//! Parameterized geometries: local params are references into the global state vector.
//!
//! Affine distinction: [`Vec2`] is a free vector (displacement / direction);
//! [`Point2`] is a location. Prefer `b - a` over constructing components by hand.
//!
//! Curve addressing: [`CurveParam::Normalized`] is the canonical chart \(t \in [0,1]\)
//! (native formula domain affinely mapped). [`CurveParam::Uniform`] is constant-speed
//! / arc-length fraction on \([0,1]\). For line segments the two coincide.
//!
//! Concrete kinds (layout + formulas) live under [`crate::geometries`].

use crate::connectivity::Connectivity;
use crate::geometries::GeometryBody;
use std::ops::{Add, Mul, Neg, Range, Sub};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryId(pub usize);

/// Presentation role: result figure vs construction helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GeometryRole {
    #[default]
    Result,
    Construction,
}

/// Document geometry: shared identity, role, State block offset, plus kind-specific body.
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub id: GeometryId,
    pub role: GeometryRole,
    /// Start index of this geometry's contiguous parameter block in State.
    pub param_offset: usize,
    pub body: GeometryBody,
}

impl Geometry {
    pub fn id(&self) -> GeometryId {
        self.id
    }

    pub fn role(&self) -> GeometryRole {
        self.role
    }

    pub fn set_role(&mut self, role: GeometryRole) {
        self.role = role;
    }

    pub fn parameter_count(&self) -> usize {
        self.body.parameter_count()
    }

    /// Contiguous State index window for this geometry's parameters.
    pub fn param_block(&self) -> Range<usize> {
        let o = self.param_offset;
        o..o + self.parameter_count()
    }

    pub fn connectivity(&self) -> Connectivity {
        let block = self.param_block();
        Connectivity::new(block.collect())
    }

    /// After a contiguous block was removed from State, shift this geometry's
    /// local→global indices that lay after the hole.
    pub fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
        if self.param_offset > removed_start {
            self.param_offset -= removed_len;
        }
    }
}

#[cfg(test)]
#[path = "tests_geom.rs"]
mod tests_geom;
