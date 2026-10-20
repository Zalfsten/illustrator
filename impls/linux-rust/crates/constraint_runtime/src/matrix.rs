//! Dense matrix views and the block-sparse Jacobian.
//!
//! [`DenseMatrix`] is a borrowed dense window (contiguous rows). The global
//! Jacobian is a [`BlockSparseMatrix`]: one dense row block per contribution,
//! each with its own column map. Formula / constraint Jacobians write into a
//! block's dense submatrix (no scatter on write).
//!
//! For the parameter / residual vector see [`crate::vector`].

use std::ops::{Index, IndexMut};

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

    pub fn nrows(&self) -> usize {
        self.nrows
    }

    pub fn ncols(&self) -> usize {
        self.ncols
    }

    /// Sub-window with local indices starting at `(row0, col0)`.
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

    /// Write a dense \(R \times C\) block at `(0,0)`, times `scale`.
    ///
    /// `vals` is row-major: `vals[i][k]` → `self[(i, k)]`. The window may be
    /// larger than \(R \times C\) (e.g. left half of a joined contribution).
    /// Const generics compile to direct stores — no heap, no dynamic dispatch.
    #[inline]
    pub fn set_scaled<const R: usize, const C: usize>(&mut self, scale: f64, vals: [[f64; C]; R]) {
        debug_assert!(self.nrows >= R);
        debug_assert!(self.ncols >= C);
        for (i, row) in vals.into_iter().enumerate() {
            for (k, v) in row.into_iter().enumerate() {
                self[(i, k)] = scale * v;
            }
        }
    }

    /// Set every entry of this window to `value`.
    pub fn fill(&mut self, value: f64) {
        for i in 0..self.nrows {
            let start = (self.row0 + i) * self.stride + self.col0;
            self.data[start..start + self.ncols].fill(value);
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

/// Placement of one row block inside a [`BlockSparseMatrix`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockMeta {
    row0: usize,
    nrows: usize,
    ncols: usize,
    /// Offset of the block's values in the matrix's value storage.
    val_off: usize,
    /// Offset of the block's column map in the matrix's column storage.
    col_off: usize,
}

/// Sparse \(m \times n\) matrix made of dense **row blocks**.
///
/// Block `b` covers rows `row0..row0 + nrows` (each row belongs to exactly one
/// block) and the columns listed in its column map; its values are stored
/// contiguously, row-major. The matrix is the **sum** of all blocks placed at
/// their rows / mapped columns, so a column listed twice in one block
/// accumulates both local columns.
///
/// Write a block through [`Self::block_mut`], the dense submatrix
/// \(J[\text{rows}_b, \text{cols}_b]\). There is deliberately no mutable
/// global `(i, j)` access: a structural zero has no storage and a duplicate
/// column is a sum of two slots.
#[derive(Clone, Debug, Default)]
pub struct BlockSparseMatrix {
    nrows: usize,
    ncols: usize,
    blocks: Vec<BlockMeta>,
    cols: Vec<usize>,
    values: Vec<f64>,
}

/// Read-only view of one row block of a [`BlockSparseMatrix`].
#[derive(Clone, Copy, Debug)]
pub struct Block<'a> {
    row0: usize,
    nrows: usize,
    cols: &'a [usize],
    values: &'a [f64],
}

impl<'a> Block<'a> {
    /// Global rows covered by this block.
    pub fn rows(&self) -> std::ops::Range<usize> {
        self.row0..self.row0 + self.nrows
    }

    /// Global column of each local column.
    pub fn cols(&self) -> &'a [usize] {
        self.cols
    }

    /// Local row count (`rows().len()`).
    pub fn nrows(&self) -> usize {
        self.nrows
    }

    /// Local column count (`cols().len()`).
    pub fn ncols(&self) -> usize {
        self.cols.len()
    }

    /// Row-major `nrows × ncols` values.
    pub fn values(&self) -> &'a [f64] {
        self.values
    }

    /// Values of each local row (`ncols` entries each), in row order.
    /// Empty for a block without columns.
    pub fn row_values(&self) -> std::slice::ChunksExact<'a, f64> {
        self.values.chunks_exact(self.ncols().max(1))
    }
}

