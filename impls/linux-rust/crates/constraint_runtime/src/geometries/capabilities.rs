//! Capability-scoped expression factories and shared formula catalogs.
//!
//! - [`CurveExprs`]: on-curve / tangent / normal / length.
//! - [`EndpointExprs`]: start / end (open/join curves only).
//!
//! [`EndpointExprs`] defaults start/end to on-curve at normalized \(t = 0\) / \(t = 1\);
//! kinds MAY override (e.g. segment sparse endpoint formulas). Closed curves implement
//! [`CurveExprs`] only. Factories are kind-associated (not object-safe on
//! [`super::Geometry`]). Prefer [`impl_curve_exprs!`] / [`impl_endpoint_exprs!`];
//! evaluation stays on the kind.

use super::shell::GeometryId;
use super::{PointExpr, ScalarExpr, VecExpr};
use crate::geom::CurveParam;

/// Curve chart expression factories (position / frame / length along the chart).
pub trait CurveExprs {
    fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr;
    fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr;
    fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr;
    fn length_expr(id: GeometryId) -> ScalarExpr;
}

/// Geometric join endpoints (not required for intrinsically periodic closed curves).
///
/// Defaults use [`CurveExprs::on_curve_expr`] at normalized endpoints. Override when
/// the kind has a cheaper formula (connectivity / Jacobian), as [`super::Segment`] does.
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

/// Shared scalar-formula tags for curve length (and future curve scalars).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CurveScalarFormula {
    Length,
}

/// Implement [`CurveExprs`] for `$ty`.
///
/// `$curve_wrap` / `$vec_wrap` / `$scalar_wrap` are kind constructors on
/// [`PointExpr`] / [`VecExpr`] / [`ScalarExpr`] (e.g. `segment_curve`, `segment`,
/// `segment_scalar`).
macro_rules! impl_curve_exprs {
    ($ty:ty, $curve_wrap:ident, $vec_wrap:ident, $scalar_wrap:ident) => {
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

            fn length_expr(id: $crate::geometries::GeometryId) -> $crate::geometries::ScalarExpr {
                $crate::geometries::ScalarExpr::$scalar_wrap(
                    id,
                    $crate::geometries::capabilities::CurveScalarFormula::Length,
                )
            }
        }
    };
}
pub(crate) use impl_curve_exprs;

/// Implement [`EndpointExprs`] for `$ty`.
///
/// - `impl_endpoint_exprs!(Type)` — default start/end via on-curve \(t = 0/1\).
/// - `impl_endpoint_exprs!(Type, endpoint_wrap)` — override with kind endpoint formulas
///   (e.g. `segment_endpoint` + [`EndpointPointFormula`]).
macro_rules! impl_endpoint_exprs {
    ($ty:ty) => {
        impl $crate::geometries::EndpointExprs for $ty {}
    };
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
