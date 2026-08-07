use crate::{ErrorKind, Expr, ExprKind, LanguageError, Span};

pub fn parse(source: &str) -> Result<Vec<Expr>, LanguageError> {
    let mut parser = Parser { source, cursor: 0 };
    let mut expressions = Vec::new();
    parser.skip_ignored();
    while parser.cursor < source.len() {
        expressions.push(parser.expression()?);
        parser.skip_ignored();
    }
    Ok(expressions)
}

struct Parser<'a> {
    source: &'a str,
    cursor: usize,
}

impl Parser<'_> {
    fn expression(&mut self) -> Result<Expr, LanguageError> {
        self.skip_ignored();
        let start = self.cursor;
        match self.peek() {
            Some('(') => self.list(start),
            Some(')') => Err(self.error(
                "unexpected closing parenthesis · неочікувана закривна дужка · unerwartete schließende Klammer",
                start,
                start + 1,
            )),
            Some('\'') => self.quoted(start),
            Some('"') => self.string(start),
            Some(_) => self.atom(start),
            None => Err(self.error(
                "expected an expression · очікувався вираз · Ausdruck erwartet",
                start,
                start,
            )),
        }
    }

    /// Reader sugar is normalized here, so the evaluator only needs `quote`.
    /// Синтаксичний цукор нормалізується тут, тому обчислювачу достатньо `quote`.
    /// Reader-Syntaxzucker wird hier normalisiert, sodass der Evaluator nur `quote` benötigt.
    fn quoted(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        let value = self.expression()?;
        let end = value.span.end;
        Ok(Expr {
            span: Span { start, end },
            kind: ExprKind::List(vec![
                Expr {
                    kind: ExprKind::Symbol("quote".into()),
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                value,
            ].into()),
        })
    }

    fn list(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        let mut items = Vec::new();
        loop {
            self.skip_ignored();
            match self.peek() {
                Some(')') => {
                    self.bump();
                    return Ok(Expr {
                        kind: ExprKind::List(items.into()),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }
                Some(_) => items.push(self.expression()?),
                None => {
                    return Err(self.error(
                        "unclosed list · незакритий список · nicht geschlossene Liste",
                        start,
                        self.cursor,
                    ))
                }
            }
        }
    }

    fn string(&mut self, start: usize) -> Result<Expr, LanguageError> {
        self.bump();
        let mut value = String::new();
        while let Some(character) = self.bump() {
            match character {
                '"' => {
                    return Ok(Expr {
                        kind: ExprKind::String(value.into()),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    })
                }
                '\\' => match self.bump() {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(other) => value.push(other),
                    None => {
                        return Err(self.error(
                            "unfinished string escape · незавершена escape-послідовність · unvollständige Escape-Sequenz",
                            start,
                            self.cursor,
                        ))
                    }
                },
                other => value.push(other),
            }
        }
        Err(self.error(
            "unclosed string · незакритий рядок · nicht geschlossene Zeichenkette",
            start,
            self.cursor,
        ))
    }

    fn atom(&mut self, start: usize) -> Result<Expr, LanguageError> {
        while let Some(character) = self.peek() {
            if character.is_whitespace() || matches!(character, '(' | ')' | ';') {
                break;
            }
            self.bump();
        }
        let token = &self.source[start..self.cursor];
        let kind = if let Some((num, den)) = token.split_once('/') {
            if let (Ok(n), Ok(d)) = (num.parse::<i64>(), den.parse::<i64>()) {
                if let Some(r) = crate::value::Rational::new(n, d) {
                    ExprKind::Rational(r)
                } else {
                    ExprKind::Symbol(token.into())
                }
            } else {
                token.parse::<f64>().map(ExprKind::Number).unwrap_or_else(|_| ExprKind::Symbol(token.into()))
            }
        } else {
            token.parse::<f64>().map(ExprKind::Number).unwrap_or_else(|_| ExprKind::Symbol(token.into()))
        };
        Ok(Expr {
            kind,
            span: Span {
                start,
                end: self.cursor,
            },
        })
    }

    fn skip_ignored(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.bump();
            }
            if self.peek() != Some(';') {
                break;
            }
            while self.peek().is_some_and(|character| character != '\n') {
                self.bump();
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.cursor += character.len_utf8();
        Some(character)
    }

    fn error(&self, message: &str, start: usize, end: usize) -> LanguageError {
        LanguageError::new(ErrorKind::Parse, message, Span { start, end })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_one(source: &str) -> Expr {
        let expressions = parse(source).expect("parsing should succeed");
        assert_eq!(expressions.len(), 1, "expected exactly one top-level form");
        expressions.into_iter().next().unwrap()
    }

    #[test]
    fn parses_integers_and_floats_as_numbers() {
        assert!(matches!(parse_one("42").kind, ExprKind::Number(n) if n == 42.0));
        assert!(matches!(parse_one("-3.5").kind, ExprKind::Number(n) if n == -3.5));
    }

    #[test]
    fn parses_slash_notation_as_exact_rational() {
        let ExprKind::Rational(rational) = parse_one("5/336").kind else {
            panic!("expected a rational literal");
        };
        assert_eq!((rational.numerator, rational.denominator), (5, 336));
    }

    #[test]
    fn zero_denominator_falls_back_to_a_plain_symbol() {
        // `1/0` is not a valid Rational (see value::Rational::new), so the reader
        // treats it as an ordinary symbol instead of failing the whole parse.
        // `1/0` не є коректним Rational (див. value::Rational::new), тому reader
        // трактує його як звичайний символ, а не провалює весь парсинг.
        // `1/0` ist kein gültiges Rational (siehe value::Rational::new), daher
        // behandelt der Reader es als gewöhnliches Symbol statt das Parsing scheitern zu lassen.
        assert!(matches!(parse_one("1/0").kind, ExprKind::Symbol(s) if &*s == "1/0"));
    }

    #[test]
    fn parses_symbols() {
        assert!(matches!(parse_one("foo-bar?").kind, ExprKind::Symbol(s) if &*s == "foo-bar?"));
    }

    #[test]
    fn parses_strings_with_escapes() {
        let ExprKind::String(value) = parse_one(r#""line\n\ttab\"quote""#).kind else {
            panic!("expected a string literal");
        };
        assert_eq!(&*value, "line\n\ttab\"quote");
    }

    #[test]
    fn unclosed_string_is_a_parse_error() {
        let error = parse(r#""unterminated"#).unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn parses_nested_lists() {
        let ExprKind::List(items) = parse_one("(1 (2 3) 4)").kind else {
            panic!("expected a list");
        };
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[1].kind, ExprKind::List(inner) if inner.len() == 2));
    }

    #[test]
    fn unclosed_list_is_a_parse_error() {
        let error = parse("(1 2 3").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn unexpected_closing_paren_is_a_parse_error() {
        let error = parse(")").unwrap_err();
        assert_eq!(error.kind, ErrorKind::Parse);
    }

    #[test]
    fn quote_sugar_desugars_to_quote_form() {
        let ExprKind::List(items) = parse_one("'(1 2)").kind else {
            panic!("expected a list");
        };
        assert_eq!(items.len(), 2);
        assert!(matches!(&items[0].kind, ExprKind::Symbol(s) if &**s == "quote"));
    }

    #[test]
    fn semicolon_comments_are_skipped() {
        let expressions = parse("; a comment\n42 ; trailing comment").expect("should parse");
        assert_eq!(expressions.len(), 1);
        assert!(matches!(expressions[0].kind, ExprKind::Number(n) if n == 42.0));
    }

    #[test]
    fn unicode_symbols_and_comments_are_supported() {
        let expressions = parse("; коментар\nпривіт").expect("should parse");
        assert!(matches!(&expressions[0].kind, ExprKind::Symbol(s) if &**s == "привіт"));
    }

    #[test]
    fn parses_multiple_top_level_expressions() {
        let expressions = parse("1 2 3").expect("should parse");
        assert_eq!(expressions.len(), 3);
    }

    #[test]
    fn empty_source_parses_to_no_expressions() {
        assert_eq!(parse("   ; only a comment\n").expect("should parse"), vec![]);
    }
}
