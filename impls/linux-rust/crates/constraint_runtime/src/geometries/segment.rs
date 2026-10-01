//! Line segment: State layout, chart sampling, and point/vector formulas.

use super::{Chart, GeometryBody, PointExpr, PointJacobian, VecExpr, VecJacobian};
use crate::geom::{CurveParam, GeometryId, Point2, Vec2};
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

    fn unpack(params: &[f64]) -> Self::Init {
        debug_assert_eq!(params.len(), Self::PARAMETER_COUNT);
        (
            Point2::new(params[0], params[1]),
            Point2::new(params[2], params[3]),
        )
    }
}

impl Segment {
    /// Point formula for the start endpoint of the segment identified by `id`.
    pub fn start_expr(id: GeometryId) -> PointExpr {
        PointFormula::StartPoint { geometry: id }.into()
    }

    /// Point formula for the end endpoint of the segment identified by `id`.
    pub fn end_expr(id: GeometryId) -> PointExpr {
        PointFormula::EndPoint { geometry: id }.into()
    }

    /// Point formula for a chart sample on the segment identified by `id`.
    pub fn on_curve_expr(id: GeometryId, param: CurveParam) -> PointExpr {
        PointFormula::OnCurve {
            geometry: id,
            param,
        }
        .into()
    }

    /// Free-vector formula: segment direction (`end - start`).
    pub fn direction_expr(id: GeometryId) -> VecExpr {
        VecFormula::Direction { geometry: id }.into()
    }

    /// Free-vector formula: curve tangent at `param`.
    pub fn tangent_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecFormula::TangentAt {
            geometry: id,
            param,
        }
        .into()
    }

    /// Free-vector formula: curve normal at `param`.
    pub fn normal_expr(id: GeometryId, param: CurveParam) -> VecExpr {
        VecFormula::NormalAt {
            geometry: id,
            param,
        }
        .into()
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
    pub(crate) fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Point2 {
        match *self {
            PointFormula::StartPoint { geometry } => {
                let (a, _) = endpoints(store, geometry, p);
                a
            }
            PointFormula::EndPoint { geometry } => {
                let (_, b) = endpoints(store, geometry, p);
                b
            }
            PointFormula::OnCurve { geometry, param } => {
                let (a, b) = endpoints(store, geometry, p);
                let t = param.chart_t();
                Point2::new((1.0 - t) * a.x + t * b.x, (1.0 - t) * a.y + t * b.y)
            }
        }
    }

    pub(crate) fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        match *self {
            PointFormula::StartPoint { geometry } => {
                let o = segment_block(store, geometry).start;
                vec![o, o + 1]
            }
            PointFormula::EndPoint { geometry } => {
                let o = segment_block(store, geometry).start + 2;
                vec![o, o + 1]
            }
            PointFormula::OnCurve { geometry, .. } => {
                store
                    .expect_chart::<Segment>(geometry)
                    .connectivity()
                    .global_indices
            }
        }
    }

    pub(crate) fn eval_point_jacobian(
        &self,
        store: &impl GeometryStore,
        _p: &State,
    ) -> PointJacobian {
        match *self {
            PointFormula::StartPoint { .. } | PointFormula::EndPoint { .. } => {
                PointJacobian::endpoint_identity()
            }
            PointFormula::OnCurve { geometry, param } => {
                let g = store.expect_chart::<Segment>(geometry);
                let n = g.connectivity().len();
                debug_assert_eq!(n, Segment::PARAMETER_COUNT);
                let mut j = PointJacobian::zeros(n);
                let t = param.chart_t();
                j.set_dx_dy(0, 1.0 - t, 0.0);
                j.set_dx_dy(1, 0.0, 1.0 - t);
                j.set_dx_dy(2, t, 0.0);
                j.set_dx_dy(3, 0.0, t);
                j
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
        /// Chart parameter; unused on a segment (γ' is constant).
        #[allow(dead_code)]
        param: CurveParam,
    },
    NormalAt {
        geometry: GeometryId,
        /// Chart parameter; unused on a segment (normal from constant tangent).
        #[allow(dead_code)]
        param: CurveParam,
    },
}

impl VecFormula {
    pub(crate) fn evaluate(&self, store: &impl GeometryStore, p: &State) -> Vec2 {
        match *self {
            VecFormula::Direction { geometry } | VecFormula::TangentAt { geometry, .. } => {
                // γ'(t) = B - A (constant on a segment); length = segment length
                let (a, b) = endpoints(store, geometry, p);
                b - a
            }
            VecFormula::NormalAt { geometry, .. } => {
                let (a, b) = endpoints(store, geometry, p);
                (b - a).left_normal()
            }
        }
    }

    pub(crate) fn param_indices(&self, store: &impl GeometryStore) -> Vec<usize> {
        store
            .expect_chart::<Segment>(self.referenced_geometry())
            .connectivity()
            .global_indices
    }

    pub(crate) fn eval_vec_jacobian(
        &self,
        _store: &impl GeometryStore,
        _p: &State,
    ) -> VecJacobian {
        match *self {
            VecFormula::Direction { .. } | VecFormula::TangentAt { .. } => {
                // v = end - start on params [x1,y1,x2,y2]
                let mut j = VecJacobian::zeros(Segment::PARAMETER_COUNT);
                j.set_dx_dy(0, -1.0, 0.0);
                j.set_dx_dy(1, 0.0, -1.0);
                j.set_dx_dy(2, 1.0, 0.0);
                j.set_dx_dy(3, 0.0, 1.0);
                j
            }
            VecFormula::NormalAt { .. } => {
                // n = (-vy, vx) of direction ⇒ rotate each column of J_dir leftward
                let mut j = VecJacobian::zeros(Segment::PARAMETER_COUNT);
                j.set_dx_dy(0, 0.0, -1.0);
                j.set_dx_dy(1, 1.0, 0.0);
                j.set_dx_dy(2, 0.0, 1.0);
                j.set_dx_dy(3, -1.0, 0.0);
                j
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

fn segment_block(store: &impl GeometryStore, id: GeometryId) -> std::ops::Range<usize> {
    store.expect_chart::<Segment>(id).param_block()
}

fn endpoints(store: &impl GeometryStore, id: GeometryId, p: &State) -> (Point2, Point2) {
    Segment::unpack(&p.as_slice()[segment_block(store, id)])
}

#[cfg(test)]
#[path = "tests_segment.rs"]
mod tests_segment;
