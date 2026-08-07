//! Exact/inexact numeric handling for `+`, `-`, `*`, and `/`.
//! Обробка точних/неточних чисел для `+`, `-`, `*` та `/`.
//! Verarbeitung exakter/inexakter Zahlen für `+`, `-`, `*` und `/`.

use super::evaluate;
use crate::{Environment, ErrorKind, Expr, LanguageError, Rational, Span, Value};

pub(super) fn evaluate_arithmetic(
    operator: &str,
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if operator == "-" && arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "- expects at least one argument · - очікує щонайменше один аргумент · - erwartet mindestens ein Argument",
            span,
        ));
    }
    let values = arguments
        .iter()
        .map(|argument| numeric_value(evaluate(argument, environment)?, argument.span))
        .collect::<Result<Vec<_>, _>>()?;

    // Exact integers and rationals stay exact. One inexact operand deliberately makes the result inexact.
    // Точні цілі та раціональні лишаються точними. Один неточний операнд навмисно робить результат неточним.
    // Exakte Ganz- und rationale Zahlen bleiben exakt. Ein unexakter Operand macht das Ergebnis bewusst unexakt.
    if values
        .iter()
        .any(|value| matches!(value, Numeric::Inexact(_)))
    {
        let values = values
            .iter()
            .map(|value| value.as_f64())
            .collect::<Vec<_>>();
        let result = match operator {
            "+" => values.iter().sum(),
            "*" => values.iter().product(),
            "-" if values.len() == 1 => -values[0],
            "-" => values[1..]
                .iter()
                .fold(values[0], |result, value| result - value),
            _ => unreachable!("known arithmetic operator"),
        };
        return Ok(Value::Number(result));
    }

    let exact = values
        .into_iter()
        .map(Numeric::into_exact)
        .collect::<Vec<_>>();
    let result = match operator {
        "+" => exact
            .into_iter()
            .try_fold(Rational::integer(0), Rational::checked_add),
        "*" => exact
            .into_iter()
            .try_fold(Rational::integer(1), Rational::checked_mul),
        "-" if exact.len() == 1 => exact[0].checked_neg(),
        "-" => exact[1..]
            .iter()
            .try_fold(exact[0], |result, value| result.checked_sub(*value)),
        _ => unreachable!("known arithmetic operator"),
    }
    .ok_or_else(|| arithmetic_overflow(span))?;
    Ok(exact_value(result))
}

#[derive(Clone, Copy)]
enum Numeric {
    Exact(Rational),
    Inexact(f64),
}

impl Numeric {
    fn as_f64(self) -> f64 {
        match self {
            Self::Exact(value) => value.as_f64(),
            Self::Inexact(value) => value,
        }
    }

    fn into_exact(self) -> Rational {
        match self {
            Self::Exact(value) => value,
            Self::Inexact(_) => unreachable!("inexact operands handled before exact arithmetic"),
        }
    }
}

fn numeric_value(value: Value, span: Span) -> Result<Numeric, LanguageError> {
    match value {
        Value::Rational(value) => Ok(Numeric::Exact(value)),
        Value::Number(value)
            if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 =>
        {
            Ok(Numeric::Exact(Rational::integer(value as i64)))
        }
        Value::Number(value) => Ok(Numeric::Inexact(value)),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "arithmetic expects numbers · арифметика очікує числа · Arithmetik erwartet Zahlen",
            span,
        )),
    }
}

fn exact_value(value: Rational) -> Value {
    if value.denominator == 1 {
        Value::Number(value.numerator as f64)
    } else {
        Value::Rational(value)
    }
}

fn arithmetic_overflow(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        "exact arithmetic overflow · переповнення точної арифметики · Überlauf der exakten Arithmetik",
        span,
    )
}

pub(super) fn evaluate_division(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.is_empty() {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / очікує щонайменше один аргумент · / erwartet mindestens ein Argument",
            span,
        ));
    }
    let mut values = arguments.iter().map(|argument| {
        let value = evaluate(argument, environment)?;
        match value {
            Value::Rational(value) => Ok(value),
            Value::Number(value) if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 => {
                Ok(Rational::integer(value as i64))
            }
            _ => Err(LanguageError::new(
                ErrorKind::Type,
                "/ expects exact integers or rational numbers · / очікує точні цілі або раціональні числа · / erwartet exakte Ganz- oder rationale Zahlen",
                argument.span,
            )),
        }
    });
    // The empty-arguments case is rejected above, but the iterator is re-derived here
    // rather than trusting that earlier check, so a future reorder cannot turn this into a panic.
    // Порожній список аргументів відхиляється вище, але ітератор тут перевіряється
    // окремо, тож майбутнє перевпорядкування коду не перетвориться на паніку.
    // Der Fall leerer Argumente wird oben abgelehnt, aber der Iterator wird hier erneut
    // geprüft, sodass eine spätere Umordnung dies nicht in einen Panic verwandeln kann.
    let Some(first) = values.next() else {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "/ expects at least one argument · / очікує щонайменше один аргумент · / erwartet mindestens ein Argument",
            span,
        ));
    };
    let mut result = first?;
    if arguments.len() == 1 {
        result = Rational::integer(1)
            .checked_div(result)
            .ok_or_else(|| division_error(span))?;
    } else {
        for divisor in values {
            result = result
                .checked_div(divisor?)
                .ok_or_else(|| division_error(span))?;
        }
    }
    Ok(exact_value(result))
}

fn division_error(span: Span) -> LanguageError {
    LanguageError::new(
        ErrorKind::InvalidForm,
        "division by zero or rational overflow · ділення на нуль або переповнення дробу · Division durch null oder Bruchüberlauf",
        span,
    )
}
