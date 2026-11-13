//! Interval arithmetic with outward rounding and poison outside open domains.
//!
//! Bounds are computed in round-to-nearest and then widened (no CPU rounding
//! modes, so this works in wasm): by one ulp for correctly rounded IEEE
//! operations, by four ulps for library functions, by `|n| + 2` ulps for
//! integer powers. An argument interval that is not entirely inside an open
//! domain of the spec yields [`Interval::POISON`], which propagates.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::number::{nthroot_f64, Number};

/// A closed interval `[lo, hi]`, or poison (any non-finite bound).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    lo: f64,
    hi: f64,
}

/// Ulps of widening for library functions whose accuracy is not guaranteed
/// to be correctly rounded.
const LIB_ULPS: f64 = 4.0;

impl Interval {
    /// The whole line: the result of an operation outside its open domain.
    pub const POISON: Self = Self {
        lo: f64::NEG_INFINITY,
        hi: f64::INFINITY,
    };

    /// `[lo, hi]`.
    ///
    /// # Panics
    /// If `lo > hi` or a bound is NaN.
    pub fn new(lo: f64, hi: f64) -> Self {
        assert!(lo <= hi, "invalid interval [{lo}, {hi}]");
        Self { lo, hi }
    }

    pub fn lo(&self) -> f64 {
        self.lo
    }

    pub fn hi(&self) -> f64 {
        self.hi
    }

    /// Whether `x` lies in the interval.
    pub fn contains(&self, x: f64) -> bool {
        self.lo <= x && x <= self.hi
    }

    fn contains_zero(&self) -> bool {
        self.lo <= 0.0 && 0.0 <= self.hi
    }

    /// `[lo, hi]` widened outward by about `ulps` ulps (relative to the
    /// bound, plus as many subnormal steps); NaN gives poison.
    fn widen(lo: f64, hi: f64, ulps: f64) -> Self {
        if lo.is_nan() || hi.is_nan() {
            return Self::POISON;
        }
        let step = |x: f64| x.abs() * ulps * f64::EPSILON + ulps * 5e-324;
        Self {
            lo: (lo - step(lo)).next_down(),
            hi: (hi + step(hi)).next_up(),
        }
    }

    /// Hull of the values, widened.
    fn hull(values: &[f64], ulps: f64) -> Self {
        if values.iter().any(|v| v.is_nan()) {
            return Self::POISON;
        }
        let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        Self::widen(lo, hi, ulps)
    }

    /// Monotone increasing `f`, widened by library ulps.
    fn increasing(self, f: fn(f64) -> f64) -> Self {
        Self::widen(f(self.lo), f(self.hi), LIB_ULPS)
    }

    fn clamp(self, lo: f64, hi: f64) -> Self {
        Self {
            lo: self.lo.max(lo),
            hi: self.hi.min(hi),
        }
    }
}

/// Whether `[lo, hi]` may contain `phase + k·period` for an integer `k`.
/// Errs towards `true`: rounding in `π` and in the division can only make
/// the answer more conservative.
fn may_contain_periodic(lo: f64, hi: f64, phase: f64, period: f64) -> bool {
    let a = (lo - phase) / period;
    let b = (hi - phase) / period;
    let slack = 1e-12 * (1.0 + a.abs().max(b.abs()));
    (b + slack).floor() >= (a - slack).ceil()
}

/// Range of `sin(x + shift)` with maxima at `max_phase + 2kπ`.
fn sin_like(x: Interval, f: fn(f64) -> f64, max_phase: f64) -> Interval {
    if x.hi - x.lo >= TAU {
        return Interval::new(-1.0, 1.0);
    }
    let mut r = Interval::hull(&[f(x.lo), f(x.hi)], LIB_ULPS);
    if may_contain_periodic(x.lo, x.hi, max_phase, TAU) {
        r.hi = 1.0;
    }
    if may_contain_periodic(x.lo, x.hi, max_phase + PI, TAU) {
        r.lo = -1.0;
    }
    r.clamp(-1.0, 1.0)
}

impl Number for Interval {
    fn constant(c: f64) -> Self {
        Self { lo: c, hi: c }
    }

    fn approx(c: f64) -> Self {
        Self::widen(c, c, 1.0)
    }

    fn is_bad(&self) -> bool {
        !(self.lo.is_finite() && self.hi.is_finite())
    }

    fn add(self, o: Self) -> Self {
        if self.is_bad() || o.is_bad() {
            return Self::POISON;
        }
        Self::widen(self.lo + o.lo, self.hi + o.hi, 1.0)
    }

    fn sub(self, o: Self) -> Self {
        if self.is_bad() || o.is_bad() {
            return Self::POISON;
        }
        Self::widen(self.lo - o.hi, self.hi - o.lo, 1.0)
    }

    fn mul(self, o: Self) -> Self {
        if self.is_bad() || o.is_bad() {
            return Self::POISON;
        }
        Self::hull(
            &[self.lo * o.lo, self.lo * o.hi, self.hi * o.lo, self.hi * o.hi],
            1.0,
        )
    }

    fn div(self, o: Self) -> Self {
        if self.is_bad() || o.is_bad() || o.contains_zero() {
            return Self::POISON;
        }
        Self::hull(
            &[self.lo / o.lo, self.lo / o.hi, self.hi / o.lo, self.hi / o.hi],
            1.0,
        )
    }

