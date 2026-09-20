//! FEM-style assembly of global residual and Jacobian from local contributions.

use crate::constraint::Constraint;
use crate::document::{Document, DocumentStructure};
use crate::geom::Segment;
use crate::solver::SystemEval;
use crate::SOLVE_TOL;

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
        let hard = if self.soft.is_empty() { 1.0 } else { 1e4 };
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
        let hard = if self.soft.is_empty() { 1.0 } else { 1e4 };

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
mod tests {
    use super::*;
    use crate::constraint::{EndpointRef, SegmentEnd};
    use crate::geom::{Point2, PointExpr, SegmentEndKind, SegmentId};

    #[test]
    fn residual_arity_stacks() {
        let mut doc = Document::new();
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let s1 = doc.add_segment(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
        doc.join_endpoints(
            EndpointRef::new(s0, SegmentEnd::End),
            EndpointRef::new(s1, SegmentEnd::Start),
        );
        doc.add_constraint(Constraint::Parallel {
            a: SegmentId(0),
            b: SegmentId(1),
        });
        doc.add_constraint(Constraint::Orthogonal {
            a: SegmentId(0),
            b: SegmentId(1),
        });
        doc.add_constraint(Constraint::FixPoint {
            point: PointExpr::SegmentEnd {
                segment: s0,
                end: SegmentEndKind::Start,
            },
            target: Point2::new(0.0, 0.0),
        });
        let soft: [SoftContribution; 0] = [];
        let sys = AssembledSystem::from_document(&doc, &soft);
        assert_eq!(sys.n_residuals(), 2 + 1 + 1 + 2);
    }

    #[test]
    fn assembled_jacobian_matches_global_fd() {
        let mut doc = Document::new();
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let _s1 = doc.add_segment(Point2::new(1.0, 0.5), Point2::new(2.0, 0.5));
        doc.join_endpoints(
            EndpointRef::new(s0, SegmentEnd::End),
            EndpointRef::new(SegmentId(1), SegmentEnd::Start),
        );
        doc.add_constraint(Constraint::FixPoint {
            point: PointExpr::SegmentEnd {
                segment: s0,
                end: SegmentEndKind::Start,
            },
            target: Point2::new(0.0, 0.0),
        });

        let soft: [SoftContribution; 0] = [];
        let sys = AssembledSystem::from_structure(&doc.structure, doc.state().len(), &soft);
        let p = doc.state().to_vec();
        let m = sys.n_residuals();
        let n = sys.n_params();
        let mut j = vec![0.0; m * n];
        sys.jacobian(&p, &mut j);

        let eps = 1e-7;
        let mut r0 = vec![0.0; m];
        sys.residual(&p, &mut r0);
        for col in 0..n {
            let mut p2 = p.clone();
            p2[col] += eps;
            let mut rp = vec![0.0; m];
            sys.residual(&p2, &mut rp);
            for row in 0..m {
                let fd = (rp[row] - r0[row]) / eps;
                let an = j[row * n + col];
                assert!(
                    (fd - an).abs() < 1e-5,
                    "row={row} col={col} fd={fd} an={an}"
                );
            }
        }
    }
}
