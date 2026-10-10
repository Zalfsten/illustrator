//! Parallel free vectors (one residual: 2D cross).

use crate::constraint::ConstraintKind;
use crate::expr::VecExpr;
use crate::geometry::GeometryId;
use crate::mapper::Mapper;
use crate::matrix::MatrixAccess;
use crate::store::GeometryStore;
use crate::vector::{VecAccess, Vector};

#[derive(Clone, Debug, PartialEq)]
pub struct Parallel {
    pub a: VecExpr,
    pub b: VecExpr,
}

impl ConstraintKind for Parallel {
    const RESIDUAL_ARITY: usize = 1;

    fn param_connectivity<S: GeometryStore + ?Sized>(&self, store: &S) -> Vec<usize> {
        Mapper::join_indices([self.a.param_indices(store), self.b.param_indices(store)])
    }

    fn eval_residual<S: GeometryStore + ?Sized, R: VecAccess>(
        &self,
        store: &S,
        p: &Vector,
        out: &mut R,
    ) {
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        out[0] = va.cross(vb);
    }

    fn accumulate_jacobian<S: GeometryStore + ?Sized, J: MatrixAccess>(
        &self,
        store: &S,
        p: &Vector,
        j: &mut J,
    ) {
        // r = va × vb; ∂r/∂va = vb.right_normal(), ∂r/∂vb = va.left_normal()
        let va = self.a.evaluate(store, p);
        let vb = self.b.evaluate(store, p);
        let na = self.a.param_count();
        self.a
            .accumulate_contracted(store, p, j, vb.right_normal(), 1.0);
        let mut jb = j.block_slice(0, na, j.nrows(), j.ncols() - na);
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
