//! FEM-style assembly of global residual and Jacobian from local contributions.

use crate::document::Document;
use crate::matrix::Matrix;
use crate::solver::SystemEval;
use crate::vector::{VecAccess, Vector};
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

    fn eval_residual<R: VecAccess>(&self, p: &Vector, out: &mut R) {
        out[0] = self.weight.sqrt() * (p[self.index] - self.target);
    }

    fn param_index(&self) -> usize {
        self.index
    }
}

/// Assembled structural (+ optional soft) system implementing [`SystemEval`].
///
/// Holds `&Document` for the discrete graph; evaluates against an explicit
/// Vector (committed or working copy) passed to residual/jacobian.
///
/// Parameter connectivity is cached at construction — it depends only on the
/// document graph, not on `p`, so Jacobian assembly does not rebuild index lists.
pub struct AssembledSystem<'a> {
    pub doc: &'a Document,
    pub soft: &'a [SoftContribution],
    n_res: usize,
    /// Starting residual row for each structural constraint.
    structural_row0: Vec<usize>,
    /// Cached [`Constraint::param_connectivity`] per structural constraint.
    structural_conn: Vec<Vec<usize>>,
    soft_row0: usize,
}

impl<'a> AssembledSystem<'a> {
    pub fn new(doc: &'a Document, soft: &'a [SoftContribution]) -> Self {
        debug_assert_eq!(doc.vector().len(), doc.parameter_count());
        let n_c = doc.constraints().len();
        let mut structural_row0 = Vec::with_capacity(n_c);
        let mut structural_conn = Vec::with_capacity(n_c);
        let mut row = 0usize;
        for c in doc.constraints().values() {
            let body = c.body();
            structural_row0.push(row);
            row += body.residual_arity();
            structural_conn.push(body.param_connectivity(doc));
        }
        let soft_row0 = row;
        row += soft
            .iter()
            .map(SoftContribution::residual_arity)
            .sum::<usize>();
        Self {
            doc,
            soft,
            n_res: row,
            structural_row0,
            structural_conn,
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

    fn residual(&self, p: &Vector, r: &mut Vector) {
        assert_eq!(r.len(), self.n_res);
        r.fill(0.0);
        let mut r_view = r.as_dense();
        for (ci, entry) in self.doc.constraints().values().enumerate() {
            let c = entry.body();
            let m = c.residual_arity();
            let row0 = self.structural_row0[ci];
            let mut out = r_view.block_slice(row0, m);
            c.eval_residual(self.doc, p, &mut out);
        }
        for (si, s) in self.soft.iter().enumerate() {
            let row = self.soft_row0 + si;
            let mut out = r_view.block_slice(row, 1);
            s.eval_residual(p, &mut out);
        }
    }

    fn jacobian(&self, p: &Vector, j: &mut Matrix) {
        assert_eq!(j.nrows(), self.n_res);
        assert_eq!(j.ncols(), self.n_params());
        j.fill(0.0);
        let mut mat = j.as_dense();

        for (ci, entry) in self.doc.constraints().values().enumerate() {
            let c = entry.body();
            let conn = &self.structural_conn[ci];
            let m_local = c.residual_arity();
            let row0 = self.structural_row0[ci];
            let mut jac = mat.scatter_slice(row0, m_local, conn);
            c.accumulate_jacobian(self.doc, p, &mut jac);
        }

        for (si, s) in self.soft.iter().enumerate() {
            let row = self.soft_row0 + si;
            let conn = [s.param_index()];
            let mut jac = mat.scatter_slice(row, 1, &conn);
            jac[(0, 0)] = s.weight.sqrt();
        }
    }
}

pub fn structural_residual_norm(doc: &Document) -> f64 {
    structural_residual_norm_at(doc, doc.vector())
}

pub fn structural_residual_norm_at(doc: &Document, p: &Vector) -> f64 {
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::new(doc, &soft);
    let mut r = Vector::zeros(sys.n_residuals());
    sys.residual(p, &mut r);
    r.iter().map(|x| x * x).sum::<f64>().sqrt()
}

pub fn is_structurally_satisfied(doc: &Document) -> bool {
    structural_residual_norm(doc) <= SOLVE_TOL
}

#[cfg(test)]
#[path = "tests_assemble.rs"]
mod tests_assemble;
