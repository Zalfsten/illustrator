use super::*;

#[test]
fn source_is_kept_verbatim() {
    let e = Expression::parse("x .*  sin( x )", "x").unwrap();
    assert_eq!(e.source(), "x .*  sin( x )");
    assert_eq!(e.variable(), "x");
    assert_eq!(e.eval_with(2.0), 2.0 * 2f64.sin());
}

#[test]
fn equality_compares_source_and_variable() {
    let a = Expression::parse("x*sin(x)", "x").unwrap();
    assert_eq!(a, Expression::parse("x*sin(x)", "x").unwrap());
    assert_ne!(a, Expression::parse("x * sin(x)", "x").unwrap());
    assert_ne!(
        Expression::parse("2", "x").unwrap(),
        Expression::parse("2", "t").unwrap()
    );
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn parse_errors_surface() {
    let e = Expression::parse("2x", "x").unwrap_err();
    assert_eq!(e.kind, ParseErrorKind::ImplicitMultiplication);
    assert_eq!(e.to_string(), "missing operator (write `*` to multiply)");
}

#[test]
#[should_panic(expected = "not an identifier")]
fn variable_must_be_an_identifier() {
    let _ = Expression::parse("1", "2x");
}

#[test]
fn eval_in_reuses_the_scratch_buffer() {
    let e = Expression::parse("x*sin(x) + exp(-x^2)", "x").unwrap();
    let mut scratch = Vec::with_capacity(e.node_count());
    let first = e.eval_in(0.7, &mut scratch).unwrap();
    let ptr = scratch.as_ptr();
    for x in [0.1, 0.2, 0.3] {
        assert_eq!(e.eval_in(x, &mut scratch), e.eval(x));
    }
    assert_eq!(scratch.as_ptr(), ptr, "no reallocation");
    assert_eq!(scratch.capacity(), e.node_count());
    assert_eq!(first, e.eval(0.7).unwrap());
}
