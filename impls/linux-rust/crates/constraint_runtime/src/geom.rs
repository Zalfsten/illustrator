//! Parameterized geometries: local params are references into the global state vector.
//!
//! Affine distinction: [`Vec2`] is a free vector (displacement / direction);
//! [`Point2`] is a location. Prefer `b - a` over constructing components by hand.

use crate::connectivity::Connectivity;
use std::ops::{Add, Mul, Neg, Sub};

pub const SEGMENT_STRIDE: usize = 4;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SegmentId(pub usize);

/// Line segment: endpoint chart, parameters live at `p[param_offset..param_offset+4]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub id: SegmentId,
    pub param_offset: usize,
}

impl Segment {
    pub fn connectivity(&self) -> Connectivity {
        let o = self.param_offset;
        Connectivity::new(vec![o, o + 1, o + 2, o + 3])
    }

    pub fn start(&self, p: &[f64]) -> Point2 {
        Point2::new(p[self.param_offset], p[self.param_offset + 1])
    }

    pub fn end(&self, p: &[f64]) -> Point2 {
        Point2::new(p[self.param_offset + 2], p[self.param_offset + 3])
    }

    /// Free direction vector from start to end (`end - start`).
    pub fn direction(&self, p: &[f64]) -> Vec2 {
        self.end(p) - self.start(p)
    }

    pub fn endpoints(&self) -> (PointExpr, PointExpr) {
        (
            PointExpr::SegmentEnd {
                segment: self.id,
                end: SegmentEndKind::Start,
            },
            PointExpr::SegmentEnd {
                segment: self.id,
                end: SegmentEndKind::End,
            },
        )
    }

    pub fn endpoint_param_indices(&self, end: SegmentEndKind) -> [usize; 2] {
        match end {
            SegmentEndKind::Start => [self.param_offset, self.param_offset + 1],
            SegmentEndKind::End => [self.param_offset + 2, self.param_offset + 3],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SegmentEndKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointExpr {
    SegmentEnd {
        segment: SegmentId,
        end: SegmentEndKind,
    },
}

impl PointExpr {
    pub fn evaluate(&self, segments: &[Segment], p: &[f64]) -> Point2 {
        match *self {
            PointExpr::SegmentEnd { segment, end } => {
                let s = segments
                    .iter()
                    .find(|s| s.id == segment)
                    .expect("segment for PointExpr");
                match end {
                    SegmentEndKind::Start => s.start(p),
                    SegmentEndKind::End => s.end(p),
                }
            }
        }
    }

    pub fn param_indices(&self, segments: &[Segment]) -> [usize; 2] {
        match *self {
            PointExpr::SegmentEnd { segment, end } => {
                let s = segments
                    .iter()
                    .find(|s| s.id == segment)
                    .expect("segment for PointExpr");
                s.endpoint_param_indices(end)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_state_vector_changes_endpoints() {
        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let mut p = vec![0.0, 0.0, 1.0, 0.0];
        assert_eq!(seg.end(&p), Point2::new(1.0, 0.0));
        p[2] = 3.0;
        p[3] = 4.0;
        assert_eq!(seg.end(&p), Point2::new(3.0, 4.0));
        assert_eq!(seg.connectivity().global_indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn point_minus_point_is_vec_direction_is_vec() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(4.0, 6.0);
        let v: Vec2 = b - a;
        assert_eq!(v, Vec2::new(3.0, 4.0));
        assert_eq!(a + v, b);

        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let p = vec![0.0, 0.0, 1.0, 0.0];
        let d: Vec2 = seg.direction(&p);
        assert_eq!(d, Vec2::new(1.0, 0.0));
        assert_eq!(d.cross(Vec2::new(0.0, 1.0)), 1.0);
        assert_eq!(d.dot(Vec2::new(0.0, 1.0)), 0.0);
    }
}
