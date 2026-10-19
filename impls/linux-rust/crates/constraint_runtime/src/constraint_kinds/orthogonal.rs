//! Orthogonal free vectors (one residual: dot product).

use crate::constraint::{pair_connectivity, unique_geometry_ids, ConstraintKind};
use crate::document::Document;
use crate::expr::VecExpr;
use crate::geometry::GeometryId;
use crate::matrix::MatrixAccess;
use crate::vector::Vector;

#[derive(Clone, Debug, PartialEq)]
pub struct Orthogonal {
    pub a: VecExpr,
    pub b: VecExpr,
}

impl ConstraintKind for Orthogonal {
    const RESIDUAL_ARITY: usize = 1;

    fn param_connectivity(&self, doc: &Document) -> Vec<usize> {
        pair_connectivity(doc, &self.a, &self.b)
    }

    fn eval_residual(&self, doc: &Document, p: &Vector, out: &mut [f64]) {
        let va = self.a.evaluate(doc, p);
        let vb = self.b.evaluate(doc, p);
        out[0] = va.dot(vb);
    }

    fn accumulate_jacobian<J: MatrixAccess>(&self, doc: &Document, p: &Vector, j: &mut J) {
        // r = va · vb; ∂r/∂va = vb, ∂r/∂vb = va
        let ja = self.a.jet(doc, p);
        let jb = self.b.jet(doc, p);
        ja.contract_into(j, 0, jb.value, 1.0);
        jb.contract_into(j, ja.cols(), ja.value, 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        unique_geometry_ids([self.a.referenced_geometry(), self.b.referenced_geometry()])
    }
}
