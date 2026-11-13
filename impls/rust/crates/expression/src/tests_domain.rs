use std::f64::consts::{PI, TAU};

use crate::{DomainCheckError, Expression};

fn check(src: &str, x0: f64, x1: f64) -> Result<(), DomainCheckError> {
    Expression::parse(src, "x").unwrap().check_domain(x0, x1)
}

fn rejected(src: &str, x0: f64, x1: f64) -> ((f64, f64), Option<&'static str>) {
    match check(src, x0, x1) {
        Err(DomainCheckError::NotProven { range, function }) => (range, function),
        other => panic!("{src} on [{x0}, {x1}]: {other:?}"),
    }
}

#[test]
fn smooth_functions_are_accepted() {
    for (src, a, b) in [
        ("x .* sin(x)", 0.0, TAU),
        ("exp(-x^2)", -5.0, 5.0),
        ("tan(x)", -1.0, 1.0),
        ("log(x)", 0.1, 10.0),
        ("x - x", -1.0, 1.0),
        ("atan2(x, 1)", -5.0, 5.0),
        ("sqrt(x)", 1e-3, 1.0),
        ("x^2.5 + nthroot(x, 3)", 0.5, 4.0),
        ("1 ./ (1 + x.^2)", -10.0, 10.0),
        ("asin(x/2)", -1.9, 1.9),
    ] {
        assert_eq!(check(src, a, b), Ok(()), "{src} on [{a}, {b}]");
    }
}

#[test]
fn singularity_at_a_range_end_is_rejected() {
    let ((lo, hi), f) = rejected("sqrt(x)", 0.0, 1.0);
    assert!(lo <= 0.0 && 0.0 <= hi, "[{lo}, {hi}]");
    assert_eq!(f, Some("sqrt"));
}

#[test]
fn poles_corners_and_cuts_are_rejected() {
    let ((lo, hi), f) = rejected("1/x", -1.0, 1.0);
    assert!(lo <= 0.0 && 0.0 <= hi, "[{lo}, {hi}]");
    assert_eq!(f, Some("division"));

    let ((lo, hi), f) = rejected("abs(x)", -1.0, 1.0);
    assert!(lo <= 0.0 && 0.0 <= hi);
    assert_eq!(f, Some("abs"));

    let ((lo, hi), f) = rejected("tan(x)", 1.0, 2.0);
    assert!(lo <= PI / 2.0 && PI / 2.0 <= hi, "[{lo}, {hi}]");
    assert_eq!(f, Some("tan"));

    let ((lo, _), f) = rejected("log(x)", -1.0, 1.0);
    assert!(lo <= 0.0);
    assert_eq!(f, Some("log"));

    let (_, f) = rejected("atan2(x, -1)", -1.0, 1.0);
    assert_eq!(f, Some("atan2"));
}

#[test]
fn a_narrow_pole_is_not_missed() {
    let ((lo, hi), f) = rejected("1/(x - 0.3333)", 0.0, 1.0);
    assert!(lo <= 0.3333 && 0.3333 <= hi, "[{lo}, {hi}]");
    assert_eq!(f, Some("division"));
}

#[test]
fn invalid_ranges() {
    for (a, b) in [(1.0, 1.0), (2.0, 1.0), (f64::NAN, 1.0), (0.0, f64::INFINITY)] {
        assert_eq!(check("x", a, b), Err(DomainCheckError::InvalidRange), "[{a}, {b}]");
    }
}

#[test]
fn overflow_is_rejected_and_named() {
    let (_, f) = rejected("exp(x)", 0.0, 1000.0);
    assert_eq!(f, Some("exp"));
}
