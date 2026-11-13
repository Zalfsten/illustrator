use super::*;

fn value(src: &str, var: &str, x: f64) -> f64 {
    let nodes = parse(src, var).unwrap_or_else(|e| panic!("{src}: {e}"));
    eval::<f64>(&nodes, x, &mut Vec::new()).0
}

fn fails(src: &str) -> ParseError {
    parse(src, "x").expect_err(src)
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-15 * b.abs().max(1.0)
}

#[test]
fn spec_examples_evaluate() {
    assert!(close(value("x .* sin(x) + 2^-1", "x", 1.0), 1f64.sin() + 0.5));
    assert!(close(value(".5 + 5. + 1e-3 + 1.5E+3", "x", 0.0), 1505.501));
    assert_eq!(value("5.*x", "x", 2.0), 10.0);
    assert_eq!(value("t^2", "t", 3.0), 9.0);
    assert_eq!(value("nthroot(x, 3)", "x", -8.0), -2.0);
    assert_eq!(value("x^3", "x", -2.0), -8.0);
    assert!(value("x^(1/3)", "x", -8.0).is_nan());
    assert!(value("(-8)^(1/3)", "x", 0.0).is_nan());
}

#[test]
fn precedence_follows_matlab() {
    assert_eq!(value("2^3^2", "x", 0.0), 64.0);
    assert_eq!(value("-x^2", "x", 3.0), -9.0);
    assert_eq!(value("2^-1", "x", 0.0), 0.5);
    assert_eq!(value("2*-3", "x", 0.0), -6.0);
    assert_eq!(value("2--3", "x", 0.0), 5.0);
    assert_eq!(value("1-2-3", "x", 0.0), -4.0);
    assert_eq!(value("12/3/2", "x", 0.0), 2.0);
    assert_eq!(value("2+3*4^2", "x", 0.0), 50.0);
    assert_eq!(value("-2^-2", "x", 0.0), -0.25);
    assert_eq!(value("(2^-2)^2", "x", 0.0), 0.0625);
    assert_eq!(value("2^(-2)^2", "x", 0.0), 0.0625);
    assert_eq!(value("+x", "x", 4.0), 4.0);
    assert_eq!(value("--x", "x", 4.0), 4.0);
    assert_eq!(value("2.^x./4", "x", 3.0), 2.0);
}

#[test]
fn signed_exponent_followed_by_power_is_rejected() {
    let e = fails("2^-2^2");
    assert_eq!(e.kind, ParseErrorKind::AmbiguousPower);
    assert_eq!(e.span, 4..5);
}

#[test]
fn constant_integer_exponents_fold_into_integer_powers() {
    assert_eq!(parse("x^2", "x").unwrap(), vec![Node::Var, Node::PowInt(0, 2)]);
    assert_eq!(parse("x^(4/2)", "x").unwrap(), vec![Node::Var, Node::PowInt(0, 2)]);
    assert_eq!(parse("x^-1", "x").unwrap(), vec![Node::Var, Node::PowInt(0, -1)]);
    assert_eq!(
        parse("x^0.5", "x").unwrap(),
        vec![Node::Var, Node::Const(0.5), Node::PowReal(0, 1)]
    );
    assert!(matches!(parse("2^x", "x").unwrap()[2], Node::PowReal(0, 1)));
    assert_eq!(parse("nthroot(x, 6/2)", "x").unwrap(), vec![Node::Var, Node::NthRoot(0, 3)]);
}

#[test]
fn rejected_constructs_name_kind_and_position() {
    use ParseErrorKind::*;
    let cases: [(&str, ParseErrorKind, Range<usize>); 16] = [
        ("2x", ImplicitMultiplication, 1..2),
        ("2(x)", ImplicitMultiplication, 1..2),
        ("x y", ImplicitMultiplication, 2..3),
        (
            "e^x",
            RejectedName {
                name: "e".into(),
                hint: Some("write `exp(x)` for the exponential"),
            },
            0..1,
        ),
        (
            "2*i",
            RejectedName {
                name: "i".into(),
                hint: Some("complex numbers are not supported"),
            },
            2..3,
        ),
        (
            "j",
            RejectedName {
                name: "j".into(),
                hint: Some("complex numbers are not supported"),
            },
            0..1,
        ),
        (
            "sign(x)",
            RejectedName {
                name: "sign".into(),
                hint: Some("discontinuous functions are not supported"),
            },
            0..4,
        ),
        ("x'", Unsupported("transpose"), 1..2),
        ("x\\2", Unsupported("left division"), 1..2),
        ("x**2", Unsupported("`**` (write `^`)"), 1..3),
        ("x==1", Unsupported("comparison"), 1..3),
        ("[x]", Unsupported("a matrix or bracket"), 0..1),
        ("gamma(x)", UnknownIdentifier("gamma".into()), 0..5),
        ("x + y", UnknownIdentifier("y".into()), 4..5),
        ("sin + 1", FunctionWithoutCall("sin"), 0..3),
        ("pi()", Unsupported("calling `pi` (write `pi`)"), 0..2),
    ];
    for (src, kind, span) in cases {
        let e = fails(src);
        assert_eq!((e.kind, e.span), (kind, span), "{src}");
    }
}

#[test]
fn arity_and_nthroot_degree() {
    let e = fails("atan2(x)");
    assert_eq!(
        e.kind,
        ParseErrorKind::WrongArity {
            function: "atan2",
            expected: 2,
            found: 1
        }
    );
    assert_eq!(e.span, 0..8);
    assert!(matches!(fails("sin(x, 2)").kind, ParseErrorKind::WrongArity { found: 2, .. }));
    let e = fails("nthroot(x, 1.5)");
    assert_eq!((e.kind, e.span), (ParseErrorKind::NthrootDegree, 11..14));
    assert_eq!(fails("nthroot(x, x)").kind, ParseErrorKind::NthrootDegree);
    assert_eq!(fails("nthroot(x, 0)").kind, ParseErrorKind::NthrootDegree);
}

#[test]
fn structural_errors() {
    let e = fails("sin(x");
    assert_eq!((e.kind, e.span), (ParseErrorKind::MissingCloseParen, 5..5));
    let e = fails("x +");
    assert_eq!((e.kind, e.span), (ParseErrorKind::ExpectedOperand, 3..3));
    let e = fails("x + * 2");
    assert_eq!((e.kind, e.span), (ParseErrorKind::ExpectedOperand, 4..5));
    let e = fails("x)");
    assert_eq!((e.kind, e.span), (ParseErrorKind::UnexpectedToken, 1..2));
    assert_eq!(fails("").kind, ParseErrorKind::ExpectedOperand);
    assert_eq!(fails("sin()").kind, ParseErrorKind::ExpectedOperand);
}

#[test]
fn every_catalog_function_parses() {
    for (name, _) in Func1::ALL {
        assert!(parse(&format!("{name}(x)"), "x").is_ok(), "{name}");
    }
    for src in ["atan2(x, 1)", "hypot(x, 2)", "nthroot(x, -3)"] {
        assert!(parse(src, "x").is_ok(), "{src}");
    }
}
