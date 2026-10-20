//! Normal equations \(G = J^\top J\), \(g = J^\top r\) from a contribution-blocked
//! Jacobian (FEM-style assembly, like a stiffness matrix).
//!
//! The upper-triangular CSC pattern of \(G\) (plus the full diagonal, so
//! damping always has a slot) and, per block, the \(k_b \times k_b\) target slots
//! are computed once in [`NormalSystem::new`]. [`NormalSystem::assemble`] then
//! adds \(B^\top B\) of every block through those slots without allocating.
//! Duplicate block columns map to the same slot and therefore accumulate.

use crate::matrix::BlockSparseMatrix;
use crate::vector::Vector;

/// Slot marker for a lower-triangle pair (skipped; its mirror is stored).
const LOWER: usize = usize::MAX;

pub(crate) struct NormalSystem {
    n: usize,
    col_ptr: Vec<usize>,
    row_idx: Vec<usize>,
    /// Per block, row-major `k × k` slots into `values` (or [`LOWER`]).
    slots: Vec<usize>,
    slot_off: Vec<usize>,
    diag_slot: Vec<usize>,
    values: Vec<f64>,
    rhs: Vec<f64>,
}

impl NormalSystem {
    /// Normal-equation pattern for the block structure of `j`.
    pub(crate) fn new(j: &BlockSparseMatrix) -> Self {
        let n = j.ncols();
        let mut rows_of: Vec<Vec<usize>> = (0..n).map(|c| vec![c]).collect();
        for b in j.blocks() {
            let cols = b.cols();
            for &ra in cols {
                for &rc in cols {
                    if ra < rc {
                        rows_of[rc].push(ra);
                    }
                }
            }
        }
        let mut col_ptr = Vec::with_capacity(n + 1);
        let mut row_idx = Vec::new();
        col_ptr.push(0);
        for rows in &mut rows_of {
            rows.sort_unstable();
            rows.dedup();
            row_idx.extend_from_slice(rows);
            col_ptr.push(row_idx.len());
        }
        let slot = |r: usize, c: usize| -> usize {
            let col = &row_idx[col_ptr[c]..col_ptr[c + 1]];
            col_ptr[c] + col.binary_search(&r).expect("pair in normal pattern")
        };
        let diag_slot: Vec<usize> = (0..n).map(|i| slot(i, i)).collect();
        let mut slots = Vec::new();
        let mut slot_off = Vec::with_capacity(j.num_blocks());
        for b in j.blocks() {
            slot_off.push(slots.len());
            let cols = b.cols();
            for &ra in cols {
                for &rc in cols {
                    slots.push(if ra <= rc { slot(ra, rc) } else { LOWER });
                }
            }
        }
        let nnz = row_idx.len();
        Self {
            n,
            col_ptr,
            row_idx,
            slots,
            slot_off,
            diag_slot,
            values: vec![0.0; nnz],
            rhs: vec![0.0; n],
        }
    }

    /// Overwrite \(G = J^\top J\) and \(g = J^\top r\) (same block structure as
    /// in [`Self::new`]).
    pub(crate) fn assemble(&mut self, j: &BlockSparseMatrix, r: &Vector) {
        debug_assert_eq!(j.ncols(), self.n);
        j.transpose_mul(r, &mut self.rhs);
        self.values.fill(0.0);
        for (b, &off) in j.blocks().zip(&self.slot_off) {
            let (m, k) = (b.nrows(), b.ncols());
            let slots = &self.slots[off..off + k * k];
            for a in 0..k {
                for c in 0..k {
                    let s = slots[a * k + c];
                    if s == LOWER {
                        continue;
                    }
                    let mut v = 0.0;
                    for i in 0..m {
                        v += b[(i, a)] * b[(i, c)];
                    }
                    self.values[s] += v;
                }
            }
        }
    }

    pub(crate) fn n(&self) -> usize {
        self.n
    }

    /// CSC column pointers of the upper-triangular pattern.
    pub(crate) fn col_ptr(&self) -> &[usize] {
        &self.col_ptr
    }

    /// CSC row indices of the upper-triangular pattern (sorted per column).
    pub(crate) fn row_idx(&self) -> &[usize] {
        &self.row_idx
    }

    /// Values of \(G\) in pattern order.
    pub(crate) fn values(&self) -> &[f64] {
        &self.values
    }

    /// Slot of \(G_{ii}\) in [`Self::values`].
    pub(crate) fn diag_slot(&self, i: usize) -> usize {
        self.diag_slot[i]
    }

    /// \(g = J^\top r\).
    pub(crate) fn rhs(&self) -> &[f64] {
        &self.rhs
    }
}

#[cfg(test)]
#[path = "tests_normal.rs"]
mod tests_normal;
