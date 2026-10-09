use crate::constraint_kinds::{Congruence, ConstraintInst, EqualLength, FixPoint, Orthogonal, Parallel};
use crate::document::Document;
use crate::geom::{CurveParam, Point2};
use crate::geometry_kinds::Segment;
use crate::geometry::{Chart, CurveExprs, EndpointExprs, GeometryId, GeometryRole, GeometryShell};
use crate::matrix::Matrix;
use crate::vector::{DenseVec, Vector};

fn two_segs() -> (Document, Vector) {
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
    let p = Vector::from(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    (doc, p)
}

#[test]
fn jacobian_matches_fd_for_catalog() {
    let (doc, p) = two_segs();
    let constraints: [ConstraintInst; 6] = [
        Congruence {
            a: Segment::start_expr(GeometryId(0)),
            b: Segment::start_expr(GeometryId(1)),
        }
        .into(),
        Parallel {
            a: Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)),
            b: Segment::tangent_expr(GeometryId(1), CurveParam::Normalized(0.0)),
        }
        .into(),
        Orthogonal {
            a: Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)),
            b: Segment::tangent_expr(GeometryId(1), CurveParam::Normalized(0.0)),
        }
        .into(),
        FixPoint {
            point: Segment::start_expr(GeometryId(0)),
            target: Point2::new(0.0, 0.0),
        }
        .into(),
        Congruence {
            a: Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(0.5)),
            b: Segment::start_expr(GeometryId(1)),
        }
        .into(),
        EqualLength {
            a: Segment::length_expr(GeometryId(0)),
            b: Segment::length_expr(GeometryId(1)),
        }
        .into(),
    ];

    let eps = 1e-7;
    let n_global = p.len();
    for c in &constraints {
        let conn = c.param_connectivity(&doc);
        let m = c.residual_arity();
        let mut mat = Matrix::zeros(m, n_global);
        {
            let mut view = mat.as_dense();
            let mut jac = view.scatter_slice(0, m, &conn);
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
            for i in 0..m {
                let fd = (rp[i] - r0[i]) / eps;
                let analytic = mat[(i, g)];
                assert!(
                    (analytic - fd).abs() < 1e-5,
                    "constraint {c:?} col {g} row {i}: analytic={analytic} fd={fd}"
                );
            }
        }
    }
}
