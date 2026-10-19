//! FEM-style assembly of global residual and Jacobian from local contributions.

use crate::document::Document;
use crate::expr::{LengthExpr, PointExpr};
use crate::geom::Point2;
use crate::matrix::Matrix;
use crate::solver::SystemEval;
use crate::vector::Vector;
use crate::SOLVE_TOL;

/// Soft residual on a single State slot: \(\sqrt{w}\,(p_i - t)\).
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

    fn eval_residual(&self, p: &Vector, out: &mut [f64]) {
        out[0] = self.weight.sqrt() * (p[self.index] - self.target);
    }

    fn param_index(&self) -> usize {
        self.index
    }
}

/// Soft residual that follows an opaque point expression: \(\sqrt{w}\,(E(p) - t)\).
///
/// Prefer this for pointer drag — formula-local indices are not world \((x,y)\)
/// for every kind (e.g. arc endpoints depend on center / radius / heading).
#[derive(Clone, Debug)]
pub struct SoftPointFollow {
    pub point: PointExpr,
    pub target: Point2,
    pub weight: f64,
}

impl SoftPointFollow {
    fn residual_arity(&self) -> usize {
        2
    }
}

/// Soft one-sided floor on a length expression: \(\sqrt{w}\,\max(0,\ell_{\min}-L)\).
///
/// Used during Interact to resist curve collapse without changing structural
/// Parallel / Orthogonal geometry. Inactive when \(L \ge \ell_{\min}\).
#[derive(Clone, Debug)]
pub struct SoftLengthFloor {
    pub length: LengthExpr,
    pub min_length: f64,
    pub weight: f64,
}

impl SoftLengthFloor {
    fn residual_arity(&self) -> usize {
        1
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
    pub soft_points: &'a [SoftPointFollow],
    pub soft_lengths: &'a [SoftLengthFloor],
    n_res: usize,
    /// Starting residual row for each structural constraint.
    structural_row0: Vec<usize>,
    /// Cached [`Constraint::param_connectivity`] per structural constraint.
    structural_conn: Vec<Vec<usize>>,
    soft_row0: usize,
    soft_point_row0: Vec<usize>,
    soft_point_conn: Vec<Vec<usize>>,
    soft_length_row0: Vec<usize>,
    soft_length_conn: Vec<Vec<usize>>,
}

impl<'a> AssembledSystem<'a> {
    pub fn new(doc: &'a Document, soft: &'a [SoftContribution]) -> Self {
        Self::with_soft(doc, soft, &[], &[])
    }

    pub fn with_soft(
        doc: &'a Document,
        soft: &'a [SoftContribution],
        soft_points: &'a [SoftPointFollow],
        soft_lengths: &'a [SoftLengthFloor],
    ) -> Self {
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
        let mut soft_point_row0 = Vec::with_capacity(soft_points.len());
        let mut soft_point_conn = Vec::with_capacity(soft_points.len());
        for sp in soft_points {
            soft_point_row0.push(row);
            row += sp.residual_arity();
            soft_point_conn.push(sp.point.param_indices(doc));
        }
        let mut soft_length_row0 = Vec::with_capacity(soft_lengths.len());
        let mut soft_length_conn = Vec::with_capacity(soft_lengths.len());
        for sl in soft_lengths {
            soft_length_row0.push(row);
            row += sl.residual_arity();
            soft_length_conn.push(sl.length.param_indices(doc));
        }
        Self {
            doc,
            soft,
            soft_points,
            soft_lengths,
            n_res: row,
            structural_row0,
            structural_conn,
            soft_row0,
            soft_point_row0,
            soft_point_conn,
            soft_length_row0,
            soft_length_conn,
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
        for (si, sp) in self.soft_points.iter().enumerate() {
            let row0 = self.soft_point_row0[si];
            let out = &mut r[row0..row0 + 2];
            let pt = sp.point.evaluate(self.doc, p);
            let s = sp.weight.sqrt();
            out[0] = s * (pt.x - sp.target.x);
            out[1] = s * (pt.y - sp.target.y);
        }
        for (si, sl) in self.soft_lengths.iter().enumerate() {
            let row0 = self.soft_length_row0[si];
            let out = &mut r[row0..row0 + 1];
            let len = sl.length.evaluate(self.doc, p).value();
            let deficit = (sl.min_length - len).max(0.0);
            out[0] = sl.weight.sqrt() * deficit;
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

        for (si, sp) in self.soft_points.iter().enumerate() {
            let row0 = self.soft_point_row0[si];
            let conn = &self.soft_point_conn[si];
            let mut jac = mat.scatter_slice(row0, 2, conn);
            sp.point
                .accumulate_jacobian(self.doc, p, &mut jac, sp.weight.sqrt());
        }

        for (si, sl) in self.soft_lengths.iter().enumerate() {
            let row0 = self.soft_length_row0[si];
            let conn = &self.soft_length_conn[si];
            let len = sl.length.evaluate(self.doc, p).value();
            if len < sl.min_length {
                let mut jac = mat.scatter_slice(row0, 1, conn);
                // r = √w (ℓ_min − L) ⇒ ∂r/∂p = −√w ∂L/∂p
                sl.length
                    .accumulate_jacobian(self.doc, p, &mut jac, -sl.weight.sqrt());
            }
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
