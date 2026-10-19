//! Kind-agnostic formula tags and capability-scoped expression factories.
//!
//! - [`CurveExprs`]: on-curve / tangent / normal / length.
//! - [`EndpointExprs`]: start / end (open/join curves only).
//!
//! Tags name *what* to evaluate; the geometry kind behind an expression's
//! [`GeometryId`] decides *how* (see [`crate::geometry::Geometry`]). Factory
//! traits are opt-in per kind so e.g. closed curves cannot hand out endpoints.

use super::formula::Geometry;
use super::shell::GeometryId;
use crate::expr::{LengthExpr, PointExpr, VecExpr};
use crate::geom::CurveParam;

/// Point formulas (curve position and geometric endpoints).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointFormula {
    OnCurve { param: CurveParam },
    /// Kind-efficient start point (may depend on fewer DOFs than `OnCurve(0)`).
    Start,
    /// Kind-efficient end point.
    End,
}

/// Free-vector formulas for the curve frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VecFormula {
    TangentAt { param: CurveParam },
    NormalAt { param: CurveParam },
}

/// Length formulas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LengthFormula {
    /// Total curve arc length (`length_expr`).
    Length,
    /// Signed radius Length chart atom (circular arc).
    Radius,
}

/// Heading-angle formulas (kinds that expose absolute orientation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HeadingFormula {
    StartHeading,
}

/// Turn-angle formulas (kinds that expose free angular displacement).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TurnFormula {
    Sweep,
}

/// Curve chart expression factories (position / frame / length along the chart).
pub trait CurveExprs: Geometry {
    fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr::new(id, PointFormula::OnCurve { param })
    }

    fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::new(id, VecFormula::TangentAt { param })
    }

    fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::new(id, VecFormula::NormalAt { param })
    }

    fn length_expr(id: GeometryId) -> LengthExpr {
        LengthExpr::new(id, LengthFormula::Length)
    }
}

/// Geometric join endpoints (not for intrinsically periodic closed curves).
pub trait EndpointExprs: CurveExprs {
    fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, PointFormula::Start)
    }

    fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, PointFormula::End)
    }
}
