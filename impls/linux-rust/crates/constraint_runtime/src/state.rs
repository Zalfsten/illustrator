//! Global continuous state vector \(p\) for a sketch.

use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Contiguous continuous parameters for a sketch (Model E global unknowns).
///
/// Owned working copies and the Document's committed state share this type.
/// Evaluate / assemble / solver APIs take `&State` / `&mut State` (indexing via
/// [`Deref`] to `[f64]` — same cost as a slice borrow).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State(Vec<f64>);

impl State {
    pub fn new() -> Self {
        Self(Vec::new())
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
            "State length mismatch: {} vs {}",
            self.0.len(),
            p.len()
        );
        self.0.copy_from_slice(p);
    }

    pub fn copy_from(&mut self, other: &State) {
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

impl From<Vec<f64>> for State {
    fn from(v: Vec<f64>) -> Self {
        Self(v)
    }
}

impl From<&[f64]> for State {
    fn from(p: &[f64]) -> Self {
        Self(p.to_vec())
    }
}

impl FromIterator<f64> for State {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Deref for State {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for State {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<usize> for State {
    type Output = f64;

    fn index(&self, i: usize) -> &f64 {
        &self.0[i]
    }
}

impl IndexMut<usize> for State {
    fn index_mut(&mut self, i: usize) -> &mut f64 {
        &mut self.0[i]
    }
}

impl AsRef<[f64]> for State {
    fn as_ref(&self) -> &[f64] {
        &self.0
    }
}

impl AsMut<[f64]> for State {
    fn as_mut(&mut self) -> &mut [f64] {
        &mut self.0
    }
}
