use super::*;
use crate::document::DocumentStructure;
use crate::geom::{Geometry, GeometryId, Segment, SEGMENT_STRIDE};

fn two_segs() -> (DocumentStructure, Vec<f64>) {
    let mut structure = DocumentStructure::new();
    structure.push_geometry(Geometry::Segment(Segment {
        id: GeometryId(0),
        param_offset: 0,
    }));
    structure.push_geometry(Geometry::Segment(Segment {
        id: GeometryId(1),
        param_offset: SEGMENT_STRIDE,
    }));
    // s0: (0,0)-(1,0), s1: (0,0)-(0,1) perpendicular sharing origin conceptually
    let p = vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    (structure, p)
}

#[test]
fn jacobian_matches_fd_for_catalog() {
    let (structure, p) = two_segs();
    let constraints = [
        Constraint::Congruence {
            a: PointExpr::Endpoint {
                geometry: GeometryId(0),
                end: EndpointKind::Start,
            },
            b: PointExpr::Endpoint {
                geometry: GeometryId(1),
                end: EndpointKind::Start,
            },
        },
        Constraint::Parallel {
            a: GeometryId(0),
            b: GeometryId(1),
        },
        Constraint::Orthogonal {
            a: GeometryId(0),
            b: GeometryId(1),
        },
        Constraint::FixPoint {
            point: PointExpr::Endpoint {
                geometry: GeometryId(0),
                end: EndpointKind::Start,
            },
            target: Point2::new(0.0, 0.0),
        },
        Constraint::Congruence {
            a: PointExpr::OnCurve {
                geometry: GeometryId(0),
                param: crate::geom::CurveParam::Uniform(0.5),
            },
            b: PointExpr::Endpoint {
                geometry: GeometryId(1),
                end: EndpointKind::Start,
            },
        },
    ];

    let eps = 1e-7;
    for c in &constraints {
        let conn = c.param_connectivity(&structure);
        let m = c.residual_arity();
        let n = conn.len();
        let mut j_an = vec![0.0; m * n];
        c.eval_jacobian(&structure, &p, &mut j_an);

        let mut r0 = vec![0.0; m];
        c.eval_residual(&structure, &p, &mut r0);

        for (col, &g) in conn.global_indices.iter().enumerate() {
            let mut p2 = p.clone();
            p2[g] += eps;
            let mut rp = vec![0.0; m];
            c.eval_residual(&structure, &p2, &mut rp);
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
