//! Відображення AST: використовує лише власний parser/AST pinned SENS.
 //! Ніякого дубльованого парсера, опрацювання семантики або машинних літералів у TUI.
use sens::{parse, Expr, ExprKind};

pub fn render(source: &str) -> Vec<String> {
    match parse(source) {
        Ok(forms) if forms.is_empty() => vec!["Порожній файл".to_owned()],
        Ok(forms) => {
            let mut lines = Vec::new();
            for expr in &forms {
                append_expr(expr, source, 0, &mut lines);
            }
            lines
        }
        Err(error) => {
            let (line, column) = error.line_col(source);
            let mut lines = vec![format!("Синтаксис SENS: {}:{}", line, column)];
            lines.extend(error.render(source).lines().map(str::to_owned));
            lines
        }
    }
}

fn append_expr(expr: &Expr, source: &str, depth: usize, lines: &mut Vec<String>) {
    if lines.len() >= 2500 { return; }
    let indent = "  ".repeat(depth.min(24));
    match &expr.kind {
        ExprKind::List(items) => {
            lines.push(format!("{indent}Список ({})", items.len()));
            for item in items.iter() { append_expr(item, source, depth + 1, lines); }
        }
        ExprKind::Pair(head, tail) => {
            lines.push(format!("{indent}Пара"));
            append_expr(head, source, depth + 1, lines);
            append_expr(tail, source, depth + 1, lines);
        }
        ExprKind::Call(_, args) | ExprKind::DomainCall(_, args) => {
            lines.push(format!("{indent}Виклик ({})", args.len()));
            for arg in args.iter() { append_expr(arg, source, depth + 1, lines); }
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
        let lines = render("(атом? 'мама)\n(+ 2 3)");
        assert!(lines.iter().any(|s| s.contains("Список")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("атом?")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("мама")), "{lines:?}");
        assert!(!lines.iter().any(|s| s.contains("Синтаксис SENS")), "{lines:?}");
    }

    #[test]
    fn parser_errors_are_reported_with_real_sens_positions() {
        let lines = render("(атом? 'мама)\n(");
        assert!(lines.iter().any(|s| s.contains("Синтаксис SENS: 2:1")), "{lines:?}");
        assert!(lines.iter().any(|s| s.contains("line/riadok/Zeile")), "{lines:?}");
    }
}
