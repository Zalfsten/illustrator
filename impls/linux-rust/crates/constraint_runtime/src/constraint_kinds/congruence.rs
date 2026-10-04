//! Point-point coincidence (two residual rows).

use crate::connectivity::Connectivity;
use crate::constraint::Constraint;
use crate::geometry::{GeometryId, PointExpr};
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct Congruence {
    pub a: PointExpr,
    pub b: PointExpr,
}

impl Constraint for Congruence {
    fn residual_arity(&self) -> usize {
        2
    }

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Connectivity {
        let mut idx = self.a.param_indices(store);
        idx.extend(self.b.param_indices(store));
        Connectivity::new(idx)
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        let pa = self.a.evaluate(store, p);
        let pb = self.b.evaluate(store, p);
        out[0] = pa.x - pb.x;
        out[1] = pa.y - pb.y;
    }

    fn accumulate_jacobian(
        &self,
        store: &dyn GeometryStore,
        p: &State,
        n_local: usize,
        out: &mut [f64],
    ) {
        let ja = self.a.eval_point_jacobian(store, p);
        let jb = self.b.eval_point_jacobian(store, p);
        let na = ja.n();
        let nb = jb.n();
        debug_assert_eq!(na, self.a.param_indices(store).len());
        debug_assert_eq!(nb, self.b.param_indices(store).len());
        for i in 0..na {
            out[0 * n_local + i] = ja.dx()[i];
            out[1 * n_local + i] = ja.dy()[i];
        }
        for j in 0..nb {
            out[0 * n_local + na + j] = -jb.dx()[j];
            out[1 * n_local + na + j] = -jb.dy()[j];
        }
    }

    fn references_geometry(&self, id: GeometryId) -> bool {
        self.a.referenced_geometry() == id || self.b.referenced_geometry() == id
    }

    fn co_dragged_points(&self, dragged: &PointExpr) -> Vec<PointExpr> {
        if &self.a == dragged {
            vec![self.b]
        } else if &self.b == dragged {
            vec![self.a]
        } else {
            Vec::new()
        }
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}
