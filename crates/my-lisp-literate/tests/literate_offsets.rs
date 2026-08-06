use my_lisp::Value;
use my_lisp_literate::SourceMode;

#[test]
fn test_literate_offsets() {
    let source = r#"
# Literate Test

This is a test document.

```my-lisp
(def foo (lambda (x)
  (+ x 1)))
```

Some more text.

```my-lisp
(foo 41)
```
"#;

    let mut session = my_lisp::Session::default();
    let result = my_lisp_literate::eval_literate(source, SourceMode::Literate, &mut session).expect("should evaluate successfully");
    
    assert_eq!(result.0.value, Value::Number(42.0));
}

#[test]
fn test_fallback_no_markdown() {
    let source = "(+ 10 20)";
    let mut session = my_lisp::Session::default();
    let result = my_lisp_literate::eval_literate(source, SourceMode::PureLisp, &mut session).expect("should evaluate fallback");
    assert_eq!(result.0.value, Value::Number(30.0));
}

#[test]
fn test_error_offsets_remapped() {
    let source = r#"
# Error Test

```my-lisp
(+ 1 2)
```

Now an error:

```my-lisp
(foo-bar 42)
```
"#;

    let mut session = my_lisp::Session::default();
    let error = my_lisp_literate::eval_literate(source, SourceMode::Literate, &mut session).expect_err("should fail on unknown symbol");
    
    // Check if the span matches the original source
    let error_text = &source[error.span.start..error.span.end];
    assert_eq!(error_text, "foo-bar");
}
#[test]
fn test_literate_evaluation() {
    let source = "# Literate Lisp\n\nThis is a literate program.\n\n```my-lisp\n(def x 10)\n(* x 2)\n```\n\nIt ignores non-code blocks.";
    let mut session = my_lisp::Session::default();
    let (res, _) = my_lisp_literate::eval_literate(source, SourceMode::Literate, &mut session).unwrap();
    assert_eq!(res.value.to_string(), "20");
}
