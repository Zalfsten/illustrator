//! Row-major matrices and dense / scatter matrix views.
//!
//! [`Matrix`] owns matrix storage. Borrowed windows are [`DenseMatrix`]
//! (contiguous) or [`ScatterMatrix`] (local column index → connectivity; not CSR).
//! Formula / constraint Jacobians write through the shared [`MatrixAccess`].
//!
//! For the parameter / residual vector see [`crate::vector`].

use std::ops::{Index, IndexMut};

/// Owned row-major matrix.
#[derive(Clone, Debug)]
pub struct Matrix {
    data: Vec<f64>,
    nrows: usize,
    ncols: usize,
}

impl Matrix {
    pub fn zeros(nrows: usize, ncols: usize) -> Self {
        Self {
            data: vec![0.0; nrows * ncols],
            nrows,
            ncols,
        }
    }

    pub fn nrows(&self) -> usize {
        self.nrows
    }

    pub fn ncols(&self) -> usize {
        self.ncols
    }

    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.data
    }

    pub fn fill(&mut self, value: f64) {
        self.data.fill(value);
    }

    /// Borrow the whole matrix as a dense view.
    pub fn as_dense(&mut self) -> DenseMatrix<'_> {
        DenseMatrix::packed(&mut self.data, self.nrows, self.ncols)
    }

    fn flat(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.nrows && j < self.ncols);
        i * self.ncols + j
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        &self.data[self.flat(i, j)]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

/// Dense local index space over a contiguous rectangular window of a parent
/// buffer (or a packed standalone matrix when offsets are zero and
/// `stride == ncols`).
pub struct DenseMatrix<'a> {
    data: &'a mut [f64],
    stride: usize,
    row0: usize,
    col0: usize,
    nrows: usize,
    ncols: usize,
}

impl<'a> DenseMatrix<'a> {
    /// Packed standalone dense matrix (row-major, no parent padding).
    pub fn packed(data: &'a mut [f64], nrows: usize, ncols: usize) -> Self {
        debug_assert_eq!(data.len(), nrows.saturating_mul(ncols));
        Self {
            data,
            stride: ncols,
            row0: 0,
            col0: 0,
            nrows,
            ncols,
        }
    }

    /// Scatter view: local columns map through borrowed `conn` into this window.
    pub fn scatter_slice<'c>(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &'c [usize],
    ) -> ScatterMatrix<'_, 'c> {
        debug_assert!(row0 + nrows <= self.nrows);
        debug_assert!(conn.iter().all(|&c| c < self.ncols));
        ScatterMatrix {
            data: self.data,
            stride: self.stride,
            row0: self.row0 + row0,
            col0: self.col0,
            nrows,
            conn,
        }
    }

    fn flat(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.nrows && j < self.ncols);
        (self.row0 + i) * self.stride + (self.col0 + j)
    }
}

impl Index<(usize, usize)> for DenseMatrix<'_> {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        &self.data[self.flat(i, j)]
    }
}

impl IndexMut<(usize, usize)> for DenseMatrix<'_> {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

/// Scatter local \((i,j)\) into parent rows / columns `col0 + conn[j]`.
///
/// `conn` is borrowed from the cached assembly connectivity. Column windows use
/// [`MatrixAccess::block_slice`] (subslice of `conn`, no alloc).
pub struct ScatterMatrix<'d, 'c> {
    data: &'d mut [f64],
    stride: usize,
    row0: usize,
    col0: usize,
    nrows: usize,
    conn: &'c [usize],
}

impl ScatterMatrix<'_, '_> {
    fn flat(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.nrows && j < self.conn.len());
        (self.row0 + i) * self.stride + (self.col0 + self.conn[j])
    }
}

impl Index<(usize, usize)> for ScatterMatrix<'_, '_> {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        &self.data[self.flat(i, j)]
    }
}

impl IndexMut<(usize, usize)> for ScatterMatrix<'_, '_> {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

/// Writable matrix window on a dense local \((i, j)\) index space: `m[(i, k)]`
/// plus same-kind block slices.
///
/// Backends: [`DenseMatrix`], [`ScatterMatrix`]. Used via generics
/// (`J: MatrixAccess`) so every access monomorphizes. Scatter views are opened
/// with inherent [`DenseMatrix::scatter_slice`].
pub trait MatrixAccess:
    Index<(usize, usize), Output = f64> + IndexMut<(usize, usize), Output = f64>
{
    type BlockSlice<'a>: MatrixAccess
    where
        Self: 'a;

    fn nrows(&self) -> usize;
    fn ncols(&self) -> usize;

    /// Sub-window with local indices starting at `(row0, col0)`.
    fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> Self::BlockSlice<'_>;

    /// Write a dense \(R \times C\) block at `(0,0)`, times `scale`.
    ///
    /// `vals` is row-major: `vals[i][k]` → `j[(i, k)]`. The view may be larger
    /// than \(R \times C\) (e.g. left half of a joined contribution). Const
    /// generics monomorphize to direct stores — no heap, no dynamic dispatch.
    #[inline]
    fn set_scaled<const R: usize, const C: usize>(
        &mut self,
        scale: f64,
        vals: [[f64; C]; R],
    ) {
        debug_assert!(self.nrows() >= R);
        debug_assert!(self.ncols() >= C);
        for (i, row) in vals.into_iter().enumerate() {
            for (k, v) in row.into_iter().enumerate() {
                self[(i, k)] = scale * v;
            }
        }
    }
}

impl MatrixAccess for DenseMatrix<'_> {
    type BlockSlice<'a>
        = DenseMatrix<'a>
    where
        Self: 'a;

    fn nrows(&self) -> usize {
        self.nrows
    }

    fn ncols(&self) -> usize {
        self.ncols
    }

    fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> DenseMatrix<'_> {
        debug_assert!(row0 + block_rows <= self.nrows);
        debug_assert!(col0 + block_cols <= self.ncols);
        DenseMatrix {
            data: self.data,
            stride: self.stride,
            row0: self.row0 + row0,
            col0: self.col0 + col0,
            nrows: block_rows,
            ncols: block_cols,
        }
    }
}

impl<'c> MatrixAccess for ScatterMatrix<'_, 'c> {
    type BlockSlice<'a>
        = ScatterMatrix<'a, 'c>
    where
        Self: 'a;

    fn nrows(&self) -> usize {
        self.nrows
    }

    fn ncols(&self) -> usize {
        self.conn.len()
    }

    /// Row window plus `conn` subslice (still scattering into the parent).
    fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> ScatterMatrix<'_, 'c> {
        debug_assert!(row0 + block_rows <= self.nrows);
        debug_assert!(col0 + block_cols <= self.conn.len());
        ScatterMatrix {
            data: self.data,
            stride: self.stride,
            row0: self.row0 + row0,
            col0: self.col0,
            nrows: block_rows,
            conn: &self.conn[col0..col0 + block_cols],
        }
    }
}

#[cfg(test)]
#[path = "tests_matrix.rs"]
mod tests_matrix;
