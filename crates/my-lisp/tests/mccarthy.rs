use my_lisp::{eval_program, parse, ErrorKind, Rational, Session, Value};

fn eval(source: &str) -> Value {
    eval_program(source, &mut Session::default()).unwrap().value
}

#[test]
fn division_is_an_exact_reduced_rational() {
    assert_eq!(eval("(/ 5 6 8 7)"), Value::Rational(Rational::new(5, 336).unwrap()));
    assert_eq!(eval("(/ 8 4)"), Value::Number(2.0));
    assert_eq!(eval("(/ (/ 2 3))"), Value::Rational(Rational::new(3, 2).unwrap()));
}

#[test]
fn bootstrap_library_is_written_and_executed_in_my_lisp() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    assert_eq!(
        eval_program("(second '(radio antenna))", &mut session).unwrap().value,
        Value::Symbol("antenna".into())
    );
    assert_eq!(eval_program("(not '())", &mut session).unwrap().value, Value::Bool(true));
}

#[test]
fn reader_supports_unicode_comments_and_quote_sugar() {
    let expressions = parse("; коментар\n'радіо").unwrap();
    assert_eq!(expressions.len(), 1);
    assert_eq!(eval("'радіо"), Value::Symbol("радіо".into()));
}

#[test]
fn implements_mccarthys_seven_primitives() {
    assert_eq!(eval("(quote radio)"), Value::Symbol("radio".into()));
    assert_eq!(eval("(atom 'radio)"), Value::Bool(true));
    assert_eq!(eval("(atom '())"), Value::Bool(true));
    assert_eq!(eval("(atom '(radio antenna))"), Value::Bool(false));
    assert_eq!(eval("(eq 'radio 'radio)"), Value::Bool(true));
    assert_eq!(eval("(eq 'radio 'antenna)"), Value::Bool(false));
    assert_eq!(
        eval("(car '(radio antenna))"),
        Value::Symbol("radio".into())
    );
    assert_eq!(
        eval("(cdr '(radio antenna))"),
        Value::list([Value::Symbol("antenna".into())])
    );
    assert_eq!(
        eval("(cons 'radio '(antenna))"),
        Value::list([
            Value::Symbol("radio".into()),
            Value::Symbol("antenna".into())
        ])
    );
    assert_eq!(
        eval("(cond (() 'wrong) (t 'right))"),
        Value::Symbol("right".into())
    );
}

#[test]
fn reports_structured_errors_with_source_spans() {
    let error = eval_program("(car '())", &mut Session::default()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Type);
    assert_eq!((error.span.start, error.span.end), (0, 9));

    let parse_error = parse("(cons 'a").unwrap_err();
    assert_eq!(parse_error.kind, ErrorKind::Parse);
    assert_eq!(parse_error.span.start, 0);
}

#[test]
fn lexical_child_reads_parent_without_mutating_it() {
    let parent = my_lisp::Environment::root();
    let child = parent.child();
    child.define("station", Value::Symbol("UR5ABC".into()));
    assert_eq!(child.get("t"), Some(Value::Bool(true)));
    assert_eq!(parent.get("station"), None);
}

#[test]
fn lambda_captures_lexical_environment_and_keeps_parameters_local() {
    let mut session = Session::default();
    session
        .environment
        .define("station", Value::Symbol("radio".into()));

    let result = eval_program(
        "((lambda (suffix) (cons station suffix)) '(antenna))",
        &mut session,
    )
    .unwrap();

    assert_eq!(
        result.value,
        Value::list([
            Value::Symbol("radio".into()),
            Value::Symbol("antenna".into())
        ])
    );
    assert_eq!(session.environment.get("suffix"), None);
}

#[test]
fn lambda_is_a_first_class_value() {
    assert_eq!(
        eval("((lambda (apply-once) (apply-once 'radio)) (lambda (x) (cons x '())))"),
        Value::list([Value::Symbol("radio".into())])
    );
}

#[test]
fn lambda_reports_invalid_parameters_and_arity() {
    let duplicate = eval_program("(lambda (x x) x)", &mut Session::default()).unwrap_err();
    assert_eq!(duplicate.kind, ErrorKind::InvalidForm);
    assert!(duplicate.message.contains("повторний параметр"));

    let invalid = eval_program("(lambda (1) 1)", &mut Session::default()).unwrap_err();
    assert_eq!(invalid.kind, ErrorKind::InvalidForm);

    let arity = eval_program("((lambda (x) x))", &mut Session::default()).unwrap_err();
    assert_eq!(arity.kind, ErrorKind::Arity);
}
