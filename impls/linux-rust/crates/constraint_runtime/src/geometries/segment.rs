//! Line segment: State layout, chart sampling, and point/vector formulas.

use std::ops::Range;

use super::{Chart, PointExpr, VecExpr};
use super::shell::{geometry_shell_instance, GeometryId, GeometryShell};
use crate::geom::{CurveParam, Point2, PointJacobian, Vec2, VecJacobian};

/// Line-segment instance: document shell plus segment chart formulas.
///
/// Parameters live at `p[shell.param_offset..shell.param_offset+PARAMETER_COUNT]`.
/// Shell fields via [`super::AsShell`] (from [`geometry_shell_instance!`]); Document
/// ops via the blanket [`super::Geometry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment {
    pub shell: GeometryShell,
}

geometry_shell_instance!(Segment);

impl Chart for Segment {
    type Init = (Point2, Point2);

    /// Four endpoint scalars: `x1,y1,x2,y2`.
    const PARAMETER_COUNT: usize = 4;

    fn pack((a, b): Self::Init) -> impl AsRef<[f64]> {
        [a.x, a.y, b.x, b.y]
    }

    fn unpack(params: &[f64]) -> Self::Init {
        debug_assert_eq!(params.len(), Self::PARAMETER_COUNT);
        (
            Point2::new(params[0], params[1]),
            Point2::new(params[2], params[3]),
        )
    }
}

impl Segment {
    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        (id, PointFormula::StartPoint).into()
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        (id, PointFormula::EndPoint).into()
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        (id, PointFormula::OnCurve { param }).into()
    }

    /// Free-vector formula: segment direction (`end - start`).
    pub fn direction_expr(id: GeometryId) -> VecExpr {
        (id, VecFormula::Direction).into()
    }

    /// Free-vector formula: curve tangent at `param`.
    pub fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        (id, VecFormula::TangentAt { param }).into()
    }

    /// Free-vector formula: curve normal at `param`.
    pub fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        (id, VecFormula::NormalAt { param }).into()
    }
}

/// Segment-owned point formulas (private to the crate; wrapped by [`PointExpr`]).
/// Geometry identity lives on the outer expr; formulas are pure chart ops.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PointFormula {
    StartPoint,
    EndPoint,
    OnCurve { param: CurveParam },
}

impl PointFormula {
    /// Evaluate against already-unpacked chart endpoints `(start, end)`.
    pub(crate) fn evaluate(&self, (a, b): (Point2, Point2)) -> Point2 {
        match *self {
            PointFormula::StartPoint => a,
            PointFormula::EndPoint => b,
            PointFormula::OnCurve { param } => {
                let t = param.chart_t();
                Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
            }
        }
    }

    /// Local→global indices given the segment's State parameter window.
    pub(crate) fn param_indices(&self, block: Range<usize>) -> Vec<usize> {
        match *self {
            PointFormula::StartPoint => {
                let o = block.start;
                vec![o, o + 1]
            }
            PointFormula::EndPoint => {
                let o = block.start + 2;
                vec![o, o + 1]
            }
            PointFormula::OnCurve { .. } => block.collect(),
        }
    }

    pub(crate) fn eval_point_jacobian(&self) -> PointJacobian {
        match *self {
            PointFormula::StartPoint | PointFormula::EndPoint => PointJacobian::endpoint_identity(),
            PointFormula::OnCurve { param } => {
                let mut j = PointJacobian::zeros(Segment::PARAMETER_COUNT);
                let t = param.chart_t();
                j.set_dx_dy(0, 1.0 - t, 0.0);
                j.set_dx_dy(1, 0.0, 1.0 - t);
                j.set_dx_dy(2, t, 0.0);
                j.set_dx_dy(3, 0.0, t);
                j
            }
        }
    }
}

/// Segment-owned free-vector formulas (private to the crate; wrapped by [`VecExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VecFormula {
    Direction,
    TangentAt {
        /// Chart parameter; unused on a segment (γ' is constant).
        #[allow(dead_code)]
        param: CurveParam,
    },
    NormalAt {
        /// Chart parameter; unused on a segment (normal from constant tangent).
        #[allow(dead_code)]
        param: CurveParam,
    },
}

impl VecFormula {
    /// Evaluate against already-unpacked chart endpoints `(start, end)`.
    pub(crate) fn evaluate(&self, (a, b): (Point2, Point2)) -> Vec2 {
        match *self {
            // γ'(t) = B - A (constant on a segment); length = segment length
            VecFormula::Direction | VecFormula::TangentAt { .. } => b - a,
            VecFormula::NormalAt { .. } => (b - a).left_normal(),
        }
    }

    pub(crate) fn param_indices(&self, block: Range<usize>) -> Vec<usize> {
        block.collect()
    }

    pub(crate) fn eval_vec_jacobian(&self) -> VecJacobian {
        match *self {
            VecFormula::Direction | VecFormula::TangentAt { .. } => {
                // v = end - start on params [x1,y1,x2,y2]
                let mut j = VecJacobian::zeros(Segment::PARAMETER_COUNT);
                j.set_dx_dy(0, -1.0, 0.0);
                j.set_dx_dy(1, 0.0, -1.0);
                j.set_dx_dy(2, 1.0, 0.0);
                j.set_dx_dy(3, 0.0, 1.0);
                j
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of direction ⇒ rotate each column of J_dir leftward
                let mut j = VecJacobian::zeros(Segment::PARAMETER_COUNT);
                j.set_dx_dy(0, 0.0, -1.0);
                j.set_dx_dy(1, 1.0, 0.0);
                j.set_dx_dy(2, 0.0, 1.0);
                j.set_dx_dy(3, -1.0, 0.0);
                j
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
