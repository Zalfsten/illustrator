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
        Constraint::Congruence {
            a: PointExpr::OnCurve {
                segment: SegmentId(0),
                param: crate::geom::CurveParam::Uniform(0.5),
            },
            b: PointExpr::SegmentEnd {
                segment: SegmentId(1),
                end: SegmentEndKind::Start,
            },
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
