use super::*;
use crate::constraint_kinds::{Congruence, EqualLength, FixPoint, Orthogonal, Parallel};
use crate::geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry_kinds::{Arc, Segment};
use crate::geometry::{CurveExprs, EndpointExprs, GeometryId, GeometryRole};
use crate::vector::Vector;

#[test]
fn residual_arity_stacks() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Parallel {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
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
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let _s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.5), Point2::new(2.0, 0.5)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(GeometryId(1)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });

    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    let p = doc.vector().clone();
    let m = sys.n_residuals();
    let n = sys.n_params();
    let mut jb = sys.new_jacobian();
    sys.jacobian(&p, &mut jb);
    let mut j = vec![0.0; m * n];
    jb.add_to_dense(&mut j);

    let eps = 1e-7;
    let mut r0 = Vector::zeros(m);
    sys.residual(&p, &mut r0);
    for col in 0..n {
        let mut p2 = p.clone();
        p2[col] += eps;
        let mut rp = Vector::zeros(m);
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

/// Central finite differences of the assembled residual vs the analytical Jacobian.
fn assert_jacobian_matches_fd(sys: &AssembledSystem<'_>, p: &Vector) {
    let m = sys.n_residuals();
    let n = sys.n_params();
    let mut jb = sys.new_jacobian();
    sys.jacobian(p, &mut jb);
    let mut j = vec![0.0; m * n];
    jb.add_to_dense(&mut j);
    let eps = 1e-6;
    let (mut rp, mut rm) = (Vector::zeros(m), Vector::zeros(m));
    for col in 0..n {
        let (mut pp, mut pm) = (p.clone(), p.clone());
        pp[col] += eps;
        pm[col] -= eps;
        sys.residual(&pp, &mut rp);
        sys.residual(&pm, &mut rm);
        for row in 0..m {
            let fd = (rp[row] - rm[row]) / (2.0 * eps);
            let an = j[row * n + col];
            assert!((fd - an).abs() < 1e-6, "row={row} col={col} fd={fd} an={an}");
        }
    }
}

#[test]
fn duplicate_connectivity_columns_accumulate() {
    // Start and end of one arc share center / radius / start-heading columns.
    let mut doc = Document::new();
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.3, -0.2),
            Length::new(1.5),
            HeadingAngle::new(0.4),
            TurnAngle::new(2.0),
        ),
    );
    doc.add_constraint(Congruence {
        a: Arc::start_expr(arc),
        b: Arc::end_expr(arc),
    });
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(&doc, &soft);
    assert_jacobian_matches_fd(&sys, doc.vector());
}

#[test]
fn blocked_jacobian_matches_fd_for_all_contribution_kinds() {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(2.0, 0.5)));
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(1.0, 1.0), Length::new(1.2), HeadingAngle::new(-0.3), TurnAngle::new(1.7)),
    );
    doc.add_constraint(FixPoint { point: Segment::start_expr(seg), target: Point2::new(0.1, -0.2) });
    doc.add_constraint(Congruence { a: Segment::end_expr(seg), b: Arc::start_expr(arc) });
    doc.add_constraint(Parallel {
        a: Segment::tangent_expr(seg, CurveParam::Normalized(0.0)),
        b: Arc::tangent_expr(arc, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Orthogonal {
        a: Arc::tangent_expr(arc, CurveParam::Normalized(0.2)),
        b: Arc::normal_expr(arc, CurveParam::Normalized(0.9)),
    });
    doc.add_constraint(EqualLength { a: Segment::length_expr(seg), b: Arc::length_expr(arc) });
    // Self-referencing: duplicate columns inside one block.
    doc.add_constraint(EqualLength { a: Arc::length_expr(arc), b: Arc::length_expr(arc) });

    let soft = [SoftContribution { index: 1, target: 0.4, weight: 0.3 }];
    let soft_points = [SoftPointFollow { point: Arc::end_expr(arc), target: Point2::new(0.0, 2.0), weight: 0.5 }];
    let soft_lengths = [
        // Active (L < floor) and inactive (L > floor) one-sided terms.
        SoftLengthFloor { length: Segment::length_expr(seg), min_length: 10.0, weight: 0.7 },
        SoftLengthFloor { length: Arc::length_expr(arc), min_length: 0.01, weight: 0.7 },
    ];
    let sys = AssembledSystem::with_soft(&doc, &soft, &soft_points, &soft_lengths);
    assert_eq!(sys.new_jacobian().num_blocks(), 6 + 1 + 1 + 2);

    let base = doc.vector().clone();
    for trial in 0..3 {
        let mut p = base.clone();
        for i in 0..p.len() {
            let k = (trial * 31 + i * 17) as f64;
            p[i] += 0.05 * (k * 0.618).sin();
        }
        assert_jacobian_matches_fd(&sys, &p);
    }
}
