//! Capability-scoped expression factories for geometry kinds.
//!
//! - [`CurveExprs`]: every curve (including closed/periodic) — on-curve / tangent / normal.
//! - [`EndpointExprs`]: open/join-capable curves only — start / end.
//!
//! Closed curves implement [`CurveExprs`] only. Factories are kind-associated
//! (not object-safe on [`super::Geometry`]).

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
