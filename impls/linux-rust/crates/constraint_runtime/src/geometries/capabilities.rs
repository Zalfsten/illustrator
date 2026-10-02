//! Capability-scoped expression factories and shared formula catalogs.
//!
//! - [`CurveExprs`] + [`CurvePointFormula`] / [`VecFormula`]: on-curve / tangent / normal.
//! - [`EndpointExprs`] + [`EndpointPointFormula`]: start / end (open/join curves only).
//!
//! Closed curves implement [`CurveExprs`] only — they never see endpoint formula tags.
//! Factories are kind-associated (not object-safe on [`super::Geometry`]). Prefer
//! [`impl_curve_exprs!`] / [`impl_endpoint_exprs!`]; evaluation stays on the kind.

use super::shell::GeometryId;
use super::{PointExpr, VecExpr};
use crate::geom::CurveParam;

/// Curve chart expression factories (position / frame along the chart).
pub trait CurveExprs {
    fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr;
    fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr;
    fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr;
}

/// Geometric join endpoints (not required for intrinsically periodic closed curves).
pub trait EndpointExprs {
    fn start_expr(id: GeometryId) -> PointExpr;
    fn end_expr(id: GeometryId) -> PointExpr;
}

/// Shared point-formula tags for [`CurveExprs`] (kind-agnostic; eval on the kind).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CurvePointFormula {
    OnCurve { param: CurveParam },
}

/// Shared point-formula tags for [`EndpointExprs`].
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

/// Implement [`CurveExprs`] for `$ty`.
///
/// `$curve_wrap` / `$vec_wrap` are kind constructors on [`PointExpr`] / [`VecExpr`]
/// (e.g. `segment_curve`, `segment`).
macro_rules! impl_curve_exprs {
    ($ty:ty, $curve_wrap:ident, $vec_wrap:ident) => {
        impl $crate::geometries::CurveExprs for $ty {
            fn on_curve_expr(
                id: $crate::geometries::GeometryId,
                param: $crate::geom::CurveParam,
            ) -> $crate::geometries::PointExpr {
                $crate::geometries::PointExpr::$curve_wrap(
                    id,
                    $crate::geometries::capabilities::CurvePointFormula::OnCurve { param },
                )
            }

            fn tangent_expr(
                id: $crate::geometries::GeometryId,
                param: $crate::geom::CurveParam,
            ) -> $crate::geometries::VecExpr {
                $crate::geometries::VecExpr::$vec_wrap(
                    id,
                    $crate::geometries::capabilities::VecFormula::TangentAt { param },
                )
            }

            fn normal_expr(
                id: $crate::geometries::GeometryId,
                param: $crate::geom::CurveParam,
            ) -> $crate::geometries::VecExpr {
                $crate::geometries::VecExpr::$vec_wrap(
                    id,
                    $crate::geometries::capabilities::VecFormula::NormalAt { param },
                )
            }
        }
    };
}
pub(crate) use impl_curve_exprs;

/// Implement [`EndpointExprs`] for `$ty`.
///
/// `$endpoint_wrap` is a kind constructor on [`PointExpr`] (e.g. `segment_endpoint`).
macro_rules! impl_endpoint_exprs {
    ($ty:ty, $endpoint_wrap:ident) => {
        impl $crate::geometries::EndpointExprs for $ty {
            fn start_expr(id: $crate::geometries::GeometryId) -> $crate::geometries::PointExpr {
                $crate::geometries::PointExpr::$endpoint_wrap(
                    id,
                    $crate::geometries::capabilities::EndpointPointFormula::StartPoint,
                )
            }

            fn end_expr(id: $crate::geometries::GeometryId) -> $crate::geometries::PointExpr {
                $crate::geometries::PointExpr::$endpoint_wrap(
                    id,
                    $crate::geometries::capabilities::EndpointPointFormula::EndPoint,
                )
            }
        }
    };
}
pub(crate) use impl_endpoint_exprs;
