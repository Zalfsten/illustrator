//! Point-point coincidence (two residual rows).

use crate::constraint::{pair_connectivity, unique_geometry_ids, ConstraintKind};
use crate::document::Document;
use crate::expr::PointExpr;
use crate::geometry::GeometryId;
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

#[derive(Clone, Debug, PartialEq)]
pub struct Congruence {
    pub a: PointExpr,
    pub b: PointExpr,
}

impl Congruence {
    /// Partner expression if `point` is one side of this identification.
    pub fn other(&self, point: &PointExpr) -> Option<PointExpr> {
        if &self.a == point {
            Some(self.b)
        } else if &self.b == point {
            Some(self.a)
        } else {
            None
        }
    }
}

impl ConstraintKind for Congruence {
    const RESIDUAL_ARITY: usize = 2;
    const NAME_PREFIX: &'static str = "join";

    fn param_connectivity(&self, doc: &Document) -> Vec<usize> {
        pair_connectivity(doc, &self.a, &self.b)
    }

    fn eval_residual(&self, doc: &Document, p: &Vector, out: &mut [f64]) {
        let pa = self.a.evaluate(doc, p);
        let pb = self.b.evaluate(doc, p);
        out[0] = pa.x - pb.x;
        out[1] = pa.y - pb.y;
    }

    fn accumulate_jacobian(&self, doc: &Document, p: &Vector, j: &mut DenseMatrix<'_>) {
        // r = a - b ⇒ ∂r/∂a = Ja, ∂r/∂b = -Jb
        let na = self.a.accumulate_jacobian(doc, p, j, 1.0);
        let mut jb = j.block_slice(0, na, j.nrows(), j.ncols() - na);
        self.b.accumulate_jacobian(doc, p, &mut jb, -1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        unique_geometry_ids([self.a.referenced_geometry(), self.b.referenced_geometry()])
    }
}
