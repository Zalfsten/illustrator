//! Fix a point expression to a target location (two residual rows).

use crate::constraint::ConstraintKind;
use crate::geom::Point2;
use crate::expr::PointExpr;
use crate::geometry::GeometryId;
use crate::matrix::MatrixAccess;
use crate::vector::{VecAccess, Vector};
use crate::store::GeometryStore;

#[derive(Clone, Debug, PartialEq)]
pub struct FixPoint {
    pub point: PointExpr,
    pub target: Point2,
}

impl ConstraintKind for FixPoint {
    const RESIDUAL_ARITY: usize = 2;

    fn param_connectivity<S: GeometryStore + ?Sized>(&self, store: &S) -> Vec<usize> {
        self.point.param_indices(store)
    }

    fn eval_residual<S: GeometryStore + ?Sized, R: VecAccess>(
        &self,
        store: &S,
        p: &Vector,
        out: &mut R,
    ) {
        let q = self.point.evaluate(store, p);
        out[0] = q.x - self.target.x;
        out[1] = q.y - self.target.y;
    }

    fn accumulate_jacobian<S: GeometryStore + ?Sized, J: MatrixAccess>(
        &self,
        store: &S,
        p: &Vector,
        j: &mut J,
    ) {
        self.point.accumulate_jacobian(store, p, j, 1.0);
    }

    fn referenced_geometries(&self) -> Vec<GeometryId> {
        vec![self.point.referenced_geometry()]
    }
}
