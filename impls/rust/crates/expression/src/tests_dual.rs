use crate::{DomainError, Expression};

type F = fn(f64) -> f64;

/// (source, f, f′, f″, two interior points)
fn cases() -> Vec<(&'static str, F, F, F, [f64; 2])> {
    use std::f64::consts::{LN_10, LN_2};
    vec![
        ("exp(x)", |x| x.exp(), |x| x.exp(), |x| x.exp(), [0.3, -1.2]),
        ("log(x)", |x| x.ln(), |x| 1.0 / x, |x| -1.0 / (x * x), [0.3, 2.5]),
        ("log10(x)", |x| x.log10(), |x| 1.0 / (x * LN_10), |x| -1.0 / (x * x * LN_10), [0.3, 2.5]),
        ("log2(x)", |x| x.log2(), |x| 1.0 / (x * LN_2), |x| -1.0 / (x * x * LN_2), [0.3, 2.5]),
        ("sqrt(x)", |x| x.sqrt(), |x| 0.5 / x.sqrt(), |x| -0.25 / x.powf(1.5), [0.3, 2.5]),
        ("sin(x)", |x| x.sin(), |x| x.cos(), |x| -x.sin(), [0.3, -2.0]),
        ("cos(x)", |x| x.cos(), |x| -x.sin(), |x| -x.cos(), [0.3, -2.0]),
        ("tan(x)", |x| x.tan(), |x| 1.0 / x.cos().powi(2), |x| 2.0 * x.tan() / x.cos().powi(2), [0.3, -1.2]),
        ("asin(x)", |x| x.asin(), |x| 1.0 / (1.0 - x * x).sqrt(), |x| x / (1.0 - x * x).powf(1.5), [0.3, -0.7]),
        ("acos(x)", |x| x.acos(), |x| -1.0 / (1.0 - x * x).sqrt(), |x| -x / (1.0 - x * x).powf(1.5), [0.3, -0.7]),
        ("atan(x)", |x| x.atan(), |x| 1.0 / (1.0 + x * x), |x| -2.0 * x / (1.0 + x * x).powi(2), [0.3, -2.0]),
        ("sinh(x)", |x| x.sinh(), |x| x.cosh(), |x| x.sinh(), [0.3, -2.0]),
        ("cosh(x)", |x| x.cosh(), |x| x.sinh(), |x| x.cosh(), [0.3, -2.0]),
        ("tanh(x)", |x| x.tanh(), |x| 1.0 / x.cosh().powi(2), |x| -2.0 * x.tanh() / x.cosh().powi(2), [0.3, -2.0]),
        ("abs(x)", |x| x.abs(), |x| x.signum(), |_| 0.0, [-0.7, 1.3]),
        ("x^3", |x| x.powi(3), |x| 3.0 * x * x, |x| 6.0 * x, [0.3, -2.0]),
        ("x^-2", |x| x.powi(-2), |x| -2.0 * x.powi(-3), |x| 6.0 * x.powi(-4), [0.3, -2.0]),
        ("x^2.5", |x| x.powf(2.5), |x| 2.5 * x.powf(1.5), |x| 3.75 * x.sqrt(), [0.3, 2.0]),
        ("2^x", |x| 2f64.powf(x), |x| LN_2 * 2f64.powf(x), |x| LN_2 * LN_2 * 2f64.powf(x), [0.3, -2.0]),
        (
            "x^x",
            |x| x.powf(x),
            |x| x.powf(x) * (x.ln() + 1.0),
            |x| x.powf(x) * ((x.ln() + 1.0).powi(2) + 1.0 / x),
            [0.3, 2.0],
        ),
        ("1/x", |x| 1.0 / x, |x| -1.0 / (x * x), |x| 2.0 / x.powi(3), [0.3, -2.0]),
        ("atan2(x, 2)", |x| x.atan2(2.0), |x| 2.0 / (4.0 + x * x), |x| -4.0 * x / (4.0 + x * x).powi(2), [0.3, -2.0]),
        ("atan2(1, x)", |x| 1f64.atan2(x), |x| -1.0 / (1.0 + x * x), |x| 2.0 * x / (1.0 + x * x).powi(2), [0.3, -1.5]),
        ("hypot(x, 2)", |x| x.hypot(2.0), |x| x / x.hypot(2.0), |x| 4.0 / x.hypot(2.0).powi(3), [0.3, -2.0]),
        (
            "nthroot(x, 3)",
            |x| x.cbrt(),
            |x| x.cbrt() / (3.0 * x),
            |x| -2.0 * x.cbrt() / (9.0 * x * x),
            [-8.0, 2.0],
        ),
        ("nthroot(x, -1)", |x| 1.0 / x, |x| -1.0 / (x * x), |x| 2.0 / x.powi(3), [0.3, -2.0]),
    ]
}

