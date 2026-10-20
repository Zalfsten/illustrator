//! Sparse \(LL^\top\) behind a crate-private wrapper (faer).
//!
//! Symbolic analysis (fill-reducing ordering + elimination structure) runs once
//! per pattern in [`SparseLlt::new`]; [`SparseLlt::factor_solve`] performs the
//! numeric factorisation and the triangular solves into pre-allocated storage.
//! LM code depends only on this module, not on faer types.

use faer::dyn_stack::{MemBuffer, MemStack, StackReq};
use faer::linalg::cholesky::llt::factor::LltRegularization;
use faer::sparse::linalg::cholesky::{
    factorize_symbolic_cholesky, CholeskySymbolicParams, SymbolicCholesky, SymmetricOrdering,
};
use faer::sparse::{SparseColMatRef, SymbolicSparseColMatRef};
use faer::{Conj, MatMut, Par, Side};

/// Symmetric positive-definite solver for a fixed upper-triangular CSC pattern.
pub(crate) struct SparseLlt {
    n: usize,
    col_ptr: Vec<usize>,
    row_idx: Vec<usize>,
    symbolic: Option<SymbolicCholesky<usize>>,
    l_values: Vec<f64>,
    scratch: MemBuffer,
}

impl SparseLlt {
    /// `col_ptr` / `row_idx`: CSC of the **upper** triangle (row ≤ column),
    /// rows sorted within each column, every diagonal entry present.
    pub(crate) fn new(n: usize, col_ptr: Vec<usize>, row_idx: Vec<usize>) -> Self {
        assert_eq!(col_ptr.len(), n + 1);
        if n == 0 {
            return Self {
                n,
                col_ptr,
                row_idx,
                symbolic: None,
                l_values: Vec::new(),
                scratch: MemBuffer::new(StackReq::EMPTY),
            };
        }
        let pattern = SymbolicSparseColMatRef::new_checked(n, n, &col_ptr, None, &row_idx);
        let symbolic = factorize_symbolic_cholesky(
            pattern,
            Side::Upper,
            SymmetricOrdering::Amd,
            CholeskySymbolicParams::default(),
        )
        .expect("symbolic Cholesky of normal-equation pattern");
        let req = StackReq::or(
            symbolic.factorize_numeric_llt_scratch::<f64>(Par::Seq, Default::default()),
            symbolic.solve_in_place_scratch::<f64>(1, Par::Seq),
        );
        Self {
            n,
            l_values: vec![0.0; symbolic.len_val()],
            symbolic: Some(symbolic),
            col_ptr,
            row_idx,
            scratch: MemBuffer::new(req),
        }
    }

    /// Stored entries of the upper-triangular pattern (`values` length).
    #[cfg(test)]
    pub(crate) fn nnz(&self) -> usize {
        self.row_idx.len()
    }

    /// Factorise \(A\) (values in pattern order) and overwrite `rhs` with
    /// \(A^{-1}\,\text{rhs}\). Returns `false` (rhs unspecified) if \(A\) is not
    /// numerically positive definite.
    pub(crate) fn factor_solve(&mut self, values: &[f64], rhs: &mut [f64]) -> bool {
        assert_eq!(values.len(), self.row_idx.len());
        assert_eq!(rhs.len(), self.n);
        let Some(symbolic) = self.symbolic.as_ref() else {
            return true;
        };
        let pattern = SymbolicSparseColMatRef::new_checked(
            self.n,
            self.n,
            &self.col_ptr,
            None,
            &self.row_idx,
        );
        let a = SparseColMatRef::new(pattern, values);
        let stack = MemStack::new(&mut self.scratch);
        let Ok(llt) = symbolic.factorize_numeric_llt(
            &mut self.l_values,
            a,
            Side::Upper,
            LltRegularization {
                dynamic_regularization_delta: 0.0,
                dynamic_regularization_epsilon: 0.0,
            },
            Par::Seq,
            stack,
            Default::default(),
        ) else {
            return false;
        };
        let x = MatMut::from_column_major_slice_mut(rhs, self.n, 1);
        llt.solve_in_place_with_conj(Conj::No, x, Par::Seq, MemStack::new(&mut self.scratch));
        true
    }
}

#[cfg(test)]
#[path = "tests_sparse_chol.rs"]
mod tests_sparse_chol;
