//! `lambda` construction and applying closures/macros to arguments.
//! Побудова `lambda` та застосування замикань/макросів до аргументів.
//! Bau von `lambda` und Anwendung von Closures/Makros auf Argumente.

use super::{evaluate, special_forms::quoted, EvalStep};
use crate::{Closure, Environment, ErrorKind, Expr, ExprKind, LanguageError, Span, Value};
use std::{collections::HashSet, rc::Rc};

pub(super) fn create_lambda(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() < 2 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "lambda expects parameters and a body · lambda очікує параметри й тіло · lambda erwartet Parameter und einen Rumpf",
            span,
        ));
    }
    let ExprKind::List(parameter_forms) = &arguments[0].kind else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "lambda parameters must be a list · параметри lambda мають бути списком · lambda-Parameter müssen eine Liste sein",
            arguments[0].span,
        ));
    };
    let mut parameters = Vec::with_capacity(parameter_forms.len());
    let mut unique = HashSet::new();
    for parameter in parameter_forms.iter() {
        let ExprKind::Symbol(name) = &parameter.kind else {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "lambda parameter must be a symbol · параметр lambda має бути символом · lambda-Parameter muss ein Symbol sein",
                parameter.span,
            ));
        };
        if !unique.insert(name.clone()) {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                format!("duplicate lambda parameter · повторний параметр lambda · doppelter lambda-Parameter: {name}"),
                parameter.span,
            ));
        }
        parameters.push(name.clone());
    }
    Ok(Value::Closure(Rc::new(Closure {
        parameters,
        body: arguments[1..].into(),
        environment: environment.clone(),
    })))
}

pub(super) fn apply(
    function: Value,
    arguments: &[Expr],
    calling_environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    let Value::Closure(ref closure) = function else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "expression is not callable · вираз не можна викликати · Ausdruck ist nicht aufrufbar",
            span,
        ));
    };
    if arguments.len() != closure.parameters.len() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            format!(
                "lambda: expected / очікувалося / erwartet {}; received / отримано / erhalten {}",
                closure.parameters.len(),
                arguments.len()
            ),
            span,
        ));
    }

    // Arguments belong to the caller; parameters belong to the captured lexical frame.
    // Аргументи належать виклику, а параметри — захопленому лексичному фрейму.
    // Argumente gehören zum Aufrufer, Parameter zum erfassten lexikalischen Frame.
    let local_environment = closure.environment.child();
    for (parameter, argument) in closure.parameters.iter().zip(arguments.iter()) {
        let value = evaluate(argument, calling_environment)?;
        local_environment.define(parameter.clone(), value);
    }
    let last = last_body_expression(&closure.body, &local_environment, span)?;
    // Tail positions become data for the evaluator loop instead of recursive Rust calls.
    // Хвостові позиції стають даними для циклу evaluator, а не рекурсивними викликами Rust.
    // Tail-Positionen werden zu Daten für die Evaluator-Schleife statt zu rekursiven Rust-Aufrufen.
    Ok(EvalStep::TailCall {
            expression: last.clone(),
            environment: local_environment,
    })
}

/// Runs every body expression except the last for its side effects, then returns
/// the last one for the caller to evaluate in tail position. `create_lambda` always
/// builds a non-empty body, but this returns a `LanguageError` instead of panicking
/// so a future invariant change degrades gracefully rather than crashing the process.
/// Виконує всі вирази тіла, крім останнього, заради побічних ефектів, і повертає
/// останній, щоб виклик обчислив його в хвостовій позиції. `create_lambda` завжди
/// будує непорожнє тіло, але тут повертається `LanguageError`, а не паніка, щоб
/// майбутня зміна інваріанту деградувала плавно, а не аварійно завершувала процес.
/// Führt alle Rumpf-Ausdrücke außer dem letzten wegen ihrer Seiteneffekte aus und
/// gibt den letzten für die Auswertung in Tail-Position zurück. `create_lambda` baut
/// stets einen nicht leeren Rumpf, dennoch wird hier ein `LanguageError` statt eines
/// Panics zurückgegeben, damit eine künftige Invariantenänderung sanft statt abstürzend degradiert.
fn last_body_expression<'a>(
    body: &'a [Expr],
    environment: &Environment,
    span: Span,
) -> Result<&'a Expr, LanguageError> {
    let Some((last, leading)) = body.split_last() else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "lambda body must not be empty · тіло lambda не може бути порожнім · lambda-Rumpf darf nicht leer sein",
            span,
        ));
    };
    for expression in leading {
        evaluate(expression, environment)?;
    }
    Ok(last)
}

pub(super) fn apply_macro(
    closure: Rc<Closure>,
    arguments: &[Expr],
    calling_environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    if arguments.len() != closure.parameters.len() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            format!(
                "defmacro: expected / очікувалося / erwartet {}; received / отримано / erhalten {}",
                closure.parameters.len(),
                arguments.len()
            ),
            span,
        ));
    }

    let local_environment = closure.environment.child();
    for (parameter, argument) in closure.parameters.iter().zip(arguments.iter()) {
        let value = quoted(argument); // Do NOT evaluate arguments
        local_environment.define(parameter.clone(), value);
    }

    let last = last_body_expression(&closure.body, &local_environment, span)?;

    let expanded_value = evaluate(last, &local_environment)?;
    let expanded_expr = value_to_expr(expanded_value, span)?;

    Ok(EvalStep::TailCall {
        expression: expanded_expr,
        environment: calling_environment.clone(),
    })
}

fn value_to_expr(value: Value, span: Span) -> Result<Expr, LanguageError> {
    let kind = match &value {
        Value::Nil => ExprKind::List(Rc::new([])),
        Value::Bool(true) => ExprKind::Symbol("t".into()),
        Value::Bool(false) => ExprKind::List(Rc::new([])),
        Value::Number(number) => ExprKind::Number(*number),
        Value::Rational(rational) => ExprKind::Rational(*rational),
        Value::String(val) => ExprKind::String(val.clone()),
        Value::Symbol(symbol) => ExprKind::Symbol(symbol.clone()),
        Value::Pair(_, _) => {
            let mut items = Vec::new();
            let mut current = value.clone();
            loop {
                match &current {
                    Value::Pair(h, t) => {
                        items.push(value_to_expr((**h).clone(), span)?);
                        current = (**t).clone();
                    }
                    Value::Nil => break,
                    _ => {
                        return Err(LanguageError::new(
                            ErrorKind::InvalidForm,
                            "macros must return proper lists · макроси повинні повертати правильні списки · Makros müssen korrekte Listen zurückgeben",
                            span,
                        ))
                    }
                }
            }
            ExprKind::List(items.into())
        }
        Value::Closure(_) | Value::Macro(_) => {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "macros cannot return closures or macros · макроси не можуть повертати замикання або макроси · Makros dürfen keine Closures oder Makros zurückgeben",
                span,
            ))
        }
    };
    Ok(Expr { kind, span })
}
