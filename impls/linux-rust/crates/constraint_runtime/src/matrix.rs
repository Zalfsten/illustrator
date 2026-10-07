//! Row-major matrices and dense / scatter views.
//!
//! [`Matrix`] owns matrix storage; the owned parameter / residual vector is
//! [`crate::state::State`]. Borrowed windows are [`DenseJac`] / [`DenseVec`]
//! (contiguous) or [`ScatterJac`] / [`ScatterVec`] (local index → connectivity;
//! not CSR). Shared semantics: [`JacView`] / [`VecView`].
//!
//! Matrix types support `m[(i, j)]`; vector views support `v[i]`.

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

    pub fn as_slice(&self) -> &[f64] {
        &self.data
    }

    /// Borrow the whole matrix as a dense view.
    pub fn as_dense(&mut self) -> DenseJac<'_> {
        DenseJac::packed(&mut self.data, self.nrows, self.ncols)
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
pub struct DenseJac<'a> {
    data: &'a mut [f64],
    stride: usize,
    row0: usize,
    col0: usize,
    nrows: usize,
    ncols: usize,
}

impl<'a> DenseJac<'a> {
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
    pub fn dense_block(
        &mut self,
        row0: usize,
        col0: usize,
        block_rows: usize,
        block_cols: usize,
    ) -> DenseJac<'_> {
        debug_assert!(row0 + block_rows <= self.nrows);
        debug_assert!(col0 + block_cols <= self.ncols);
        DenseJac {
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
    pub fn scatter<'c>(
        &mut self,
        row0: usize,
        nrows: usize,
        conn: &'c [usize],
    ) -> ScatterJac<'_, 'c> {
        debug_assert!(row0 + nrows <= self.nrows);
        ScatterJac {
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

impl Index<(usize, usize)> for DenseJac<'_> {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        &self.data[self.flat(i, j)]
    }
}

impl IndexMut<(usize, usize)> for DenseJac<'_> {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

/// Scatter local \((i,j)\) into parent rows / columns `col0 + conn[j]`.
pub struct ScatterJac<'d, 'c> {
    data: &'d mut [f64],
    stride: usize,
    row0: usize,
    col0: usize,
    nrows: usize,
    conn: &'c [usize],
}

impl ScatterJac<'_, '_> {
    fn flat(&self, i: usize, j: usize) -> usize {
        debug_assert!(i < self.nrows && j < self.conn.len());
        (self.row0 + i) * self.stride + (self.col0 + self.conn[j])
    }
}

impl Index<(usize, usize)> for ScatterJac<'_, '_> {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        &self.data[self.flat(i, j)]
    }
}

impl IndexMut<(usize, usize)> for ScatterJac<'_, '_> {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        let idx = self.flat(i, j);
        &mut self.data[idx]
    }
}

/// Matrix semantics on a dense local \((i, j)\) index space (`dyn`-friendly).
///
/// Primitives are [`cell`](Self::cell) / [`cell_mut`](Self::cell_mut) so
/// [`Index`] / [`IndexMut`] work on `dyn JacView` (`j[(i, k)] = …`).
pub trait JacView {
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

impl Index<(usize, usize)> for dyn JacView + '_ {
    type Output = f64;
    fn index(&self, (i, j): (usize, usize)) -> &f64 {
        self.cell(i, j)
    }
}

impl IndexMut<(usize, usize)> for dyn JacView + '_ {
    fn index_mut(&mut self, (i, j): (usize, usize)) -> &mut f64 {
        self.cell_mut(i, j)
    }
}

/// Column window over a [`JacView`]: local column `j` maps to `inner` column `col0 + j`.
///
/// Lets expressions write at formula-local columns `0..` while constraints place
/// successive blocks via the view (no `col0` on accumulate APIs).
pub struct JacCols<'a> {
    inner: &'a mut dyn JacView,
    col0: usize,
    ncols: usize,
}

impl<'a> JacCols<'a> {
    /// Window starting at `col0` with width `inner.ncols() - col0`.
    pub fn from(inner: &'a mut dyn JacView, col0: usize) -> Self {
        let ncols = inner.ncols().saturating_sub(col0);
        debug_assert!(col0 + ncols <= inner.ncols());
        Self {
            inner,
            col0,
            ncols,
        }
    }
}

impl JacView for JacCols<'_> {
    fn nrows(&self) -> usize {
        self.inner.nrows()
    }

    fn ncols(&self) -> usize {
        self.ncols
    }

    fn cell(&self, i: usize, j: usize) -> &f64 {
        self.inner.cell(i, self.col0 + j)
    }

    fn cell_mut(&mut self, i: usize, j: usize) -> &mut f64 {
        self.inner.cell_mut(i, self.col0 + j)
    }
}

