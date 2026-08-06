use std::rc::Rc;

/// Byte range in the original UTF-8 source.
/// Діапазон байтів у початковому тексті UTF-8.
/// Bytebereich im ursprünglichen UTF-8-Quelltext.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Number(f64),
    String(Rc<str>),
    Symbol(Rc<str>),
    List(Rc<[Expr]>),
}
