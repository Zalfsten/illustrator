use std::f64::consts::{FRAC_PI_2, PI};

use super::*;
use crate::document::Document;
use crate::geom::{CurveParam, Length};
use crate::geometry::{CurveExprs, GeometryId, GeometryRole};
use crate::geometry_kinds::{Arc, Label, Segment};
use crate::vector::Vector;

const TOL: f64 = 1e-12;

fn close(a: Point2, b: Point2) -> bool {
    (a - b).length() < TOL
}

/// Whether `q` lies on one of `pieces`.
fn on_shape(pieces: &[ShapePiece], q: Point2) -> bool {
    pieces.iter().any(|piece| match *piece {
        ShapePiece::Line { start, end } => {
            let d = end - start;
            let s = ((q - start).dot(d) / d.dot(d)).clamp(0.0, 1.0);
            close(start + d * s, q)
        }
        ShapePiece::Arc {
            center,
            radius,
            start,
            sweep,
        } => {
            let v = q - center;
            // Turn from the start heading in the sweep's direction, in [0, 2π).
            let sweep = sweep.radians();
            let mut u = (sweep.signum() * (v.y.atan2(v.x) - start.radians())).rem_euclid(2.0 * PI);
            if u > 2.0 * PI - 1e-9 {
                u = 0.0;
            }
            (v.length() - radius).abs() < TOL && u <= sweep.abs() + 1e-9
        }
    })
}

fn sample(doc: &Document, p: &Vector, id: GeometryId, t: f64) -> Point2 {
    with_curve(id, t).evaluate(doc, p)
}

fn with_curve(id: GeometryId, t: f64) -> crate::expr::PointExpr {
    // The factory is kind-agnostic: the tag is the same for every curve kind.
    Segment::on_curve_expr(id, CurveParam::Normalized(t))
}

fn drawing() -> (Document, GeometryId, GeometryId, GeometryId) {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(1.0, 2.0), Point2::new(4.0, 6.0)),
    );
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(-1.0, 0.5),
            Length::new(2.0),
            HeadingAngle::new(0.3),
            TurnAngle::new(-1.2 * PI),
        ),
    );
    let lbl = doc.add_geometry::<Label>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), HeadingAngle::new(0.0)),
    );
    (doc, seg, arc, lbl)
}

#[test]
fn segment_shape_is_one_straight_piece() {
    let (doc, seg, _, _) = drawing();
    let shape = doc.curve_shape(seg, doc.committed_vector()).unwrap();
    assert_eq!(
        shape,
        vec![ShapePiece::Line {
            start: Point2::new(1.0, 2.0),
            end: Point2::new(4.0, 6.0)
        }]
    );
}

#[test]
fn arc_shape_is_one_arc_piece() {
    let (doc, _, arc, _) = drawing();
    let shape = doc.curve_shape(arc, doc.committed_vector()).unwrap();
    assert_eq!(
        shape,
        vec![ShapePiece::Arc {
            center: Point2::new(-1.0, 0.5),
            radius: 2.0,
            start: HeadingAngle::new(0.3),
            sweep: TurnAngle::new(-1.2 * PI),
        }]
    );
}

#[test]
fn label_has_no_curve_shape() {
    let (doc, _, _, lbl) = drawing();
    assert_eq!(doc.curve_shape(lbl, doc.committed_vector()), None);
    assert_eq!(doc.curve_shape(GeometryId(99), doc.committed_vector()), None);
}

#[test]
fn curve_curvature_by_identity() {
    let (doc, seg, arc, lbl) = drawing();
    let p = doc.committed_vector();
    for t in [0.0, 0.5, 1.0] {
        assert_eq!(doc.curve_curvature(seg, p, t), Some(0.0));
        let k = doc.curve_curvature(arc, p, t).unwrap();
        assert!((k - 0.5).abs() < TOL, "arc curvature {k} at t = {t}");
    }
    assert_eq!(doc.curve_curvature(lbl, p, 0.5), None);
    assert_eq!(doc.curve_curvature(GeometryId(99), p, 0.5), None);
}

#[test]
fn curve_curvature_leaves_the_drawing_unchanged() {
    let (doc, _, arc, _) = drawing();
    let before = doc.committed_vector().clone();
    let shape_before = doc.curve_shape(arc, &before);
    let _ = doc.curve_curvature(arc, doc.committed_vector(), 0.25);
    assert_eq!(doc.committed_vector(), &before);
    assert_eq!(doc.curve_shape(arc, doc.committed_vector()), shape_before);
}

