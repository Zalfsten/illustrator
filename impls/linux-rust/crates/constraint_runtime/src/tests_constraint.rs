use super::*;
use crate::constraint_kinds::{Congruence, EqualLength, FixPoint, Orthogonal, Parallel};
use crate::document::Document;
use crate::geom::{CurveParam, Point2};
use crate::geometry_kinds::Segment;
use crate::geometry::{Chart, CurveExprs, EndpointExprs, GeometryId, GeometryRole, GeometryShell};
use crate::matrix::{DenseVec, Matrix};
use crate::state::State;

fn two_segs() -> (Document, State) {
    let mut doc = Document::new();
    doc.push_geometry(Segment {
        shell: GeometryShell::new(GeometryId(0), GeometryRole::Result, 0),
    });
    doc.push_geometry(Segment {
        shell: GeometryShell::new(
            GeometryId(1),
            GeometryRole::Result,
            Segment::PARAMETER_COUNT,
        ),
    });
    // s0: (0,0)-(1,0), s1: (0,0)-(0,1) perpendicular sharing origin conceptually
    let p = State::from(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    (doc, p)
}

#[test]
fn jacobian_matches_fd_for_catalog() {
    let (doc, p) = two_segs();
    let constraints: [Box<dyn Constraint>; 6] = [
        Box::new(Congruence {
            a: Segment::start_expr(GeometryId(0)),
            b: Segment::start_expr(GeometryId(1)),
        }),
        Box::new(Parallel {
            a: Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)),
            b: Segment::tangent_expr(GeometryId(1), CurveParam::Normalized(0.0)),
        }),
        Box::new(Orthogonal {
            a: Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)),
            b: Segment::tangent_expr(GeometryId(1), CurveParam::Normalized(0.0)),
        }),
        Box::new(FixPoint {
            point: Segment::start_expr(GeometryId(0)),
            target: Point2::new(0.0, 0.0),
        }),
        Box::new(Congruence {
            a: Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(0.5)),
            b: Segment::start_expr(GeometryId(1)),
        }),
        Box::new(EqualLength {
            a: Segment::length_expr(GeometryId(0)),
            b: Segment::length_expr(GeometryId(1)),
        }),
    ];

    let eps = 1e-7;
    let n_global = p.len();
    for c in &constraints {
        let conn = c.param_connectivity(&doc);
        let m = c.residual_arity();
        let mut mat = Matrix::zeros(m, n_global);
        {
            let mut view = mat.as_dense();
            let mut jac = view.scatter(0, m, &conn);
            c.accumulate_jacobian(&doc, &p, &mut jac);
        }

        let mut r0 = vec![0.0; m];
        {
            let mut out = DenseVec::packed(&mut r0);
            c.eval_residual(&doc, &p, &mut out);
        }

        for &g in &conn {
            let mut p2 = p.clone();
            p2[g] += eps;
            let mut rp = vec![0.0; m];
            {
                let mut out = DenseVec::packed(&mut rp);
                c.eval_residual(&doc, &p2, &mut out);
            }
            for row in 0..m {
                let fd = (rp[row] - r0[row]) / eps;
                let an = mat.as_slice()[row * n_global + g];
                assert!(
                    (fd - an).abs() < 1e-5,
                    "constraint {c:?} row={row} col={g} fd={fd} an={an}"
                );
            }
        }
    }
}