fn rel_close(got: f64, want: f64, tol: f64) -> bool {
    (got - want).abs() <= tol * want.abs().max(1.0)
}

#[test]
fn derivatives_match_hand_formulas() {
    for (src, f, f1, f2, points) in cases() {
        let e = Expression::parse(src, "x").unwrap();
        for x in points {
            let j = e.eval(x).unwrap_or_else(|err| panic!("{src} at {x}: {err}"));
            assert!(rel_close(j.value, f(x), 1e-14), "{src} value at {x}: {} vs {}", j.value, f(x));
            assert!(rel_close(j.d1, f1(x), 1e-12), "{src} f' at {x}: {} vs {}", j.d1, f1(x));
            assert!(rel_close(j.d2, f2(x), 1e-12), "{src} f'' at {x}: {} vs {}", j.d2, f2(x));
        }
    }
}

#[test]
fn derivatives_match_central_differences() {
    for (src, _, _, _, points) in cases() {
        let e = Expression::parse(src, "x").unwrap();
        let v = |x: f64| e.eval(x).unwrap().value;
        for x in points {
            let j = e.eval(x).unwrap();
            let h = 1e-5 * x.abs().max(1.0);
            let d1 = (v(x + h) - v(x - h)) / (2.0 * h);
            let h2 = 1e-3 * x.abs().max(1.0);
            let d2 = (v(x + h2) - 2.0 * v(x) + v(x - h2)) / (h2 * h2);
            assert!(rel_close(j.d1, d1, 1e-6), "{src} f' at {x}: {} vs {d1}", j.d1);
            assert!(rel_close(j.d2, d2, 1e-4), "{src} f'' at {x}: {} vs {d2}", j.d2);
        }
    }
}

#[test]
fn product_rule_spec_scenario() {
    let j = Expression::parse("x*sin(x)", "x").unwrap().eval(2.0).unwrap();
    let (s, c) = (2f64.sin(), 2f64.cos());
    assert!(rel_close(j.value, 2.0 * s, 1e-15));
    assert!(rel_close(j.d1, s + 2.0 * c, 1e-15));
    assert!(rel_close(j.d2, 2.0 * c - 2.0 * s, 1e-15));
}

#[test]
fn domain_errors_name_the_function() {
    let err = |src: &str, x: f64| Expression::parse(src, "x").unwrap().eval(x).unwrap_err();
    assert_eq!(err("log(x)", 0.0), DomainError { function: Some("log") });
    assert_eq!(err("1 + sqrt(x)", -1.0), DomainError { function: Some("sqrt") });
    assert_eq!(err("(-8)^(1/3)", 0.0), DomainError { function: Some("power") });
    assert_eq!(err("x^(1/3)", -8.0), DomainError { function: Some("power") });
    assert_eq!(err("sqrt(x)", 0.0), DomainError { function: Some("sqrt") });
    assert_eq!(err("abs(x)", 0.0), DomainError { function: Some("abs") });
    assert_eq!(err("nthroot(x, 2)", -4.0), DomainError { function: Some("nthroot") });
    assert_eq!(err("1/x", 0.0), DomainError { function: Some("division") });
    assert_eq!(err("exp(x)", 1000.0), DomainError { function: Some("exp") });
    assert_eq!(err("x*x", 1e200), DomainError { function: None });
}
