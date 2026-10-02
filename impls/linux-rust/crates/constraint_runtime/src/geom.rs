//! Shared planar types: affine points/vectors, curve parameters, formula Jacobians.
//!
//! Affine distinction: [`Vec2`] is a free vector (displacement / direction);
//! [`Point2`] is a location. Prefer `b - a` over constructing components by hand.
//!
//! Curve addressing: [`CurveParam::Normalized`] is the canonical chart \(t \in [0,1]\)
//! (native formula domain affinely mapped). [`CurveParam::Uniform`] is constant-speed
//! / arc-length fraction on \([0,1]\). For line segments the two coincide.
//!
//! [`PointJacobian`] / [`VecJacobian`] are stack \(2 \times n\) carriers for analytical
//! derivatives of point/vector formulas (runtime width \(n\), fixed stack cap).
//! Concrete kinds and opaque expressions fill/return them; Document/kind IR lives
//! under [`crate::geometries`].

use std::ops::{Add, Mul, Neg, Sub};

/// Free vector in the plane (displacement, direction, difference of points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Signed 2D cross product magnitude \(u_x v_y - u_y v_x\).
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn length2(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length2().sqrt()
    }

    /// Leftward planar rotate \((-y, x)\) (CCW 90°).
    pub fn left_normal(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

/// Affine point (location). Representable as a position vector relative to an origin,
/// but not interchangeable with [`Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dist2(self, other: Self) -> f64 {
        (self - other).length2()
    }
}

impl Sub for Point2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add<Vec2> for Point2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Vec2> for Point2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// Parameter addressing a location on a curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CurveParam {
    /// Canonical chart \(t \in [0,1]\) (normalized native domain).
    Normalized(f64),
    /// Constant-speed / arc-length fraction \(u \in [0,1]\).
    Uniform(f64),
}

impl CurveParam {
    /// Chart \(t\) when Normalized ≡ Uniform (line segments today).
    pub fn chart_t(self) -> f64 {
        match self {
            CurveParam::Normalized(t) | CurveParam::Uniform(t) => t,
        }
    }
}

/// Max width (local parameters) of a single point/vector-formula Jacobian.
/// Chart formulas stay small (segment endpoint: 2, on-curve: 4); raise when a
/// kind needs more formula-local DOFs. Erased form: runtime \(n\) ≤ this cap.
pub const MAX_POINT_JACOBIAN_PARAMS: usize = 8;

/// Analytical Jacobian of a point map: \(2 \times n\) stacked as two rows `dx`,
/// `dy` with \(\texttt{dx}[i]=\partial x/\partial p_i\),
/// \(\texttt{dy}[i]=\partial y/\partial p_i\). Stored on the stack; `n` is the
/// formula-local parameter count (same order as the expression's param indices).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointJacobian {
    dx: [f64; MAX_POINT_JACOBIAN_PARAMS],
    dy: [f64; MAX_POINT_JACOBIAN_PARAMS],
    n: usize,
}

impl PointJacobian {
    /// Zero \(2 \times n\) Jacobian (`n` must be ≤ [`MAX_POINT_JACOBIAN_PARAMS`]).
    pub fn zeros(n: usize) -> Self {
        assert!(
            n <= MAX_POINT_JACOBIAN_PARAMS,
            "point Jacobian width {n} exceeds MAX_POINT_JACOBIAN_PARAMS ({MAX_POINT_JACOBIAN_PARAMS})"
        );
        Self {
            dx: [0.0; MAX_POINT_JACOBIAN_PARAMS],
            dy: [0.0; MAX_POINT_JACOBIAN_PARAMS],
            n,
        }
    }

    /// Identity map \((p_0,p_1)\mapsto(x,y)\): \(J = I_2\).
    pub fn endpoint_identity() -> Self {
        let mut j = Self::zeros(2);
        j.dx[0] = 1.0;
        j.dy[1] = 1.0;
        j
    }

    pub fn n(&self) -> usize {
        self.n
    }

    pub fn dx(&self) -> &[f64] {
        &self.dx[..self.n]
    }

    pub fn dy(&self) -> &[f64] {
        &self.dy[..self.n]
    }

    /// Set column `i`: \(\partial(x,y)/\partial p_i = (\partial x, \partial y)\).
    pub(crate) fn set_dx_dy(&mut self, i: usize, dx_i: f64, dy_i: f64) {
        assert!(i < self.n);
        self.dx[i] = dx_i;
        self.dy[i] = dy_i;
    }
}

/// Analytical Jacobian of a free-vector map: same \(2 \times n\) layout as
/// [`PointJacobian`] (rows \(\partial v_x\), \(\partial v_y\)).
pub type VecJacobian = PointJacobian;

#[cfg(test)]
#[path = "tests_geom.rs"]
mod tests_geom;