impl JacView for DenseJac<'_> {
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

impl JacView for ScatterJac<'_, '_> {
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

// --- vector views ------------------------------------------------------------

/// Dense local index space over a contiguous segment of a parent vector.
pub struct DenseVec<'a> {
    data: &'a mut [f64],
    start: usize,
    len: usize,
}

impl<'a> DenseVec<'a> {
    pub fn new(data: &'a mut [f64], start: usize, len: usize) -> Self {
        debug_assert!(start + len <= data.len());
        Self { data, start, len }
    }

    pub fn packed(data: &'a mut [f64]) -> Self {
        let len = data.len();
        Self::new(data, 0, len)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Contiguous sub-segment with local indices `0..len`.
    pub fn dense_block(&mut self, start: usize, len: usize) -> DenseVec<'_> {
        debug_assert!(start + len <= self.len);
        DenseVec::new(self.data, self.start + start, len)
    }

    /// Scatter view: local `i` maps to `start + conn[i]` in the parent buffer.
    pub fn scatter<'c>(&mut self, conn: &'c [usize]) -> ScatterVec<'_, 'c> {
        ScatterVec {
            data: self.data,
            start: self.start,
            conn,
        }
    }

    fn flat(&self, i: usize) -> usize {
        debug_assert!(i < self.len);
        self.start + i
    }
}

impl Index<usize> for DenseVec<'_> {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        &self.data[self.flat(i)]
    }
}

impl IndexMut<usize> for DenseVec<'_> {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        let idx = self.flat(i);
        &mut self.data[idx]
    }
}

/// Scatter local `i` into parent entry `start + conn[i]`.
pub struct ScatterVec<'d, 'c> {
    data: &'d mut [f64],
    start: usize,
    conn: &'c [usize],
}

impl<'d, 'c> ScatterVec<'d, 'c> {
    /// Absolute connectivity into `data` (`start == 0`).
    pub fn new(data: &'d mut [f64], conn: &'c [usize]) -> Self {
        Self {
            data,
            start: 0,
            conn,
        }
    }

    fn flat(&self, i: usize) -> usize {
        debug_assert!(i < self.conn.len());
        let g = self.start + self.conn[i];
        debug_assert!(g < self.data.len());
        g
    }
}

impl Index<usize> for ScatterVec<'_, '_> {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        &self.data[self.flat(i)]
    }
}

impl IndexMut<usize> for ScatterVec<'_, '_> {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        let idx = self.flat(i);
        &mut self.data[idx]
    }
}

/// Vector semantics on a dense local index space (`dyn`-friendly).
///
/// Primitives are [`cell`](Self::cell) / [`cell_mut`](Self::cell_mut) so
/// [`Index`] / [`IndexMut`] work on `dyn VecView` (`v[i] = …`).
pub trait VecView {
    fn len(&self) -> usize;
    fn cell(&self, i: usize) -> &f64;
    fn cell_mut(&mut self, i: usize) -> &mut f64;
    fn get(&self, i: usize) -> f64 {
        *self.cell(i)
    }
    fn set(&mut self, i: usize, v: f64) {
        *self.cell_mut(i) = v;
    }
    fn add(&mut self, i: usize, v: f64) {
        *self.cell_mut(i) += v;
    }
    fn fill(&mut self, value: f64) {
        for i in 0..self.len() {
            *self.cell_mut(i) = value;
        }
    }
}

impl Index<usize> for dyn VecView + '_ {
    type Output = f64;
    fn index(&self, i: usize) -> &f64 {
        self.cell(i)
    }
}

impl IndexMut<usize> for dyn VecView + '_ {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        self.cell_mut(i)
    }
}

impl VecView for DenseVec<'_> {
    fn len(&self) -> usize {
        self.len
    }

    fn cell(&self, i: usize) -> &f64 {
        &self.data[self.flat(i)]
    }

    fn cell_mut(&mut self, i: usize) -> &mut f64 {
        let idx = self.flat(i);
        &mut self.data[idx]
    }

    fn fill(&mut self, value: f64) {
        let start = self.start;
        let end = start + self.len;
        self.data[start..end].fill(value);
    }
}

impl VecView for ScatterVec<'_, '_> {
    fn len(&self) -> usize {
        self.conn.len()
    }

    fn cell(&self, i: usize) -> &f64 {
        &self.data[self.flat(i)]
    }

    fn cell_mut(&mut self, i: usize) -> &mut f64 {
        let idx = self.flat(i);
        &mut self.data[idx]
    }
}

#[cfg(test)]
#[path = "tests_matrix.rs"]
mod tests_matrix;
