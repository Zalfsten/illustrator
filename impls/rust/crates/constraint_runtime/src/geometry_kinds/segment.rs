//! Line segment: Vector layout, chart sampling, and formula evaluation.

use std::ops::Range;

use crate::geom::{Length, Point2, Vec2};
use crate::geometry::capabilities::geometry_caps;
use crate::geometry::chart::chart_atoms;
use crate::geometry::{Geometry, LengthFormula, PointFormula, VecFormula};
use crate::geometry_kinds::GeometryKind;
use crate::matrix::DenseMatrix;

/// Line-segment kind (type-level marker; instances live in
/// [`crate::geometry_kinds::GeometryInst`]).
///
/// Layout: [`Point2`] atoms [`Self::start`] / [`Self::end`] via `chart_atoms!`.
/// Full-chart Jacobian columns are formula-local `0..4` (start.x, start.y,
/// end.x, end.y). Endpoint formulas use only their own [`Point2`] slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment;

chart_atoms! {
    Segment {
        start: Point2,
        end: Point2,
    }
}

// Capability factory impls + runtime Caps.
geometry_caps!(Segment: endpoints, line);

impl Geometry for Segment {
    const KIND: GeometryKind = GeometryKind::Segment;
    const NAME_PREFIX: &'static str = "seg";

    fn point(f: PointFormula, (a, b): Self::Init) -> Point2 {
        match f {
            PointFormula::OnCurve { param } => {
                let t = param.chart_t();
                Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
            }
            PointFormula::Start => a,
            PointFormula::End => b,
            PointFormula::Center => unreachable!("segment has no circular carrier"),
        }
    }

    fn point_jacobian(f: PointFormula, _c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        match f {
            PointFormula::OnCurve { param } => {
                let t = param.chart_t();
                // (1-t)I | t I
                j.set_scaled(
                    scale,
                    [
                        [1.0 - t, 0.0, t, 0.0],
                        [0.0, 1.0 - t, 0.0, t],
                    ],
                );
            }
            PointFormula::Start | PointFormula::End => {
                j.set_scaled(scale, [[1.0, 0.0], [0.0, 1.0]]);
            }
            PointFormula::Center => unreachable!("segment has no circular carrier"),
        }
    }

    fn point_params(f: PointFormula) -> Range<usize> {
        match f {
            PointFormula::OnCurve { .. } => Self::start.span(Self::end),
            PointFormula::Start => Self::start.range(),
            PointFormula::End => Self::end.range(),
            PointFormula::Center => unreachable!("segment has no circular carrier"),
        }
    }

    fn vec(f: VecFormula, (a, b): Self::Init) -> Vec2 {
        match f {
            // γ'(t) = B - A (constant on a segment); length = segment length.
            // Direction is the same vector, as a t-independent invariant.
            VecFormula::TangentAt { .. } | VecFormula::Direction => b - a,
            VecFormula::NormalAt { .. } => (b - a).left_normal(),
        }
    }

    fn vec_jacobian(f: VecFormula, _c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        match f {
            VecFormula::TangentAt { .. } | VecFormula::Direction => {
                // ∂(B-A)/∂(A,B) = -I | I
                j.set_scaled(
                    scale,
                    [[-1.0, 0.0, 1.0, 0.0], [0.0, -1.0, 0.0, 1.0]],
                );
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of tangent ⇒ left-rotate each J_tan column
                j.set_scaled(
                    scale,
                    [[0.0, 1.0, 0.0, -1.0], [-1.0, 0.0, 1.0, 0.0]],
                );
            }
        }
    }

    fn length(f: LengthFormula, (a, b): Self::Init) -> Length {
        match f {
            LengthFormula::Length => Length::new((b - a).length()),
            LengthFormula::Radius => unreachable!("segment chart has no radius length atom"),
        }
    }

    fn length_jacobian(
        f: LengthFormula,
        (a, b): Self::Init,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        match f {
            LengthFormula::Length => {
                let d = b - a;
                let len = d.length();
                if len > 0.0 {
                    let ux = d.x / len;
                    let uy = d.y / len;
                    j.set_scaled(scale, [[-ux, -uy, ux, uy]]);
                }
            }
            LengthFormula::Radius => unreachable!("segment chart has no radius length atom"),
        }
    }
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
