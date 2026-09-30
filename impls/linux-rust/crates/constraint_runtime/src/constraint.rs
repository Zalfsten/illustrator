//! Structural constraints: local residuals and analytical local Jacobians.

use crate::connectivity::Connectivity;
use crate::geometries::PointExpr;
use crate::geometries::Segment;
use crate::geom::{GeometryId, Point2};
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    Congruence { a: PointExpr, b: PointExpr },
    Parallel { a: GeometryId, b: GeometryId },
    Orthogonal { a: GeometryId, b: GeometryId },
    FixPoint { point: PointExpr, target: Point2 },
}

fn segment_direction(store: &impl GeometryStore, id: GeometryId, p: &State) -> crate::geom::Vec2 {
    Segment::direction_expr(id).evaluate(store, p)
}

impl Constraint {
    pub fn residual_arity(&self) -> usize {
        match self {
            Constraint::Congruence { .. } | Constraint::FixPoint { .. } => 2,
            Constraint::Parallel { .. } | Constraint::Orthogonal { .. } => 1,
        }
    }

    /// Column connectivity: global parameter indices involved in this contribution.
    pub fn param_connectivity(&self, store: &impl GeometryStore) -> Connectivity {
        match self {
            Constraint::Congruence { a, b } => {
                let mut idx = a.param_indices(store);
                idx.extend(b.param_indices(store));
                Connectivity::new(idx)
            }
            Constraint::Parallel { a, b } | Constraint::Orthogonal { a, b } => {
                let mut idx = store.expect_geometry(*a).connectivity().global_indices;
                idx.extend(store.expect_geometry(*b).connectivity().global_indices);
                Connectivity::new(idx)
            }
            Constraint::FixPoint { point, .. } => Connectivity::new(point.param_indices(store)),
        }
    }

    /// Local residual `r_N` in contribution order (length = residual_arity).
    pub fn eval_residual(&self, store: &impl GeometryStore, p: &State, out: &mut [f64]) {
        match self {
            Constraint::Congruence { a, b } => {
                let pa = a.evaluate(store, p);
                let pb = b.evaluate(store, p);
                out[0] = pa.x - pb.x;
                out[1] = pa.y - pb.y;
            }
            Constraint::Parallel { a, b } => {
                let va = segment_direction(store, *a, p);
                let vb = segment_direction(store, *b, p);
                out[0] = va.cross(vb);
            }
            Constraint::Orthogonal { a, b } => {
                let va = segment_direction(store, *a, p);
                let vb = segment_direction(store, *b, p);
                out[0] = va.dot(vb);
            }
            Constraint::FixPoint { point, target } => {
                let q = point.evaluate(store, p);
                out[0] = q.x - target.x;
                out[1] = q.y - target.y;
            }
        }
    }

    /// Local Jacobian `J_N` row-major, shape `(residual_arity × param_connectivity.len())`.
    pub fn eval_jacobian(&self, store: &impl GeometryStore, p: &State, out: &mut [f64]) {
        let n_local = self.param_connectivity(store).len();
        let m = self.residual_arity();
        assert_eq!(out.len(), m * n_local);
        out.fill(0.0);

        match self {
            Constraint::Congruence { a, b } => {
                let ja = a.eval_point_jacobian(store, p);
                let jb = b.eval_point_jacobian(store, p);
                let na = ja.n();
                let nb = jb.n();
                debug_assert_eq!(na, a.param_indices(store).len());
                debug_assert_eq!(nb, b.param_indices(store).len());
                for i in 0..na {
                    out[0 * n_local + i] = ja.dx()[i];
                    out[1 * n_local + i] = ja.dy()[i];
                }
                for j in 0..nb {
                    out[0 * n_local + na + j] = -jb.dx()[j];
                    out[1 * n_local + na + j] = -jb.dy()[j];
                }
            }
            Constraint::Parallel { a, b } => {
                let va = segment_direction(store, *a, p);
                let vb = segment_direction(store, *b, p);
                out[0] = -vb.y;
                out[1] = vb.x;
                out[2] = vb.y;
                out[3] = -vb.x;
                out[4] = va.y;
                out[5] = -va.x;
                out[6] = -va.y;
                out[7] = va.x;
            }
            Constraint::Orthogonal { a, b } => {
                let va = segment_direction(store, *a, p);
                let vb = segment_direction(store, *b, p);
                out[0] = -vb.x;
                out[1] = -vb.y;
                out[2] = vb.x;
                out[3] = vb.y;
                out[4] = -va.x;
                out[5] = -va.y;
                out[6] = va.x;
                out[7] = va.y;
            }
            Constraint::FixPoint { point, .. } => {
                let j = point.eval_point_jacobian(store, p);
                debug_assert_eq!(j.n(), point.param_indices(store).len());
                for i in 0..j.n() {
                    out[0 * n_local + i] = j.dx()[i];
                    out[1 * n_local + i] = j.dy()[i];
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_constraint.rs"]
mod tests_constraint;
