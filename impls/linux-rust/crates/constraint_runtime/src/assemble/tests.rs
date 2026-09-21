use super::*;
use crate::constraint::{Constraint, EndpointRef, SegmentEnd};
use crate::geom::{Point2, PointExpr, SegmentEndKind, SegmentId};

#[test]
fn residual_arity_stacks() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let s1 = doc.add_segment(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
    doc.join_endpoints(
        EndpointRef::new(s0, SegmentEnd::End),
        EndpointRef::new(s1, SegmentEnd::Start),
    );
    doc.add_constraint(Constraint::Parallel {
        a: SegmentId(0),
        b: SegmentId(1),
    });
    doc.add_constraint(Constraint::Orthogonal {
        a: SegmentId(0),
        b: SegmentId(1),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    assert_eq!(sys.n_residuals(), 2 + 1 + 1 + 2);
}

#[test]
fn assembled_jacobian_matches_global_fd() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let _s1 = doc.add_segment(Point2::new(1.0, 0.5), Point2::new(2.0, 0.5));
    doc.join_endpoints(
        EndpointRef::new(s0, SegmentEnd::End),
        EndpointRef::new(SegmentId(1), SegmentEnd::Start),
    );
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::SegmentEnd {
            segment: s0,
            end: SegmentEndKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });

    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_structure(&doc.structure, doc.state().len(), &soft);
    let p = doc.state().to_vec();
    let m = sys.n_residuals();
    let n = sys.n_params();
    let mut j = vec![0.0; m * n];
    sys.jacobian(&p, &mut j);

    let eps = 1e-7;
    let mut r0 = vec![0.0; m];
    sys.residual(&p, &mut r0);
    for col in 0..n {
        let mut p2 = p.clone();
        p2[col] += eps;
        let mut rp = vec![0.0; m];
        sys.residual(&p2, &mut rp);
        for row in 0..m {
            let fd = (rp[row] - r0[row]) / eps;
            let an = j[row * n + col];
            assert!(
                (fd - an).abs() < 1e-5,
                "row={row} col={col} fd={fd} an={an}"
            );
        }
    }
}
