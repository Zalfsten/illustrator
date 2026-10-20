//! FEM-style assembly of global residual and Jacobian from local contributions.

use crate::document::Document;
use crate::expr::{LengthExpr, PointExpr};
use crate::geom::Point2;
use crate::matrix::BlockSparseMatrix;
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
/// VectorsParallel / VectorsOrthogonal geometry. Inactive when \(L \ge \ell_{\min}\).
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
/// Contributions are stacked in order: structural constraints, soft slots,
/// soft point follows, soft length floors — one residual-row range and one
/// Jacobian block each. Parameter connectivity depends only on the document
/// graph; it is gathered once into the block structure of
/// [`SystemEval::new_jacobian`] (columns = State indices, no reduction).
pub struct AssembledSystem<'a> {
    pub doc: &'a Document,
    pub soft: &'a [SoftContribution],
    pub soft_points: &'a [SoftPointFollow],
    pub soft_lengths: &'a [SoftLengthFloor],
    n_res: usize,
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
        let n_res = doc
            .constraints()
            .values()
            .map(|c| c.body().residual_arity())
            .sum::<usize>()
            + soft.iter().map(SoftContribution::residual_arity).sum::<usize>()
            + soft_points.iter().map(SoftPointFollow::residual_arity).sum::<usize>()
            + soft_lengths.iter().map(SoftLengthFloor::residual_arity).sum::<usize>();
        Self {
            doc,
            soft,
            soft_points,
            soft_lengths,
            n_res,
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

    fn new_jacobian(&self) -> BlockSparseMatrix {
        let doc = self.doc;
        let mut j = BlockSparseMatrix::new(doc.parameter_count());
        for c in doc.constraints().values() {
            let body = c.body();
            j.push_block(body.residual_arity(), &body.param_connectivity(doc));
        }
        for s in self.soft {
            j.push_block(s.residual_arity(), &[s.param_index()]);
        }
        for sp in self.soft_points {
            j.push_block(sp.residual_arity(), &sp.point.param_indices(doc));
        }
        for sl in self.soft_lengths {
            j.push_block(sl.residual_arity(), &sl.length.param_indices(doc));
        }
        debug_assert_eq!(j.nrows(), self.n_res);
        j
    }

    fn residual(&self, p: &Vector, r: &mut Vector) {
        assert_eq!(r.len(), self.n_res);
        let mut row = 0;
        let mut rows = |m: usize| {
            row += m;
            row - m..row
        };
        for entry in self.doc.constraints().values() {
            let c = entry.body();
            c.eval_residual(self.doc, p, &mut r[rows(c.residual_arity())]);
        }
        for s in self.soft {
            s.eval_residual(p, &mut r[rows(1)]);
        }
        for sp in self.soft_points {
            let out = &mut r[rows(2)];
            let pt = sp.point.evaluate(self.doc, p);
            let s = sp.weight.sqrt();
            out[0] = s * (pt.x - sp.target.x);
            out[1] = s * (pt.y - sp.target.y);
        }
        for sl in self.soft_lengths {
            let len = sl.length.evaluate(self.doc, p).value();
            let deficit = (sl.min_length - len).max(0.0);
            r[rows(1)][0] = sl.weight.sqrt() * deficit;
        }
    }

    /// Each block is zeroed before its contribution writes, so `+=` writers
    /// (e.g. contracted vector Jacobians) start clean and inactive one-sided
    /// terms stay zero.
    fn jacobian(&self, p: &Vector, j: &mut BlockSparseMatrix) {
        assert_eq!(j.nrows(), self.n_res);
        let mut blocks = 0..j.num_blocks();
        let mut next = || blocks.next().expect("Jacobian from new_jacobian()");

        for entry in self.doc.constraints().values() {
            let mut jac = j.block_mut(next());
            jac.fill(0.0);
            entry.body().accumulate_jacobian(self.doc, p, &mut jac);
        }

        for s in self.soft {
            let mut jac = j.block_mut(next());
            jac[(0, 0)] = s.weight.sqrt();
        }

        for sp in self.soft_points {
            let mut jac = j.block_mut(next());
            jac.fill(0.0);
            sp.point
                .accumulate_jacobian(self.doc, p, &mut jac, sp.weight.sqrt());
        }

        for sl in self.soft_lengths {
            let mut jac = j.block_mut(next());
            jac.fill(0.0);
            let len = sl.length.evaluate(self.doc, p).value();
            if len < sl.min_length {
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
