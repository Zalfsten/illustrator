//! Second-order forward automatic differentiation over any base number type.
//!
//! `Dual2 { v, d1, d2 }` carries a value and its first and second derivative
//! with respect to the variable. With `T = f64` this is the point jet; with
//! `T = Interval` it bounds value and derivatives over a range.

use crate::number::Number;

/// Value with first and second derivative with respect to the variable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dual2<T> {
    pub v: T,
    pub d1: T,
    pub d2: T,
}

impl<T: Number> Dual2<T> {
    /// The variable itself at `x` (derivatives 1 and 0).
    pub fn variable(x: T) -> Self {
        Self {
            v: x,
            d1: T::constant(1.0),
            d2: T::constant(0.0),
        }
    }

    /// Chain rule for `f(self)` given `f(v)`, `f′(v)` and `f″(v)`.
    fn chain(self, f: T, f1: T, f2: T) -> Self {
        Self {
            v: f,
            d1: f1.mul(self.d1),
            d2: f2.mul(self.d1.powi(2)).add(f1.mul(self.d2)),
        }
    }

    fn c(x: f64) -> T {
        T::constant(x)
    }
}

impl<T: Number> Number for Dual2<T> {
    fn constant(c: f64) -> Self {
        Self {
            v: T::constant(c),
            d1: T::constant(0.0),
            d2: T::constant(0.0),
        }
    }

    fn is_bad(&self) -> bool {
        self.v.is_bad() || self.d1.is_bad() || self.d2.is_bad()
    }

    fn add(self, o: Self) -> Self {
        Self {
            v: self.v.add(o.v),
            d1: self.d1.add(o.d1),
            d2: self.d2.add(o.d2),
        }
    }

    fn sub(self, o: Self) -> Self {
        Self {
            v: self.v.sub(o.v),
            d1: self.d1.sub(o.d1),
            d2: self.d2.sub(o.d2),
        }
    }

    /// `(ab)′ = a′b + ab′`, `(ab)″ = a″b + 2a′b′ + ab″`.
    fn mul(self, o: Self) -> Self {
        Self {
            v: self.v.mul(o.v),
            d1: self.d1.mul(o.v).add(self.v.mul(o.d1)),
            d2: self
                .d2
                .mul(o.v)
                .add(Self::c(2.0).mul(self.d1).mul(o.d1))
                .add(self.v.mul(o.d2)),
        }
    }

    /// `q = a/b`, `q′ = (a′ − q b′)/b`, `q″ = (a″ − 2 q′ b′ − q b″)/b`.
    fn div(self, o: Self) -> Self {
        let q = self.v.div(o.v);
        let q1 = self.d1.sub(q.mul(o.d1)).div(o.v);
        let q2 = self
            .d2
            .sub(Self::c(2.0).mul(q1).mul(o.d1))
            .sub(q.mul(o.d2))
            .div(o.v);
        Self { v: q, d1: q1, d2: q2 }
    }

    fn neg(self) -> Self {
        Self {
            v: self.v.neg(),
            d1: self.d1.neg(),
            d2: self.d2.neg(),
        }
    }

    fn powi(self, n: i32) -> Self {
        match n {
            0 => Self::constant(1.0),
            1 => self,
            _ => {
                let nf = f64::from(n);
                let f1 = Self::c(nf).mul(self.v.powi(n - 1));
                let f2 = Self::c(nf * (nf - 1.0)).mul(self.v.powi(n - 2));
                self.chain(self.v.powi(n), f1, f2)
            }
        }
    }

    /// `a^b = exp(b ln a)`: with `g = b ln a`, `f′ = f g′`, `f″ = f (g″ + g′²)`.
    fn powf(self, e: Self) -> Self {
        let f = self.v.powf(e.v);
        let g = e.mul(self.log());
        Self {
            v: f,
            d1: f.mul(g.d1),
            d2: f.mul(g.d2.add(g.d1.powi(2))),
        }
    }

    fn exp(self) -> Self {
        let e = self.v.exp();
        self.chain(e, e, e)
    }

    fn log(self) -> Self {
        let r = Self::c(1.0).div(self.v);
        self.chain(self.v.log(), r, r.powi(2).neg())
    }

    fn log10(self) -> Self {
        let r = T::approx(std::f64::consts::LOG10_E).div(self.v);
        let f2 = r.div(self.v).neg();
        self.chain(self.v.log10(), r, f2)
    }

    fn log2(self) -> Self {
        let r = T::approx(std::f64::consts::LOG2_E).div(self.v);
        let f2 = r.div(self.v).neg();
        self.chain(self.v.log2(), r, f2)
    }

