//! Structural constraints: local residuals and analytical local Jacobians.

use crate::connectivity::Connectivity;
use crate::geometry::{LengthExpr, PointExpr, VecExpr};
use crate::geom::Point2;
use crate::state::State;
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    Congruence { a: PointExpr, b: PointExpr },
    Parallel { a: VecExpr, b: VecExpr },
    Orthogonal { a: VecExpr, b: VecExpr },
    FixPoint { point: PointExpr, target: Point2 },
    EqualLength { a: LengthExpr, b: LengthExpr },
}

impl Constraint {
    pub fn residual_arity(&self) -> usize {
        match self {
            Constraint::Congruence { .. } | Constraint::FixPoint { .. } => 2,
            Constraint::Parallel { .. }
            | Constraint::Orthogonal { .. }
            | Constraint::EqualLength { .. } => 1,
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
                let mut idx = a.param_indices(store);
                idx.extend(b.param_indices(store));
                Connectivity::new(idx)
            }
            Constraint::FixPoint { point, .. } => Connectivity::new(point.param_indices(store)),
            Constraint::EqualLength { a, b } => {
                let mut idx = a.param_indices(store);
                idx.extend(b.param_indices(store));
                Connectivity::new(idx)
            }
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
                let va = a.evaluate(store, p);
                let vb = b.evaluate(store, p);
                out[0] = va.cross(vb);
            }
            Constraint::Orthogonal { a, b } => {
                let va = a.evaluate(store, p);
                let vb = b.evaluate(store, p);
                out[0] = va.dot(vb);
            }
            Constraint::FixPoint { point, target } => {
                let q = point.evaluate(store, p);
                out[0] = q.x - target.x;
                out[1] = q.y - target.y;
            }
            Constraint::EqualLength { a, b } => {
                out[0] = a.evaluate(store, p).value() - b.evaluate(store, p).value();
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
                let va = a.evaluate(store, p);
                let vb = b.evaluate(store, p);
                let ja = a.eval_vec_jacobian(store, p);
                let jb = b.eval_vec_jacobian(store, p);
                let na = ja.n();
                let nb = jb.n();
                debug_assert_eq!(na, a.param_indices(store).len());
                debug_assert_eq!(nb, b.param_indices(store).len());
                // r = va × vb; ∂r/∂va = (vb.y, -vb.x), ∂r/∂vb = (-va.y, va.x)
                for i in 0..na {
                    out[i] = vb.y * ja.dx()[i] - vb.x * ja.dy()[i];
                }
                for j in 0..nb {
                    out[na + j] = -va.y * jb.dx()[j] + va.x * jb.dy()[j];
                }
            }
            Constraint::Orthogonal { a, b } => {
                let va = a.evaluate(store, p);
                let vb = b.evaluate(store, p);
                let ja = a.eval_vec_jacobian(store, p);
                let jb = b.eval_vec_jacobian(store, p);
                let na = ja.n();
                let nb = jb.n();
                debug_assert_eq!(na, a.param_indices(store).len());
                debug_assert_eq!(nb, b.param_indices(store).len());
                // r = va · vb; ∂r/∂va = vb, ∂r/∂vb = va
                for i in 0..na {
                    out[i] = vb.x * ja.dx()[i] + vb.y * ja.dy()[i];
                }
                for j in 0..nb {
                    out[na + j] = va.x * jb.dx()[j] + va.y * jb.dy()[j];
                }
            }
            Constraint::FixPoint { point, .. } => {
                let j = point.eval_point_jacobian(store, p);
                debug_assert_eq!(j.n(), point.param_indices(store).len());
                for i in 0..j.n() {
                    out[0 * n_local + i] = j.dx()[i];
                    out[1 * n_local + i] = j.dy()[i];
                }
            }
            Constraint::EqualLength { a, b } => {
                let ja = a.eval_length_jacobian(store, p);
                let jb = b.eval_length_jacobian(store, p);
                let na = ja.n();
                let nb = jb.n();
                debug_assert_eq!(na, a.param_indices(store).len());
                debug_assert_eq!(nb, b.param_indices(store).len());
                for i in 0..na {
                    out[i] = ja.ds()[i];
                }
                for j in 0..nb {
                    out[na + j] = -jb.ds()[j];
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_constraint.rs"]
mod tests_constraint;
