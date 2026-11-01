//! Exact curve shapes: pieces from a fixed, kind-independent vocabulary
//! (geometry-ir: Curves describe their exact shape).
//!
//! Kinds with the curve capability override [`Geometry::shape`]; exports read
//! shapes by identity through
//! [`Document::curve_shape`](crate::document::Document::curve_shape).
//!
//! [`Geometry::shape`]: crate::geometry::Geometry::shape

use crate::geom::{HeadingAngle, Point2, TurnAngle, Vec2};

/// One piece of a curve's exact shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShapePiece {
    /// Straight piece from `start` to `end`.
    Line { start: Point2, end: Point2 },
    /// Circular-arc piece: polar heading `start + s·sweep` (s ∈ [0, 1]) at
    /// distance `radius` (≥ 0) from `center`.
    Arc {
        center: Point2,
        radius: f64,
        start: HeadingAngle,
        sweep: TurnAngle,
    },
}

impl ShapePiece {
    /// Arc length of the piece.
    pub fn length(&self) -> f64 {
        match *self {
            Self::Line { start, end } => (end - start).length(),
            Self::Arc { radius, sweep, .. } => radius * sweep.radians().abs(),
        }
    }

    /// Point at fraction `s` ∈ [0, 1] of the piece (uniform in arc length).
    pub fn point_at(&self, s: f64) -> Point2 {
        match *self {
            Self::Line { start, end } => start + (end - start) * s,
            Self::Arc {
                center,
                radius,
                start,
                sweep,
            } => {
                let a = start.radians() + s * sweep.radians();
                center + Vec2::new(radius * a.cos(), radius * a.sin())
            }
        }
    }

    pub fn start_point(&self) -> Point2 {
        self.point_at(0.0)
    }

    pub fn end_point(&self) -> Point2 {
        self.point_at(1.0)
    }

    /// The part between fractions `s0 ≤ s1` of the piece, as a piece of the
    /// same kind.
    pub fn sub(&self, s0: f64, s1: f64) -> Self {
        match *self {
            Self::Line { .. } => Self::Line {
                start: self.point_at(s0),
                end: self.point_at(s1),
            },
            Self::Arc {
                center,
                radius,
                start,
                sweep,
            } => Self::Arc {
                center,
                radius,
                start: HeadingAngle::new(start.radians() + s0 * sweep.radians()),
                sweep: TurnAngle::new((s1 - s0) * sweep.radians()),
            },
        }
    }
}

#[cfg(test)]
#[path = "tests_shape.rs"]
mod tests_shape;
