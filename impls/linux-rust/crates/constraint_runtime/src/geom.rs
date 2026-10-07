//! Shared planar types: affine points/vectors and curve parameters.
//!
//! Affine distinction: [`Vec2`] is a free vector (displacement / direction);
//! [`Point2`] is a location. Prefer `b - a` over constructing components by hand.
//!
//! Curve addressing: [`CurveParam::Normalized`] is the canonical chart \(t \in [0,1]\)
//! (native formula domain affinely mapped). [`CurveParam::Uniform`] is constant-speed
//! / arc-length fraction on \([0,1]\). For line segments the two coincide.
//!
//! Scalar chart parameters: [`Angle`] (radians) and [`Length`] (signed State DOF).
//! State packing of all of these lives in [`crate::geometry::StateAtom`].
//! Formula Jacobians write into [`crate::matrix::JacView`] (see expressions /
//! geometry kinds).

use std::ops::{Add, Mul, Neg, Sub};

/// Oriented angle in radians (State DOF; no wrap at the type level).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Angle(f64);

impl Angle {
    pub fn new(radians: f64) -> Self {
        Self(radians)
    }

    pub fn radians(self) -> f64 {
        self.0
    }
}

/// Length parameter as a signed State DOF (geometric non-negativity is separate).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length(f64);

impl Length {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// Free vector in the plane (displacement, direction, difference of points).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    /// Component index of \(x\) within a packed [`Vec2`] atom.
    pub const COMP_X: usize = 0;
    /// Component index of \(y\) within a packed [`Vec2`] atom.
    pub const COMP_Y: usize = 1;

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

    /// Rightward planar rotate \((y, -x)\) (CW 90°).
    ///
    /// Gradient of `u.cross(v)` w.r.t. `u` is `v.right_normal()`; w.r.t. `v` is
    /// `u.left_normal()`.
    pub fn right_normal(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
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
    /// Component index of \(x\) within a packed [`Point2`] atom.
    pub const COMP_X: usize = 0;
    /// Component index of \(y\) within a packed [`Point2`] atom.
    pub const COMP_Y: usize = 1;

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

#[cfg(test)]
#[path = "tests_geom.rs"]
mod tests_geom;
