//! Capability-scoped expression factories and shared formula catalogs.
//!
//! - [`CurveExprs`]: on-curve / tangent / normal / length.
//! - [`EndpointExprs`]: start / end (open/join curves only).
//!
//! [`EndpointExprs`] defaults start/end to on-curve at normalized \(t = 0\) / \(t = 1\);
//! kinds MAY override via [`EndpointFormulaKey`] (installed by
//! [`register_expr_kinds!`](super::expr) when `endpoint:` is set).
//! Closed curves implement [`CurveExprs`] only (via [`CurveFormulaKey`] from the
//! same registry). Factories are kind-associated (not on the document catalog).

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
/// Defaults use [`CurveExprs::on_curve_expr`] at normalized endpoints. Override when
/// the kind has a cheaper formula (connectivity / Jacobian), as
/// [`crate::geometry_kinds::Segment`] does via [`EndpointFormulaKey`].
pub trait EndpointExprs: CurveExprs {
    fn start_expr(id: GeometryId) -> PointExpr {
        Self::on_curve_expr(id, CurveParam::Normalized(0.0))
    }

    fn end_expr(id: GeometryId) -> PointExpr {
        Self::on_curve_expr(id, CurveParam::Normalized(1.0))
    }
}

/// Shared point-formula tags for [`CurveExprs`] (kind-agnostic; eval on the kind).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CurvePointFormula {
    OnCurve { param: CurveParam },
}

/// Shared point-formula tags for [`EndpointExprs`] overrides (kind-efficient endpoints).
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

/// Shared length-formula tags for curve length capabilities.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CurveLengthFormula {
    Length,
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

/// Kind-efficient endpoint formula keying (overrides [`EndpointExprs`] defaults).
///
/// Implemented by [`register_expr_kinds!`](super::expr) when `endpoint:` is set.
/// For default on-curve endpoints, omit `endpoint:` and write
/// `impl EndpointExprs for $ty {}`.
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
