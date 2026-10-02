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
/// the kind has a cheaper formula (connectivity / Jacobian), as
/// [`crate::geometry_kinds::Segment`] does.
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
/// `$point_key` / `$vec_key` / `$scalar_key` are variants of
/// [`super::expr::PointFormulaKeyed`] / [`super::expr::VecFormulaKeyed`] /
/// [`super::expr::ScalarFormulaKeyed`] (e.g. `SegmentCurve`, `Segment`, `Segment`).
macro_rules! impl_curve_exprs {
    ($ty:ty, $point_key:ident, $vec_key:ident, $scalar_key:ident) => {
        impl $crate::geometry::CurveExprs for $ty {
            fn on_curve_expr(
                id: $crate::geometry::GeometryId,
                param: $crate::geom::CurveParam,
            ) -> $crate::geometry::PointExpr {
                $crate::geometry::PointExpr::new(
                    id,
                    $crate::geometry::expr::PointFormulaKeyed::$point_key(
                        $crate::geometry::capabilities::CurvePointFormula::OnCurve { param },
                    ),
                )
            }

            fn tangent_expr(
                id: $crate::geometry::GeometryId,
                param: $crate::geom::CurveParam,
            ) -> $crate::geometry::VecExpr {
                $crate::geometry::VecExpr::new(
                    id,
                    $crate::geometry::expr::VecFormulaKeyed::$vec_key(
                        $crate::geometry::capabilities::VecFormula::TangentAt { param },
                    ),
                )
            }

            fn normal_expr(
                id: $crate::geometry::GeometryId,
                param: $crate::geom::CurveParam,
            ) -> $crate::geometry::VecExpr {
                $crate::geometry::VecExpr::new(
                    id,
                    $crate::geometry::expr::VecFormulaKeyed::$vec_key(
                        $crate::geometry::capabilities::VecFormula::NormalAt { param },
                    ),
                )
            }

            fn length_expr(id: $crate::geometry::GeometryId) -> $crate::geometry::ScalarExpr {
                $crate::geometry::ScalarExpr::new(
                    id,
                    $crate::geometry::expr::ScalarFormulaKeyed::$scalar_key(
                        $crate::geometry::capabilities::CurveScalarFormula::Length,
                    ),
                )
            }
        }
    };
}
pub(crate) use impl_curve_exprs;

/// Implement [`EndpointExprs`] for `$ty`.
///
/// - `impl_endpoint_exprs!(Type)` — default start/end via on-curve \(t = 0/1\).
/// - `impl_endpoint_exprs!(Type, $point_key)` — override with kind endpoint formulas
///   (`$point_key` is a [`super::expr::PointFormulaKeyed`] variant, e.g. `SegmentEndpoint`).
macro_rules! impl_endpoint_exprs {
    ($ty:ty) => {
        impl $crate::geometry::EndpointExprs for $ty {}
    };
    ($ty:ty, $point_key:ident) => {
        impl $crate::geometry::EndpointExprs for $ty {
            fn start_expr(id: $crate::geometry::GeometryId) -> $crate::geometry::PointExpr {
                $crate::geometry::PointExpr::new(
                    id,
                    $crate::geometry::expr::PointFormulaKeyed::$point_key(
                        $crate::geometry::capabilities::EndpointPointFormula::StartPoint,
                    ),
                )
            }

            fn end_expr(id: $crate::geometry::GeometryId) -> $crate::geometry::PointExpr {
                $crate::geometry::PointExpr::new(
                    id,
                    $crate::geometry::expr::PointFormulaKeyed::$point_key(
                        $crate::geometry::capabilities::EndpointPointFormula::EndPoint,
                    ),
                )
            }
        }
    };
}
pub(crate) use impl_endpoint_exprs;
