//! Evaluator entry points and the special-form dispatcher.
//! Точки входу evaluator і диспетчер спеціальних форм.
//! Einstiegspunkte des Evaluators und der Sonderformen-Dispatcher.
//!
//! The evaluator is split by concern: this module owns the trampoline loop and
//! dispatch table, `arithmetic` owns exact/inexact number handling, `special_forms`
//! owns the McCarthy primitives plus `def`/`defmacro`/`cond`, and `closures` owns
//! `lambda` construction and function/macro application.
//! Evaluator розділено за відповідальністю: цей модуль володіє циклом trampoline
//! та таблицею диспетчеризації, `arithmetic` — точними/неточними числами,
//! `special_forms` — примітивами Маккарті та `def`/`defmacro`/`cond`, а `closures` —
//! побудовою `lambda` і застосуванням функцій/макросів.
//! Der Evaluator ist nach Zuständigkeit aufgeteilt: dieses Modul besitzt die
//! Trampolin-Schleife und die Dispatch-Tabelle, `arithmetic` die exakte/inexakte
//! Zahlenverarbeitung, `special_forms` die McCarthy-Primitive sowie `def`/`defmacro`/
//! `cond`, und `closures` den Bau von `lambda` und die Anwendung von Funktionen/Makros.

mod arithmetic;
mod closures;
mod special_forms;

