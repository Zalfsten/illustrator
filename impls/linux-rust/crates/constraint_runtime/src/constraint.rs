//! Object-safe structural constraints (no catalog enum).
//!
//! Concrete kinds live under [`crate::constraint_kinds`] and implement
//! [`ConstraintKind`]; [`Constraint`] follows via blanket (like
//! [`crate::geometry::GeometryKind`] / [`crate::geometry::Geometry`]).
//! Document stores [`ConstraintEntry`] (`ConstraintId` + `Box<dyn Constraint>`).

use std::any::Any;
use std::fmt::Debug;

use crate::geometry::GeometryId;
use crate::matrix::{JacView, VecView};
use crate::state::State;
use crate::store::GeometryStore;

/// Deduplicate geometry ids while preserving first-seen order.
pub(crate) fn unique_geometry_ids(
    ids: impl IntoIterator<Item = GeometryId>,
) -> Vec<GeometryId> {
    let mut out = Vec::new();
    for id in ids {
        if !out.contains(&id) {
            out.push(id);
        }
    }
    out
}

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
    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut dyn VecView);

    /// Write this contribution's Jacobian into `j` (dense or scatter local indices).
    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut dyn JacView);

    /// Geometry instances this constraint depends on (incidence / reverse index).
    fn referenced_geometries(&self) -> Vec<GeometryId>;

    /// True if this constraint references geometry `id` (orphan drop on remove).
    fn references_geometry(&self, id: GeometryId) -> bool {
        self.referenced_geometries().contains(&id)
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
    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut dyn VecView);

    /// Write this contribution's Jacobian into `j` (dense or scatter local indices).
    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut dyn JacView);

    /// Geometry instances this constraint depends on (incidence / reverse index).
    fn referenced_geometries(&self) -> Vec<GeometryId>;

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

    fn eval_residual(&self, store: &dyn GeometryStore, p: &State, out: &mut dyn VecView) {
        ConstraintKind::eval_residual(self, store, p, out)
    }

    fn accumulate_jacobian(&self, store: &dyn GeometryStore, p: &State, j: &mut dyn JacView) {
        ConstraintKind::accumulate_jacobian(self, store, p, j)
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        ConstraintKind::referenced_geometries(self)
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
