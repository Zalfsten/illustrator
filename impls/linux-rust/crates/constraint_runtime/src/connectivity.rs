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
mod tests {
    use super::*;

    #[test]
    fn gather_reads_mapped_entries() {
        let p = vec![10.0, 20.0, 30.0, 40.0];
        let c = Connectivity::new(vec![1, 3]);
        assert_eq!(c.gather(&p), vec![20.0, 40.0]);
    }
}
