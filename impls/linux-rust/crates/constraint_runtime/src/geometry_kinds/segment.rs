//! Line segment: State layout, chart sampling, and formula evaluation.

use crate::geom::{Length, Point2, PointJacobian, ScalarJacobian, Vec2, VecJacobian};
use crate::geometry::capabilities::{
    impl_curve_exprs, impl_endpoint_exprs, CurveLengthFormula, CurvePointFormula,
    EndpointPointFormula, VecFormula,
};
use crate::geometry::chart::chart_atoms;
use crate::geometry::shell::{geometry_shell_instance, GeometryShell};
use crate::geometry::{Chart, ChartSlot};

/// Line-segment instance: document shell plus segment chart formulas.
///
/// Parameters live at `p[shell.param_offset..shell.param_offset+PARAMETER_COUNT]`.
/// Layout: [`Point2`] atoms [`Self::start`] / [`Self::end`] via [`chart_atoms!`];
/// pack / unpack / [`Self::SLOTS`] follow from that.
/// Shell fields via [`crate::geometry::AsShell`] (from [`geometry_shell_instance!`]);
/// together with [`Chart`] that yields [`crate::geometry::GeometryKind`] /
/// [`crate::geometry::Geometry`]. Expression factories via [`impl_curve_exprs!`] /
/// [`impl_endpoint_exprs!`] (endpoint override: sparse start/end formulas, not
/// default on-curve \(t = 0/1\)).
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
    pub(crate) fn eval_curve_point(f: CurvePointFormula, (a, b): (Point2, Point2)) -> Point2 {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let t = param.chart_t();
                Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
            }
        }
    }

    /// Formula-local parameter indices (into the chart block, not global State).
    pub(crate) fn curve_point_param_indices(f: CurvePointFormula) -> Vec<usize> {
        match f {
            CurvePointFormula::OnCurve { .. } => ChartSlot::union_indices(Self::SLOTS),
        }
    }

    pub(crate) fn eval_curve_point_jacobian(f: CurvePointFormula) -> PointJacobian {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let mut j = PointJacobian::zeros(Self::PARAMETER_COUNT);
                let t = param.chart_t();
                j.set_dx_dy(Self::start.component(Point2::COMP_X), 1.0 - t, 0.0);
                j.set_dx_dy(Self::start.component(Point2::COMP_Y), 0.0, 1.0 - t);
                j.set_dx_dy(Self::end.component(Point2::COMP_X), t, 0.0);
                j.set_dx_dy(Self::end.component(Point2::COMP_Y), 0.0, t);
                j
            }
        }
    }

    pub(crate) fn eval_endpoint_point(f: EndpointPointFormula, (a, b): (Point2, Point2)) -> Point2 {
        match f {
            EndpointPointFormula::StartPoint => a,
            EndpointPointFormula::EndPoint => b,
        }
    }

    /// Formula-local parameter indices (into the chart block, not global State).
    pub(crate) fn endpoint_point_param_indices(f: EndpointPointFormula) -> Vec<usize> {
        match f {
            EndpointPointFormula::StartPoint => Self::start.indices(),
            EndpointPointFormula::EndPoint => Self::end.indices(),
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

    /// Formula-local parameter indices (into the chart block, not global State).
    pub(crate) fn vec_param_indices(_f: VecFormula) -> Vec<usize> {
        ChartSlot::union_indices(Self::SLOTS)
    }

    pub(crate) fn eval_vec_jacobian(f: VecFormula) -> VecJacobian {
        match f {
            VecFormula::TangentAt { .. } => {
                // v = end - start
                let mut j = VecJacobian::zeros(Self::PARAMETER_COUNT);
                j.set_dx_dy(Self::start.component(Point2::COMP_X), -1.0, 0.0);
                j.set_dx_dy(Self::start.component(Point2::COMP_Y), 0.0, -1.0);
                j.set_dx_dy(Self::end.component(Point2::COMP_X), 1.0, 0.0);
                j.set_dx_dy(Self::end.component(Point2::COMP_Y), 0.0, 1.0);
                j
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of tangent ⇒ rotate each column of J_tan leftward
                let mut j = VecJacobian::zeros(Self::PARAMETER_COUNT);
                j.set_dx_dy(Self::start.component(Point2::COMP_X), 0.0, -1.0);
                j.set_dx_dy(Self::start.component(Point2::COMP_Y), 1.0, 0.0);
                j.set_dx_dy(Self::end.component(Point2::COMP_X), 0.0, 1.0);
                j.set_dx_dy(Self::end.component(Point2::COMP_Y), -1.0, 0.0);
                j
            }
        }
    }

    pub(crate) fn eval_length(f: CurveLengthFormula, (a, b): (Point2, Point2)) -> Length {
        match f {
            CurveLengthFormula::Length => Length::new((b - a).length()),
        }
    }

    /// Formula-local parameter indices (into the chart block, not global State).
    pub(crate) fn length_param_indices(f: CurveLengthFormula) -> Vec<usize> {
        match f {
            CurveLengthFormula::Length => ChartSlot::union_indices(Self::SLOTS),
        }
    }

    pub(crate) fn eval_length_jacobian(
        f: CurveLengthFormula,
        (a, b): (Point2, Point2),
    ) -> ScalarJacobian {
        match f {
            CurveLengthFormula::Length => {
                let mut j = ScalarJacobian::zeros(Self::PARAMETER_COUNT);
                let d = b - a;
                let len = d.length();
                if len > 0.0 {
                    let ux = d.x / len;
                    let uy = d.y / len;
                    j.set_ds(Self::start.component(Point2::COMP_X), -ux);
                    j.set_ds(Self::start.component(Point2::COMP_Y), -uy);
                    j.set_ds(Self::end.component(Point2::COMP_X), ux);
                    j.set_ds(Self::end.component(Point2::COMP_Y), uy);
                }
                j
            }
        }
    }
}

impl_curve_exprs!(Segment, SegmentCurve, Segment, Segment);
impl_endpoint_exprs!(Segment, SegmentEndpoint);

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
