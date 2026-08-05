//! Independent, capability-free core of the my-lisp language.
//! Незалежне ядро мови my-lisp без доступу до можливостей операційної системи.
//! Unabhängiger Sprachkern von my-lisp ohne Zugriff auf Betriebssystemfunktionen.
//!
//! The crate deliberately knows nothing about Tauri, files, the network, or UI.
//! Крейт навмисно нічого не знає про Tauri, файли, мережу чи інтерфейс.
//! Das Crate kennt bewusst weder Tauri noch Dateien, Netzwerk oder Benutzeroberfläche.

mod environment;
mod error;
mod eval;
mod parser;
mod syntax;
mod value;

pub use environment::{Environment, Session};
pub use error::{ErrorKind, LanguageError};
pub use eval::{eval_program, EvalResult};
pub use parser::parse;
pub use syntax::{Expr, ExprKind, Span};
pub use value::{Closure, Rational, Value};