    /// `s′ = 1/(2s)`, `s″ = −1/(4 s³)`.
    fn sqrt(self) -> Self {
        let s = self.v.sqrt();
        let f1 = Self::c(0.5).div(s);
        let f2 = Self::c(-0.25).div(s.powi(3));
        self.chain(s, f1, f2)
    }

    fn sin(self) -> Self {
        let (s, c) = (self.v.sin(), self.v.cos());
        self.chain(s, c, s.neg())
    }

    fn cos(self) -> Self {
        let (s, c) = (self.v.sin(), self.v.cos());
        self.chain(c, s.neg(), c.neg())
    }

    /// `t′ = 1 + t²`, `t″ = 2t(1 + t²)`.
    fn tan(self) -> Self {
        let t = self.v.tan();
        let f1 = Self::c(1.0).add(t.powi(2));
        let f2 = Self::c(2.0).mul(t).mul(f1);
        self.chain(t, f1, f2)
    }

    /// `asin′ = 1/√(1−v²)`, `asin″ = v/(1−v²)^{3/2}`.
    fn asin(self) -> Self {
        let w = Self::c(1.0).sub(self.v.powi(2));
        let r = Self::c(1.0).div(w.sqrt());
        let f2 = self.v.mul(r.powi(3));
        self.chain(self.v.asin(), r, f2)
    }

    fn acos(self) -> Self {
        let w = Self::c(1.0).sub(self.v.powi(2));
        let r = Self::c(1.0).div(w.sqrt());
        let f2 = self.v.mul(r.powi(3));
        self.chain(self.v.acos(), r.neg(), f2.neg())
    }

    /// `atan′ = 1/(1+v²)`, `atan″ = −2v/(1+v²)²`.
    fn atan(self) -> Self {
        let r = Self::c(1.0).div(Self::c(1.0).add(self.v.powi(2)));
        let f2 = Self::c(-2.0).mul(self.v).mul(r.powi(2));
        self.chain(self.v.atan(), r, f2)
    }

    fn sinh(self) -> Self {
        let (s, c) = (self.v.sinh(), self.v.cosh());
        self.chain(s, c, s)
    }

    fn cosh(self) -> Self {
        let (s, c) = (self.v.sinh(), self.v.cosh());
        self.chain(c, s, c)
    }

    /// `t′ = 1 − t²`, `t″ = −2t(1 − t²)`.
    fn tanh(self) -> Self {
        let t = self.v.tanh();
        let f1 = Self::c(1.0).sub(t.powi(2));
        let f2 = Self::c(-2.0).mul(t).mul(f1);
        self.chain(t, f1, f2)
    }

    fn abs(self) -> Self {
        let a = self.v.abs();
        self.chain(a, self.v.div(a), Self::c(0.0))
    }

    /// With `r = x² + y²`, `n = x y′ − y x′`: `θ′ = n/r`,
    /// `θ″ = (n′ r − n r′)/r²` where `n′ = x y″ − y x″` and `r′ = 2(x x′ + y y′)`.
    fn atan2(self, x: Self) -> Self {
        let y = self;
        let theta = y.v.atan2(x.v);
        let r = x.v.powi(2).add(y.v.powi(2));
        let n = x.v.mul(y.d1).sub(y.v.mul(x.d1));
        let n1 = x.v.mul(y.d2).sub(y.v.mul(x.d2));
        let r1 = Self::c(2.0).mul(x.v.mul(x.d1).add(y.v.mul(y.d1)));
        Self {
            v: theta,
            d1: n.div(r),
            d2: n1.mul(r).sub(n.mul(r1)).div(r.powi(2)),
        }
    }

    /// `h′ = (a a′ + b b′)/h`, `h″ = (a′² + a a″ + b′² + b b″ − h′²)/h`.
    fn hypot(self, b: Self) -> Self {
        let a = self;
        let h = a.v.hypot(b.v);
        let h1 = a.v.mul(a.d1).add(b.v.mul(b.d1)).div(h);
        let h2 = a
            .d1
            .powi(2)
            .add(a.v.mul(a.d2))
            .add(b.d1.powi(2))
            .add(b.v.mul(b.d2))
            .sub(h1.powi(2))
            .div(h);
        Self { v: h, d1: h1, d2: h2 }
    }

    /// `r = x^{1/n}`: `r′ = r/(n x)`, `r″ = r′ (1/n − 1)/x`.
    fn nthroot(self, n: i32) -> Self {
        match n {
            1 => self,
            -1 => Self::constant(1.0).div(self),
            _ => {
                let r = self.v.nthroot(n);
                let f1 = r.div(Self::c(f64::from(n)).mul(self.v));
                let f2 = f1.mul(T::approx(1.0 / f64::from(n) - 1.0)).div(self.v);
                self.chain(r, f1, f2)
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_dual.rs"]
mod tests_dual;
