//! Structural constraints: local residuals and analytical local Jacobians.

use crate::connectivity::Connectivity;
use crate::geom::{Point2, PointExpr, SegmentEndKind, SegmentId, SegmentStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SegmentEnd {
    Start,
    End,
}

impl From<SegmentEnd> for SegmentEndKind {
    fn from(value: SegmentEnd) -> Self {
        match value {
            SegmentEnd::Start => SegmentEndKind::Start,
            SegmentEnd::End => SegmentEndKind::End,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EndpointRef {
    pub segment: SegmentId,
    pub end: SegmentEnd,
}

impl EndpointRef {
    pub fn new(segment: SegmentId, end: SegmentEnd) -> Self {
        Self { segment, end }
    }

    pub fn as_expr(self) -> PointExpr {
        PointExpr::SegmentEnd {
            segment: self.segment,
            end: self.end.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    Congruence { a: PointExpr, b: PointExpr },
    Parallel { a: SegmentId, b: SegmentId },
    Orthogonal { a: SegmentId, b: SegmentId },
    FixPoint { point: PointExpr, target: Point2 },
}

impl Constraint {
    pub fn residual_arity(&self) -> usize {
        match self {
            Constraint::Congruence { .. } | Constraint::FixPoint { .. } => 2,
            Constraint::Parallel { .. } | Constraint::Orthogonal { .. } => 1,
        }
    }

    /// Column connectivity: global parameter indices involved in this contribution.
    pub fn param_connectivity(&self, store: &impl SegmentStore) -> Connectivity {
        match self {
            Constraint::Congruence { a, b } => {
                let mut idx = a.param_indices(store);
                idx.extend(b.param_indices(store));
                Connectivity::new(idx)
            }
            Constraint::Parallel { a, b } | Constraint::Orthogonal { a, b } => {
                let sa = store.expect_segment(*a);
                let sb = store.expect_segment(*b);
                let mut idx = sa.connectivity().global_indices;
                idx.extend(sb.connectivity().global_indices);
                Connectivity::new(idx)
            }
            Constraint::FixPoint { point, .. } => {
                Connectivity::new(point.param_indices(store))
            }
        }
    }

    /// Local residual `r_N` in contribution order (length = residual_arity).
    pub fn eval_residual(&self, store: &impl SegmentStore, p: &[f64], out: &mut [f64]) {
        match self {
            Constraint::Congruence { a, b } => {
                let pa = a.evaluate(store, p);
                let pb = b.evaluate(store, p);
                out[0] = pa.x - pb.x;
                out[1] = pa.y - pb.y;
            }
            Constraint::Parallel { a, b } => {
                let va = store.expect_segment(*a).direction(p);
                let vb = store.expect_segment(*b).direction(p);
                out[0] = va.cross(vb);
            }
            Constraint::Orthogonal { a, b } => {
                let va = store.expect_segment(*a).direction(p);
                let vb = store.expect_segment(*b).direction(p);
                out[0] = va.dot(vb);
            }
            Constraint::FixPoint { point, target } => {
                let q = point.evaluate(store, p);
                out[0] = q.x - target.x;
                out[1] = q.y - target.y;
            }
        }
    }

    /// Local Jacobian `J_N` row-major, shape `(residual_arity × param_connectivity.len())`.
    pub fn eval_jacobian(&self, store: &impl SegmentStore, p: &[f64], out: &mut [f64]) {
        let n_local = self.param_connectivity(store).len();
        let m = self.residual_arity();
        assert_eq!(out.len(), m * n_local);
        out.fill(0.0);

        match self {
            Constraint::Congruence { a, b } => {
                let na = a.param_indices(store).len();
                let nb = b.param_indices(store).len();
                let mut dax = vec![0.0; na];
                let mut day = vec![0.0; na];
                let mut dbx = vec![0.0; nb];
                let mut dby = vec![0.0; nb];
                a.eval_point_jacobian(store, p, &mut dax, &mut day);
                b.eval_point_jacobian(store, p, &mut dbx, &mut dby);
                for i in 0..na {
                    out[0 * n_local + i] = dax[i];
                    out[1 * n_local + i] = day[i];
                }
                for j in 0..nb {
                    out[0 * n_local + na + j] = -dbx[j];
                    out[1 * n_local + na + j] = -dby[j];
                }
            }
            Constraint::Parallel { a, b } => {
                let va = store.expect_segment(*a).direction(p);
                let vb = store.expect_segment(*b).direction(p);
                out[0] = -vb.y;
                out[1] = vb.x;
                out[2] = vb.y;
                out[3] = -vb.x;
                out[4] = va.y;
                out[5] = -va.x;
                out[6] = -va.y;
                out[7] = va.x;
            }
            Constraint::Orthogonal { a, b } => {
                let va = store.expect_segment(*a).direction(p);
                let vb = store.expect_segment(*b).direction(p);
                out[0] = -vb.x;
                out[1] = -vb.y;
                out[2] = vb.x;
                out[3] = vb.y;
                out[4] = -va.x;
                out[5] = -va.y;
                out[6] = va.x;
                out[7] = va.y;
            }
            Constraint::FixPoint { point, .. } => {
                let n = point.param_indices(store).len();
                let mut dx = vec![0.0; n];
                let mut dy = vec![0.0; n];
                point.eval_point_jacobian(store, p, &mut dx, &mut dy);
                for i in 0..n {
                    out[0 * n_local + i] = dx[i];
                    out[1 * n_local + i] = dy[i];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
