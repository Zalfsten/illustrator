//! Point-point coincidence (two residual rows).

use crate::constraint::ConstraintKind;
use crate::geometry::{GeometryId, PointExpr};
use crate::mapper::Mapper;
use crate::matrix::MatrixAccess;
use crate::vector::{VecAccess, Vector};
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct Congruence {
    pub a: PointExpr,
    pub b: PointExpr,
}

impl Congruence {
    /// Partner expression if `point` is one side of this identification.
    pub fn other(&self, point: &PointExpr) -> Option<PointExpr> {
        if &self.a == point {
            Some(self.b)
        } else if &self.b == point {
            Some(self.a)
        } else {
            None
        }
    }
}

impl ConstraintKind for Congruence {
    const RESIDUAL_ARITY: usize = 2;

    fn param_connectivity<S: GeometryStore + ?Sized>(&self, store: &S) -> Vec<usize> {
        Mapper::join_indices([self.a.param_indices(store), self.b.param_indices(store)])
    }

    fn eval_residual<S: GeometryStore + ?Sized, R: VecAccess>(
        &self,
        store: &S,
        p: &Vector,
        out: &mut R,
    ) {
        let pa = self.a.evaluate(store, p);
        let pb = self.b.evaluate(store, p);
        out[0] = pa.x - pb.x;
        out[1] = pa.y - pb.y;
    }

    fn accumulate_jacobian<S: GeometryStore + ?Sized, J: MatrixAccess>(
        &self,
        store: &S,
        p: &Vector,
        j: &mut J,
    ) {
        // r = a - b ⇒ ∂r/∂a = Ja, ∂r/∂b = -Jb
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
