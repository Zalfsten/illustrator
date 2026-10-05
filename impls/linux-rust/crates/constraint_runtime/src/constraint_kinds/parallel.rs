//! Parallel free vectors (one residual: 2D cross).

use crate::constraint::ConstraintKind;
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

    fn accumulate_jacobian(
        &self,
        store: &dyn GeometryStore,
        p: &State,
        _n_local: usize,
        out: &mut [f64],
    ) {
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        let ja = self.a.eval_vec_jacobian(store, p);
        let jb = self.b.eval_vec_jacobian(store, p);
        let na = ja.n();
        let nb = jb.n();
        debug_assert_eq!(na, self.a.param_indices(store).len());
        debug_assert_eq!(nb, self.b.param_indices(store).len());
        // r = va × vb; ∂r/∂va = (vb.y, -vb.x), ∂r/∂vb = (-va.y, va.x)
        for i in 0..na {
            out[i] = vb.y * ja.dx()[i] - vb.x * ja.dy()[i];
        }
        for j in 0..nb {
            out[na + j] = -va.y * jb.dx()[j] + va.x * jb.dy()[j];
        }
    }

    fn references_geometry(&self, id: GeometryId) -> bool {
        self.a.referenced_geometry() == id || self.b.referenced_geometry() == id
    }
}
