use super::*;
use crate::constraint_kinds::FixPoint;
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry::{
    CircularExprs, CurveExprs, EndpointExprs, GeometryId, GeometryRole, LineExprs,
};
use crate::geometry_kinds::{Arc, Segment};

/// `seg1` from (1, 2) to (4, 6), `arc1` around (−1, 3), and constraint `fix1`.
fn doc() -> (Document, GeometryId, GeometryId) {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(1.0, 2.0), Point2::new(4.0, 6.0)),
    );
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(-1.0, 3.0),
            Length::new(2.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(1.0),
        ),
    );
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(seg),
        target: Point2::new(1.0, 2.0),
    });
    (doc, seg, arc)
}

fn resolve(doc: &Document, a: &str) -> Address {
    doc.resolve_address(a)
        .unwrap_or_else(|e| panic!("{a}: {e}"))
}

#[test]
fn suffixes_resolve_like_the_static_factories() {
    use Address::{Length as L, Point as P, Vector as V};
    let (doc, seg, arc) = doc();
    let t = CurveParam::Normalized(0.25);
    let cases = [
        ("seg1.start", P(Segment::start_expr(seg))),
        ("seg1.end", P(Segment::end_expr(seg))),
        ("seg1.at(0.25)", P(Segment::on_curve_expr(seg, t))),
        ("seg1.direction", V(Segment::direction_expr(seg))),
        ("seg1.tangent(0.25)", V(Segment::tangent_expr(seg, t))),
        ("seg1.normal(0.25)", V(Segment::normal_expr(seg, t))),
        ("seg1.length", L(Segment::length_expr(seg))),
        ("arc1.start", P(Arc::start_expr(arc))),
        ("arc1.end", P(Arc::end_expr(arc))),
        ("arc1.center", P(Arc::center_expr(arc))),
        ("arc1.at(0.25)", P(Arc::on_curve_expr(arc, t))),
        ("arc1.tangent(0.25)", V(Arc::tangent_expr(arc, t))),
        ("arc1.normal(0.25)", V(Arc::normal_expr(arc, t))),
        ("arc1.length", L(Arc::length_expr(arc))),
        ("arc1.radius", L(Arc::radius_expr(arc))),
    ];
    for (a, expected) in cases {
        assert_eq!(resolve(&doc, a), expected, "{a}");
    }
}

#[test]
fn endpoint_address_evaluates_to_the_end_point() {
    let (doc, ..) = doc();
    let Address::Point(e) = resolve(&doc, "seg1.end") else {
        panic!("not a point");
    };
    let p = e.evaluate(&doc, doc.vector());
    assert_eq!((p.x, p.y), (4.0, 6.0));
}

#[test]
fn vector_address_is_not_a_point() {
    let (doc, ..) = doc();
    assert!(matches!(
        resolve(&doc, "seg1.direction"),
        Address::Vector(_)
    ));
}

#[test]
fn malformed_addresses() {
    let (doc, ..) = doc();
    for a in [
        "seg1",
        "seg1.",
        ".start",
        "seg1.at(",
        "seg1.at()",
        "seg1.at(x)",
        "seg1.foo",
        "seg1.start(0.5)",
        "seg1.at",
        "seg1.at(0.5).x",
        "seg 1.start",
        "1seg.start",
    ] {
        assert_eq!(
            doc.resolve_address(a),
            Err(AddressError::Malformed),
            "{a:?}"
        );
    }
}

#[test]
fn unknown_name_and_constraint_name() {
    let (doc, ..) = doc();
    assert_eq!(
        doc.resolve_address("nope.start"),
        Err(AddressError::UnknownName("nope".into()))
    );
    assert_eq!(
        doc.resolve_address("fix1.start"),
        Err(AddressError::NotAGeometry("fix1".into()))
    );
}

#[test]
fn curve_parameter_out_of_range() {
    let (doc, ..) = doc();
    for a in [
        "seg1.at(1.5)",
        "seg1.at(-0.1)",
        "seg1.tangent(2)",
        "seg1.at(inf)",
    ] {
        assert!(
            matches!(
                doc.resolve_address(a),
                Err(AddressError::ParamOutOfRange(_))
            ),
            "{a}"
        );
    }
    assert!(matches!(
        doc.resolve_address("seg1.at(NaN)"),
        Err(AddressError::ParamOutOfRange(t)) if t.is_nan()
    ));
    assert!(doc.resolve_address("seg1.at(0)").is_ok());
    assert!(doc.resolve_address("seg1.at(1)").is_ok());
}

#[test]
fn missing_capability_is_reported() {
    let (doc, ..) = doc();
    for (a, name, capability) in [
        ("arc1.direction", "arc1", "line"),
        ("seg1.center", "seg1", "circular"),
        ("seg1.radius", "seg1", "circular"),
    ] {
        assert_eq!(
            doc.resolve_address(a),
            Err(AddressError::MissingCapability {
                name: name.into(),
                capability
            }),
            "{a}"
        );
    }
}

#[test]
fn print_resolve_round_trip() {
    let (doc, ..) = doc();
    for a in [
        "seg1.start",
        "seg1.end",
        "seg1.at(0.25)",
        "seg1.at(0)",
        "seg1.at(1)",
        "arc1.center",
        "seg1.direction",
        "seg1.tangent(0.3)",
        "arc1.normal(0.7)",
        "seg1.length",
        "arc1.radius",
    ] {
        let printed = match resolve(&doc, a) {
            Address::Point(e) => doc.point_address(&e),
            Address::Vector(e) => doc.vec_address(&e),
            Address::Length(e) => doc.length_address(&e),
        };
        assert_eq!(printed.as_deref(), Some(a));
    }
}

#[test]
fn uniform_parameters_have_no_address() {
    let (doc, seg, _) = doc();
    let u = CurveParam::Uniform(0.5);
    assert_eq!(doc.point_address(&Segment::on_curve_expr(seg, u)), None);
    assert_eq!(doc.vec_address(&Segment::tangent_expr(seg, u)), None);
}
