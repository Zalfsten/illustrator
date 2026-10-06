//! FEM-style assembly of global residual and Jacobian from local contributions.

use crate::constraint::LocalJacobian;
use crate::document::Document;
use crate::solver::SystemEval;
use crate::state::State;
use crate::SOLVE_TOL;

/// Soft residual: sqrt(w) * (p[index] - target); local J is scalar sqrt(w).
///
/// `weight` already includes the soft-vs-hard strength (see [`crate::SoftTerms`]).
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

    fn eval_residual(&self, p: &State, out: &mut [f64]) {
        out[0] = self.weight.sqrt() * (p[self.index] - self.target);
    }

    fn eval_jacobian(&self, _p: &State, out: &mut [f64]) {
        out[0] = self.weight.sqrt();
    }

    fn param_index(&self) -> usize {
        self.index
    }
}

/// Assembled structural (+ optional soft) system implementing [`SystemEval`].
///
/// Holds `&Document` for the discrete graph; evaluates against an explicit
/// State (committed or working copy) passed to residual/jacobian.
pub struct AssembledSystem<'a> {
    pub doc: &'a Document,
    pub soft: &'a [SoftContribution],
    n_res: usize,
    /// Starting residual row for each structural constraint.
    structural_row0: Vec<usize>,
    soft_row0: usize,
}

impl<'a> AssembledSystem<'a> {
    pub fn new(doc: &'a Document, soft: &'a [SoftContribution]) -> Self {
        debug_assert_eq!(doc.state().len(), doc.parameter_count());
        let mut structural_row0 = Vec::with_capacity(doc.constraints().len());
        let mut row = 0usize;
        for c in doc.constraints().values() {
            structural_row0.push(row);
            row += c.body().residual_arity();
        }
        let soft_row0 = row;
        row += soft.iter().map(SoftContribution::residual_arity).sum::<usize>();
        Self {
            doc,
            soft,
            n_res: row,
            structural_row0,
            soft_row0,
        }
    }

    pub fn from_document(doc: &'a Document, soft: &'a [SoftContribution]) -> Self {
        Self::new(doc, soft)
    }
}

impl SystemEval for AssembledSystem<'_> {
    fn n_params(&self) -> usize {
        self.doc.parameter_count()
    }

    fn n_residuals(&self) -> usize {
        self.n_res
    }

    fn residual(&self, p: &State, r: &mut [f64]) {
        assert_eq!(r.len(), self.n_res);
        r.fill(0.0);
        for (ci, entry) in self.doc.constraints().values().enumerate() {
            let c = entry.body();
            let m = c.residual_arity();
            let row0 = self.structural_row0[ci];
            c.eval_residual(self.doc, p, &mut r[row0..row0 + m]);
        }
        for (si, s) in self.soft.iter().enumerate() {
            let row = self.soft_row0 + si;
            s.eval_residual(p, &mut r[row..row + 1]);
        }
    }

    fn jacobian(&self, p: &State, j: &mut [f64]) {
        let n = self.n_params();
        let m = self.n_res;
        assert_eq!(j.len(), m * n);
        j.fill(0.0);

        for (ci, entry) in self.doc.constraints().values().enumerate() {
            let c = entry.body();
            let conn = c.param_connectivity(self.doc);
            let m_local = c.residual_arity();
            let row0 = self.structural_row0[ci];
            let mut jac = LocalJacobian::new(j, n, row0, m_local, &conn);
            c.accumulate_jacobian(self.doc, p, &mut jac);
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
    structural_residual_norm_at(doc, doc.state())
}

pub fn structural_residual_norm_at(doc: &Document, p: &State) -> f64 {
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::new(doc, &soft);
    let mut r = vec![0.0; sys.n_residuals()];
    sys.residual(p, &mut r);
    r.iter().map(|x| x * x).sum::<f64>().sqrt()
}

pub fn is_structurally_satisfied(doc: &Document) -> bool {
    structural_residual_norm(doc) <= SOLVE_TOL
}

#[cfg(test)]
#[path = "tests_assemble.rs"]
mod tests_assemble;
