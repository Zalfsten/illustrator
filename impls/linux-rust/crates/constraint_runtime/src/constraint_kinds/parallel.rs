//! Parallel free vectors (one residual: 2D cross).

use crate::constraint::ConstraintKind;
use crate::geometry::{GeometryId, VecExpr};
use crate::mapper::Mapper;
use crate::matrix::{JacCols, JacView, VecView};
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

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut dyn VecView) {
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        out[0] = va.cross(vb);
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut dyn JacView) {
        // r = va × vb; ∂r/∂va = vb.right_normal(), ∂r/∂vb = va.left_normal()
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        let na = self.a.param_indices(store).len();
        self.a
            .accumulate_contracted(store, p, j, vb.right_normal(), 1.0);
        let mut jb = JacCols::from(j, na);
        self.b
            .accumulate_contracted(store, p, &mut jb, va.left_normal(), 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
