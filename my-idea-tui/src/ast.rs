//! Відображення AST: використовує лише власний parser/AST pinned SENS.
 //! Ніякого дубльованого парсера, опрацювання семантики або машинних літералів у TUI.
use sens::{parse, Expr, ExprKind};
use crate::i18n;

/// Рядок AST разом зі спаном вузла, який його породив, — для «живого» підсвічування.
#[derive(Clone, Debug)]
pub struct AstLine {
    pub text: String,
    pub span: Option<(usize, usize)>,
}

impl AstLine {
    fn plain(text: String) -> Self {
        AstLine { text, span: None }
    }
}

/// Текстове подання AST (панель, розбір відповідей swarm-node).
pub fn render(source: &str, lang: &str) -> Vec<String> {
    render_spans(source, lang)
        .into_iter()
        .map(|line| line.text)
        .collect()
}

/// Той самий обхід, але зі спанами вузлів (щоб підсвітити форму під курсором).
pub fn render_spans(source: &str, lang: &str) -> Vec<AstLine> {
    match parse(source) {
        Ok(forms) if forms.is_empty() => {
            vec![AstLine::plain(i18n::t(lang, "ast-empty").to_owned())]
        }
        Ok(forms) => sens::ast_nodes(source, &forms)
            .into_iter()
            .map(|node| AstLine {
                text: format!("{}{}", "  ".repeat(node.depth.saturating_sub(1)), ast_text(lang, &node)),
                span: Some(node.span),
            })
            .collect(),
        Err(error) => {
            let (line, column) = error.line_col(source);
            let mut lines = vec![AstLine::plain(format!(
                "{}: {}:{}",
                i18n::t(lang, "ast-syntax"),
                line,
                column
            ))];
            lines.extend(
                error
                    .render(source)
                    .lines()
                    .map(|raw| AstLine::plain(raw.to_owned())),
            );
            lines
        }
    }
}

/// Підпис вузла — UI-мова хоста (uk→de→en); класифікацію дає SENS.
fn ast_text(lang: &str, node: &sens::AstNode) -> String {
    match &node.kind {
        sens::AstNodeKind::List { items, head } => match head {
            Some(head) if !head.is_empty() => {
                format!("{} ({}) \u{b7} {}", i18n::t(lang, "ast-list"), items, clip(head))
            }
            _ => format!("{} ({})", i18n::t(lang, "ast-list"), items),
        },
        sens::AstNodeKind::Pair => i18n::t(lang, "ast-pair").to_owned(),
        sens::AstNodeKind::Call { arguments, .. }
        | sens::AstNodeKind::DomainCall { arguments, .. } => {
            format!("{} ({})", i18n::t(lang, "ast-call"), arguments)
        }
        _ => clip(&node.excerpt),
    }
}

fn clip(raw: &str) -> String {
    raw.chars().take(80).collect::<String>().replace('\n', "\u{21b5}")
}

/// Голова найглибшої форми, що містить байтовий зсув курсора (живий AST).
pub fn head_at_cursor(source: &str, cursor: usize) -> Option<String> {
    let forms = parse(source).ok()?;
    let mut found = None;
    for expr in &forms {
        visit_at_cursor(expr, source, cursor, &mut found);
    }
    found
}

fn visit_at_cursor(expr: &Expr, source: &str, cursor: usize, found: &mut Option<String>) {
    if !(expr.span.start <= cursor && cursor <= expr.span.end) {
        return;
    }
    match &expr.kind {
        ExprKind::List(items) | ExprKind::Call(_, items) | ExprKind::DomainCall(_, items) => {
            if let Some(raw) = items
                .first()
                .and_then(|first| source.get(first.span.start..first.span.end))
            {
                let head = raw.trim();
                if !head.is_empty() {
                    *found = Some(head.chars().take(48).collect());
                }
            }
            for item in items.iter() {
                visit_at_cursor(item, source, cursor, found);
            }
        }
        ExprKind::Pair(head, tail) => {
            visit_at_cursor(head, source, cursor, found);
            visit_at_cursor(tail, source, cursor, found);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::render;

    #[test]
    fn real_pinned_sens_ast_preserves_ukrainian_source() {
        let lines = render("(атом? 'мама)\n(+ 2 3)", "uk");
        assert!(lines.iter().any(|s| s.contains("Список")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("Список (2) · атом?")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("Список (3) · +")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("мама")), "{lines:?}");
        assert!(!lines.iter().any(|s| s.contains("Синтаксис SENS")), "{lines:?}");
    }

    #[test]
    fn nested_reader_lists_show_original_ukrainian_heads_without_new_semantics() {
        let lines = render("(умова (атом? їжак) (цитата мама))", "uk");
        assert!(lines.iter().any(|s| s.contains("Список (3) · умова")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("Список (2) · атом?")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("Список (2) · цитата")), "{lines:?}");
    }

    #[test]
    fn parser_errors_are_reported_with_real_sens_positions() {
        let lines = render("(атом? 'мама)\n(", "uk");
        assert!(lines.iter().any(|s| s.contains("Синтаксис SENS: 2:1")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("line/riadok/Zeile")), "{lines:?}");
    }

    #[test]
    fn ui_labels_change_but_pinned_ukrainian_sens_ast_does_not() {
        let source = "(умова (атом? їжак) (цитата мама))";
        let uk = render(source, "uk");
        let de = render(source, "de");
        let en = render(source, "en");
        assert!(uk.iter().any(|s| s.contains("Список (3) · умова")), "{uk:?}");
        assert!(de.iter().any(|s| s.contains("Liste (3) · умова")), "{de:?}");
        assert!(en.iter().any(|s| s.contains("List (3) · умова")), "{en:?}");
        assert!(de.iter().any(|s| s.contains("цитата")), "{de:?}");
        assert!(en.iter().any(|s| s.contains("атом?")), "{en:?}");
        assert_eq!(render("", "de"), vec!["Leere Datei"]);
        assert_eq!(render("", "en"), vec!["Empty file"]);
        assert!(render("(атом? 'мама)\n(", "en")
            .iter().any(|s| s.starts_with("SENS syntax: 2:1")));
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::head_at_cursor;

    #[test]
    fn finds_innermost_form_head_at_cursor() {
        let source = "(визначити площа (добуток 3 4))\n";
        let inner = source.find("добуток").unwrap() + 1;
        assert_eq!(head_at_cursor(source, inner).as_deref(), Some("добуток"));
        let outer = source.find("визначити").unwrap() + 1;
        assert_eq!(head_at_cursor(source, outer).as_deref(), Some("визначити"));
        assert_eq!(head_at_cursor("   ", 1), None);
    }
}