use crate::{parse, Environment, ErrorKind, Expr, ExprKind, LanguageError, Session, Span, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct EvalResult {
    pub value: Value,
    pub output: Vec<String>,
}

pub fn eval_parsed_expressions(
    expressions: &[Expr],
    session: &mut Session,
) -> Result<EvalResult, LanguageError> {
    let mut value = Value::Nil;
    for expression in expressions {
        value = evaluate(expression, &session.environment)?;
    }
    Ok(EvalResult {
        value,
        output: session.output.clone(),
    })
}

/// Evaluates source string by parsing it and running the resulting expressions.
/// Обчислює сирцевий рядок через парсинг та виконання отриманих виразів.
/// Wertet den Quelltext durch Parsing und Ausführung der Ausdrücke aus.
pub fn eval_program(source: &str, session: &mut Session) -> Result<EvalResult, LanguageError> {
    let expressions = parse(source)?;
    eval_parsed_expressions(&expressions, session)
}

pub(crate) enum EvalStep {
    Value(Value),
    TailCall {
        expression: Expr,
        environment: Environment,
    },
}

pub(crate) fn evaluate(expression: &Expr, environment: &Environment) -> Result<Value, LanguageError> {
    let (mut owned_expression, mut owned_environment) =
        match evaluate_step(expression, environment)? {
            EvalStep::Value(value) => return Ok(value),
            EvalStep::TailCall {
                expression,
                environment,
            } => (expression, environment),
        };

    loop {
        match evaluate_step(&owned_expression, &owned_environment)? {
            EvalStep::Value(value) => return Ok(value),
            EvalStep::TailCall {
                expression: next,
                environment: next_environment,
            } => {
                owned_expression = next;
                owned_environment = next_environment;
            }
        }
    }
}

pub(crate) fn evaluate_step(
    expression: &Expr,
    environment: &Environment,
) -> Result<EvalStep, LanguageError> {
    match &expression.kind {
        ExprKind::Number(number) => Ok(EvalStep::Value(Value::Number(*number))),
        ExprKind::Rational(rational) => Ok(EvalStep::Value(Value::Rational(*rational))),
        ExprKind::String(value) => Ok(EvalStep::Value(Value::String(value.clone()))),
        ExprKind::Symbol(symbol) => environment.get(symbol).map(EvalStep::Value).ok_or_else(|| {
            LanguageError::new(
                ErrorKind::UnknownSymbol,
                format!("unknown symbol · невідомий символ · unbekanntes Symbol: {symbol}"),
                expression.span,
            )
        }),
        ExprKind::List(items) if items.is_empty() => Ok(EvalStep::Value(Value::Nil)),
        ExprKind::List(items) => evaluate_list(items, environment, expression.span),
    }
}

fn evaluate_list(
    items: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    let arguments = &items[1..];
    // Special forms stay explicit because they control which arguments are evaluated.
    // Спеціальні форми лишаються явними, бо вони керують обчисленням аргументів.
    // Sonderformen bleiben explizit, weil sie die Auswertung ihrer Argumente steuern.
    match items[0].kind.as_symbol() {
        Some("quote") => {
            special_forms::exact_arity("quote", arguments, 1, span)?;
            Ok(EvalStep::Value(special_forms::quoted(&arguments[0])))
        }
        Some("lambda") => closures::create_lambda(arguments, environment, span).map(EvalStep::Value),
        Some("def") => {
            special_forms::evaluate_definition(arguments, environment, span).map(EvalStep::Value)
        }
        Some("defmacro") => {
            special_forms::evaluate_defmacro(arguments, environment, span).map(EvalStep::Value)
        }
        Some("list") => {
            special_forms::evaluate_list_func(arguments, environment, span).map(EvalStep::Value)
        }
        Some("cond") => special_forms::evaluate_cond(arguments, environment, span),
        Some("atom") => {
            special_forms::exact_arity("atom", arguments, 1, span)?;
            Ok(EvalStep::Value(Value::Bool(
                evaluate(&arguments[0], environment)?.is_atom(),
            )))
        }
        Some("eq") => special_forms::evaluate_eq(arguments, environment, span).map(EvalStep::Value),
        Some("car") => special_forms::evaluate_car(arguments, environment, span).map(EvalStep::Value),
        Some("cdr") => special_forms::evaluate_cdr(arguments, environment, span).map(EvalStep::Value),
        Some("cons") => {
            special_forms::evaluate_cons(arguments, environment, span).map(EvalStep::Value)
        }
        Some("/") => arithmetic::evaluate_division(arguments, environment, span).map(EvalStep::Value),
        // Binding the operator symbol in the pattern avoids re-deriving it with
        // an `.expect()`, so a future refactor of `as_symbol` cannot turn this into a panic.
        // Захоплення символа оператора прямо в патерні уникає повторного `.expect()`,
        // тож майбутня зміна `as_symbol` не зможе перетворити це на паніку.
        // Das Binden des Operator-Symbols im Pattern vermeidet ein erneutes `.expect()`,
        // sodass eine spätere Änderung an `as_symbol` dies nicht zu einem Panic machen kann.
        Some(operator @ ("+" | "-" | "*")) => {
            arithmetic::evaluate_arithmetic(operator, arguments, environment, span).map(EvalStep::Value)
        }
        _ => {
            let function = evaluate(&items[0], environment)?;
            match &function {
                Value::Macro(closure) => {
                    closures::apply_macro(closure.clone(), arguments, environment, span)
                }
                _ => closures::apply(function, arguments, environment, span),
            }
        }
    }
}

trait ExprKindExt {
    fn as_symbol(&self) -> Option<&str>;
}

impl ExprKindExt for ExprKind {
    fn as_symbol(&self) -> Option<&str> {
        match self {
            ExprKind::Symbol(symbol) => Some(symbol),
            _ => None,
        }
    }
}

#[cfg(test)]
mod single_pass_eval_tests {
    use super::*;

    #[test]
    fn single_pass_eval_parsed_expressions_evaluates_preparsed_ast() {
        let source = "(def x (/ 1 3)) (cons x '())";
        let forms = parse(source).expect("parsing should succeed");
        let mut session = Session::default();
        let result = eval_parsed_expressions(&forms, &mut session)
            .expect("eval_parsed_expressions should succeed");
        assert_eq!(result.value.to_string(), "(1/3)");
    }

    #[test]
    fn macros_expand_and_evaluate_correctly() {
        let source = r#"
            (defmacro unless (condition body)
                (list 'cond
                    (list condition '())
                    (list 't body)))
            (unless () 'success)
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session).expect("eval should succeed");
        assert_eq!(result.value.to_string(), "success");
    }

    #[test]
    fn macro_expansion_preserves_exact_rationals() {
        let source = r#"
            (defmacro half-of-third ()
                (/ 1 6))
            (half-of-third)
        "#;
        let mut session = Session::default();
        let result = eval_program(source, &mut session).expect("eval should succeed");
        assert_eq!(result.value.to_string(), "1/6");
    }
}
