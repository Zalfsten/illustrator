//! Hot-path allocation check: after warm-up, Jacobian / residual evaluation on a
//! pre-sized system must not touch the heap.

mod common;

use common::staircase::staircase;
use constraint_runtime::{
    AnchorExprs, AssembledSystem, Congruence, CurveExprs, CurveParam, Document, EndpointExprs, Function,
    FunctionAttrs, GeometryRole, HeadingAngle, KindAttrs, LineExprs, Point2, Segment, SoftContribution,
    SoftLengthFloor, SoftPointFollow, SystemEval, Vector, VectorsParallel,
};
#[path = "../src/test_alloc.rs"]
mod test_alloc;

use test_alloc::allocations;

#[test]
fn jacobian_and_residual_do_not_allocate() {
    let (doc, ids) = staircase(100);
    let soft = [SoftContribution {
        index: 3,
        target: 0.5,
        weight: 1e-3,
    }];
    let points = [SoftPointFollow {
        point: Segment::end_expr(*ids.last().unwrap()),
        target: Point2::new(60.0, 40.0),
        weight: 1.0,
    }];
    let lengths: Vec<SoftLengthFloor> = ids
        .iter()
        .map(|&id| SoftLengthFloor {
            length: Segment::length_expr(id),
            min_length: 0.5,
            weight: 1.0,
        })
        .collect();
    let sys = AssembledSystem::with_soft(&doc, &soft, &points, &lengths);
    let p = doc.clone_vector();
    let mut r = Vector::zeros(sys.n_residuals());
    let mut j = sys.new_jacobian();

    sys.residual(&p, &mut r);
    sys.jacobian(&p, &mut j);

    let n = allocations(|| {
        for _ in 0..5 {
            sys.residual(&p, &mut r);
            sys.jacobian(&p, &mut j);
        }
    });
    assert_eq!(n, 0, "hot-path evaluation allocated {n} times");

    // Scoped to every constraint: same rows, still no allocation.
    let scope: Vec<_> = doc.constraints().iter().map(|(id, _)| id).collect();
    let sys = AssembledSystem::with_soft(&doc, &soft, &points, &lengths).scoped(&scope);
    sys.residual(&p, &mut r);
    sys.jacobian(&p, &mut j);
    let n = allocations(|| {
        for _ in 0..5 {
            sys.residual(&p, &mut r);
            sys.jacobian(&p, &mut j);
        }
    });
    assert_eq!(n, 0, "scoped evaluation allocated {n} times");
}

#[test]
fn function_curve_evaluation_does_not_allocate() {
    let mut doc = Document::new();
    let attrs = FunctionAttrs::new("sin(x)", (0.0, std::f64::consts::TAU), (1.0, 1.0)).unwrap();
    let f = doc.add_geometry_with::<Function>(
        GeometryRole::Result,
        (Point2::new(0.1, 0.2), HeadingAngle::new(0.05)),
        KindAttrs::Function(attrs),
    );
    let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(-1.0, 0.0), Point2::new(0.0, 0.0)));
    let s2 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(6.3, 0.0), Point2::new(7.0, 1.0)));
    doc.add_constraint(Congruence { a: Function::start_expr(f), b: Segment::end_expr(s1) });
    doc.add_constraint(Congruence { a: Function::end_expr(f), b: Segment::start_expr(s2) });
    doc.add_constraint(Congruence {
        a: Function::on_curve_expr(f, CurveParam::Normalized(0.3)),
        b: Segment::end_expr(s2),
    });
    doc.add_constraint(VectorsParallel { a: Function::direction_expr(f), b: Segment::direction_expr(s1) });
    let points = [SoftPointFollow { point: Function::origin_expr(f), target: Point2::new(0.0, 0.0), weight: 1.0 }];
    let lengths = [SoftLengthFloor { length: Function::length_expr(f), min_length: 1.0, weight: 1.0 }];
    let sys = AssembledSystem::with_soft(&doc, &[], &points, &lengths);
    let p = doc.clone_vector();
    let mut r = Vector::zeros(sys.n_residuals());
    let mut j = sys.new_jacobian();
    // Warm-up fills the thread-local expression scratch.
    sys.residual(&p, &mut r);
    sys.jacobian(&p, &mut j);
    let n = allocations(|| {
        for _ in 0..5 {
            sys.residual(&p, &mut r);
            sys.jacobian(&p, &mut j);
        }
    });
    assert_eq!(n, 0, "function curve evaluation allocated {n} times");
}

#[test]
fn counter_detects_allocation() {
    let n = allocations(|| {
        std::hint::black_box(vec![0u8; 16]);
    });
    assert!(n >= 1);
}
