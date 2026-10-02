use super::*;
use crate::document::DocumentStructure;
use crate::geom::CurveParam;
use crate::geometry_kinds::Segment;
use crate::geometry::{Chart, CurveExprs, EndpointExprs, GeometryId, GeometryRole, GeometryShell};
use crate::state::State;

fn two_segs() -> (DocumentStructure, State) {
    let mut structure = DocumentStructure::new();
    structure.push_geometry(Segment {
        shell: GeometryShell::new(GeometryId(0), GeometryRole::Result, 0),
    });
    structure.push_geometry(Segment {
        shell: GeometryShell::new(
            GeometryId(1),
            GeometryRole::Result,
            Segment::PARAMETER_COUNT,
        ),
    });
    // s0: (0,0)-(1,0), s1: (0,0)-(0,1) perpendicular sharing origin conceptually
    let p = State::from(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    (structure, p)
}

#[test]
fn jacobian_matches_fd_for_catalog() {
    let (structure, p) = two_segs();
    let constraints = [
        Constraint::Congruence {
            a: Segment::start_expr(GeometryId(0)),
            b: Segment::start_expr(GeometryId(1)),
        },
        Constraint::Parallel {
            a: Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)),
            b: Segment::tangent_expr(GeometryId(1), CurveParam::Normalized(0.0)),
        },
        Constraint::Orthogonal {
            a: Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)),
            b: Segment::tangent_expr(GeometryId(1), CurveParam::Normalized(0.0)),
        },
        Constraint::FixPoint {
            point: Segment::start_expr(GeometryId(0)),
            target: Point2::new(0.0, 0.0),
        },
        Constraint::Congruence {
            a: Segment::on_curve_expr(GeometryId(0), crate::geom::CurveParam::Uniform(0.5)),
            b: Segment::start_expr(GeometryId(1)),
        },
        Constraint::EqualLength {
            a: Segment::length_expr(GeometryId(0)),
            b: Segment::length_expr(GeometryId(1)),
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
