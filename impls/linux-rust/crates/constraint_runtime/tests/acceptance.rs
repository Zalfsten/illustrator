//! Headless acceptance scenarios.

use constraint_runtime::{
    AssembledSystem, Constraint, Document, DragSession, EndpointRef, InteractiveSolve, Point2,
    PointExpr, EndpointKind, GeometryId, SoftContribution, SystemEval, SOLVE_TOL,
};

fn residual_ok(doc: &Document) -> bool {
    let soft: [SoftContribution; 0] = [];
    let sys = AssembledSystem::from_document(doc, &soft);
    let mut r = vec![0.0; sys.n_residuals()];
    sys.residual(doc.state(), &mut r);
    r.iter().map(|x| x * x).sum::<f64>().sqrt() <= SOLVE_TOL * 10.0
}

#[test]
fn right_angle_soft_drag_keeps_orthogonal_and_congruence() {
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    let s1 = doc.add_segment(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
    doc.join_endpoints(
        EndpointRef::new(s0, EndpointKind::End),
        EndpointRef::new(s1, EndpointKind::Start),
    );
    doc.add_constraint(Constraint::Orthogonal {
        a: GeometryId(0),
        b: GeometryId(1),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::Endpoint {
            geometry: s0,
            end: EndpointKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });

    let mut interact = InteractiveSolve::default();
    interact.structural_only(&mut doc).expect("initial solve");
    assert!(residual_ok(&doc));

    let mut drag = DragSession::new();
    let free = PointExpr::Endpoint {
        geometry: s1,
        end: EndpointKind::End,
    };
    drag.pointer_down(&doc, free);
    drag.pointer_move(&mut interact, &mut doc, (0.2, 1.5))
        .expect("move");
    drag.pointer_up(&mut interact, &mut doc, (0.2, 1.5))
        .expect("up");

    assert!(residual_ok(&doc));
    let a = doc.segment(s0).unwrap().end(doc.state());
    let b = doc.segment(s1).unwrap().start(doc.state());
    assert!((a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4);
}

#[test]
fn rectangle_drag_keeps_structure_and_stays_near() {
    let mut doc = Document::new();
    let ab = doc.add_segment(Point2::new(0.0, 1.0), Point2::new(2.0, 1.0));
    let bc = doc.add_segment(Point2::new(2.0, 1.0), Point2::new(2.0, 0.0));
    let cd = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(0.0, 0.0));
    let da = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(0.0, 1.0));

    doc.join_endpoints(
        EndpointRef::new(ab, EndpointKind::End),
        EndpointRef::new(bc, EndpointKind::Start),
    );
    doc.join_endpoints(
        EndpointRef::new(bc, EndpointKind::End),
        EndpointRef::new(cd, EndpointKind::Start),
    );
    doc.join_endpoints(
        EndpointRef::new(cd, EndpointKind::End),
        EndpointRef::new(da, EndpointKind::Start),
    );
    doc.join_endpoints(
        EndpointRef::new(da, EndpointKind::End),
        EndpointRef::new(ab, EndpointKind::Start),
    );

    doc.add_constraint(Constraint::Parallel { a: ab, b: cd });
    doc.add_constraint(Constraint::Parallel { a: da, b: bc });
    doc.add_constraint(Constraint::Orthogonal { a: ab, b: da });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::Endpoint {
            geometry: ab,
            end: EndpointKind::Start,
        },
        target: Point2::new(0.0, 1.0),
    });

    let mut interact = InteractiveSolve::default();
    interact.structural_only(&mut doc).expect("initial rectangle");
    let prev = doc.clone_state();
    assert!(residual_ok(&doc));

    let mut drag = DragSession::new();
    let corner = PointExpr::Endpoint {
        geometry: bc,
        end: EndpointKind::End,
    };
    drag.pointer_down(&doc, corner);
    let mouse = (2.15, -0.1);
    drag.pointer_move(&mut interact, &mut doc, mouse)
        .expect("rect move");
    drag.pointer_up(&mut interact, &mut doc, mouse)
        .expect("rect up");

    assert!(residual_ok(&doc));
    let now = doc.state();
    let dist: f64 = prev
        .iter()
        .zip(now.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt();
    assert!(dist < 2.0, "dist={dist}");
}


#[test]
fn right_angle_horizontal_drag_moves_x() {
    // Regression: stay+mouse on the same DOFs + Orthogonal trapped the free
    // end on a vertical line (axis-aligned right angle). Stay must skip mouse
    // indices so the joint can rotate under the hard constraint.
    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(120.0, 320.0), Point2::new(320.0, 320.0));
    let s1 = doc.add_segment(Point2::new(320.0, 320.0), Point2::new(320.0, 140.0));
    doc.join_endpoints(
        EndpointRef::new(s0, EndpointKind::End),
        EndpointRef::new(s1, EndpointKind::Start),
    );
    doc.add_constraint(Constraint::Orthogonal {
        a: GeometryId(0),
        b: GeometryId(1),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::Endpoint {
            geometry: s0,
            end: EndpointKind::Start,
        },
        target: Point2::new(120.0, 320.0),
    });

    let mut interact = InteractiveSolve::default();
    interact.structural_only(&mut doc).expect("initial");
    let free = PointExpr::Endpoint {
        geometry: s1,
        end: EndpointKind::End,
    };
    let before = free.evaluate(&doc.structure, doc.state());

    let mut drag = DragSession::new();
    drag.pointer_down(&doc, free);
    drag.pointer_move(&mut interact, &mut doc, (200.0, 140.0))
        .expect("move");
    let after = free.evaluate(&doc.structure, doc.state());
    assert!(
        (after.x - before.x).abs() > 40.0,
        "expected horizontal motion, C {before:?} -> {after:?}"
    );
    assert!(residual_ok(&doc));

    // Large further drag should still track (not stick at first feasible point).
    drag.pointer_move(&mut interact, &mut doc, (80.0, 100.0))
        .expect("large move");
    let far = free.evaluate(&doc.structure, doc.state());
    assert!(
        (far.x - 80.0).abs() < 40.0 && (far.y - 100.0).abs() < 40.0,
        "expected to follow large drag, got {far:?}"
    );
    assert!(residual_ok(&doc));
}

#[test]
fn congruence_to_uniform_midpoint_pins_segment() {
    use constraint_runtime::CurveParam;

    let mut doc = Document::new();
    let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(4.0, 0.0));
    let s1 = doc.add_segment(Point2::new(1.0, 2.0), Point2::new(5.0, 2.0));
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::Endpoint {
            geometry: s0,
            end: EndpointKind::Start,
        },
        target: Point2::new(0.0, 0.0),
    });
    doc.add_constraint(Constraint::FixPoint {
        point: PointExpr::Endpoint {
            geometry: s1,
            end: EndpointKind::Start,
        },
        target: Point2::new(1.0, 2.0),
    });
    doc.add_constraint(Constraint::Congruence {
        a: PointExpr::OnCurve {
            geometry: s0,
            param: CurveParam::Uniform(0.5),
        },
        b: PointExpr::Endpoint {
            geometry: s1,
            end: EndpointKind::Start,
        },
    });

    let mut interact = InteractiveSolve::default();
    interact.structural_only(&mut doc).expect("midpoint congruence");
    assert!(residual_ok(&doc));
    let mid_pt = PointExpr::OnCurve {
        geometry: s0,
        param: CurveParam::Uniform(0.5),
    }
    .evaluate(&doc.structure, doc.state());
    assert!((mid_pt.x - 1.0).abs() < 1e-4 && (mid_pt.y - 2.0).abs() < 1e-4);
}
