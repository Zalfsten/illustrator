//! Object-safe structural constraints (no catalog enum).
//!
//! Concrete kinds live under [`crate::constraint_kinds`]. Document stores
//! `Box<dyn Constraint>` like geometries store `Box<dyn Geometry>`.

use std::fmt::Debug;

use crate::connectivity::Connectivity;
use crate::geometry::{GeometryId, PointExpr};
use crate::state::State;
use crate::store::GeometryStore;

/// Document-held structural constraint (object-safe).
///
/// Implement on a concrete kind ([`crate::constraint_kinds`]); Document boxes
/// via [`Self::clone_box`]. Assembly only calls residual / Jacobian / connectivity.
pub trait Constraint: Debug {
    fn residual_arity(&self) -> usize;

    /// Column connectivity: global parameter indices involved in this contribution.
    fn param_connectivity(&self, store: &dyn GeometryStore) -> Connectivity;

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
    fn co_dragged_points(&self, _dragged: &PointExpr) -> Vec<PointExpr> {
        Vec::new()
    }

    fn clone_box(&self) -> Box<dyn Constraint>;
}

#[cfg(test)]
#[path = "tests_constraint.rs"]
mod tests_constraint;
