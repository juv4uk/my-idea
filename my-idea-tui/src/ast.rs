//! Відображення AST: використовує лише власний parser/AST pinned SENS.
 //! Ніякого дубльованого парсера, опрацювання семантики або машинних літералів у TUI.
use sens::{parse, Expr, ExprKind};
use crate::i18n;

pub fn render(source: &str, lang: &str) -> Vec<String> {
    match parse(source) {
        Ok(forms) if forms.is_empty() => vec![i18n::t(lang, "ast-empty").to_owned()],
        Ok(forms) => {
            let mut lines = Vec::new();
            for expr in &forms {
                append_expr(expr, source, lang, 0, &mut lines);
            }
            lines
        }
        Err(error) => {
            let (line, column) = error.line_col(source);
            let mut lines = vec![format!("{}: {}:{}", i18n::t(lang, "ast-syntax"), line, column)];
            lines.extend(error.render(source).lines().map(str::to_owned));
            lines
        }
    }
}

fn append_expr(expr: &Expr, source: &str, lang: &str, depth: usize, lines: &mut Vec<String>) {
    if lines.len() >= 2500 { return; }
    let indent = "  ".repeat(depth.min(24));
    match &expr.kind {
        ExprKind::List(items) => {
            // Reader уже визначив межі першого елемента. Беремо написання
            // виключно з оригінального .lisp, не вигадуємо SENS/доменний код.
            // Це важливо для українських назв функцій і вкладених форм.
            let head = items.first()
                .and_then(|first| source.get(first.span.start..first.span.end))
                .map(|raw| raw.chars().take(48).collect::<String>().replace('\n', "↵"));
            let label = match head {
                Some(head) if !head.is_empty() => format!("{} ({}) · {head}", i18n::t(lang, "ast-list"), items.len()),
                _ => format!("{} ({})", i18n::t(lang, "ast-list"), items.len()),
            };
            lines.push(format!("{indent}{label}"));
            for item in items.iter() { append_expr(item, source, lang, depth + 1, lines); }
        }
        ExprKind::Pair(head, tail) => {
            lines.push(format!("{indent}{}", i18n::t(lang, "ast-pair")));
            append_expr(head, source, lang, depth + 1, lines);
            append_expr(tail, source, lang, depth + 1, lines);
        }
        ExprKind::Call(_, args) | ExprKind::DomainCall(_, args) => {
            lines.push(format!("{indent}{} ({})", i18n::t(lang, "ast-call"), args.len()));
            for arg in args.iter() { append_expr(arg, source, lang, depth + 1, lines); }
        }
        _ => {
            // Показуємо оригінальну українську проєкцію, а не видумані доменні коди.
            let raw = source.get(expr.span.start..expr.span.end).unwrap_or("");
            let label: String = raw.chars().take(80).collect::<String>().replace('\n', "↵");
            lines.push(format!("{indent}{label}"));
        }
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
