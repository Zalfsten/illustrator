//! Row-major matrices and dense / scatter matrix views.
//!
//! [`Matrix`] owns matrix storage. Borrowed windows are [`DenseMatrix`]
//! (contiguous) or [`ScatterMatrix`] (local column index → connectivity; not CSR).
//! Shared semantics: [`MatrixView`] / [`MatrixWrite`].
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
        assert_eq!(data.len(), nrows.saturating_mul(ncols));
        Self {
            data,
            stride: ncols,
            row0: 0,
            col0: 0,
            nrows,
            ncols,
        }
    }

    /// Contiguous sub-block with local dense indices.
    pub fn block_slice(
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

    /// Scatter view: local columns map through `conn` into this window's columns
    /// (then into the parent buffer via `col0` / `stride`).
    pub fn scatter_slice(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &[usize],
    ) -> ScatterMatrix<'_> {
        debug_assert!(row0 + nrows <= self.nrows);
        debug_assert!(conn.iter().all(|&c| c < self.ncols));
        ScatterMatrix {
            data: self.data,
            stride: self.stride,
            row0: self.row0 + row0,
            col0: self.col0,
            nrows,
            conn: conn.to_vec(),
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
/// `conn` is owned so [`scatter_slice`](Self::scatter_slice) can compose
/// connectivity without borrowing nested maps.
pub struct ScatterMatrix<'d> {
    data: &'d mut [f64],
    stride: usize,
    row0: usize,
    col0: usize,
    nrows: usize,
    conn: Vec<usize>,
}

impl<'d> ScatterMatrix<'d> {
    /// Local row/column window (same scatter kind).
    pub fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> ScatterMatrix<'_> {
        debug_assert!(row0 + block_rows <= self.nrows);
        debug_assert!(col0 + block_cols <= self.conn.len());
        ScatterMatrix {
            data: self.data,
            stride: self.stride,
            row0: self.row0 + row0,
            col0: self.col0,
            nrows: block_rows,
            conn: self.conn[col0..col0 + block_cols].to_vec(),
        }
    }

    /// Scatter through this view's local columns: `conn[j]` selects parent-local
    /// column, then the existing map into the dense parent.
    pub fn scatter_slice(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &[usize],
    ) -> ScatterMatrix<'_> {
        debug_assert!(row0 + nrows <= self.nrows);
        debug_assert!(conn.iter().all(|&c| c < self.conn.len()));
        let composed = conn.iter().map(|&j| self.conn[j]).collect();
        ScatterMatrix {
            data: self.data,
            stride: self.stride,
            row0: self.row0 + row0,
            col0: self.col0,
            nrows,
            conn: composed,
        }
    }

    fn flat(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.nrows && j < self.conn.len());
        (self.row0 + i) * self.stride + (self.col0 + self.conn[j])
    }
}

impl Index<(usize, usize)> for ScatterMatrix<'_> {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        &self.data[self.flat(i, j)]
    }
}

impl IndexMut<(usize, usize)> for ScatterMatrix<'_> {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

/// Matrix semantics on a dense local \((i, j)\) index space.
///
/// Used via generics (`J: MatrixView` / [`MatrixWrite`]) so accesses monomorphize.
/// Primitives are [`cell`](Self::cell) / [`cell_mut`](Self::cell_mut); concrete
/// types also implement [`Index`] / [`IndexMut`] for `j[(i, k)]`.
pub trait MatrixView {
    fn nrows(&self) -> usize;
    fn ncols(&self) -> usize;
    fn cell(&self, i: usize, j: usize) -> &f64;
    fn cell_mut(&mut self, i: usize, j: usize) -> &mut f64;
    fn get(&self, i: usize, j: usize) -> f64 {
        *self.cell(i, j)
    }
    fn set(&mut self, i: usize, j: usize, v: f64) {
        *self.cell_mut(i, j) = v;
    }
    fn add(&mut self, i: usize, j: usize, v: f64) {
        *self.cell_mut(i, j) += v;
    }
    fn fill(&mut self, value: f64) {
        for i in 0..self.nrows() {
            for j in 0..self.ncols() {
                *self.cell_mut(i, j) = value;
            }
        }
    }
}

/// Writable matrix buffer: [`MatrixView`] plus `m[(i, k)]` indexing and slices.
///
/// Concrete backends: [`DenseMatrix`], [`ScatterMatrix`]. `block_slice` stays same-kind;
/// `scatter_slice` always yields [`ScatterMatrix`].
pub trait MatrixWrite:
    MatrixView + Index<(usize, usize), Output = f64> + IndexMut<(usize, usize), Output = f64>
{
    type BlockSlice<'a>: MatrixWrite
    where
        Self: 'a;
    type ScatterSlice<'a>: MatrixWrite
    where
        Self: 'a;

    fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> Self::BlockSlice<'_>;

    fn scatter_slice(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &[usize],
    ) -> Self::ScatterSlice<'_>;

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

impl MatrixView for DenseMatrix<'_> {
    fn nrows(&self) -> usize {
        self.nrows
    }

    fn ncols(&self) -> usize {
        self.ncols
    }

    fn cell(&self, i: usize, j: usize) -> &f64 {
        &self.data[self.flat(i, j)]
    }

    fn cell_mut(&mut self, i: usize, j: usize) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }

    /// Each local row is a contiguous `ncols`-run in the parent buffer.
    fn fill(&mut self, value: f64) {
        if self.nrows == 0 || self.ncols == 0 {
            return;
        }
        // Packed whole buffer: one contiguous span.
        if self.row0 == 0
            && self.col0 == 0
            && self.stride == self.ncols
            && self.data.len() == self.nrows * self.ncols
        {
            self.data.fill(value);
            return;
        }
        for i in 0..self.nrows {
            let start = (self.row0 + i) * self.stride + self.col0;
            self.data[start..start + self.ncols].fill(value);
        }
    }
}

impl MatrixView for ScatterMatrix<'_> {
    fn nrows(&self) -> usize {
        self.nrows
    }

    fn ncols(&self) -> usize {
        self.conn.len()
    }

    fn cell(&self, i: usize, j: usize) -> &f64 {
        &self.data[self.flat(i, j)]
    }

    fn cell_mut(&mut self, i: usize, j: usize) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

impl MatrixWrite for DenseMatrix<'_> {
    type BlockSlice<'a>
        = DenseMatrix<'a>
    where
        Self: 'a;
    type ScatterSlice<'a>
        = ScatterMatrix<'a>
    where
        Self: 'a;

    fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> Self::BlockSlice<'_> {
        DenseMatrix::block_slice(self, row0, col0, block_rows, block_cols)
    }

    fn scatter_slice(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &[usize],
    ) -> Self::ScatterSlice<'_> {
        DenseMatrix::scatter_slice(self, row0, nrows, conn)
    }
}

impl MatrixWrite for ScatterMatrix<'_> {
    type BlockSlice<'a>
        = ScatterMatrix<'a>
    where
        Self: 'a;
    type ScatterSlice<'a>
        = ScatterMatrix<'a>
    where
        Self: 'a;

    fn block_slice(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> Self::BlockSlice<'_> {
        ScatterMatrix::block_slice(self, row0, col0, block_rows, block_cols)
    }

    fn scatter_slice(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &[usize],
    ) -> Self::ScatterSlice<'_> {
        ScatterMatrix::scatter_slice(self, row0, nrows, conn)
    }
}

#[cfg(test)]
#[path = "tests_matrix.rs"]
mod tests_matrix;
