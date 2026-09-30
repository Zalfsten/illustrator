//! Line segment: State layout, chart sampling, and point/vector formulas.

use super::{Chart, GeometryBody, PointExpr, PointJacobian, VecExpr};
use crate::geom::{CurveParam, GeometryId, Point2, Vec2};
#[cfg(test)]
use crate::geom::{Geometry, GeometryRole};
use crate::state::State;
use crate::store::GeometryStore;

/// Line-segment chart. Parameters live at
/// `p[offset..offset+PARAMETER_COUNT]` on the enclosing [`Geometry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Segment;

impl Chart for Segment {
    type Init = (Point2, Point2);

    const BODY: GeometryBody = GeometryBody::Segment;

    /// Four endpoint scalars: `x1,y1,x2,y2`.
    const PARAMETER_COUNT: usize = 4;

    fn pack((a, b): Self::Init) -> impl AsRef<[f64]> {
        [a.x, a.y, b.x, b.y]
    }
}

impl Segment {
    /// Build a document [`Geometry`] with a segment body (crate tests).
    #[cfg(test)]
    pub(crate) fn into_geometry(id: GeometryId, role: GeometryRole, param_offset: usize) -> Geometry {
        Geometry {
            id,
            role,
            param_offset,
            body: GeometryBody::Segment,
        }
    }

    pub fn all_param_indices(offset: usize) -> [usize; 4] {
        debug_assert_eq!(Self::PARAMETER_COUNT, 4);
        let block = offset..offset + Self::PARAMETER_COUNT;
        [
            block.start,
            block.start + 1,
            block.start + 2,
            block.start + 3,
        ]
    }

    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        PointExpr::from_segment(PointFormula::StartPoint(StartPoint { geometry: id }))
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        PointExpr::from_segment(PointFormula::EndPoint(EndPoint { geometry: id }))
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointExpr::from_segment(PointFormula::OnCurve(OnCurve {
            geometry: id,
            param,
        }))
    }

    /// Free-vector formula: segment direction (`end - start`).
    pub fn direction_expr(id: GeometryId) -> VecExpr {
        VecExpr::from_segment(VecFormula::Direction(Direction { geometry: id }))
    }

    /// Free-vector formula: curve tangent at `param`.
    pub fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::from_segment(VecFormula::TangentAt(TangentAt {
            geometry: id,
            param,
        }))
    }

    /// Free-vector formula: curve normal at `param`.
    pub fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecExpr::from_segment(VecFormula::NormalAt(NormalAt {
            geometry: id,
            param,
        }))
    }
}

/// Segment-owned point formulas (private to the crate; wrapped by [`PointExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PointFormula {
    StartPoint(StartPoint),
    EndPoint(EndPoint),
    OnCurve(OnCurve),
}

