//! Parallel free vectors (one residual: 2D cross).

use crate::constraint::{ConstraintKind, LocalJacobian};
use crate::geometry::{GeometryId, VecExpr};
use crate::mapper::Mapper;
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct Parallel {
    pub a: VecExpr,
    pub b: VecExpr,
}

impl ConstraintKind for Parallel {
    const RESIDUAL_ARITY: usize = 1;

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize> {
        Mapper::join_indices([self.a.param_indices(store), self.b.param_indices(store)])
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        out[0] = va.cross(vb);
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut LocalJacobian<'_>) {
        // r = va × vb; ∂r/∂va = vb.right_normal(), ∂r/∂vb = va.left_normal()
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        let ja = self.a.eval_vec_jacobian(store, p);
        let jb = self.b.eval_vec_jacobian(store, p);
        let mut place = j.placer();
        place.place_scalar(&ja.contract(vb.right_normal()), 1.0);
        place.place_scalar(&jb.contract(va.left_normal()), 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
