//! Circular arc: Vector layout, chart sampling, and formula evaluation.

use std::ops::Range;

use crate::expr::{HeadingAngleExpr, TurnAngleExpr};
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle, Vec2};
use crate::geometry::capabilities::geometry_caps;
use crate::geometry::chart::chart_atoms;
use crate::geometry::{
    Geometry, GeometryId, HeadingFormula, LengthFormula, PointFormula, TurnFormula, VecFormula,
};
use crate::geometry_kinds::GeometryKind;
use crate::matrix::DenseMatrix;

/// Circular-arc kind (type-level marker; instances live in
/// [`crate::geometry_kinds::GeometryInst`]).
///
/// Layout via `chart_atoms!`: [`Self::center`], [`Self::radius`] (Length),
/// [`Self::start`] (heading), [`Self::sweep`] (turn). The radius is non-negative
/// in committed State ([`Geometry::canonical`] rewrites `R < 0` to
/// `(−R, θ₀ + π)` on commit / add). Solver iterates may hold `R < 0`: position
/// uses \(R\) as stored (smooth through 0; a negative radius flips the point
/// through the center), so the start heading equals the start point's polar
/// angle only for `R ≥ 0`. Length and radius use magnitudes:
/// \(L = |R|\,|\Delta\theta|\), radius \(= |R|\).
///
/// Chart: \(\theta(t)=\theta_0 + t\,\Delta\theta\),
/// \(\gamma(t)=C + R\,(\cos\theta,\sin\theta)\).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Arc;

chart_atoms! {
    Arc {
        center: Point2,
        radius: Length,
        start: HeadingAngle,
        sweep: TurnAngle,
    }
}

// Capability factory impls + runtime Caps.
geometry_caps!(Arc: endpoints, circular);

impl Arc {
    /// Heading at the arc start (chart atom).
    pub fn start_heading_expr(id: GeometryId) -> HeadingAngleExpr {
        HeadingAngleExpr::new(id, HeadingFormula::StartHeading)
    }

    /// Signed sweep turn (chart atom).
    pub fn sweep_expr(id: GeometryId) -> TurnAngleExpr {
        TurnAngleExpr::new(id, TurnFormula::Sweep)
    }

    #[inline]
    fn theta(t: f64, start: HeadingAngle, sweep: TurnAngle) -> f64 {
        start.radians() + t * sweep.radians()
    }

    /// Chart parameter of a point formula (endpoints are `t = 0 / 1`).
    #[inline]
    fn point_t(f: PointFormula) -> f64 {
        match f {
            PointFormula::OnCurve { param } => param.chart_t(),
            PointFormula::Start => 0.0,
            PointFormula::End => 1.0,
            PointFormula::Center => unreachable!("center is not a curve location"),
        }
    }
}

impl Geometry for Arc {
    const KIND: GeometryKind = GeometryKind::Arc;

    /// `R < 0` → `(C, −R, θ₀ + π, Δθ)`: same curve, non-negative radius.
    fn canonical((center, radius, start, sweep): Self::Init) -> Self::Init {
        if radius.value() < 0.0 {
            (
                center,
                Length::new(-radius.value()),
                HeadingAngle::new(start.radians() + std::f64::consts::PI),
                sweep,
            )
        } else {
            (center, radius, start, sweep)
        }
    }

    fn point(f: PointFormula, (center, radius, start, sweep): Self::Init) -> Point2 {
        if let PointFormula::Center = f {
            return center;
        }
        let th = Self::theta(Self::point_t(f), start, sweep);
        let r = radius.value();
        Point2::new(center.x + r * th.cos(), center.y + r * th.sin())
    }

    /// \(2 \times 5\) (or \(2 \times 4\) for `Start`, which is independent of sweep).
    fn point_jacobian(
        f: PointFormula,
        (_center, radius, start, sweep): Self::Init,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        if let PointFormula::Center = f {
            // local cols: center.x, center.y
            j.set_scaled(scale, [[1.0, 0.0], [0.0, 1.0]]);
            return;
        }
        let t = Self::point_t(f);
        let r = radius.value();
        let th = Self::theta(t, start, sweep);
        let (c, s) = (th.cos(), th.sin());
        match f {
            // local cols: center.x,y, R, θ0 (no sweep)
            PointFormula::Start => {
                j.set_scaled(scale, [[1.0, 0.0, c, -r * s], [0.0, 1.0, s, r * c]]);
            }
            // ∂γ/∂(cx,cy,R,θ0,Δθ)
            PointFormula::OnCurve { .. } | PointFormula::End => {
                j.set_scaled(
                    scale,
                    [
                        [1.0, 0.0, c, -r * s, -r * s * t],
                        [0.0, 1.0, s, r * c, r * c * t],
                    ],
                );
            }
            PointFormula::Center => unreachable!("handled above"),
        }
    }

