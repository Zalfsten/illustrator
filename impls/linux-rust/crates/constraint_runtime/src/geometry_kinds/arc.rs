//! Circular arc: Vector layout, chart sampling, and formula evaluation.

use crate::geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle, Vec2};
use crate::geometry::capabilities::{
    CurveLengthFormula, CurvePointFormula, EndpointPointFormula, HeadingAngleFormula,
    TurnAngleFormula, VecFormula,
};
use crate::geometry::chart::chart_atoms;
use crate::geometry::shell::{geometry_shell_instance, GeometryShell};
use crate::geometry::StateAtom;
use crate::matrix::MatrixAccess;

/// Circular-arc instance: document shell plus arc chart formulas.
///
/// Parameters live at `p[shell.param_offset..shell.param_offset+PARAMETER_COUNT]`.
/// Layout via [`chart_atoms!`]: [`Self::center`], [`Self::radius`] (signed Length),
/// [`Self::start`] (heading), [`Self::sweep`] (turn). Position uses \(R\) as stored
/// (signed radius flips the point through the center). Length uses magnitudes:
/// \(L = |R|\,|\Delta\theta|\).
///
/// Chart: \(\theta(t)=\theta_0 + t\,\Delta\theta\),
/// \(\gamma(t)=C + R\,(\cos\theta,\sin\theta)\).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Arc {
    pub shell: GeometryShell,
}

geometry_shell_instance!(Arc);

chart_atoms! {
    Arc {
        center: Point2,
        radius: Length,
        start: HeadingAngle,
        sweep: TurnAngle,
    }
}

type ArcChart = (Point2, Length, HeadingAngle, TurnAngle);

impl Arc {
    #[inline]
    fn theta(t: f64, start: HeadingAngle, sweep: TurnAngle) -> f64 {
        start.radians() + t * sweep.radians()
    }

    #[inline]
    fn point_at(center: Point2, r: f64, theta: f64) -> Point2 {
        Point2::new(center.x + r * theta.cos(), center.y + r * theta.sin())
    }

    pub(crate) fn eval_curve_point(
        f: CurvePointFormula,
        (center, radius, start, sweep): ArcChart,
    ) -> Point2 {
        match f {
            CurvePointFormula::OnCurve { param } => {
                let t = param.chart_t();
                let th = Self::theta(t, start, sweep);
                Self::point_at(center, radius.value(), th)
            }
        }
    }

    /// Write \(2 \times 5\) curve-point Jacobian into `j` (SLOTS-local columns).
    pub(crate) fn accumulate_curve_point_jacobian<J: MatrixAccess>(
        f: CurvePointFormula,
        (center, radius, start, sweep): ArcChart,
        j: &mut J,
        scale: f64,
    ) {
        let _ = center;
        match f {
            CurvePointFormula::OnCurve { param } => {
                let t = param.chart_t();
                let r = radius.value();
                let th = Self::theta(t, start, sweep);
                let (c, s) = (th.cos(), th.sin());
                // ∂γ/∂(cx,cy,R,θ0,Δθ)
                j.set_scaled(
                    scale,
                    [
                        [1.0, 0.0, c, -r * s, -r * s * t],
                        [0.0, 1.0, s, r * c, r * c * t],
                    ],
                );
            }
        }
    }

    pub(crate) fn eval_endpoint_point(
        f: EndpointPointFormula,
        chart: ArcChart,
    ) -> Point2 {
        match f {
            EndpointPointFormula::StartPoint => Self::eval_curve_point(
                CurvePointFormula::OnCurve {
                    param: CurveParam::Normalized(0.0),
                },
                chart,
            ),
            EndpointPointFormula::EndPoint => Self::eval_curve_point(
                CurvePointFormula::OnCurve {
                    param: CurveParam::Normalized(1.0),
                },
                chart,
            ),
        }
    }

