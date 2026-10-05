//! Object-safe structural constraints (no catalog enum).
//!
//! Concrete kinds live under [`crate::constraint_kinds`] and implement
//! [`ConstraintKind`]; [`Constraint`] follows via blanket (like
//! [`crate::geometry::GeometryKind`] / [`crate::geometry::Geometry`]).
//! Document stores `Box<dyn Constraint>`.

use std::fmt::Debug;

use crate::geometry::{GeometryId, PointExpr};
use crate::state::State;
use crate::store::GeometryStore;

/// Concrete constraint kind ready for Document storage.
///
/// Not object-safe ([`Self::RESIDUAL_ARITY`]). Implement this on each kind;
/// [`Constraint`] (including [`residual_arity`](Constraint::residual_arity) and
/// [`clone_box`](Constraint::clone_box)) follows via the blanket below.
pub trait ConstraintKind: Clone + Debug + 'static {
    /// Residual row count for this contribution.
    const RESIDUAL_ARITY: usize;

    /// Global State columns involved in this contribution (assembly scatter map).
    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize>;

    /// Local residual `r_N` in contribution order (length = [`Self::RESIDUAL_ARITY`]).
    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]);

    /// Write analytical Jacobian into a zeroed `out` buffer (`n_local` = connectivity width).
    fn accumulate_jacobian(
        &self,
        store: &dyn GeometryStore,
        p: &State,
        n_local: usize,
        out: &mut [f64],
    );

    /// True if this constraint references geometry `id` (orphan drop on remove).
    fn references_geometry(&self, id: GeometryId) -> bool;

    /// Extra point expressions that should share soft mouse drag with `dragged`
    /// (Congruence). Default: none.
    fn co_dragged_points(&self, _dragged: &PointExpr) -> Vec<PointExpr> {
        Vec::new()
    }
}

/// Document-held structural constraint (object-safe).
///
/// Supplied automatically for every [`ConstraintKind`].
pub trait Constraint: Debug {
    /// Object-safe accessor for the kind's [`ConstraintKind::RESIDUAL_ARITY`].
    fn residual_arity(&self) -> usize;

    /// Global State columns involved in this contribution (assembly scatter map).
    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize>;

    /// Local residual `r_N` in contribution order (length = residual_arity).
    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]);

    /// Local Jacobian `J_N` row-major, shape `(residual_arity × param_connectivity.len())`.
    fn eval_jacobian(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        let n_local = self.param_connectivity(store).len();
        let m = self.residual_arity();
        assert_eq!(out.len(), m * n_local);
        out.fill(0.0);
        self.accumulate_jacobian(store, p, n_local, out);
    }

    /// Write analytical Jacobian into a zeroed `out` buffer (`n_local` = connectivity width).
    fn accumulate_jacobian(
        &self,
        store: &dyn GeometryStore,
        p: &State,
        n_local: usize,
        out: &mut [f64],
    );

    /// True if this constraint references geometry `id` (orphan drop on remove).
    fn references_geometry(&self, id: GeometryId) -> bool;

    /// Extra point expressions that should share soft mouse drag with `dragged`
    /// (Congruence). Default: none.
    fn co_dragged_points(&self, dragged: &PointExpr) -> Vec<PointExpr>;

    fn clone_box(&self) -> Box<dyn Constraint>;
}

impl<T: ConstraintKind> Constraint for T {
    fn residual_arity(&self) -> usize {
        T::RESIDUAL_ARITY
    }

    fn param_connectivity(&self, store: &dyn GeometryStore) -> Vec<usize> {
        ConstraintKind::param_connectivity(self, store)
    }

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut [f64]) {
        ConstraintKind::eval_residual(self, store, p, out)
    }

    fn accumulate_jacobian(
        &self,
        store: &dyn GeometryStore,
        p: &State,
        n_local: usize,
        out: &mut [f64],
    ) {
        ConstraintKind::accumulate_jacobian(self, store, p, n_local, out)
    }

    fn references_geometry(&self, id: GeometryId) -> bool {
        ConstraintKind::references_geometry(self, id)
    }

    fn co_dragged_points(&self, dragged: &PointExpr) -> Vec<PointExpr> {
        ConstraintKind::co_dragged_points(self, dragged)
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
#[path = "tests_constraint.rs"]
mod tests_constraint;
