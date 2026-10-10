//! Equal length expressions (one residual).

use crate::constraint::ConstraintKind;
use crate::expr::LengthExpr;
use crate::geometry::GeometryId;
use crate::mapper::Mapper;
use crate::matrix::MatrixAccess;
use crate::vector::{VecAccess, Vector};
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct EqualLength {
    pub a: LengthExpr,
    pub b: LengthExpr,
}

impl ConstraintKind for EqualLength {
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
        out[0] = self.a.evaluate(store, p).value() - self.b.evaluate(store, p).value();
    }

    fn accumulate_jacobian<S: GeometryStore + ?Sized, J: MatrixAccess>(
        &self,
        store: &S,
        p: &Vector,
        j: &mut J,
    ) {
        // r = ℓ_a - ℓ_b ⇒ ∂r/∂a = Ja, ∂r/∂b = -Jb
        let na = self.a.param_count();
        self.a.accumulate_jacobian(store, p, j, 1.0);
        let mut jb = j.block_slice(0, na, j.nrows(), j.ncols() - na);
        self.b.accumulate_jacobian(store, p, &mut jb, -1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
