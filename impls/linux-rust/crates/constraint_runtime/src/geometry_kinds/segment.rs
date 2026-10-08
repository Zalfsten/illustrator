//! Line segment: Vector layout, chart sampling, and formula evaluation.

use crate::geom::{Length, Point2, Vec2};
use crate::geometry::capabilities::{
    CurveLengthFormula, CurvePointFormula, EndpointPointFormula, VecFormula,
};
use crate::geometry::chart::chart_atoms;
use crate::geometry::shell::{geometry_shell_instance, GeometryShell};
use crate::matrix::MatrixWrite;

/// Line-segment instance: document shell plus segment chart formulas.
///
/// Parameters live at `p[shell.param_offset..shell.param_offset+PARAMETER_COUNT]`.
/// Layout: [`Point2`] atoms [`Self::start`] / [`Self::end`] via [`chart_atoms!`];
/// pack / unpack / [`Self::SLOTS`] follow from that.
/// Shell fields via [`crate::geometry::AsShell`] (from [`geometry_shell_instance!`]);
/// together with [`Chart`] that yields [`crate::geometry::GeometryKind`] /
/// [`crate::geometry::Geometry`]. Expression keying comes from
/// [`register_expr_kinds!`](crate::geometry::expr) (`endpoint:` → sparse start/end).
///
/// Full-chart Jacobian columns are formula-local `0..4` in [`Self::SLOTS`] order
/// (start.x, start.y, end.x, end.y). Endpoint formulas use local `0..2` (packed
/// [`Point2`]). Placement into contribution/global space is the view's job.
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

    /// Write \(2 \times 4\) curve-point Jacobian into `j` (SLOTS-local columns).
    pub(crate) fn accumulate_curve_point_jacobian<J: MatrixWrite>(
        f: CurvePointFormula,
        j: &mut J,
        scale: f64,
    ) {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let t = param.chart_t();
                j[(0, 0)] = scale * (1.0 - t);
                j[(1, 1)] = scale * (1.0 - t);
                j[(0, 2)] = scale * t;
                j[(1, 3)] = scale * t;
            }
        }
    }

    pub(crate) fn eval_endpoint_point(f: EndpointPointFormula, (a, b): (Point2, Point2)) -> Point2 {
        match f {
            EndpointPointFormula::StartPoint => a,
            EndpointPointFormula::EndPoint => b,
        }
    }

    /// Write \(2 \times 2\) endpoint identity Jacobian into `j` (Point2-local columns).
    pub(crate) fn accumulate_endpoint_point_jacobian<J: MatrixWrite>(
        f: EndpointPointFormula,
        j: &mut J,
        scale: f64,
    ) {
        match f {
            EndpointPointFormula::StartPoint | EndpointPointFormula::EndPoint => {
                j[(0, 0)] = scale;
                j[(1, 1)] = scale;
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

    pub(crate) fn eval_vec(f: VecFormula, (a, b): (Point2, Point2)) -> Vec2 {
        match f {
            // γ'(t) = B - A (constant on a segment); length = segment length
            VecFormula::TangentAt { .. } => b - a,
            VecFormula::NormalAt { .. } => (b - a).left_normal(),
        }
    }

    /// Write \(2 \times 4\) free-vector Jacobian into `j` (SLOTS-local columns).
    pub(crate) fn accumulate_vec_jacobian<J: MatrixWrite>(
        f: VecFormula,
        j: &mut J,
        scale: f64,
    ) {
        match f {
            VecFormula::TangentAt { .. } => {
                j[(0, 0)] = scale * -1.0;
                j[(1, 1)] = scale * -1.0;
                j[(0, 2)] = scale;
                j[(1, 3)] = scale;
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of tangent ⇒ rotate each column of J_tan leftward
                j[(1, 0)] = scale * -1.0;
                j[(0, 1)] = scale;
                j[(1, 2)] = scale;
                j[(0, 3)] = scale * -1.0;
            }
        }
    }

    pub(crate) fn eval_length(f: CurveLengthFormula, (a, b): (Point2, Point2)) -> Length {
        match f {
            CurveLengthFormula::Length => Length::new((b - a).length()),
        }
    }

    /// Write \(1 \times 4\) length Jacobian into row 0 of `j` (SLOTS-local columns).
    pub(crate) fn accumulate_length_jacobian<J: MatrixWrite>(
        f: CurveLengthFormula,
        (a, b): (Point2, Point2),
        j: &mut J,
        scale: f64,
    ) {
        match f {
            CurveLengthFormula::Length => {
                let d = b - a;
                let len = d.length();
                if len > 0.0 {
                    let ux = d.x / len;
                    let uy = d.y / len;
                    j[(0, 0)] = scale * -ux;
                    j[(0, 1)] = scale * -uy;
                    j[(0, 2)] = scale * ux;
                    j[(0, 3)] = scale * uy;
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