impl PointFormula {
    pub(crate) fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        match self {
            PointFormula::StartPoint(f) => f.evaluate(store, p),
            PointFormula::EndPoint(f) => f.evaluate(store, p),
            PointFormula::OnCurve(f) => f.evaluate(store, p),
        }
    }

    pub(crate) fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match self {
            PointFormula::StartPoint(f) => f.param_indices(store),
            PointFormula::EndPoint(f) => f.param_indices(store),
            PointFormula::OnCurve(f) => f.param_indices(store),
        }
    }

    pub(crate) fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        p: &State,
    ) -> PointJacobian {
        match self {
            PointFormula::StartPoint(f) => f.eval_point_jacobian(store, p),
            PointFormula::EndPoint(f) => f.eval_point_jacobian(store, p),
            PointFormula::OnCurve(f) => f.eval_point_jacobian(store, p),
        }
    }

    pub(crate) fn referenced_geometry(&self) -> GeometryId {
        match self {
            PointFormula::StartPoint(f) => f.geometry,
            PointFormula::EndPoint(f) => f.geometry,
            PointFormula::OnCurve(f) => f.geometry,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StartPoint {
    geometry: GeometryId,
}

impl StartPoint {
    fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        start(segment_block(store, self.geometry).start, p)
    }

    fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        let o = segment_block(store, self.geometry).start;
        vec![o, o + 1]
    }

    fn eval_point_jacobian(
        &self,
        _store: &impl GeometryStore,
        _p: &State,
    ) -> PointJacobian {
        PointJacobian::endpoint_identity()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EndPoint {
    geometry: GeometryId,
}

impl EndPoint {
    fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        end(segment_block(store, self.geometry).start, p)
    }

    fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        let o = segment_block(store, self.geometry).start + 2;
        vec![o, o + 1]
    }

    fn eval_point_jacobian(
        &self,
        _store: &impl GeometryStore,
        _p: &State,
    ) -> PointJacobian {
        PointJacobian::endpoint_identity()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OnCurve {
    geometry: GeometryId,
    param: CurveParam,
}

impl OnCurve {
    fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        position_at(segment_block(store, self.geometry).start, self.param.chart_t(), p)
    }

    fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        store
            .expect_geometry(self.geometry)
            .connectivity()
            .global_indices
    }

    fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        _p: &State,
    ) -> PointJacobian {
        let g = store.expect_geometry(self.geometry);
        debug_assert!(matches!(g.body, GeometryBody::Segment));
        let n = g.connectivity().len();
        debug_assert_eq!(n, Segment::PARAMETER_COUNT);
        let mut j = PointJacobian::zeros(n);
        let t = self.param.chart_t();
        j.set_dx_dy(0, 1.0 - t, 0.0);
        j.set_dx_dy(1, 0.0, 1.0 - t);
        j.set_dx_dy(2, t, 0.0);
        j.set_dx_dy(3, 0.0, t);
        j
    }
}

/// Segment-owned free-vector formulas (private to the crate; wrapped by [`VecExpr`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum VecFormula {
    Direction(Direction),
    TangentAt(TangentAt),
    NormalAt(NormalAt),
}

impl VecFormula {
    pub(crate) fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        match self {
            VecFormula::Direction(f) => f.evaluate(store, p),
            VecFormula::TangentAt(f) => f.evaluate(store, p),
            VecFormula::NormalAt(f) => f.evaluate(store, p),
        }
    }

    pub(crate) fn referenced_geometry(&self) -> GeometryId {
        match self {
            VecFormula::Direction(f) => f.geometry,
            VecFormula::TangentAt(f) => f.geometry,
            VecFormula::NormalAt(f) => f.geometry,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Direction {
    geometry: GeometryId,
}

impl Direction {
    fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        direction(segment_block(store, self.geometry).start, p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TangentAt {
    geometry: GeometryId,
    /// Chart parameter; unused on a segment (γ' is constant).
    #[allow(dead_code)]
    param: CurveParam,
}

impl TangentAt {
    fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        // γ'(t) = B - A (constant on a segment); length = segment length
        direction(segment_block(store, self.geometry).start, p)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NormalAt {
    geometry: GeometryId,
    /// Chart parameter; unused on a segment (normal from constant tangent).
    #[allow(dead_code)]
    param: CurveParam,
}

impl NormalAt {
    fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        direction(segment_block(store, self.geometry).start, p).left_normal()
    }
}

fn segment_block(store: &impl GeometryStore, id: GeometryId) -> std::ops::Range<usize> {
    let g = store.expect_geometry(id);
    debug_assert!(matches!(g.body, GeometryBody::Segment));
    g.param_block()
}

fn start(offset: usize, p: &State) -> Point2 {
    Point2::new(p[offset], p[offset + 1])
}

fn end(offset: usize, p: &State) -> Point2 {
    Point2::new(p[offset + 2], p[offset + 3])
}

fn direction(offset: usize, p: &State) -> Vec2 {
    end(offset, p) - start(offset, p)
}

fn position_at(offset: usize, t: f64, p: &State) -> Point2 {
    let a = start(offset, p);
    let b = end(offset, p);
    Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