    fn neg(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        Self {
            lo: -self.hi,
            hi: -self.lo,
        }
    }

    fn powi(self, n: i32) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        if n == 0 {
            return Self::constant(1.0);
        }
        let ulps = f64::from(n.unsigned_abs()) + 2.0;
        let (a, b) = (self.lo.powi(n), self.hi.powi(n));
        if self.contains_zero() {
            if n < 0 {
                return Self::POISON;
            }
            if n % 2 == 0 {
                let w = Self::widen(0.0, a.max(b), ulps);
                return Self { lo: 0.0, hi: w.hi };
            }
        }
        Self::hull(&[a, b], ulps)
    }

    fn powf(self, e: Self) -> Self {
        if self.is_bad() || e.is_bad() || self.lo <= 0.0 {
            return Self::POISON;
        }
        e.mul(self.log()).exp()
    }

    fn exp(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        let r = self.increasing(f64::exp);
        Self {
            lo: r.lo.max(0.0),
            hi: r.hi,
        }
    }

    fn log(self) -> Self {
        if self.is_bad() || self.lo <= 0.0 {
            return Self::POISON;
        }
        self.increasing(f64::ln)
    }

    fn log10(self) -> Self {
        if self.is_bad() || self.lo <= 0.0 {
            return Self::POISON;
        }
        self.increasing(f64::log10)
    }

    fn log2(self) -> Self {
        if self.is_bad() || self.lo <= 0.0 {
            return Self::POISON;
        }
        self.increasing(f64::log2)
    }

    fn sqrt(self) -> Self {
        if self.is_bad() || self.lo <= 0.0 {
            return Self::POISON;
        }
        let r = Self::widen(self.lo.sqrt(), self.hi.sqrt(), 1.0);
        Self {
            lo: r.lo.max(0.0),
            hi: r.hi,
        }
    }

    fn sin(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        sin_like(self, f64::sin, FRAC_PI_2)
    }

    fn cos(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        sin_like(self, f64::cos, 0.0)
    }

    fn tan(self) -> Self {
        if self.is_bad()
            || self.hi - self.lo >= PI
            || may_contain_periodic(self.lo, self.hi, FRAC_PI_2, PI)
        {
            return Self::POISON;
        }
        self.increasing(f64::tan)
    }

    fn asin(self) -> Self {
        if self.is_bad() || self.lo <= -1.0 || self.hi >= 1.0 {
            return Self::POISON;
        }
        self.increasing(f64::asin)
    }

    fn acos(self) -> Self {
        if self.is_bad() || self.lo <= -1.0 || self.hi >= 1.0 {
            return Self::POISON;
        }
        Self::widen(self.hi.acos(), self.lo.acos(), LIB_ULPS)
    }

    fn atan(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        self.increasing(f64::atan)
    }

    fn sinh(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        self.increasing(f64::sinh)
    }

    fn cosh(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        let (a, b) = (self.lo.cosh(), self.hi.cosh());
        if self.contains_zero() {
            let w = Self::widen(1.0, a.max(b), LIB_ULPS);
            return Self { lo: 1.0, hi: w.hi };
        }
        Self::hull(&[a, b], LIB_ULPS)
    }

    fn tanh(self) -> Self {
        if self.is_bad() {
            return Self::POISON;
        }
        self.increasing(f64::tanh).clamp(-1.0, 1.0)
    }

    fn abs(self) -> Self {
        if self.is_bad() || self.contains_zero() {
            return Self::POISON;
        }
        if self.lo > 0.0 {
            self
        } else {
            self.neg()
        }
    }

    /// Continuous on any box that avoids the cut `{y = 0, x ≤ 0}`; its
    /// extremes over such a box are at corners.
    fn atan2(self, x: Self) -> Self {
        let y = self;
        if y.is_bad() || x.is_bad() || (y.contains_zero() && x.lo <= 0.0) {
            return Self::POISON;
        }
        Self::hull(
            &[
                y.lo.atan2(x.lo),
                y.lo.atan2(x.hi),
                y.hi.atan2(x.lo),
                y.hi.atan2(x.hi),
            ],
            LIB_ULPS,
        )
    }

    fn hypot(self, b: Self) -> Self {
        let a = self;
        if a.is_bad() || b.is_bad() || (a.contains_zero() && b.contains_zero()) {
            return Self::POISON;
        }
        let nearest = |i: Self| {
            if i.contains_zero() {
                0.0
            } else {
                i.lo.abs().min(i.hi.abs())
            }
        };
        let farthest = |i: Self| i.lo.abs().max(i.hi.abs());
        Self::widen(
            nearest(a).hypot(nearest(b)),
            farthest(a).hypot(farthest(b)),
            LIB_ULPS,
        )
    }

    fn nthroot(self, n: i32) -> Self {
        match n {
            1 => self,
            -1 => Self::constant(1.0).div(self),
            _ => {
                if self.is_bad() || self.contains_zero() || (n % 2 == 0 && self.lo < 0.0) {
                    return Self::POISON;
                }
                Self::hull(
                    &[nthroot_f64(self.lo, n), nthroot_f64(self.hi, n)],
                    LIB_ULPS,
                )
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_interval.rs"]
mod tests_interval;
