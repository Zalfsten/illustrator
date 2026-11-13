use super::*;
use crate::dual::Dual2;
use crate::Expression;

/// Deterministic pseudo-random numbers in [0, 1) (no dependency).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A sub-interval of (a, b), often narrow.
    fn interval(&mut self, a: f64, b: f64) -> Interval {
        let lo = a + (b - a) * self.next();
        let hi = lo + (b - lo) * self.next().powi(4);
        Interval::new(lo, hi)
    }
}

fn samples(i: Interval) -> impl Iterator<Item = f64> {
    (0..20).map(move |k| {
        if k == 19 {
            i.hi()
        } else {
            i.lo() + (i.hi() - i.lo()) * f64::from(k) / 19.0
        }
    })
}

type Unary = (&'static str, fn(Interval) -> Interval, fn(f64) -> f64, f64, f64);

fn unary_cases() -> Vec<Unary> {
    vec![
        ("exp", Number::exp, f64::exp, -30.0, 30.0),
        ("log", Number::log, f64::ln, 1e-6, 1e6),
        ("log10", Number::log10, f64::log10, 1e-6, 1e6),
        ("log2", Number::log2, f64::log2, 1e-6, 1e6),
        ("sqrt", Number::sqrt, f64::sqrt, 1e-9, 1e3),
        ("sin", Number::sin, f64::sin, -50.0, 50.0),
        ("cos", Number::cos, f64::cos, -50.0, 50.0),
        ("tan", Number::tan, f64::tan, -1.5, 1.5),
        ("asin", Number::asin, f64::asin, -0.999, 0.999),
        ("acos", Number::acos, f64::acos, -0.999, 0.999),
        ("atan", Number::atan, f64::atan, -100.0, 100.0),
        ("sinh", Number::sinh, f64::sinh, -20.0, 20.0),
        ("cosh", Number::cosh, f64::cosh, -20.0, 20.0),
        ("tanh", Number::tanh, f64::tanh, -20.0, 20.0),
        ("abs+", Number::abs, f64::abs, 0.01, 100.0),
        ("abs-", Number::abs, f64::abs, -100.0, -0.01),
        ("neg", Number::neg, |x| -x, -100.0, 100.0),
        ("x^3", |i| i.powi(3), |x| x.powi(3), -10.0, 10.0),
        ("x^4", |i| i.powi(4), |x| x.powi(4), -10.0, 10.0),
        ("x^-2 +", |i| i.powi(-2), |x| x.powi(-2), 0.1, 10.0),
        ("x^-3 -", |i| i.powi(-3), |x| x.powi(-3), -10.0, -0.1),
        ("nthroot3 +", |i| i.nthroot(3), |x| nthroot_f64(x, 3), 0.01, 100.0),
        ("nthroot3 -", |i| i.nthroot(3), |x| nthroot_f64(x, 3), -100.0, -0.01),
        ("nthroot4", |i| i.nthroot(4), |x| nthroot_f64(x, 4), 0.01, 100.0),
        ("nthroot-3", |i| i.nthroot(-3), |x| nthroot_f64(x, -3), 0.01, 100.0),
    ]
}

#[test]
fn unary_functions_enclose_point_values() {
    let mut rng = Lcg(42);
    for (name, f, g, a, b) in unary_cases() {
        for _ in 0..200 {
            let i = rng.interval(a, b);
            let r = f(i);
            assert!(!r.is_bad(), "{name} on {i:?} is poison");
            for x in samples(i) {
                assert!(r.contains(g(x)), "{name}({x}) = {} not in {r:?} for {i:?}", g(x));
            }
        }
    }
}

type Binary = (&'static str, fn(Interval, Interval) -> Interval, fn(f64, f64) -> f64, [f64; 4]);

#[test]
fn binary_operations_enclose_point_values() {
    let cases: Vec<Binary> = vec![
        ("add", Number::add, |a, b| a + b, [-5.0, 5.0, -5.0, 5.0]),
        ("sub", Number::sub, |a, b| a - b, [-5.0, 5.0, -5.0, 5.0]),
        ("mul", Number::mul, |a, b| a * b, [-5.0, 5.0, -5.0, 5.0]),
        ("div", Number::div, |a, b| a / b, [-5.0, 5.0, 0.1, 5.0]),
        ("powf", Number::powf, f64::powf, [0.1, 5.0, -3.0, 3.0]),
        ("atan2 x>0", Number::atan2, f64::atan2, [-5.0, 5.0, 0.01, 5.0]),
        ("atan2 y>0", Number::atan2, f64::atan2, [0.01, 5.0, -5.0, 5.0]),
        ("atan2 y<0", Number::atan2, f64::atan2, [-5.0, -0.01, -5.0, 5.0]),
        ("hypot", Number::hypot, f64::hypot, [0.01, 5.0, -5.0, 5.0]),
    ];
    let mut rng = Lcg(7);
    for (name, f, g, [a0, a1, b0, b1]) in cases {
        for _ in 0..200 {
            let (p, q) = (rng.interval(a0, a1), rng.interval(b0, b1));
            let r = f(p, q);
            assert!(!r.is_bad(), "{name} on {p:?}, {q:?} is poison");
            for x in samples(p).step_by(4) {
                for y in samples(q).step_by(4) {
                    assert!(r.contains(g(x, y)), "{name}({x}, {y}) not in {r:?}");
                }
            }
        }
    }
}

#[test]
fn derivative_bounds_enclose_point_jets() {
    let mut rng = Lcg(3);
    for (src, a, b) in [
        ("x .* sin(x)", -6.0, 6.0),
        ("atan2(x, 2) + sqrt(x + 7)", -5.0, 5.0),
        ("exp(-x^2) ./ (1 + x^2)", -3.0, 3.0),
        ("x^x", 0.2, 3.0),
    ] {
        let e = Expression::parse(src, "x").unwrap();
        for _ in 0..50 {
            let i = rng.interval(a, b);
            let r: Dual2<Interval> = e.eval_with(Dual2::variable(i));
            assert!(!r.is_bad(), "{src} on {i:?}");
            for x in samples(i) {
                let j = e.eval(x).unwrap();
                assert!(r.v.contains(j.value), "{src} value at {x}");
                assert!(r.d1.contains(j.d1), "{src} f' at {x}");
                assert!(r.d2.contains(j.d2), "{src} f'' at {x}");
            }
        }
    }
}

#[test]
fn domain_guards_give_poison() {
    let i = Interval::new;
    let poison = [
        ("sqrt [0,1]", i(0.0, 1.0).sqrt()),
        ("log [-1,1]", i(-1.0, 1.0).log()),
        ("log10 [0,1]", i(0.0, 1.0).log10()),
        ("asin [0,1]", i(0.0, 1.0).asin()),
        ("acos [-1,0]", i(-1.0, 0.0).acos()),
        ("tan [1,2]", i(1.0, 2.0).tan()),
        ("tan wide", i(-10.0, -6.0).tan()),
        ("abs [-1,1]", i(-1.0, 1.0).abs()),
        ("abs [0,1]", i(0.0, 1.0).abs()),
        ("1/[-1,1]", Interval::constant(1.0).div(i(-1.0, 1.0))),
        ("atan2 cut", i(-1.0, 1.0).atan2(i(-1.0, 0.0))),
        ("atan2 origin", i(0.0, 1.0).atan2(i(0.0, 1.0))),
        ("hypot origin", i(-1.0, 1.0).hypot(i(-1.0, 1.0))),
        ("powf base [0,1]", i(0.0, 1.0).powf(Interval::constant(0.5))),
        ("powi -1 [-1,1]", i(-1.0, 1.0).powi(-1)),
        ("nthroot even [-1,1]", i(-1.0, 1.0).nthroot(2)),
        ("nthroot even negative", i(-2.0, -1.0).nthroot(4)),
        ("nthroot 3 [0,1]", i(0.0, 1.0).nthroot(3)),
    ];
    for (name, r) in poison {
        assert!(r.is_bad(), "{name}: {r:?}");
    }
    // Inside the domains the same functions are fine.
    assert!(!i(0.5, 1.0).sqrt().is_bad());
    assert!(!i(-1.0, 1.0).atan2(i(0.5, 1.0)).is_bad());
    assert!(!i(-1.0, 1.0).powi(2).is_bad());
    assert_eq!(i(-1.0, 2.0).powi(2).lo(), 0.0);
}

#[test]
fn poison_propagates_through_every_operation() {
    let p = Interval::POISON;
    let one = Interval::constant(1.0);
    let results = [
        p.add(one),
        one.sub(p),
        p.mul(one),
        one.div(p),
        p.neg(),
        p.powi(3),
        p.powf(one),
        one.powf(p),
        p.exp(),
        p.log(),
        p.sin(),
        p.cos(),
        p.tan(),
        p.atan(),
        p.sinh(),
        p.cosh(),
        p.tanh(),
        p.abs(),
        p.atan2(one),
        one.hypot(p),
        p.nthroot(3),
    ];
    for (k, r) in results.iter().enumerate() {
        assert!(r.is_bad(), "operation #{k}");
    }
}

#[test]
fn sine_extrema_and_full_periods() {
    let r = Interval::new(1.0, 2.0).sin();
    assert_eq!(r.hi(), 1.0);
    assert!(r.lo() <= 1f64.sin());
    let r = Interval::new(-100.0, -90.0).cos();
    assert_eq!((r.lo(), r.hi()), (-1.0, 1.0));
    let r = Interval::new(0.1, 0.2).sin();
    assert!(r.hi() < 0.2 && r.lo() > 0.09);
}
