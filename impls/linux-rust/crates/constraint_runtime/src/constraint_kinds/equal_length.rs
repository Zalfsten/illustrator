//! Equal length expressions (one residual).

use crate::constraint::ConstraintKind;
use crate::geometry::{GeometryId, LengthExpr};
use crate::mapper::Mapper;
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct EqualLength {
    pub a: LengthExpr,
    pub b: LengthExpr,
}

impl ConstraintKind for EqualLength {
    const RESIDUAL_ARITY: usize = 1;

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize> {
        Mapper::join_indices([self.a.param_indices(store), self.b.param_indices(store)])
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        out[0] = self.a.evaluate(store, p).value() - self.b.evaluate(store, p).value();
    }

    fn accumulate_jacobian(
        &self,
        store: &dyn GeometryStore,
        p: &State,
        _n_local: usize,
        out: &mut [f64],
    ) {
        let ja = self.a.eval_length_jacobian(store, p);
        let jb = self.b.eval_length_jacobian(store, p);
        let na = ja.n();
        let nb = jb.n();
        debug_assert_eq!(na, self.a.param_indices(store).len());
        debug_assert_eq!(nb, self.b.param_indices(store).len());
        for i in 0..na {
            out[i] = ja.ds()[i];
        }
        for j in 0..nb {
            out[na + j] = -jb.ds()[j];
        }
    }

    fn references_geometry(&self, id: GeometryId) -> bool {
        self.a.referenced_geometry() == id || self.b.referenced_geometry() == id
    }
}
