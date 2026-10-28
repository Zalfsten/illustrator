//! Structural constraints: kind trait + document entry (closed catalog).
//!
//! Concrete kinds live under [`crate::constraint_kinds`] and implement
//! [`ConstraintKind`]. Document stores [`ConstraintEntry`] with
//! [`crate::constraint_kinds::ConstraintInst`] (no `dyn`).

use std::fmt::Debug;

use crate::document::Document;
use crate::expr::Expr;
use crate::geometry::{Formula, GeometryId};
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

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

/// Global columns of a two-sided contribution: `a`'s block, then `b`'s.
pub(crate) fn pair_connectivity<A: Formula, B: Formula>(
    doc: &Document,
    a: &Expr<A>,
    b: &Expr<B>,
) -> Vec<usize> {
    let mut conn = a.param_indices(doc);
    conn.extend(b.param_indices(doc));
    conn
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

    /// Prefix of automatic object names (`par` → `par1`, `par2`, …).
    const NAME_PREFIX: &'static str;

    /// Global Vector columns involved in this contribution (assembly column map).
    fn param_connectivity(&self, doc: &Document) -> Vec<usize>;

    /// Local residual `r_N` in contribution order (length = [`Self::RESIDUAL_ARITY`]).
    fn eval_residual(&self, doc: &Document, p: &Vector, out: &mut [f64]);

    /// Write this contribution's Jacobian into `j` (local indices, connectivity order).
    fn accumulate_jacobian(&self, doc: &Document, p: &Vector, j: &mut DenseMatrix<'_>);

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
