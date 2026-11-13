use super::*;
use crate::parser::parse;
use crate::tree::eval;

#[test]
fn catalog_values_match_std_at_interior_points() {
    let x = 0.37_f64;
    let cases: [(f64, f64); 15] = [
        (Number::exp(x), x.exp()),
        (Number::log(x), x.ln()),
        (Number::log10(x), x.log10()),
        (Number::log2(x), x.log2()),
        (Number::sqrt(x), x.sqrt()),
        (Number::sin(x), x.sin()),
        (Number::cos(x), x.cos()),
        (Number::tan(x), x.tan()),
        (Number::asin(x), x.asin()),
        (Number::acos(x), x.acos()),
        (Number::atan(x), x.atan()),
        (Number::sinh(x), x.sinh()),
        (Number::cosh(x), x.cosh()),
        (Number::tanh(x), x.tanh()),
        (Number::abs(-x), x),
    ];
    for (i, (got, want)) in cases.into_iter().enumerate() {
        assert_eq!(got, want, "function #{i}");
    }
    assert_eq!(Number::atan2(x, -2.0), x.atan2(-2.0));
    assert_eq!(Number::hypot(x, 2.0), x.hypot(2.0));
}

#[test]
fn operators_and_powers() {
    assert_eq!(Number::add(1.5, 2.0), 3.5);
    assert_eq!(Number::sub(1.5, 2.0), -0.5);
    assert_eq!(Number::mul(1.5, 2.0), 3.0);
    assert_eq!(Number::div(1.5, 2.0), 0.75);
    assert_eq!(Number::neg(1.5), -1.5);
    assert_eq!(Number::powi(-2.0, 3), -8.0);
    assert_eq!(Number::powi(-2.0, -2), 0.25);
    assert_eq!(Number::powf(2.0, 0.5), 2f64.sqrt());
    assert!(Number::powf(-8.0, 1.0 / 3.0).is_nan());
}

#[test]
fn nthroot_gives_real_roots() {
    assert_eq!(Number::nthroot(-8.0, 3), -2.0);
    assert_eq!(Number::nthroot(27.0, 3), 3.0);
    assert_eq!(Number::nthroot(16.0, 4), 2.0);
    assert_eq!(Number::nthroot(8.0, -3), 0.5);
    assert_eq!(Number::nthroot(-5.0, 1), -5.0);
    assert!(Number::nthroot(-4.0, 2).is_nan());
}

#[test]
fn open_domain_boundaries_are_bad() {
    let bad = [
        Number::log(0.0),
        Number::log10(-1.0),
        Number::log2(0.0),
        Number::sqrt(0.0),
        Number::sqrt(-1.0),
        Number::asin(1.0),
        Number::acos(-1.0),
        Number::abs(0.0),
        Number::div(1.0, 0.0),
        Number::powi(0.0, -1),
        Number::powf(0.0, 2.5),
        Number::atan2(0.0, -1.0),
        Number::atan2(0.0, 0.0),
        Number::hypot(0.0, 0.0),
        Number::nthroot(0.0, 3),
    ];
    for (i, v) in bad.iter().enumerate() {
        assert!(v.is_bad(), "case #{i}: {v}");
    }
    assert!(!Number::nthroot(0.0, 1).is_bad());
    assert!(!Number::atan2(0.0, 1.0).is_bad());
    assert!(!Number::powi(0.0, 2).is_bad());
}

#[test]
fn evaluator_reports_the_first_bad_node() {
    let nodes = parse("1 + log(x) * sqrt(x)", "x").unwrap();
    let (v, bad) = eval::<f64>(&nodes, -1.0, &mut Vec::new());
    assert!(v.is_bad());
    let i = bad.unwrap();
    assert_eq!(nodes[i].name(), Some("log"));
    let (v, bad) = eval::<f64>(&nodes, 4.0, &mut Vec::new());
    assert_eq!((v, bad), (1.0 + 4f64.ln() * 2.0, None));
}
