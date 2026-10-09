//! Structural constraints: kind trait + document entry (closed catalog).
//!
//! Concrete kinds live under [`crate::constraint_kinds`] and implement
//! [`ConstraintKind`]. Document stores [`ConstraintEntry`] with
//! [`crate::constraint_kinds::ConstraintInst`] (no `dyn`).

use std::fmt::Debug;

use crate::geometry::GeometryId;
use crate::matrix::MatrixWrite;
use crate::vector::{VecWrite, Vector};
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

/// Document-held constraint: stable id plus catalog payload.
#[derive(Clone, Debug)]
pub struct ConstraintEntry {
    pub id: ConstraintId,
    pub body: crate::constraint_kinds::ConstraintInst,
}

impl ConstraintEntry {
    pub fn body(&self) -> &crate::constraint_kinds::ConstraintInst {
        &self.body
    }
}

/// Concrete constraint kind ready for Document storage.
///
/// Implement this on each kind; convert into
/// [`crate::constraint_kinds::ConstraintInst`] via [`From`].
pub trait ConstraintKind: Clone + Debug + 'static {
    /// Residual row count for this contribution.
    const RESIDUAL_ARITY: usize;

    /// Global Vector columns involved in this contribution (assembly scatter map).
    fn param_connectivity<S: GeometryStore + ?Sized>(&self, store: &S) -> Vec<usize>;

    /// Local residual `r_N` in contribution order (length = [`Self::RESIDUAL_ARITY`]).
    fn eval_residual<S: GeometryStore + ?Sized, R: VecWrite>(
        &self,
        store: &S,
        p: &Vector,
        out: &mut R,
    );

    /// Write this contribution's Jacobian into `j` (dense or scatter local indices).
    fn accumulate_jacobian<S: GeometryStore + ?Sized, J: MatrixWrite>(
        &self,
        store: &S,
        p: &Vector,
        j: &mut J,
    );

    /// Geometry instances this constraint depends on (incidence / reverse index).
    fn referenced_geometries(&self) -> Vec<GeometryId>;

    /// True if this constraint references geometry `id` (orphan drop on remove).
    fn references_geometry(&self, id: GeometryId) -> bool {
        self.referenced_geometries().contains(&id)
    }
}

#[cfg(test)]
#[path = "tests_constraint.rs"]
mod tests_constraint;
