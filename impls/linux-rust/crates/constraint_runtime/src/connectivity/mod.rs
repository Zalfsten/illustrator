//! Explicit local→global degree-of-freedom connectivity (FEM-style scatter map).

/// Maps local DOF indices `0..len` to global indices in the state vector `p`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connectivity {
    pub global_indices: Vec<usize>,
}

impl Connectivity {
    pub fn new(global_indices: Vec<usize>) -> Self {
        Self { global_indices }
    }

    pub fn len(&self) -> usize {
        self.global_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.global_indices.is_empty()
    }

    pub fn gather(&self, p: &[f64]) -> Vec<f64> {
        self.global_indices.iter().map(|&g| p[g]).collect()
    }
}

#[cfg(test)]
mod tests;
