//! Orthogonal free vectors (one residual: dot product).

use crate::constraint::ConstraintKind;
use crate::geometry::{GeometryId, VecExpr};
use crate::mapper::Mapper;
use crate::matrix::{JacCols, JacView, VecView};
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct Orthogonal {
    pub a: VecExpr,
    pub b: VecExpr,
}

impl ConstraintKind for Orthogonal {
    const RESIDUAL_ARITY: usize = 1;

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize> {
        Mapper::join_indices([self.a.param_indices(store), self.b.param_indices(store)])
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut dyn VecView) {
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        out[0] = va.dot(vb);
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut dyn JacView) {
        // r = va · vb; ∂r/∂va = vb, ∂r/∂vb = va
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        let na = self.a.param_indices(store).len();
        self.a.accumulate_contracted(store, p, j, vb, 1.0);
        let mut jb = JacCols::from(j, na);
        self.b.accumulate_contracted(store, p, &mut jb, va, 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
