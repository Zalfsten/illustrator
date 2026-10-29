use constraint_runtime::{
    Arc, Congruence, Document, FixPoint, GeometryRole, Point2, Segment, VectorsParallel,
};

use super::*;
use crate::command::CommandError;

fn invalid(field: &'static str) -> impl Fn(&CommandError) -> bool {
    move |e| matches!(e, CommandError::InvalidValue { field: f, .. } if *f == field)
}

fn arc(radius: f64, start_heading: f64, sweep: f64) -> ArcParams {
    ArcParams {
        center: Point2::new(1.0, 2.0),
        radius,
        start_heading,
        sweep,
    }
}

#[test]
fn geometry_params_round_trip_through_init() {
    let s = SegmentParams {
        start: Point2::new(0.1, -0.0),
        end: Point2::new(1e-300, 3.0),
    };
    assert_eq!(Segment::params(Segment::init(&s).unwrap()), s);
    let a = arc(0.5, -1.25, 2.5);
    assert_eq!(Arc::params(Arc::init(&a).unwrap()), a);
}

#[test]
fn non_finite_values_are_rejected_with_field() {
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let s = SegmentParams {
            start: Point2::new(0.0, 0.0),
            end: Point2::new(v, 0.0),
        };
        assert!(invalid("end")(&Segment::init(&s).unwrap_err()));
        assert!(invalid("radius")(
            &Arc::init(&arc(v, 0.0, 1.0)).unwrap_err()
        ));
        assert!(invalid("start_heading")(
            &Arc::init(&arc(1.0, v, 1.0)).unwrap_err()
        ));
        assert!(invalid("sweep")(&Arc::init(&arc(1.0, 0.0, v)).unwrap_err()));
    }
}

#[test]
fn negative_radius_is_rejected_zero_accepted() {
    assert!(invalid("radius")(
        &Arc::init(&arc(-1.0, 0.0, 1.0)).unwrap_err()
    ));
    assert!(Arc::init(&arc(0.0, 0.0, 1.0)).is_ok());
}

fn two_segments() -> Document {
    let mut doc = Document::new();
    let p = Point2::new;
    doc.add_geometry::<Segment>(GeometryRole::Result, (p(0.0, 0.0), p(1.0, 0.0)));
    doc.add_geometry::<Segment>(GeometryRole::Result, (p(1.0, 0.0), p(1.0, 1.0)));
    doc
}

#[test]
fn constraint_params_round_trip_through_resolve() {
    let doc = two_segments();
    let r = Resolver::new(&doc);
    let join = PointPair {
        a: "seg1.end".into(),
        b: "seg2.start".into(),
    };
    assert_eq!(
        Congruence::resolve(&join, &r).unwrap().params(&doc),
        Some(join)
    );
    let par = VecPair {
        a: "seg1.direction".into(),
        b: "seg2.direction".into(),
    };
    assert_eq!(
        VectorsParallel::resolve(&par, &r).unwrap().params(&doc),
        Some(par)
    );
    let fix = FixParams {
        point: "seg1.at(0.25)".into(),
        target: Point2::new(0.25, 0.0),
    };
    assert_eq!(FixPoint::resolve(&fix, &r).unwrap().params(&doc), Some(fix));
}

#[test]
fn fix_target_must_be_finite() {
    let doc = two_segments();
    let fix = FixParams {
        point: "seg1.start".into(),
        target: Point2::new(f64::INFINITY, 0.0),
    };
    let e = FixPoint::resolve(&fix, &Resolver::new(&doc)).unwrap_err();
    assert!(invalid("target")(&e));
}

#[test]
fn join_of_one_point_is_already_joined() {
    let doc = two_segments();
    let p = PointPair {
        a: "seg1.end".into(),
        b: "seg1.end".into(),
    };
    let e = Congruence::resolve(&p, &Resolver::new(&doc)).unwrap_err();
    assert!(matches!(e, CommandError::AlreadyJoined { .. }));
}

#[test]
fn wrong_family_is_rejected() {
    let doc = two_segments();
    let p = VecPair {
        a: "seg1.start".into(),
        b: "seg2.direction".into(),
    };
    let e = VectorsParallel::resolve(&p, &Resolver::new(&doc)).unwrap_err();
    assert!(matches!(e, CommandError::WrongFamily { .. }));
}
