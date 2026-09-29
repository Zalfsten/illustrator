//! Opaque point/vector expressions and their evaluation against State.
//!
//! Factories live on [`Segment`] (and later other charts). Constraints, Mesh, and
//! Interact call [`PointExpr::evaluate`] / [`VecExpr::evaluate`] — they do not
//! match on formula kinds. Geometry layout stays in [`crate::geom`].

use crate::geom::{CurveParam, GeometryBody, GeometryId, Point2, Segment, Vec2};
use crate::store::GeometryStore;

/// Private formula catalog for [`PointExpr`] (Model B: opaque handle, closed enum).
#[derive(Clone, Copy, Debug, PartialEq)]
enum PointFormula {
    StartPoint { geometry: GeometryId },
    EndPoint { geometry: GeometryId },
    OnCurve {
        geometry: GeometryId,
        param: CurveParam,
    },
}

/// Opaque point expression: map from State parameters to a [`Point2`].
///
/// Construct via geometry factories (e.g. [`Segment::start_expr`]); constraints
/// and Interact only call [`Self::evaluate`] / Jacobian helpers — they do not
/// match on formula kinds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointExpr(PointFormula);

impl PointExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Point2 {
        match self.0 {
            PointFormula::StartPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                segment_start(g.param_offset, p)
            }
            PointFormula::EndPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                segment_end(g.param_offset, p)
            }
            PointFormula::OnCurve { geometry, param } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                segment_position_at(g.param_offset, param.chart_t(), p)
            }
        }
    }

    pub fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match self.0 {
            PointFormula::StartPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let o = g.param_offset;
                vec![o, o + 1]
            }
            PointFormula::EndPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let o = g.param_offset + 2;
                vec![o, o + 1]
            }
            PointFormula::OnCurve { geometry, .. } => {
                store.expect_geometry(geometry).connectivity().global_indices
            }
        }
    }

    pub fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        _p: &[f64],
        dx: &mut [f64],
        dy: &mut [f64],
    ) {
        match self.0 {
            PointFormula::StartPoint { .. } | PointFormula::EndPoint { .. } => {
                assert_eq!(dx.len(), 2);
                assert_eq!(dy.len(), 2);
                dx.fill(0.0);
                dy.fill(0.0);
                dx[0] = 1.0;
                dy[1] = 1.0;
            }
            PointFormula::OnCurve { geometry, param } => {
                let g = store.expect_geometry(geometry);
                let n = g.connectivity().len();
                assert_eq!(dx.len(), n);
                assert_eq!(dy.len(), n);
                dx.fill(0.0);
                dy.fill(0.0);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let t = param.chart_t();
                dx[0] = 1.0 - t;
                dy[1] = 1.0 - t;
                dx[2] = t;
                dy[3] = t;
            }
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        match self.0 {
            PointFormula::StartPoint { geometry }
            | PointFormula::EndPoint { geometry }
            | PointFormula::OnCurve { geometry, .. } => geometry,
        }
    }
}

/// Private formula catalog for [`VecExpr`] (same opacity principle as [`PointExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
enum VecFormula {
    SegmentDirection { geometry: GeometryId },
    TangentAt {
        geometry: GeometryId,
        param: CurveParam,
    },
    NormalAt {
        geometry: GeometryId,
        param: CurveParam,
    },
}

/// Opaque free-vector expression: map from State parameters to a [`Vec2`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VecExpr(VecFormula);

impl VecExpr {
    pub fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Vec2 {
        match self.0 {
            VecFormula::SegmentDirection { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                segment_direction(g.param_offset, p)
            }
            VecFormula::TangentAt { geometry, param } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                // γ'(t) = B - A (constant on a segment); length = segment length
                let _ = param;
                segment_direction(g.param_offset, p)
            }
            VecFormula::NormalAt { geometry, param } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let _ = param;
                segment_direction(g.param_offset, p).left_normal()
            }
        }
    }

    pub fn referenced_geometry(&self) -> GeometryId {
        match self.0 {
            VecFormula::SegmentDirection { geometry }
            | VecFormula::TangentAt { geometry, .. }
            | VecFormula::NormalAt { geometry, .. } => geometry,
        }
    }
}

impl Segment {
    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr(PointFormula::StartPoint { geometry: id })
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr(PointFormula::EndPoint { geometry: id })
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr(PointFormula::OnCurve {
            geometry: id,
            param,
        })
    }

    /// Free-vector formula: segment direction (`end - start`).
    pub fn direction_expr(id: GeometryId) -> VecExpr {
        VecExpr(VecFormula::SegmentDirection { geometry: id })
    }

    /// Free-vector formula: curve tangent at `param`.
    pub fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr(VecFormula::TangentAt {
            geometry: id,
            param,
        })
    }

    /// Free-vector formula: curve normal at `param`.
    pub fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr(VecFormula::NormalAt {
            geometry: id,
            param,
        })
    }
}

fn segment_start(offset: usize, p: &[f64]) -> Point2 {
    Point2::new(p[offset], p[offset + 1])
}

fn segment_end(offset: usize, p: &[f64]) -> Point2 {
    Point2::new(p[offset + 2], p[offset + 3])
}

fn segment_direction(offset: usize, p: &[f64]) -> Vec2 {
    segment_end(offset, p) - segment_start(offset, p)
}

fn segment_position_at(offset: usize, t: f64, p: &[f64]) -> Point2 {
    let a = segment_start(offset, p);
    let b = segment_end(offset, p);
    Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
}

#[cfg(test)]
#[path = "tests_expr.rs"]
mod tests_expr;
