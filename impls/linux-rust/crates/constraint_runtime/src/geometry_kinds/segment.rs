//! Line segment: State layout, chart sampling, and formula evaluation.

use crate::geom::{Length, Point2, PointJacobian, ScalarJacobian, Vec2, VecJacobian};
use crate::geometry::capabilities::{
    CurveLengthFormula, CurvePointFormula, EndpointPointFormula, VecFormula,
};
use crate::geometry::chart::chart_atoms;
use crate::geometry::expr::FormulaEval;
use crate::geometry::shell::{geometry_shell_instance, GeometryShell};
use crate::geometry::Chart;

/// Line-segment instance: document shell plus segment chart formulas.
///
/// Parameters live at `p[shell.param_offset..shell.param_offset+PARAMETER_COUNT]`.
/// Layout: [`Point2`] atoms [`Self::start`] / [`Self::end`] via [`chart_atoms!`];
/// pack / unpack / [`Self::SLOTS`] follow from that.
/// Shell fields via [`crate::geometry::AsShell`] (from [`geometry_shell_instance!`]);
/// together with [`Chart`] that yields [`crate::geometry::GeometryKind`] /
/// [`crate::geometry::Geometry`]. Expression keying comes from
/// [`register_expr_kinds!`](crate::geometry::expr) (`endpoint:` → sparse start/end).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment {
    pub shell: GeometryShell,
}

geometry_shell_instance!(Segment);

chart_atoms! {
    Segment {
        start: Point2,
        end: Point2,
    }
}

impl Segment {
    pub(crate) fn eval_curve_point(
        f: CurvePointFormula,
        (a, b): (Point2, Point2),
    ) -> FormulaEval<Point2, PointJacobian> {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let t = param.chart_t();
                let value = Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y);
                let mut jac = PointJacobian::zeros(Self::PARAMETER_COUNT);
                jac.set_dx_dy(Self::start.component(Point2::COMP_X), 1.0 - t, 0.0);
                jac.set_dx_dy(Self::start.component(Point2::COMP_Y), 0.0, 1.0 - t);
                jac.set_dx_dy(Self::end.component(Point2::COMP_X), t, 0.0);
                jac.set_dx_dy(Self::end.component(Point2::COMP_Y), 0.0, t);
                FormulaEval::new(value, jac)
            }
        }
    }

    pub(crate) fn eval_endpoint_point(
        f: EndpointPointFormula,
        (a, b): (Point2, Point2),
    ) -> FormulaEval<Point2, PointJacobian> {
        match f {
            EndpointPointFormula::StartPoint => {
                FormulaEval::new(a, PointJacobian::endpoint_identity())
            }
            EndpointPointFormula::EndPoint => {
                FormulaEval::new(b, PointJacobian::endpoint_identity())
            }
        }
    }

    /// Sparse endpoint connectivity (override; curve/vec/length default to [`Self::SLOTS`]).
    pub(crate) fn endpoint_point_param_indices(f: EndpointPointFormula) -> Vec<usize> {
        match f {
            EndpointPointFormula::StartPoint => Self::start.indices(),
            EndpointPointFormula::EndPoint => Self::end.indices(),
        }
    }

    pub(crate) fn eval_vec(
        f: VecFormula,
        (a, b): (Point2, Point2),
    ) -> FormulaEval<Vec2, VecJacobian> {
        match f {
            // γ'(t) = B - A (constant on a segment); length = segment length
            VecFormula::TangentAt { .. } => {
                let mut jac = VecJacobian::zeros(Self::PARAMETER_COUNT);
                jac.set_dx_dy(Self::start.component(Point2::COMP_X), -1.0, 0.0);
                jac.set_dx_dy(Self::start.component(Point2::COMP_Y), 0.0, -1.0);
                jac.set_dx_dy(Self::end.component(Point2::COMP_X), 1.0, 0.0);
                jac.set_dx_dy(Self::end.component(Point2::COMP_Y), 0.0, 1.0);
                FormulaEval::new(b - a, jac)
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of tangent ⇒ rotate each column of J_tan leftward
                let mut jac = VecJacobian::zeros(Self::PARAMETER_COUNT);
                jac.set_dx_dy(Self::start.component(Point2::COMP_X), 0.0, -1.0);
                jac.set_dx_dy(Self::start.component(Point2::COMP_Y), 1.0, 0.0);
                jac.set_dx_dy(Self::end.component(Point2::COMP_X), 0.0, 1.0);
                jac.set_dx_dy(Self::end.component(Point2::COMP_Y), -1.0, 0.0);
                FormulaEval::new((b - a).left_normal(), jac)
            }
        }
    }

    pub(crate) fn eval_length(
        f: CurveLengthFormula,
        (a, b): (Point2, Point2),
    ) -> FormulaEval<Length, ScalarJacobian> {
        match f {
            CurveLengthFormula::Length => {
                let d = b - a;
                let len = d.length();
                let mut jac = ScalarJacobian::zeros(Self::PARAMETER_COUNT);
                if len > 0.0 {
                    let ux = d.x / len;
                    let uy = d.y / len;
                    jac.set_ds(Self::start.component(Point2::COMP_X), -ux);
                    jac.set_ds(Self::start.component(Point2::COMP_Y), -uy);
                    jac.set_ds(Self::end.component(Point2::COMP_X), ux);
                    jac.set_ds(Self::end.component(Point2::COMP_Y), uy);
                }
                FormulaEval::new(Length::new(len), jac)
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
