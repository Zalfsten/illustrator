//! Structural constraints: local residuals and analytical local Jacobians.

use crate::connectivity::Connectivity;
use crate::geom::{Point2, PointExpr, Segment, SegmentEndKind, SegmentId};

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
    pub fn param_connectivity(&self, segments: &[Segment]) -> Connectivity {
        match self {
            Constraint::Congruence { a, b } => {
                let ia = a.param_indices(segments);
                let ib = b.param_indices(segments);
                Connectivity::new(vec![ia[0], ia[1], ib[0], ib[1]])
            }
            Constraint::Parallel { a, b } | Constraint::Orthogonal { a, b } => {
                let sa = find(segments, *a);
                let sb = find(segments, *b);
                let mut idx = sa.connectivity().global_indices;
                idx.extend(sb.connectivity().global_indices);
                Connectivity::new(idx)
            }
            Constraint::FixPoint { point, .. } => {
                let i = point.param_indices(segments);
                Connectivity::new(vec![i[0], i[1]])
            }
        }
    }

    /// Local residual `r_N` in contribution order (length = residual_arity).
    pub fn eval_residual(&self, segments: &[Segment], p: &[f64], out: &mut [f64]) {
        match self {
            Constraint::Congruence { a, b } => {
                let pa = a.evaluate(segments, p);
                let pb = b.evaluate(segments, p);
                out[0] = pa.x - pb.x;
                out[1] = pa.y - pb.y;
            }
            Constraint::Parallel { a, b } => {
                let va = find(segments, *a).direction(p);
                let vb = find(segments, *b).direction(p);
                out[0] = va.x * vb.y - va.y * vb.x;
            }
            Constraint::Orthogonal { a, b } => {
                let va = find(segments, *a).direction(p);
                let vb = find(segments, *b).direction(p);
                out[0] = va.x * vb.x + va.y * vb.y;
            }
            Constraint::FixPoint { point, target } => {
                let q = point.evaluate(segments, p);
                out[0] = q.x - target.x;
                out[1] = q.y - target.y;
            }
        }
    }

    /// Local Jacobian `J_N` row-major, shape `(residual_arity × param_connectivity.len())`.
    pub fn eval_jacobian(&self, segments: &[Segment], p: &[f64], out: &mut [f64]) {
        let n_local = self.param_connectivity(segments).len();
        let m = self.residual_arity();
        assert_eq!(out.len(), m * n_local);
        out.fill(0.0);

        match self {
            Constraint::Congruence { .. } => {
                // r = [xa-xb, ya-yb], local order [xa,ya,xb,yb]
                out[0 * n_local + 0] = 1.0;
                out[0 * n_local + 2] = -1.0;
                out[1 * n_local + 1] = 1.0;
                out[1 * n_local + 3] = -1.0;
            }
            Constraint::Parallel { a, b } => {
                // r = vx_a * vy_b - vy_a * vx_b
                // v_a = (x2-x1, y2-y1) for segment a, similarly b
                // local: [ax1,ay1,ax2,ay2, bx1,by1,bx2,by2]
                let va = find(segments, *a).direction(p);
                let vb = find(segments, *b).direction(p);
                // d/d ax1 = -vy_b, d/d ay1 = vx_b, d/d ax2 = vy_b, d/d ay2 = -vx_b
                out[0] = -vb.y;
                out[1] = vb.x;
                out[2] = vb.y;
                out[3] = -vb.x;
                // d/d bx1 = vy_a, d/d by1 = -vx_a, d/d bx2 = -vy_a, d/d by2 = vx_a
                out[4] = va.y;
                out[5] = -va.x;
                out[6] = -va.y;
                out[7] = va.x;
            }
            Constraint::Orthogonal { a, b } => {
                // r = va · vb
                let va = find(segments, *a).direction(p);
                let vb = find(segments, *b).direction(p);
                out[0] = -vb.x;
                out[1] = -vb.y;
                out[2] = vb.x;
                out[3] = vb.y;
                out[4] = -va.x;
                out[5] = -va.y;
                out[6] = va.x;
                out[7] = va.y;
            }
            Constraint::FixPoint { .. } => {
                out[0 * n_local + 0] = 1.0;
                out[1 * n_local + 1] = 1.0;
            }
        }
    }
}

fn find(segments: &[Segment], id: SegmentId) -> &Segment {
    segments
        .iter()
        .find(|s| s.id == id)
        .expect("segment id for constraint")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::SEGMENT_STRIDE;

    fn two_segs() -> (Vec<Segment>, Vec<f64>) {
        let segments = vec![
            Segment {
                id: SegmentId(0),
                param_offset: 0,
            },
            Segment {
                id: SegmentId(1),
                param_offset: SEGMENT_STRIDE,
            },
        ];
        // s0: (0,0)-(1,0), s1: (0,0)-(0,1) perpendicular sharing origin conceptually
        let p = vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        (segments, p)
    }

    #[test]
    fn jacobian_matches_fd_for_catalog() {
        let (segments, p) = two_segs();
        let constraints = [
            Constraint::Congruence {
                a: PointExpr::SegmentEnd {
                    segment: SegmentId(0),
                    end: SegmentEndKind::Start,
                },
                b: PointExpr::SegmentEnd {
                    segment: SegmentId(1),
                    end: SegmentEndKind::Start,
                },
            },
            Constraint::Parallel {
                a: SegmentId(0),
                b: SegmentId(1),
            },
            Constraint::Orthogonal {
                a: SegmentId(0),
                b: SegmentId(1),
            },
            Constraint::FixPoint {
                point: PointExpr::SegmentEnd {
                    segment: SegmentId(0),
                    end: SegmentEndKind::Start,
                },
                target: Point2::new(0.0, 0.0),
            },
        ];

        let eps = 1e-7;
        for c in &constraints {
            let conn = c.param_connectivity(&segments);
            let m = c.residual_arity();
            let n = conn.len();
            let mut j_an = vec![0.0; m * n];
            c.eval_jacobian(&segments, &p, &mut j_an);

            let mut r0 = vec![0.0; m];
            c.eval_residual(&segments, &p, &mut r0);

            for (col, &g) in conn.global_indices.iter().enumerate() {
                let mut p2 = p.clone();
                p2[g] += eps;
                let mut rp = vec![0.0; m];
                c.eval_residual(&segments, &p2, &mut rp);
                for row in 0..m {
                    let fd = (rp[row] - r0[row]) / eps;
                    let an = j_an[row * n + col];
                    assert!(
                        (fd - an).abs() < 1e-5,
                        "constraint {c:?} row={row} col={col} fd={fd} an={an}"
                    );
                }
            }
        }
    }
}
