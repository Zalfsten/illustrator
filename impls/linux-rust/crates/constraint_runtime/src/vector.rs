//! Owned 1-D vector (parameter state \(p\), residual \(r\), …).
//!
//! Borrowed windows are plain slices (`&mut r[row0..row0 + m]`). For matrices
//! see [`crate::matrix`].

use std::ops::{Deref, DerefMut};

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

#[cfg(test)]
#[path = "tests_vector.rs"]
mod tests_vector;
