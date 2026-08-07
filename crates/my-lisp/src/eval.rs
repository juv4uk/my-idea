use crate::{
    parse, Closure, Environment, ErrorKind, Expr, ExprKind, LanguageError, Rational, Session, Span,
    Value,
};
use std::{collections::HashSet, rc::Rc};

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

enum EvalStep {
    Value(Value),
    TailCall {
        expression: Expr,
        environment: Environment,
    },
}

fn evaluate(expression: &Expr, environment: &Environment) -> Result<Value, LanguageError> {
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

fn evaluate_step(expression: &Expr, environment: &Environment) -> Result<EvalStep, LanguageError> {
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
            exact_arity("quote", arguments, 1, span)?;
            Ok(EvalStep::Value(quoted(&arguments[0])))
        }
        Some("lambda") => create_lambda(arguments, environment, span).map(EvalStep::Value),
        Some("def") => evaluate_definition(arguments, environment, span).map(EvalStep::Value),
        Some("defmacro") => evaluate_defmacro(arguments, environment, span).map(EvalStep::Value),
        Some("list") => evaluate_list_func(arguments, environment, span).map(EvalStep::Value),
        Some("cond") => evaluate_cond(arguments, environment, span),
        Some("atom") => {
            exact_arity("atom", arguments, 1, span)?;
            Ok(EvalStep::Value(Value::Bool(
                evaluate(&arguments[0], environment)?.is_atom(),
            )))
        }
        Some("eq") => evaluate_eq(arguments, environment, span).map(EvalStep::Value),
        Some("car") => evaluate_car(arguments, environment, span).map(EvalStep::Value),
        Some("cdr") => evaluate_cdr(arguments, environment, span).map(EvalStep::Value),
        Some("cons") => evaluate_cons(arguments, environment, span).map(EvalStep::Value),
        Some("/") => evaluate_division(arguments, environment, span).map(EvalStep::Value),
        // Binding the operator symbol in the pattern avoids re-deriving it with
        // an `.expect()`, so a future refactor of `as_symbol` cannot turn this into a panic.
        // Захоплення символа оператора прямо в патерні уникає повторного `.expect()`,
        // тож майбутня зміна `as_symbol` не зможе перетворити це на паніку.
        // Das Binden des Operator-Symbols im Pattern vermeidet ein erneutes `.expect()`,
        // sodass eine spätere Änderung an `as_symbol` dies nicht zu einem Panic machen kann.
        Some(operator @ ("+" | "-" | "*")) => {
            evaluate_arithmetic(operator, arguments, environment, span).map(EvalStep::Value)
        }
        _ => {
            let function = evaluate(&items[0], environment)?;
            match &function {
                Value::Macro(closure) => apply_macro(closure.clone(), arguments, environment, span),
                _ => apply(function, arguments, environment, span),
            }
        }
    }
}

fn evaluate_arithmetic(
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

fn evaluate_definition(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("def", arguments, 2, span)?;
    let ExprKind::Symbol(name) = &arguments[0].kind else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "def expects a symbol name · def очікує назву-символ · def erwartet einen Symbolnamen",
            arguments[0].span,
        ));
    };
    let value = evaluate(&arguments[1], environment)?;
    // The shared lexical frame makes recursive definitions visible to their closure after binding.
    // Спільний лексичний фрейм робить рекурсивне визначення видимим замиканню після зв’язування.
    // Der gemeinsame lexikalische Frame macht rekursive Definitionen nach der Bindung für ihre Closure sichtbar.
    environment.define(name.clone(), value.clone());
    Ok(value)
}

fn evaluate_defmacro(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    if arguments.len() < 2 {
        return Err(LanguageError::new(
            ErrorKind::Arity,
            "defmacro expects a name, parameters, and a body · defmacro очікує назву, параметри й тіло · defmacro erwartet einen Namen, Parameter und einen Rumpf",
            span,
        ));
    }
    let ExprKind::Symbol(name) = &arguments[0].kind else {
        return Err(LanguageError::new(
            ErrorKind::InvalidForm,
            "defmacro expects a symbol name · defmacro очікує назву-символ · defmacro erwartet einen Symbolnamen",
            arguments[0].span,
        ));
    };
    let closure_val = create_lambda(&arguments[1..], environment, span)?;
    let Value::Closure(closure) = &closure_val else {
        unreachable!("create_lambda always returns Closure")
    };
    let macro_val = Value::Macro(closure.clone());
    environment.define(name.clone(), macro_val.clone());
    Ok(macro_val)
}

