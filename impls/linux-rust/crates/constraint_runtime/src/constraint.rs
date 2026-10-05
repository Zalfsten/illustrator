//! Object-safe structural constraints (no catalog enum).
//!
//! Concrete kinds live under [`crate::constraint_kinds`] and implement
//! [`ConstraintKind`]; [`Constraint`] follows via blanket (like
//! [`crate::geometry::GeometryKind`] / [`crate::geometry::Geometry`]).
//! Document stores [`ConstraintEntry`] (`ConstraintId` + `Box<dyn Constraint>`).

use std::any::Any;
use std::fmt::Debug;

use crate::geom::{PointJacobian, ScalarJacobian};
use crate::geometry::GeometryId;
use crate::state::State;
use crate::store::GeometryStore;

/// Stable document identity for a structural constraint instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstraintId(pub usize);

impl From<usize> for ConstraintId {
    fn from(raw: usize) -> Self {
        Self(raw)
    }
}

impl From<ConstraintId> for usize {
    fn from(id: ConstraintId) -> Self {
        id.0
    }
}

/// Document-held constraint: stable id plus kind payload.
#[derive(Debug)]
pub struct ConstraintEntry {
    pub id: ConstraintId,
    pub body: Box<dyn Constraint>,
}

impl Clone for ConstraintEntry {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            body: self.body.clone_box(),
        }
    }
}

impl ConstraintEntry {
    pub fn body(&self) -> &dyn Constraint {
        self.body.as_ref()
    }
}

/// Mutable view of a contribution's local residual Jacobian \(J_N\).
///
/// Row-major layout of shape `(n_rows × n_cols)` with `n_cols` =
/// [`Constraint::param_connectivity`] width. Formula Jacobians are placed as
/// contiguous column blocks (see [`LocalJacobian::placer`]).
pub struct LocalJacobian<'a> {
    data: &'a mut [f64],
    n_rows: usize,
    n_cols: usize,
}

impl<'a> LocalJacobian<'a> {
    /// Borrow a zeroed (or caller-cleared) row-major buffer as \(m \times n\).
    pub fn new(data: &'a mut [f64], n_rows: usize, n_cols: usize) -> Self {
        assert_eq!(data.len(), n_rows * n_cols);
        Self {
            data,
            n_rows,
            n_cols,
        }
    }

    pub fn n_cols(&self) -> usize {
        self.n_cols
    }

    pub fn n_rows(&self) -> usize {
        self.n_rows
    }

    /// Sequential placer starting at column 0 (contribution join order).
    pub fn placer(&mut self) -> LocalJacobianPlacer<'_, 'a> {
        LocalJacobianPlacer { jac: self, col: 0 }
    }

    fn place_point(&mut self, col: usize, j: &PointJacobian, scale: f64) {
        debug_assert_eq!(self.n_rows, 2);
        debug_assert!(col + j.n() <= self.n_cols);
        let n = self.n_cols;
        for i in 0..j.n() {
            self.data[0 * n + col + i] = scale * j.dx()[i];
            self.data[1 * n + col + i] = scale * j.dy()[i];
        }
    }

    fn place_scalar(&mut self, col: usize, j: &ScalarJacobian, scale: f64) {
        debug_assert_eq!(self.n_rows, 1);
        debug_assert!(col + j.n() <= self.n_cols);
        for i in 0..j.n() {
            self.data[col + i] = scale * j.ds()[i];
        }
    }
}

/// Places formula Jacobian blocks left-to-right into a [`LocalJacobian`].
pub struct LocalJacobianPlacer<'p, 'a> {
    jac: &'p mut LocalJacobian<'a>,
    col: usize,
}

impl LocalJacobianPlacer<'_, '_> {
    /// Place a 2-row point/vector formula block and advance the column cursor.
    pub fn place_point(&mut self, j: &PointJacobian, scale: f64) {
        self.jac.place_point(self.col, j, scale);
        self.col += j.n();
    }

    /// Place a 1-row scalar formula block and advance the column cursor.
    pub fn place_scalar(&mut self, j: &ScalarJacobian, scale: f64) {
        self.jac.place_scalar(self.col, j, scale);
        self.col += j.n();
    }
}

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

    /// Fill a zeroed local Jacobian view (columns in [`Self::param_connectivity`] order).
    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut LocalJacobian<'_>);

    /// True if this constraint references geometry `id` (orphan drop on remove).
    fn references_geometry(&self, id: GeometryId) -> bool;
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
        let mut j = LocalJacobian::new(out, m, n_local);
        self.accumulate_jacobian(store, p, &mut j);
    }

    /// Fill a zeroed local Jacobian view (columns in param_connectivity order).
    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut LocalJacobian<'_>);

    /// True if this constraint references geometry `id` (orphan drop on remove).
    fn references_geometry(&self, id: GeometryId) -> bool;

    fn clone_box(&self) -> Box<dyn Constraint>;

    /// Downcast escape hatch for kind-specific queries (e.g. [`crate::Congruence`]).
    fn as_any(&self) -> &dyn Any;
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

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut LocalJacobian<'_>) {
        ConstraintKind::accumulate_jacobian(self, store, p, j)
    }

    fn references_geometry(&self, id: GeometryId) -> bool {
        ConstraintKind::references_geometry(self, id)
    }

    fn clone_box(&self) -> Box<dyn Constraint> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
#[path = "tests_constraint.rs"]
mod tests_constraint;
