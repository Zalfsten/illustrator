//! Line segment: State layout, chart sampling, and formula evaluation.

use std::ops::Range;

use super::capabilities::{
    impl_curve_exprs, impl_endpoint_exprs, CurvePointFormula, EndpointPointFormula, VecFormula,
};
use super::shell::{geometry_shell_instance, GeometryShell};
use super::Chart;
use crate::geom::{Point2, PointJacobian, Vec2, VecJacobian};

/// Line-segment instance: document shell plus segment chart formulas.
///
/// Parameters live at `p[shell.param_offset..shell.param_offset+PARAMETER_COUNT]`.
/// Shell fields via [`super::AsShell`] (from [`geometry_shell_instance!`]); together
/// with [`Chart`] that yields [`super::GeometryKind`] / [`super::Geometry`].
/// Expression factories via [`impl_curve_exprs!`] / [`impl_endpoint_exprs!`]
/// (endpoint override: sparse start/end formulas, not default on-curve \(t = 0/1\)).
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
    pub(crate) fn eval_curve_point(f: CurvePointFormula, (a, b): (Point2, Point2)) -> Point2 {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let t = param.chart_t();
                Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
            }
        }
    }

    pub(crate) fn curve_point_param_indices(
        f: CurvePointFormula,
        block: Range<usize>,
    ) -> Vec<usize> {
        match f {
            CurvePointFormula::OnCurve { .. } => block.collect(),
        }
    }

    pub(crate) fn eval_curve_point_jacobian(f: CurvePointFormula) -> PointJacobian {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let mut j = PointJacobian::zeros(Self::PARAMETER_COUNT);
                let t = param.chart_t();
                j.set_dx_dy(0, 1.0 - t, 0.0);
                j.set_dx_dy(1, 0.0, 1.0 - t);
                j.set_dx_dy(2, t, 0.0);
                j.set_dx_dy(3, 0.0, t);
                j
            }
        }
    }

    pub(crate) fn eval_endpoint_point(
        f: EndpointPointFormula,
        (a, b): (Point2, Point2),
    ) -> Point2 {
        match f {
            EndpointPointFormula::StartPoint => a,
            EndpointPointFormula::EndPoint => b,
        }
    }

    pub(crate) fn endpoint_point_param_indices(
        f: EndpointPointFormula,
        block: Range<usize>,
    ) -> Vec<usize> {
        match f {
            EndpointPointFormula::StartPoint => {
                let o = block.start;
                vec![o, o + 1]
            }
            EndpointPointFormula::EndPoint => {
                let o = block.start + 2;
                vec![o, o + 1]
            }
        }
    }

    pub(crate) fn eval_endpoint_point_jacobian(f: EndpointPointFormula) -> PointJacobian {
        match f {
            EndpointPointFormula::StartPoint | EndpointPointFormula::EndPoint => {
                PointJacobian::endpoint_identity()
            }
        }
    }

    /// Evaluate a shared free-vector formula against unpacked segment endpoints.
    pub(crate) fn eval_vec(f: VecFormula, (a, b): (Point2, Point2)) -> Vec2 {
        match f {
            // γ'(t) = B - A (constant on a segment); length = segment length
            VecFormula::TangentAt { .. } => b - a,
            VecFormula::NormalAt { .. } => (b - a).left_normal(),
        }
    }

    pub(crate) fn vec_param_indices(_f: VecFormula, block: Range<usize>) -> Vec<usize> {
        block.collect()
    }

    pub(crate) fn eval_vec_jacobian(f: VecFormula) -> VecJacobian {
        match f {
            VecFormula::TangentAt { .. } => {
                // v = end - start on params [x1,y1,x2,y2]
                let mut j = VecJacobian::zeros(Self::PARAMETER_COUNT);
                j.set_dx_dy(0, -1.0, 0.0);
                j.set_dx_dy(1, 0.0, -1.0);
                j.set_dx_dy(2, 1.0, 0.0);
                j.set_dx_dy(3, 0.0, 1.0);
                j
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of tangent ⇒ rotate each column of J_tan leftward
                let mut j = VecJacobian::zeros(Self::PARAMETER_COUNT);
                j.set_dx_dy(0, 0.0, -1.0);
                j.set_dx_dy(1, 1.0, 0.0);
                j.set_dx_dy(2, 0.0, 1.0);
                j.set_dx_dy(3, -1.0, 0.0);
                j
            }
        }
    }
}

impl_curve_exprs!(Segment, segment_curve, segment);
impl_endpoint_exprs!(Segment, segment_endpoint);

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
