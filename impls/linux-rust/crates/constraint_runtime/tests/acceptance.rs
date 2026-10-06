//! Headless acceptance scenarios.

use constraint_runtime::{
    AssembledSystem, Congruence, CurveExprs, CurveParam, Document, DragSession, EndpointExprs,
    EqualLength, FixPoint, GeometryRole, InteractiveSolve, Orthogonal, Parallel, Point2,
    SoftContribution, SystemEval, Segment, SOLVE_TOL,
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
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 0.0), Point2::new(1.0, 1.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });

    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).expect("initial solve");
    assert!(residual_ok(&doc));

    let mut drag = DragSession::new();
    let free = Segment::end_expr(s1);
    drag.pointer_down(&doc, free.clone());
    drag.pointer_move(&mut interact, &mut doc, (0.2, 1.5))
        .expect("move");
    drag.pointer_up(&mut interact, &mut doc, (0.2, 1.5))
        .expect("up");

    assert!(residual_ok(&doc));
    let a = Segment::end_expr(s0).evaluate(&doc, doc.state());
    let b = Segment::start_expr(s1).evaluate(&doc, doc.state());
    assert!((a.x - b.x).abs() < 1e-4 && (a.y - b.y).abs() < 1e-4);
}

#[test]
fn rectangle_drag_keeps_structure_and_stays_near() {
    let mut doc = Document::new();
    let ab = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 1.0), Point2::new(2.0, 1.0)));
    let bc = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 1.0), Point2::new(2.0, 0.0)));
    let cd = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(0.0, 0.0)));
    let da = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(0.0, 1.0)));

    doc.add_constraint(Congruence {
        a: Segment::end_expr(ab),
        b: Segment::start_expr(bc),
    });
    doc.add_constraint(Congruence {
        a: Segment::end_expr(bc),
        b: Segment::start_expr(cd),
    });
    doc.add_constraint(Congruence {
        a: Segment::end_expr(cd),
        b: Segment::start_expr(da),
    });
    doc.add_constraint(Congruence {
        a: Segment::end_expr(da),
        b: Segment::start_expr(ab),
    });

    doc.add_constraint(Parallel {
        a: Segment::tangent_expr(ab, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(cd, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Parallel {
        a: Segment::tangent_expr(da, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(bc, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(ab, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(da, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(ab),
        target: Point2::new(0.0, 1.0),
    });

    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).expect("initial rectangle");
    let prev = doc.clone_state();
    assert!(residual_ok(&doc));

    let mut drag = DragSession::new();
    let corner = Segment::end_expr(bc);
    drag.pointer_down(&doc, corner.clone());
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
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(120.0, 320.0), Point2::new(320.0, 320.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(320.0, 320.0), Point2::new(320.0, 140.0)));
    doc.add_constraint(Congruence {
        a: Segment::end_expr(s0),
        b: Segment::start_expr(s1),
    });
    doc.add_constraint(Orthogonal {
        a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
        b: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(120.0, 320.0),
    });

    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).expect("initial");
    let free = Segment::end_expr(s1);
    let before = free.evaluate(&doc, doc.state());

    let mut drag = DragSession::new();
    drag.pointer_down(&doc, free.clone());
    drag.pointer_move(&mut interact, &mut doc, (200.0, 140.0))
        .expect("move");
    let after = free.evaluate(&doc, doc.state());
    assert!(
        (after.x - before.x).abs() > 40.0,
        "expected horizontal motion, C {before:?} -> {after:?}"
    );
    assert!(residual_ok(&doc));

    // Large further drag should still track (not stick at first feasible point).
    drag.pointer_move(&mut interact, &mut doc, (80.0, 100.0))
        .expect("large move");
    let far = free.evaluate(&doc, doc.state());
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
    let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(4.0, 0.0)));
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(1.0, 2.0), Point2::new(5.0, 2.0)));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s1),
        target: Point2::new(1.0, 2.0),
    });
    doc.add_constraint(Congruence {
        a: Segment::on_curve_expr(s0, CurveParam::Uniform(0.5),
        ),
        b: Segment::start_expr(s1),
    });

    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).expect("midpoint congruence");
    assert!(residual_ok(&doc));
    let mid_pt = Segment::on_curve_expr(s0, CurveParam::Uniform(0.5),
    )
    .evaluate(&doc, doc.state());
    assert!((mid_pt.x - 1.0).abs() < 1e-4 && (mid_pt.y - 2.0).abs() < 1e-4);
}

#[test]
fn equal_length_keeps_two_segments_same_length() {
    let mut doc = Document::new();
    let s0 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(2.0, 0.0)),
    );
    let s1 = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 1.0), Point2::new(1.0, 1.0)),
    );
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s0),
        target: Point2::new(0.0, 0.0),
    });
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(s1),
        target: Point2::new(0.0, 1.0),
    });
    doc.add_constraint(EqualLength {
        a: Segment::length_expr(s0),
        b: Segment::length_expr(s1),
    });
    doc.add_constraint(FixPoint {
        point: Segment::end_expr(s0),
        target: Point2::new(3.0, 0.0),
    });

    let mut interact = InteractiveSolve::new();
    interact.structural_only(&mut doc).expect("equal length solve");
    assert!(residual_ok(&doc));
    let l0 = Segment::length_expr(s0)
        .evaluate(&doc, doc.state())
        .value();
    let l1 = Segment::length_expr(s1)
        .evaluate(&doc, doc.state())
        .value();
    assert!((l0 - l1).abs() < 1e-4, "l0={l0} l1={l1}");
    assert!((l0 - 3.0).abs() < 1e-4, "expected length 3, got {l0}");
}
