//! Parallel free vectors (one residual: 2D cross).

use crate::constraint::{pair_connectivity, unique_geometry_ids, ConstraintKind};
use crate::document::Document;
use crate::expr::VecExpr;
use crate::geometry::GeometryId;
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

#[derive(Clone, Debug, PartialEq)]
pub struct Parallel {
    pub a: VecExpr,
    pub b: VecExpr,
}

impl ConstraintKind for Parallel {
    const RESIDUAL_ARITY: usize = 1;

    fn param_connectivity(&self, doc: &Document) -> Vec<usize> {
        pair_connectivity(doc, &self.a, &self.b)
    }

    fn eval_residual(&self, doc: &Document, p: &Vector, out: &mut [f64]) {
        let va = self.a.evaluate(doc, p);
        let vb = self.b.evaluate(doc, p);
        out[0] = va.cross(vb);
    }

    fn accumulate_jacobian(&self, doc: &Document, p: &Vector, j: &mut DenseMatrix<'_>) {
        // r = va × vb; ∂r/∂va = vb.right_normal(), ∂r/∂vb = va.left_normal()
        let ja = self.a.jet(doc, p);
        let jb = self.b.jet(doc, p);
        ja.contract_into(j, 0, jb.value.right_normal(), 1.0);
        jb.contract_into(j, ja.cols(), ja.value.left_normal(), 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        unique_geometry_ids([self.a.referenced_geometry(), self.b.referenced_geometry()])
    }
}