/// Read block entry `(i, k)`: local row `i`, local column `k`.
impl Index<(usize, usize)> for Block<'_> {
    type Output = f64;
    #[inline]
    fn index(&self, (i, k): (usize, usize)) -> &f64 {
        debug_assert!(i < self.nrows && k < self.ncols());
        &self.values[i * self.ncols() + k]
    }
}

impl BlockSparseMatrix {
    /// Empty matrix with `ncols` columns; rows grow with [`Self::push_block`].
    pub fn new(ncols: usize) -> Self {
        Self {
            ncols,
            ..Self::default()
        }
    }

    /// Append a zero block covering the next `nrows` rows; `cols` maps its
    /// local columns to matrix columns. Returns the block index.
    pub fn push_block(&mut self, nrows: usize, cols: &[usize]) -> usize {
        debug_assert!(cols.iter().all(|&c| c < self.ncols));
        self.blocks.push(BlockMeta {
            row0: self.nrows,
            nrows,
            ncols: cols.len(),
            val_off: self.values.len(),
            col_off: self.cols.len(),
        });
        self.cols.extend_from_slice(cols);
        self.values.resize(self.values.len() + nrows * cols.len(), 0.0);
        self.nrows += nrows;
        self.blocks.len() - 1
    }

    pub fn nrows(&self) -> usize {
        self.nrows
    }

    pub fn ncols(&self) -> usize {
        self.ncols
    }

    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Read-only view of block `b`.
    pub fn block(&self, b: usize) -> Block<'_> {
        self.view(&self.blocks[b])
    }

    /// All blocks in row order.
    pub fn blocks(&self) -> impl ExactSizeIterator<Item = Block<'_>> + '_ {
        self.blocks.iter().map(|m| self.view(m))
    }

    /// Dense submatrix of block `b` (local rows / columns).
    pub fn block_mut(&mut self, b: usize) -> DenseMatrix<'_> {
        let m = self.blocks[b];
        DenseMatrix::packed(
            &mut self.values[m.val_off..m.val_off + m.nrows * m.ncols],
            m.nrows,
            m.ncols,
        )
    }

    /// Set every stored entry to `value`.
    pub fn fill(&mut self, value: f64) {
        self.values.fill(value);
    }

    /// \(g = J^\top r\) (overwrites `g`).
    pub fn transpose_mul(&self, r: &[f64], g: &mut [f64]) {
        debug_assert_eq!(r.len(), self.nrows);
        debug_assert_eq!(g.len(), self.ncols);
        g.fill(0.0);
        for b in self.blocks() {
            for (vals, row) in b.row_values().zip(b.rows()) {
                let ri = r[row];
                for (&v, &c) in vals.iter().zip(b.cols) {
                    g[c] += v * ri;
                }
            }
        }
    }

    /// Add the matrix into a row-major `nrows × ncols` buffer (caller zeroes it).
    pub fn add_to_dense(&self, out: &mut [f64]) {
        let n = self.ncols;
        debug_assert_eq!(out.len(), self.nrows * n);
        for b in self.blocks() {
            for (vals, row) in b.row_values().zip(b.rows()) {
                let dst = &mut out[row * n..(row + 1) * n];
                for (&v, &c) in vals.iter().zip(b.cols) {
                    dst[c] += v;
                }
            }
        }
    }

    fn view(&self, m: &BlockMeta) -> Block<'_> {
        Block {
            row0: m.row0,
            nrows: m.nrows,
            cols: &self.cols[m.col_off..m.col_off + m.ncols],
            values: &self.values[m.val_off..m.val_off + m.nrows * m.ncols],
        }
    }
}

#[cfg(test)]
#[path = "tests_matrix.rs"]
mod tests_matrix;