fn evaluate_list_func(
    arguments: &[Expr],
    environment: &Environment,
    _span: Span,
) -> Result<Value, LanguageError> {
    let mut values = Vec::with_capacity(arguments.len());
    for argument in arguments {
        values.push(evaluate(argument, environment)?);
    }
    Ok(Value::list(values))
}

fn evaluate_division(
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

fn create_lambda(
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

fn apply(
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

fn evaluate_eq(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("eq", arguments, 2, span)?;
    let left = evaluate(&arguments[0], environment)?;
    let right = evaluate(&arguments[1], environment)?;
    if !left.is_atom() || !right.is_atom() {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "eq expects two atoms · eq очікує два атоми · eq erwartet zwei Atome",
            span,
        ));
    }
    Ok(Value::Bool(left == right))
}

fn evaluate_car(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("car", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::Pair(ref head, _) => Ok((**head).clone()),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "car expects a non-empty list · car очікує непорожній список · car erwartet eine nicht leere Liste",
            span,
        )),
    }
}

fn evaluate_cdr(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("cdr", arguments, 1, span)?;
    match evaluate(&arguments[0], environment)? {
        Value::Pair(_, ref tail) => Ok((**tail).clone()),
        _ => Err(LanguageError::new(
            ErrorKind::Type,
            "cdr expects a non-empty list · cdr очікує непорожній список · cdr erwartet eine nicht leere Liste",
            span,
        )),
    }
}

fn evaluate_cons(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("cons", arguments, 2, span)?;
    let head = evaluate(&arguments[0], environment)?;
    let tail = evaluate(&arguments[1], environment)?;
    Ok(Value::Pair(Rc::new(head), Rc::new(tail)))
}

fn evaluate_cond(
    clauses: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<EvalStep, LanguageError> {
    for clause in clauses {
        let ExprKind::List(parts) = &clause.kind else {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "cond expects list clauses · cond очікує списки-умови · cond erwartet Listenklauseln",
                clause.span,
            ));
        };
        if parts.len() != 2 {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "cond expects (test expression) clauses · cond очікує умови (перевірка вираз) · cond erwartet Klauseln der Form (Test Ausdruck)",
                clause.span,
            ));
        }
        if evaluate(&parts[0], environment)?.is_truthy() {
            return evaluate_step(&parts[1], environment);
        }
    }
    if clauses.is_empty() {
        // The span is retained for future strict empty-cond diagnostics.
        // Діапазон збережено для майбутньої строгої діагностики порожнього `cond`.
        // Der Bereich bleibt für eine künftige strikte Diagnose eines leeren `cond` erhalten.
        let _ = span;
    }
    Ok(EvalStep::Value(Value::Nil))
}

fn exact_arity(
    operator: &str,
    arguments: &[Expr],
    expected: usize,
    span: Span,
) -> Result<(), LanguageError> {
    if arguments.len() == expected {
        return Ok(());
    }
    Err(LanguageError::new(
        ErrorKind::Arity,
        format!(
            "{operator}: expected / очікувалося / erwartet {expected}; received / отримано / erhalten {}",
            arguments.len()
        ),
        span,
    ))
}

fn quoted(expression: &Expr) -> Value {
    match &expression.kind {
        ExprKind::Number(number) => Value::Number(*number),
        ExprKind::Rational(rational) => Value::Rational(*rational),
        ExprKind::String(value) => Value::String(value.clone()),
        ExprKind::Symbol(symbol) => Value::Symbol(symbol.clone()),
        ExprKind::List(items) => Value::list(items.iter().map(quoted)),
    }
}

fn apply_macro(
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
