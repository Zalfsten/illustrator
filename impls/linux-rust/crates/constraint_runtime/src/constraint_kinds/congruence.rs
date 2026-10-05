//! Point-point coincidence (two residual rows).

use crate::constraint::{ConstraintEntry, ConstraintKind, LocalJacobian};
use crate::geometry::{GeometryId, PointExpr};
use crate::mapper::Mapper;
use crate::state::State;
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

/// Partners identified with `point` by any [`Congruence`] in `constraints`.
pub fn points_identified_with(
    constraints: &[ConstraintEntry],
    point: &PointExpr,
) -> Vec<PointExpr> {
    constraints
        .iter()
        .filter_map(|e| {
            e.body()
                .as_any()
                .downcast_ref::<Congruence>()
                .and_then(|g| g.other(point))
        })
        .collect()
}

impl ConstraintKind for Congruence {
    const RESIDUAL_ARITY: usize = 2;

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize> {
        Mapper::join_indices([self.a.param_indices(store), self.b.param_indices(store)])
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        let pa = self.a.evaluate(store, p);
        let pb = self.b.evaluate(store, p);
        out[0] = pa.x - pb.x;
        out[1] = pa.y - pb.y;
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut LocalJacobian<'_>) {
        // r = a - b ⇒ ∂r/∂a = Ja, ∂r/∂b = -Jb
        let ja = self.a.eval_point_jacobian(store, p);
        let jb = self.b.eval_point_jacobian(store, p);
        let mut place = j.placer();
        place.place_point(&ja, 1.0);
        place.place_point(&jb, -1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        crate::constraint::unique_geometry_ids([
            self.a.referenced_geometry(),
            self.b.referenced_geometry(),
        ])
    }
}
