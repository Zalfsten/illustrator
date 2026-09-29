use super::*;
use crate::constraint::Constraint;
use crate::geom::{GeometryBody, GeometryId, GeometryRole, Point2};
use crate::geometries::Segment;

#[test]
fn residual_arity_stacks() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    doc.add_constraint(Constraint::Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Constraint::Parallel {
        a: GeometryId(0),
        b: GeometryId(1),
    });
    doc.add_constraint(Constraint::Orthogonal {
        a: GeometryId(0),
        b: GeometryId(1),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    assert_eq!(sys.n_residuals(), 2 + 1 + 1 + 2);
}

#[test]
fn assembled_jacobian_matches_global_fd() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let _s1 = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(1.0, 0.5), Point2::new(2.0, 0.5)));
    doc.add_constraint(Constraint::Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(GeometryId(1)),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: Segment::start_expr(s0),
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
