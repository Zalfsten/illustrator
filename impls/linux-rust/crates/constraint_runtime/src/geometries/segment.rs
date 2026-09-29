//! Line segment: State layout, chart sampling, and point/vector formulas.

use crate::expr::{PointExpr, VecExpr};
use crate::geom::{
    CurveParam, Geometry, GeometryBody, GeometryId, GeometryRole, Point2, Vec2,
};
use crate::store::GeometryStore;

/// Line-segment chart. Parameters live at `p[offset..offset+STRIDE]` on the
/// enclosing [`Geometry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment;

impl Segment {
    /// Number of State parameters for one segment (`x1,y1,x2,y2`).
    pub const STRIDE: usize = 4;

    /// Initial State parameters for a segment from endpoints `(a, b)`.
    pub fn params(a: Point2, b: Point2) -> [f64; 4] {
        [a.x, a.y, b.x, b.y]
    }

    /// Build a document [`Geometry`] with a segment body.
    pub fn into_geometry(id: GeometryId, role: GeometryRole, param_offset: usize) -> Geometry {
        Geometry {
            id,
            role,
            param_offset,
            body: GeometryBody::Segment,
        }
    }

    pub fn all_param_indices(offset: usize) -> [usize; 4] {
        [offset, offset + 1, offset + 2, offset + 3]
    }

    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr::from_segment(PointFormula::StartPoint { geometry: id })
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr::from_segment(PointFormula::EndPoint { geometry: id })
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr::from_segment(PointFormula::OnCurve {
            geometry: id,
            param,
        })
    }

    /// Free-vector formula: segment direction (`end - start`).
    pub fn direction_expr(id: GeometryId) -> VecExpr {
        VecExpr::from_segment(VecFormula::Direction { geometry: id })
    }

    /// Free-vector formula: curve tangent at `param`.
    pub fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::from_segment(VecFormula::TangentAt {
            geometry: id,
            param,
        })
    }

    /// Free-vector formula: curve normal at `param`.
    pub fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::from_segment(VecFormula::NormalAt {
            geometry: id,
            param,
        })
    }
}

/// Segment-owned point formulas (private to the crate; wrapped by [`PointExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PointFormula {
    StartPoint { geometry: GeometryId },
    EndPoint { geometry: GeometryId },
    OnCurve {
        geometry: GeometryId,
        param: CurveParam,
    },
}

impl PointFormula {
    pub(crate) fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Point2 {
        match *self {
            PointFormula::StartPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                start(g.param_offset, p)
            }
            PointFormula::EndPoint { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                end(g.param_offset, p)
            }
            PointFormula::OnCurve { geometry, param } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                position_at(g.param_offset, param.chart_t(), p)
            }
        }
    }

    pub(crate) fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match *self {
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

    pub(crate) fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        _p: &[f64],
        dx: &mut [f64],
        dy: &mut [f64],
    ) {
        match *self {
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

    pub(crate) fn referenced_geometry(&self) -> GeometryId {
        match *self {
            PointFormula::StartPoint { geometry }
            | PointFormula::EndPoint { geometry }
            | PointFormula::OnCurve { geometry, .. } => geometry,
        }
    }
}

/// Segment-owned free-vector formulas (private to the crate; wrapped by [`VecExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VecFormula {
    Direction { geometry: GeometryId },
    TangentAt {
        geometry: GeometryId,
        param: CurveParam,
    },
    NormalAt {
        geometry: GeometryId,
        param: CurveParam,
    },
}

impl VecFormula {
    pub(crate) fn evaluate(&self, store: &impl GeometryStore, p: &[f64]) -> Vec2 {
        match *self {
            VecFormula::Direction { geometry } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                direction(g.param_offset, p)
            }
            VecFormula::TangentAt { geometry, param } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                // γ'(t) = B - A (constant on a segment); length = segment length
                let _ = param;
                direction(g.param_offset, p)
            }
            VecFormula::NormalAt { geometry, param } => {
                let g = store.expect_geometry(geometry);
                debug_assert!(matches!(g.body, GeometryBody::Segment));
                let _ = param;
                direction(g.param_offset, p).left_normal()
            }
        }
    }

    pub(crate) fn referenced_geometry(&self) -> GeometryId {
        match *self {
            VecFormula::Direction { geometry }
            | VecFormula::TangentAt { geometry, .. }
            | VecFormula::NormalAt { geometry, .. } => geometry,
        }
    }
}

fn start(offset: usize, p: &[f64]) -> Point2 {
    Point2::new(p[offset], p[offset + 1])
}

fn end(offset: usize, p: &[f64]) -> Point2 {
    Point2::new(p[offset + 2], p[offset + 3])
}

fn direction(offset: usize, p: &[f64]) -> Vec2 {
    end(offset, p) - start(offset, p)
}

fn position_at(offset: usize, t: f64, p: &[f64]) -> Point2 {
    let a = start(offset, p);
    let b = end(offset, p);
    Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
