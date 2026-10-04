//! Parallel free vectors (one residual: 2D cross).

use crate::connectivity::Connectivity;
use crate::constraint::Constraint;
use crate::geometry::{GeometryId, VecExpr};
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct Parallel {
    pub a: VecExpr,
    pub b: VecExpr,
}

impl Constraint for Parallel {
    fn residual_arity(&self) -> usize {
        1
    }

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Connectivity {
        let mut idx = self.a.param_indices(store);
        idx.extend(self.b.param_indices(store));
        Connectivity::new(idx)
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

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}
