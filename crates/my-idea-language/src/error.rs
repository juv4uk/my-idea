use crate::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Parse,
    UnknownSymbol,
    Arity,
    Type,
    InvalidForm,
}

/// Structured errors let the IDE underline the exact source range later.
/// Структурована помилка дозволить IDE підкреслити точне місце в коді.
/// Strukturierte Fehler ermöglichen der IDE später, den genauen Quellbereich zu markieren.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Span,
}

impl LanguageError {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }
}

impl fmt::Display for LanguageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} · at / позиція / Stelle {}..{}",
            self.message, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for LanguageError {}