    /// Write endpoint Jacobian into `j` (formula-local columns; see connectivity).
    pub(crate) fn accumulate_endpoint_point_jacobian<J: MatrixAccess>(
        f: EndpointPointFormula,
        (center, radius, start, sweep): ArcChart,
        j: &mut J,
        scale: f64,
    ) {
        let _ = center;
        let r = radius.value();
        match f {
            EndpointPointFormula::StartPoint => {
                let th = start.radians();
                let (c, s) = (th.cos(), th.sin());
                // local cols: center.x,y, R, θ0 (no sweep)
                j.set_scaled(
                    scale,
                    [[1.0, 0.0, c, -r * s], [0.0, 1.0, s, r * c]],
                );
            }
            EndpointPointFormula::EndPoint => {
                let t = 1.0;
                let th = Self::theta(t, start, sweep);
                let (c, s) = (th.cos(), th.sin());
                j.set_scaled(
                    scale,
                    [
                        [1.0, 0.0, c, -r * s, -r * s * t],
                        [0.0, 1.0, s, r * c, r * c * t],
                    ],
                );
            }
        }
    }

    pub(crate) fn endpoint_point_param_indices(f: EndpointPointFormula) -> Vec<usize> {
        match f {
            // start independent of sweep at t=0
            EndpointPointFormula::StartPoint => {
                let mut idx = Self::center.indices();
                idx.extend(Self::radius.indices());
                idx.extend(Self::start.indices());
                idx
            }
            EndpointPointFormula::EndPoint => {
                crate::geometry::ChartSlot::union_indices(Self::SLOTS)
            }
        }
    }

    pub(crate) fn endpoint_point_param_count(f: EndpointPointFormula) -> usize {
        match f {
            EndpointPointFormula::StartPoint => {
                Point2::DOF_COUNT + Length::DOF_COUNT + HeadingAngle::DOF_COUNT
            }
            EndpointPointFormula::EndPoint => {
                crate::geometry::ChartSlot::union_dof(Self::SLOTS)
            }
        }
    }

    pub(crate) fn eval_vec(
        f: VecFormula,
        (center, radius, start, sweep): ArcChart,
    ) -> Vec2 {
        let _ = center;
        let r = radius.value();
        let dth = sweep.radians();
        match f {
            VecFormula::TangentAt { param } => {
                let t = param.chart_t();
                let th = Self::theta(t, start, sweep);
                // γ'(t) = R Δθ (-sin θ, cos θ)
                Vec2::new(-r * dth * th.sin(), r * dth * th.cos())
            }
            VecFormula::NormalAt { param } => {
                let t = param.chart_t();
                let th = Self::theta(t, start, sweep);
                // left_normal(tangent) = R Δθ (-cos θ, -sin θ)
                Vec2::new(-r * dth * th.cos(), -r * dth * th.sin())
            }
        }
    }

    /// Write \(2 \times 5\) free-vector Jacobian into `j` (SLOTS-local columns).
    pub(crate) fn accumulate_vec_jacobian<J: MatrixAccess>(
        f: VecFormula,
        (center, radius, start, sweep): ArcChart,
        j: &mut J,
        scale: f64,
    ) {
        let _ = center;
        let r = radius.value();
        let dth = sweep.radians();
        match f {
            VecFormula::TangentAt { param } => {
                let t = param.chart_t();
                let th = Self::theta(t, start, sweep);
                let (c, s) = (th.cos(), th.sin());
                // v = R s (-sin, cos)
                j.set_scaled(
                    scale,
                    [
                        [
                            0.0,
                            0.0,
                            -dth * s,
                            -r * dth * c,
                            -r * s - r * dth * t * c,
                        ],
                        [
                            0.0,
                            0.0,
                            dth * c,
                            -r * dth * s,
                            r * c - r * dth * t * s,
                        ],
                    ],
                );
            }
            VecFormula::NormalAt { param } => {
                let t = param.chart_t();
                let th = Self::theta(t, start, sweep);
                let (c, s) = (th.cos(), th.sin());
                // n = R s (-cos, -sin)
                j.set_scaled(
                    scale,
                    [
                        [
                            0.0,
                            0.0,
                            -dth * c,
                            r * dth * s,
                            -r * c + r * dth * t * s,
                        ],
                        [
                            0.0,
                            0.0,
                            -dth * s,
                            -r * dth * c,
                            -r * s - r * dth * t * c,
                        ],
                    ],
                );
            }
        }
    }