    fn point_params(f: PointFormula) -> Range<usize> {
        match f {
            // start independent of sweep at t=0
            PointFormula::Start => Self::center.span(Self::start),
            PointFormula::OnCurve { .. } | PointFormula::End => Self::center.span(Self::sweep),
            PointFormula::Center => Self::center.range(),
        }
    }

    fn vec(f: VecFormula, (_center, radius, start, sweep): Self::Init) -> Vec2 {
        let r = radius.value();
        let dth = sweep.radians();
        match f {
            VecFormula::TangentAt { param } => {
                let th = Self::theta(param.chart_t(), start, sweep);
                // γ'(t) = R Δθ (-sin θ, cos θ)
                Vec2::new(-r * dth * th.sin(), r * dth * th.cos())
            }
            VecFormula::NormalAt { param } => {
                let th = Self::theta(param.chart_t(), start, sweep);
                // left_normal(tangent) = R Δθ (-cos θ, -sin θ)
                Vec2::new(-r * dth * th.cos(), -r * dth * th.sin())
            }
            VecFormula::Direction => unreachable!("arc has no straight carrier"),
        }
    }

    /// \(2 \times 5\) free-vector Jacobian.
    fn vec_jacobian(
        f: VecFormula,
        (_center, radius, start, sweep): Self::Init,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
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
                        [0.0, 0.0, -dth * s, -r * dth * c, -r * s - r * dth * t * c],
                        [0.0, 0.0, dth * c, -r * dth * s, r * c - r * dth * t * s],
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
                        [0.0, 0.0, -dth * c, r * dth * s, -r * c + r * dth * t * s],
                        [0.0, 0.0, -dth * s, -r * dth * c, -r * s - r * dth * t * c],
                    ],
                );
            }
            VecFormula::Direction => unreachable!("arc has no straight carrier"),
        }
    }

    fn length(f: LengthFormula, (_center, radius, _start, sweep): Self::Init) -> Length {
        match f {
            LengthFormula::Length => Length::new(radius.value().abs() * sweep.radians().abs()),
            LengthFormula::Radius => Length::new(radius.value().abs()),
        }
    }

    /// \(1 \times 5\) length Jacobian.
    fn length_jacobian(
        f: LengthFormula,
        (_center, radius, _start, sweep): Self::Init,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        match f {
            LengthFormula::Length => {
                let r = radius.value();
                let s = sweep.radians();
                let d_r = if r != 0.0 { r.signum() * s.abs() } else { 0.0 };
                let d_s = if s != 0.0 { r.abs() * s.signum() } else { 0.0 };
                j.set_scaled(scale, [[0.0, 0.0, d_r, 0.0, d_s]]);
            }
            LengthFormula::Radius => {
                let r = radius.value();
                let d_r = if r != 0.0 { r.signum() } else { 0.0 };
                j.set_scaled(scale, [[0.0, 0.0, d_r, 0.0, 0.0]]);
            }
        }
    }

    fn heading(f: HeadingFormula, (_center, _radius, start, _sweep): Self::Init) -> HeadingAngle {
        match f {
            HeadingFormula::StartHeading => start,
        }
    }

    fn heading_jacobian(f: HeadingFormula, _c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        match f {
            HeadingFormula::StartHeading => j.set_scaled(scale, [[1.0]]),
        }
    }

    fn heading_params(f: HeadingFormula) -> Range<usize> {
        match f {
            HeadingFormula::StartHeading => Self::start.range(),
        }
    }

    fn turn(f: TurnFormula, (_center, _radius, _start, sweep): Self::Init) -> TurnAngle {
        match f {
            TurnFormula::Sweep => sweep,
        }
    }

    fn turn_jacobian(f: TurnFormula, _c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        match f {
            TurnFormula::Sweep => j.set_scaled(scale, [[1.0]]),
        }
    }

    fn turn_params(f: TurnFormula) -> Range<usize> {
        match f {
            TurnFormula::Sweep => Self::sweep.range(),
        }
    }
}

#[cfg(test)]
#[path = "tests_arc.rs"]
mod tests_arc;