#[test]
fn samples_lie_on_the_shape() {
    let (doc, seg, arc, _) = drawing();
    let p = doc.committed_vector();
    for id in [seg, arc] {
        let shape = doc.curve_shape(id, p).unwrap();
        for t in [0.0, 0.25, 0.5, 1.0] {
            assert!(on_shape(&shape, sample(&doc, p, id, t)), "{id:?} t={t}");
        }
        assert!(close(shape[0].start_point(), sample(&doc, p, id, 0.0)));
        assert!(close(shape.last().unwrap().end_point(), sample(&doc, p, id, 1.0)));
    }
}

#[test]
fn shape_length_equals_curve_length() {
    let (doc, seg, arc, _) = drawing();
    let p = doc.committed_vector();
    for id in [seg, arc] {
        let total: f64 = doc.curve_shape(id, p).unwrap().iter().map(ShapePiece::length).sum();
        let len = Segment::length_expr(id).evaluate(&doc, p).value();
        assert!((total - len).abs() < TOL, "{id:?}: {total} vs {len}");
    }
}

#[test]
fn negative_intermediate_radius_gives_the_canonical_piece() {
    let (doc, _, arc, _) = drawing();
    let mut p = doc.committed_vector().clone();
    // Arc block starts after the segment's 4 parameters: radius is at 4 + 2.
    p[6] = -2.0;
    let shape = doc.curve_shape(arc, &p).unwrap();
    let ShapePiece::Arc { radius, start, .. } = shape[0] else {
        panic!("arc piece expected")
    };
    assert_eq!(radius, 2.0);
    assert!((start.radians() - (0.3 + PI)).abs() < TOL);
    for t in [0.0, 0.5, 1.0] {
        assert!(on_shape(&shape, sample(&doc, &p, arc, t)), "t={t}");
    }
    let total: f64 = shape.iter().map(ShapePiece::length).sum();
    assert!((total - Segment::length_expr(arc).evaluate(&doc, &p).value()).abs() < TOL);
}

#[test]
fn sub_piece_keeps_its_kind() {
    let quarter = ShapePiece::Arc {
        center: Point2::new(0.0, 0.0),
        radius: 1.0,
        start: HeadingAngle::new(0.0),
        sweep: TurnAngle::new(FRAC_PI_2),
    };
    let half = quarter.sub(0.5, 1.0);
    assert!((half.length() - FRAC_PI_2 / 2.0).abs() < TOL);
    assert!(close(half.end_point(), Point2::new(0.0, 1.0)));
    let line = ShapePiece::Line {
        start: Point2::new(0.0, 0.0),
        end: Point2::new(4.0, 0.0),
    };
    assert_eq!(
        line.sub(0.25, 0.5),
        ShapePiece::Line {
            start: Point2::new(1.0, 0.0),
            end: Point2::new(2.0, 0.0)
        }
    );
}

/// Exact bounding box of `piece` as `(min, max)`.
fn bounds(piece: &ShapePiece) -> (Point2, Point2) {
    let pts = piece.bounding_points();
    let fold = |f: fn(f64, f64) -> f64| {
        pts.iter().skip(1).fold(pts[0], |m, p| Point2::new(f(m.x, p.x), f(m.y, p.y)))
    };
    (fold(f64::min), fold(f64::max))
}

#[test]
fn line_bounding_points_are_its_ends() {
    let piece = ShapePiece::Line { start: Point2::new(1.0, 2.0), end: Point2::new(-3.0, 5.0) };
    assert_eq!(bounds(&piece), (Point2::new(-3.0, 2.0), Point2::new(1.0, 5.0)));
}

#[test]
fn arc_bounding_points_include_the_extremes_it_passes() {
    let arc = |start: f64, sweep: f64| ShapePiece::Arc {
        center: Point2::new(1.0, 1.0),
        radius: 2.0,
        start: HeadingAngle::new(start),
        sweep: TurnAngle::new(sweep),
    };
    // 0° → 120°: passes the top (heading 90°), not the left side.
    let (min, max) = bounds(&arc(0.0, 2.0 * PI / 3.0));
    assert!((max.y - 3.0).abs() < TOL && (max.x - 3.0).abs() < TOL);
    assert!((min.x - 0.0).abs() < TOL && (min.y - 1.0).abs() < TOL);
    // The same arc traversed clockwise has the same box.
    let (cmin, cmax) = bounds(&arc(2.0 * PI / 3.0, -2.0 * PI / 3.0));
    assert!(close(cmin, min) && close(cmax, max));
    // A full turn reaches all four sides.
    let (min, max) = bounds(&arc(0.3, 2.0 * PI));
    assert!(close(min, Point2::new(-1.0, -1.0)) && close(max, Point2::new(3.0, 3.0)));
    // A short arc between two extremes is bounded by its ends.
    let piece = arc(0.1, 0.2);
    assert_eq!(piece.bounding_points().len(), 2);
}
