//! Capability-scoped expression factories and shared formula catalogs.
//!
//! - [`CurveExprs`]: on-curve / tangent / normal / length.
//! - [`EndpointExprs`]: start / end (open/join curves only).
//!
//! [`EndpointExprs`] comes only from [`EndpointFormulaKey`] (installed by
//! [`register_expr_kinds!`](super::expr) when `endpoint:` is set). Open kinds
//! supply kind-efficient start/end formulas; there is no on-curve-at-\(t=0/1\)
//! default. Closed curves implement [`CurveExprs`] only (via
//! [`CurveFormulaKey`] from the same registry). Factories are kind-associated
//! (not on the document catalog).

use super::expr::{LengthFormulaKeyed, PointFormulaKeyed, VecFormulaKeyed};
use super::shell::GeometryId;
use crate::expr::{LengthExpr, PointExpr, VecExpr};
use crate::geom::CurveParam;

/// Curve chart expression factories (position / frame / length along the chart).
pub trait CurveExprs {
    fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr;
    fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr;
    fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr;
    fn length_expr(id: GeometryId) -> LengthExpr;
}

/// Geometric join endpoints (not required for intrinsically periodic closed curves).
///
/// Implemented only via [`EndpointFormulaKey`] (kind-efficient start/end formulas
/// and connectivity). See [`crate::geometry_kinds::Segment`] /
/// [`crate::geometry_kinds::Arc`].
pub trait EndpointExprs: CurveExprs {
    fn start_expr(id: GeometryId) -> PointExpr;
    fn end_expr(id: GeometryId) -> PointExpr;
}

/// Shared point-formula tags for [`CurveExprs`] (kind-agnostic; eval on the kind).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CurvePointFormula {
    OnCurve { param: CurveParam },
}

/// Shared point-formula tags for [`EndpointExprs`] (kind-efficient endpoints).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum EndpointPointFormula {
    StartPoint,
    EndPoint,
}

/// Shared free-vector formula tags for curve frame capabilities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VecFormula {
    TangentAt { param: CurveParam },
    NormalAt { param: CurveParam },
}

/// Shared length-formula tags for curve / chart length capabilities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CurveLengthFormula {
    /// Total curve arc length (`length_expr`).
    Length,
    /// Signed radius Length chart atom (circular arc).
    Radius,
}

/// Shared heading-angle formula tags (kinds that expose absolute orientation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum HeadingAngleFormula {
    StartHeading,
}

/// Shared turn-angle formula tags (kinds that expose free angular displacement).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TurnAngleFormula {
    Sweep,
}

/// Maps shared formula tags to kind-keyed wrappers in [`super::expr`].
///
/// Implemented by [`register_expr_kinds!`](super::expr); [`CurveExprs`] follows
/// from the blanket below.
pub(crate) trait CurveFormulaKey {
    fn curve_point(f: CurvePointFormula) -> PointFormulaKeyed;
    fn vec(f: VecFormula) -> VecFormulaKeyed;
    fn length(f: CurveLengthFormula) -> LengthFormulaKeyed;
}

impl<T: CurveFormulaKey> CurveExprs for T {
    fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr::new(id, T::curve_point(CurvePointFormula::OnCurve { param }))
    }

    fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::new(id, T::vec(VecFormula::TangentAt { param }))
    }

    fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::new(id, T::vec(VecFormula::NormalAt { param }))
    }

    fn length_expr(id: GeometryId) -> LengthExpr {
        LengthExpr::new(id, T::length(CurveLengthFormula::Length))
    }
}

/// Kind-efficient endpoint formula keying → [`EndpointExprs`].
///
/// Implemented by [`register_expr_kinds!`](super::expr) when `endpoint:` is set.
pub(crate) trait EndpointFormulaKey: CurveFormulaKey {
    fn endpoint_point(f: EndpointPointFormula) -> PointFormulaKeyed;
}

impl<T: EndpointFormulaKey> EndpointExprs for T {
    fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, T::endpoint_point(EndpointPointFormula::StartPoint))
    }

    fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr::new(id, T::endpoint_point(EndpointPointFormula::EndPoint))
    }
}