    pub(crate) fn eval_length(
        f: CurveLengthFormula,
        (_center, radius, _start, sweep): ArcChart,
    ) -> Length {
        match f {
            CurveLengthFormula::Length => {
                Length::new(radius.value().abs() * sweep.radians().abs())
            }
            CurveLengthFormula::Radius => radius,
        }
    }

    /// Write \(1 \times 5\) length Jacobian into row 0 of `j` (SLOTS-local columns).
    pub(crate) fn accumulate_length_jacobian<J: MatrixAccess>(
        f: CurveLengthFormula,
        (_center, radius, _start, sweep): ArcChart,
        j: &mut J,
        scale: f64,
    ) {
        match f {
            CurveLengthFormula::Length => {
                let r = radius.value();
                let s = sweep.radians();
                let d_r = if r != 0.0 {
                    r.signum() * s.abs()
                } else {
                    0.0
                };
                let d_s = if s != 0.0 {
                    r.abs() * s.signum()
                } else {
                    0.0
                };
                j.set_scaled(scale, [[0.0, 0.0, d_r, 0.0, d_s]]);
            }
            CurveLengthFormula::Radius => {
                j.set_scaled(scale, [[0.0, 0.0, 1.0, 0.0, 0.0]]);
            }
        }
    }

    pub(crate) fn eval_heading(
        f: HeadingAngleFormula,
        (_center, _radius, start, _sweep): ArcChart,
    ) -> HeadingAngle {
        match f {
            HeadingAngleFormula::StartHeading => start,
        }
    }

    pub(crate) fn accumulate_heading_jacobian<J: MatrixAccess>(
        f: HeadingAngleFormula,
        j: &mut J,
        scale: f64,
    ) {
        match f {
            HeadingAngleFormula::StartHeading => {
                j.set_scaled(scale, [[1.0]]);
            }
        }
    }

    pub(crate) fn heading_param_indices(f: HeadingAngleFormula) -> Vec<usize> {
        match f {
            HeadingAngleFormula::StartHeading => Self::start.indices(),
        }
    }

    pub(crate) fn heading_param_count(f: HeadingAngleFormula) -> usize {
        match f {
            HeadingAngleFormula::StartHeading => HeadingAngle::DOF_COUNT,
        }
    }

    pub(crate) fn eval_turn(
        f: TurnAngleFormula,
        (_center, _radius, _start, sweep): ArcChart,
    ) -> TurnAngle {
        match f {
            TurnAngleFormula::Sweep => sweep,
        }
    }

    pub(crate) fn accumulate_turn_jacobian<J: MatrixAccess>(
        f: TurnAngleFormula,
        j: &mut J,
        scale: f64,
    ) {
        match f {
            TurnAngleFormula::Sweep => {
                j.set_scaled(scale, [[1.0]]);
            }
        }
    }

    pub(crate) fn turn_param_indices(f: TurnAngleFormula) -> Vec<usize> {
        match f {
            TurnAngleFormula::Sweep => Self::sweep.indices(),
        }
    }

    pub(crate) fn turn_param_count(f: TurnAngleFormula) -> usize {
        match f {
            TurnAngleFormula::Sweep => TurnAngle::DOF_COUNT,
        }
    }

    /// Chord count for Mesh polyline: \(n=\mathrm{clamp}(\lceil|\Delta\theta|/(\pi/8)\rceil,2,64)\).
    pub fn mesh_sample_count(sweep: TurnAngle) -> usize {
        const STEP: f64 = std::f64::consts::PI / 8.0;
        let n = (sweep.radians().abs() / STEP).ceil() as usize;
        n.clamp(2, 64)
    }
}

#[cfg(test)]
#[path = "tests_arc.rs"]
mod tests_arc;
