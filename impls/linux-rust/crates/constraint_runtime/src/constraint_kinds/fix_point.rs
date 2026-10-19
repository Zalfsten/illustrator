//! Fix a point expression to a target location (two residual rows).

use crate::constraint::ConstraintKind;
use crate::document::Document;
use crate::expr::PointExpr;
use crate::geom::Point2;
use crate::geometry::GeometryId;
use crate::matrix::MatrixAccess;
use crate::vector::Vector;

#[derive(Clone, Debug, PartialEq)]
pub struct FixPoint {
    pub point: PointExpr,
    pub target: Point2,
}

impl ConstraintKind for FixPoint {
    const RESIDUAL_ARITY: usize = 2;

    fn param_connectivity(&self, doc: &Document) -> Vec<usize> {
        self.point.param_indices(doc)
    }

    fn eval_residual(&self, doc: &Document, p: &Vector, out: &mut [f64]) {
        let q = self.point.evaluate(doc, p);
        out[0] = q.x - self.target.x;
        out[1] = q.y - self.target.y;
    }

    fn accumulate_jacobian<J: MatrixAccess>(&self, doc: &Document, p: &Vector, j: &mut J) {
        self.point.accumulate_jacobian(doc, p, j, 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        vec![self.point.referenced_geometry()]
    }
}
