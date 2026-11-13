//! The formula catalog as a number-type trait, and its `f64` implementation.
//!
//! Every implementation applies the open domains of the spec: outside them
//! the result is invalid ([`Number::is_bad`]): NaN or ±∞ for `f64`, poison
//! for intervals. Derivative types build on top of a base implementation.

/// A number type the expression tree can be evaluated in.
pub trait Number: Copy {
    /// An exact constant (a parsed number or `pi` as MATLAB's double).
    fn constant(c: f64) -> Self;
    /// An internal constant that only approximates a real number (`ln 10`);
    /// interval types widen it.
    fn approx(c: f64) -> Self {
        Self::constant(c)
    }
    /// Whether the value is outside every open domain or not finite.
    fn is_bad(&self) -> bool;

    fn add(self, o: Self) -> Self;
    fn sub(self, o: Self) -> Self;
    fn mul(self, o: Self) -> Self;
    /// Requires `o ≠ 0`.
    fn div(self, o: Self) -> Self;
    fn neg(self) -> Self;
    /// Integer power; requires `self ≠ 0` for `n < 0`.
    fn powi(self, n: i32) -> Self;
    /// Real power; requires `self > 0`.
    fn powf(self, e: Self) -> Self;

    fn exp(self) -> Self;
    /// Natural logarithm; requires `self > 0` (also `log10`, `log2`, `sqrt`).
    fn log(self) -> Self;
    fn log10(self) -> Self;
    fn log2(self) -> Self;
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    /// Requires `self` not an odd multiple of π/2.
    fn tan(self) -> Self;
    /// Requires `self ∈ (−1, 1)` (also `acos`).
    fn asin(self) -> Self;
    fn acos(self) -> Self;
    fn atan(self) -> Self;
    fn sinh(self) -> Self;
    fn cosh(self) -> Self;
    fn tanh(self) -> Self;
    /// Requires `self ≠ 0`.
    fn abs(self) -> Self;
    /// `atan2(self, x)`; requires not (`self = 0` and `x ≤ 0`).
    fn atan2(self, x: Self) -> Self;
    /// Requires not both zero.
    fn hypot(self, b: Self) -> Self;
    /// Real `n`-th root, `n ≠ 0`; requires `self ≠ 0` for `|n| ≥ 2` and
    /// `self > 0` for even `n`.
    fn nthroot(self, n: i32) -> Self;
}

/// Real `n`-th root of `x` as MATLAB's `nthroot` (one Newton correction).
pub(crate) fn nthroot_f64(x: f64, n: i32) -> f64 {
    if n % 2 == 0 && x < 0.0 {
        return f64::NAN;
    }
    let nf = f64::from(n);
    let y = x.signum() * x.abs().powf(1.0 / nf);
    if y.is_finite() && y != 0.0 {
        // MATLAB / Octave: y - (y^n - x) / (n y^(n-1)).
        y - (y.powi(n) - x) / (nf * y.powi(n - 1))
    } else {
        y
    }
}

const NAN: f64 = f64::NAN;

impl Number for f64 {
    fn constant(c: f64) -> Self {
        c
    }
    fn is_bad(&self) -> bool {
        !self.is_finite()
    }
    fn add(self, o: Self) -> Self {
        self + o
    }
    fn sub(self, o: Self) -> Self {
        self - o
    }
    fn mul(self, o: Self) -> Self {
        self * o
    }
    fn div(self, o: Self) -> Self {
        if o == 0.0 {
            NAN
        } else {
            self / o
        }
    }
    fn neg(self) -> Self {
        -self
    }
    fn powi(self, n: i32) -> Self {
        if n < 0 && self == 0.0 {
            NAN
        } else {
            f64::powi(self, n)
        }
    }
    fn powf(self, e: Self) -> Self {
        if self > 0.0 {
            f64::powf(self, e)
        } else {
            NAN
        }
    }
    fn exp(self) -> Self {
        f64::exp(self)
    }
    fn log(self) -> Self {
        if self > 0.0 {
            f64::ln(self)
        } else {
            NAN
        }
    }
    fn log10(self) -> Self {
        if self > 0.0 {
            f64::log10(self)
        } else {
            NAN
        }
    }
    fn log2(self) -> Self {
        if self > 0.0 {
            f64::log2(self)
        } else {
            NAN
        }
    }
    fn sqrt(self) -> Self {
        if self > 0.0 {
            f64::sqrt(self)
        } else {
            NAN
        }
    }
    fn sin(self) -> Self {
        f64::sin(self)
    }
    fn cos(self) -> Self {
        f64::cos(self)
    }
    fn tan(self) -> Self {
        f64::tan(self)
    }
    fn asin(self) -> Self {
        if self.abs() < 1.0 {
            f64::asin(self)
        } else {
            NAN
        }
    }
    fn acos(self) -> Self {
        if self.abs() < 1.0 {
            f64::acos(self)
        } else {
            NAN
        }
    }
    fn atan(self) -> Self {
        f64::atan(self)
    }
    fn sinh(self) -> Self {
        f64::sinh(self)
    }
    fn cosh(self) -> Self {
        f64::cosh(self)
    }
    fn tanh(self) -> Self {
        f64::tanh(self)
    }
    fn abs(self) -> Self {
        if self == 0.0 {
            NAN
        } else {
            f64::abs(self)
        }
    }
    fn atan2(self, x: Self) -> Self {
        if self == 0.0 && x <= 0.0 {
            NAN
        } else {
            f64::atan2(self, x)
        }
    }
    fn hypot(self, b: Self) -> Self {
        if self == 0.0 && b == 0.0 {
            NAN
        } else {
            f64::hypot(self, b)
        }
    }
    fn nthroot(self, n: i32) -> Self {
        if n.abs() >= 2 && self == 0.0 {
            NAN
        } else {
            nthroot_f64(self, n)
        }
    }
}

#[cfg(test)]
#[path = "tests_number.rs"]
mod tests_number;
