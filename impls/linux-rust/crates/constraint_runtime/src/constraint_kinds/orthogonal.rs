//! Orthogonal free vectors (one residual: dot product).

use crate::constraint::ConstraintKind;
use crate::geometry::{GeometryId, VecExpr};
use crate::mapper::Mapper;
use crate::matrix::MatrixWrite;
use crate::vector::{VecWrite, Vector};
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

    fn eval_residual<R: VecWrite>(
        &self,
        store: &dyn GeometryStore,
        p: &Vector,
        out: &mut R,
    ) {
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        out[0] = va.dot(vb);
    }

    fn accumulate_jacobian<J: MatrixWrite>(
        &self,
        store: &dyn GeometryStore,
        p: &Vector,
        j: &mut J,
    ) {
        // r = va · vb; ∂r/∂va = vb, ∂r/∂vb = va
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        let na = self.a.param_indices(store).len();
        self.a.accumulate_contracted(store, p, j, vb, 1.0);
        let mut jb = j.block_slice(0, na, j.nrows(), j.ncols() - na);
        self.b.accumulate_contracted(store, p, &mut jb, va, 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
