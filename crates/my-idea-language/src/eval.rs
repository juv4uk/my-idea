use crate::{parse, Environment, ErrorKind, Expr, ExprKind, LanguageError, Session, Span, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct EvalResult {
    pub value: Value,
    pub output: Vec<String>,
}

pub fn eval_program(source: &str, session: &mut Session) -> Result<EvalResult, LanguageError> {
    let expressions = parse(source)?;
    let mut value = Value::Nil;
    for expression in &expressions {
        value = evaluate(expression, &session.environment)?;
    }
    Ok(EvalResult {
        value,
        output: session.output.clone(),
    })
}

fn evaluate(expression: &Expr, environment: &Environment) -> Result<Value, LanguageError> {
    match &expression.kind {
        ExprKind::Number(number) => Ok(Value::Number(*number)),
        ExprKind::String(value) => Ok(Value::String(value.clone())),
        ExprKind::Symbol(symbol) => environment.get(symbol).ok_or_else(|| {
            LanguageError::new(
                ErrorKind::UnknownSymbol,
                format!("unknown symbol · невідомий символ · unbekanntes Symbol: {symbol}"),
                expression.span,
            )
        }),
        ExprKind::List(items) if items.is_empty() => Ok(Value::Nil),
        ExprKind::List(items) => evaluate_list(items, environment, expression.span),
    }
}

fn evaluate_list(
    items: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    let operator = match &items[0].kind {
        ExprKind::Symbol(symbol) => symbol.as_str(),
        _ => {
            return Err(LanguageError::new(
                ErrorKind::InvalidForm,
                "operator must be a symbol · оператор має бути символом · Operator muss ein Symbol sein",
                items[0].span,
            ))
        }
    };
    let arguments = &items[1..];
    match operator {
        "quote" => {
            exact_arity(operator, arguments, 1, span)?;
            Ok(quoted(&arguments[0]))
        }
        "cond" => evaluate_cond(arguments, environment, span),
        "atom" => {
            exact_arity(operator, arguments, 1, span)?;
            Ok(Value::Bool(evaluate(&arguments[0], environment)?.is_atom()))
        }
        "eq" => {
            exact_arity(operator, arguments, 2, span)?;
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
        "car" => {
            exact_arity(operator, arguments, 1, span)?;
            match evaluate(&arguments[0], environment)? {
                Value::Pair(head, _) => Ok(*head),
                _ => Err(LanguageError::new(
                    ErrorKind::Type,
                    "car expects a non-empty list · car очікує непорожній список · car erwartet eine nicht leere Liste",
                    span,
                )),
            }
        }
        "cdr" => {
            exact_arity(operator, arguments, 1, span)?;
            match evaluate(&arguments[0], environment)? {
                Value::Pair(_, tail) => Ok(*tail),
                _ => Err(LanguageError::new(
                    ErrorKind::Type,
                    "cdr expects a non-empty list · cdr очікує непорожній список · cdr erwartet eine nicht leere Liste",
                    span,
                )),
            }
        }
        "cons" => {
            exact_arity(operator, arguments, 2, span)?;
            let head = evaluate(&arguments[0], environment)?;
            let tail = evaluate(&arguments[1], environment)?;
            Ok(Value::Pair(Box::new(head), Box::new(tail)))
        }
        _ => Err(LanguageError::new(
            ErrorKind::UnknownSymbol,
            format!("unknown operator · невідомий оператор · unbekannter Operator: {operator}"),
            items[0].span,
        )),
    }
}

fn evaluate_cond(
    clauses: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
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
            return evaluate(&parts[1], environment);
        }
    }
    if clauses.is_empty() {
        // The span is retained for future strict empty-cond diagnostics.
        // Діапазон збережено для майбутньої строгої діагностики порожнього `cond`.
        // Der Bereich bleibt für eine künftige strikte Diagnose eines leeren `cond` erhalten.
        let _ = span;
    }
    Ok(Value::Nil)
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
        ExprKind::String(value) => Value::String(value.clone()),
        ExprKind::Symbol(symbol) => Value::Symbol(symbol.clone()),
        ExprKind::List(items) => Value::list(items.iter().map(quoted)),
    }
}
