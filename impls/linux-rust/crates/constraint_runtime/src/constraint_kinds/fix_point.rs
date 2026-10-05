//! Fix a point expression to a target location (two residual rows).

use crate::constraint::{ConstraintKind, LocalJacobian};
use crate::geom::Point2;
use crate::geometry::{GeometryId, PointExpr};
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct FixPoint {
    pub point: PointExpr,
    pub target: Point2,
}

impl ConstraintKind for FixPoint {
    const RESIDUAL_ARITY: usize = 2;

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize> {
        self.point.param_indices(store)
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        let q = self.point.evaluate(store, p);
        out[0] = q.x - self.target.x;
        out[1] = q.y - self.target.y;
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut LocalJacobian<'_>) {
        let jp = self.point.eval_point_jacobian(store, p);
        j.placer().place_point(&jp, 1.0);
    }

    fn references_geometry(&self, id: GeometryId) -> bool {
        self.point.referenced_geometry() == id
    }
}
