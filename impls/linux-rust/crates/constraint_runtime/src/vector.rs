//! Owned 1-D vectors and dense / scatter vector views.
//!
//! [`Vector`] owns a contiguous `f64` buffer (parameter state \(p\), residual
//! \(r\), …). Borrowed windows are [`DenseVec`] (contiguous) or [`ScatterVec`]
//! (local index → connectivity). Shared semantics: [`VecView`] / [`VecAccess`].
//!
//! For matrices see [`crate::matrix`].

use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Owned contiguous `f64` vector (counterpart to [`crate::matrix::Matrix`]).
///
/// Used for the sketch parameter state \(p\), assembled residuals \(r\), and
/// other 1-D buffers. Indexing via [`Deref`] to `[f64]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vector(Vec<f64>);

impl Vector {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn zeros(n: usize) -> Self {
        Self(vec![0.0; n])
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn from_vec(v: Vec<f64>) -> Self {
        Self(v)
    }

    pub fn as_slice(&self) -> &[f64] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.0
    }

    /// Mutable dense view of the whole vector.
    pub fn as_dense(&mut self) -> DenseVec<'_> {
        DenseVec::packed(&mut self.0)
    }

    /// Contiguous parameter window `[start, start+len)` (geometry chart block).
    pub fn block_slice(&mut self, start: usize, len: usize) -> DenseVec<'_> {
        DenseVec::new(&mut self.0, start, len)
    }

    /// Non-contiguous DOFs via connectivity (e.g. soft mouse index set).
    pub fn scatter_slice<'c>(&mut self, conn: &'c [usize]) -> ScatterVec<'_, 'c> {
        ScatterVec::new(&mut self.0, conn)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_vec(&self) -> Vec<f64> {
        self.0.clone()
    }

    pub fn into_vec(self) -> Vec<f64> {
        self.0
    }

    pub fn copy_from_slice(&mut self, p: &[f64]) {
        assert_eq!(
            p.len(),
            self.0.len(),
            "Vector length mismatch: {} vs {}",
            self.0.len(),
            p.len()
        );
        self.0.copy_from_slice(p);
    }

    pub fn copy_from(&mut self, other: &Vector) {
        self.copy_from_slice(other.as_slice());
    }

    pub fn extend_from_slice(&mut self, params: &[f64]) {
        self.0.extend_from_slice(params);
    }

    pub fn drain(&mut self, range: std::ops::Range<usize>) {
        self.0.drain(range);
    }

    pub fn fill(&mut self, value: f64) {
        self.0.fill(value);
    }

    pub fn iter(&self) -> std::slice::Iter<'_, f64> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, f64> {
        self.0.iter_mut()
    }
}

impl From<Vec<f64>> for Vector {
    fn from(v: Vec<f64>) -> Self {
        Self(v)
    }
}

impl From<&[f64]> for Vector {
    fn from(p: &[f64]) -> Self {
        Self(p.to_vec())
    }
}

impl FromIterator<f64> for Vector {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for Vector {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<usize> for Vector {
    type Output = f64;

    fn index(&self, i: usize) -> &f64 {
        &self.0[i]
    }
}

impl IndexMut<usize> for Vector {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        &mut self.0[i]
    }
}

impl AsRef<[f64]> for Vector {
    fn as_ref(&self) -> &[f64] {
        &self.0
    }
}

impl AsMut<[f64]> for Vector {
    fn as_mut(&mut self) -> &mut [f64] {
        &mut self.0
    }
}

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
    pub fn block_slice(&mut self, start: usize, len: usize) -> DenseVec<'_> {
        debug_assert!(start + len <= self.len);
        DenseVec::new(self.data, self.start + start, len)
    }

    /// Scatter view: local `i` maps to `start + conn[i]` in the parent buffer.
    pub fn scatter_slice<'c>(&mut self, conn: &'c [usize]) -> ScatterVec<'_, 'c> {
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

/// Vector semantics on a dense local index space.
///
/// Used via generics (`V: VecView` / [`VecAccess`]) so accesses monomorphize.
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

/// Writable vector buffer: [`VecView`] plus `v[i]` indexing (e.g. residual path).
///
/// Concrete backends: [`DenseVec`], [`ScatterVec`]. Parallel to [`crate::matrix::MatrixAccess`]
/// on the matrix side — not the parameter vector [`Vector`].
pub trait VecAccess: VecView + Index<usize, Output = f64> + IndexMut<usize, Output = f64> {}
impl<T> VecAccess for T where T: VecView + Index<usize, Output = f64> + IndexMut<usize, Output = f64> {}

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
#[path = "tests_vector.rs"]
mod tests_vector;
