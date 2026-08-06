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
        let kind = token
            .parse::<f64>()
            .map(ExprKind::Number)
            .unwrap_or_else(|_| ExprKind::Symbol(token.into()));
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
