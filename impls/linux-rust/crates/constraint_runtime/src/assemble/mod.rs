//! FEM-style assembly of global residual and Jacobian from local contributions.

use crate::constraint::Constraint;
use crate::document::{Document, DocumentStructure};
use crate::geom::Segment;
use crate::solver::SystemEval;
use crate::SOLVE_TOL;

/// Relative weight of structural residuals vs soft terms during interactive solves.
/// Must stay modest vs soft Jacobian scale: at canvas/pixel coordinates (~1e2),
/// values ≫ 1e3 make soft mouse terms numerically invisible and trap drags in
/// the previous nullspace (e.g. only vertical motion on an axis-aligned right angle).
pub const HARD_WEIGHT_WITH_SOFT: f64 = 1e3;

/// Soft residual: sqrt(w) * (p[index] - target); local J is scalar sqrt(w).
#[derive(Clone, Debug)]
pub struct SoftContribution {
    pub index: usize,
    pub target: f64,
    pub weight: f64,
}

impl SoftContribution {
    fn residual_arity(&self) -> usize {
        1
    }

    fn eval_residual(&self, p: &[f64], out: &mut [f64]) {
        out[0] = self.weight.sqrt() * (p[self.index] - self.target);
    }

    fn eval_jacobian(&self, _p: &[f64], out: &mut [f64]) {
        out[0] = self.weight.sqrt();
    }

    fn param_index(&self) -> usize {
        self.index
    }
}

/// Assembled structural (+ optional soft) system implementing [`SystemEval`].
pub struct AssembledSystem<'a> {
    pub segments: &'a [Segment],
    pub constraints: &'a [Constraint],
    pub soft: &'a [SoftContribution],
    n_params: usize,
    n_res: usize,
    /// Starting residual row for each structural constraint.
    structural_row0: Vec<usize>,
    soft_row0: usize,
}

impl<'a> AssembledSystem<'a> {
    pub fn from_structure(
        structure: &'a DocumentStructure,
        n_params: usize,
        soft: &'a [SoftContribution],
    ) -> Self {
        debug_assert_eq!(n_params, structure.parameter_count());
        let mut structural_row0 = Vec::with_capacity(structure.constraints().len());
        let mut row = 0usize;
        for c in structure.constraints() {
            structural_row0.push(row);
            row += c.residual_arity();
        }
        let soft_row0 = row;
        row += soft.iter().map(SoftContribution::residual_arity).sum::<usize>();
        Self {
            segments: structure.segments(),
            constraints: structure.constraints(),
            soft,
            n_params,
            n_res: row,
            structural_row0,
            soft_row0,
        }
    }

    pub fn from_document(doc: &'a Document, soft: &'a [SoftContribution]) -> Self {
        Self::from_structure(&doc.structure, doc.state().len(), soft)
    }
}

impl SystemEval for AssembledSystem<'_> {
    fn n_params(&self) -> usize {
        self.n_params
    }

    fn n_residuals(&self) -> usize {
        self.n_res
    }

    fn residual(&self, p: &[f64], r: &mut [f64]) {
        assert_eq!(r.len(), self.n_res);
        r.fill(0.0);
        let hard = if self.soft.is_empty() {
            1.0
        } else {
            HARD_WEIGHT_WITH_SOFT
        };
        for (ci, c) in self.constraints.iter().enumerate() {
            let m = c.residual_arity();
            let row0 = self.structural_row0[ci];
            c.eval_residual(self.segments, p, &mut r[row0..row0 + m]);
            for x in &mut r[row0..row0 + m] {
                *x *= hard;
            }
        }
        for (si, s) in self.soft.iter().enumerate() {
            let row = self.soft_row0 + si;
            s.eval_residual(p, &mut r[row..row + 1]);
        }
    }

    fn jacobian(&self, p: &[f64], j: &mut [f64]) {
        let n = self.n_params;
        let m = self.n_res;
        assert_eq!(j.len(), m * n);
        j.fill(0.0);
        let hard = if self.soft.is_empty() {
            1.0
        } else {
            HARD_WEIGHT_WITH_SOFT
        };

        for (ci, c) in self.constraints.iter().enumerate() {
            let conn = c.param_connectivity(self.segments);
            let n_local = conn.len();
            let m_local = c.residual_arity();
            let mut j_local = vec![0.0; m_local * n_local];
            c.eval_jacobian(self.segments, p, &mut j_local);
            let row0 = self.structural_row0[ci];
            for i in 0..m_local {
                for (lj, &g) in conn.global_indices.iter().enumerate() {
                    j[(row0 + i) * n + g] += hard * j_local[i * n_local + lj];
                }
            }
        }

        for (si, s) in self.soft.iter().enumerate() {
            let row = self.soft_row0 + si;
            let mut j_local = [0.0];
            s.eval_jacobian(p, &mut j_local);
            j[row * n + s.param_index()] += j_local[0];
        }
    }
}

pub fn structural_residual_norm(doc: &Document) -> f64 {
    structural_residual_norm_at(&doc.structure, doc.state())
}

pub fn structural_residual_norm_at(structure: &DocumentStructure, p: &[f64]) -> f64 {
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_structure(structure, p.len(), &soft);
    let mut r = vec![0.0; sys.n_residuals()];
    sys.residual(p, &mut r);
    r.iter().map(|x| x * x).sum::<f64>().sqrt()
}

pub fn is_structurally_satisfied(doc: &Document) -> bool {
    structural_residual_norm(doc) <= SOLVE_TOL
}

#[cfg(test)]
mod tests;
