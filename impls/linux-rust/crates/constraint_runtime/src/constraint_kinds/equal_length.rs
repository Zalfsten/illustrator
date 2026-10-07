//! Equal length expressions (one residual).

use crate::constraint::ConstraintKind;
use crate::geometry::{GeometryId, LengthExpr};
use crate::mapper::Mapper;
use crate::matrix::{JacCols, JacView, VecView};
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

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut dyn VecView) {
        out[0] = self.a.evaluate(store, p).value() - self.b.evaluate(store, p).value();
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut dyn JacView) {
        // r = ℓ_a - ℓ_b ⇒ ∂r/∂a = Ja, ∂r/∂b = -Jb
        let na = self.a.param_indices(store).len();
        self.a.accumulate_jacobian(store, p, j, 1.0);
        let mut jb = JacCols::from(j, na);
        self.b.accumulate_jacobian(store, p, &mut jb, -1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
