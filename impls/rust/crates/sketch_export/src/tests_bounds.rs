use std::f64::consts::FRAC_PI_2;

use constraint_runtime::{
    Arc, Arrow, Document, DocumentSettings, GeometryId, GeometryRole, HeadingAngle, Length, Rgb, Segment,
    StyleProps, StyleRef, TurnAngle, MM_PER_PT,
};

use super::*;
use crate::scene::{scene, LabelLines, Object};

const TOL: f64 = 1e-12;

fn pt(v: f64) -> f64 {
    v * MM_PER_PT / DocumentSettings::default().scale
}

fn assert_box(b: Bounds, min: (f64, f64), max: (f64, f64)) {
    let ok = |a: f64, e: f64| (a - e).abs() < TOL;
    assert!(
        ok(b.min.x, min.0) && ok(b.min.y, min.1) && ok(b.max.x, max.0) && ok(b.max.y, max.1),
        "{b:?} != {min:?}..{max:?}"
    );
}

fn styled(doc: &mut Document, id: GeometryId, overrides: StyleProps) {
    doc.set_geometry_style(
        id,
        StyleRef {
            overrides,
            ..StyleRef::default()
        },
    )
    .unwrap();
}

fn one(part: Part) -> Scene {
    Scene {
        objects: vec![Object {
            id: GeometryId(0),
            name: "x".into(),
            parts: vec![part],
        }],
    }
}

#[test]
fn segment_bounds_are_its_butt_rectangle() {
    let mut doc = Document::new();
    doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)));
    let b = bounds(&scene(&doc, &LabelLines::new()).unwrap()).unwrap();
    let h = pt(0.4) / 2.0;
    assert_box(b, (0.0, -h), (3.0, h));
    // 30 mm × 0.4 pt at scale 10.
    assert!((b.width() * 10.0 - 30.0).abs() < 1e-9);
    assert!((b.height() * 10.0 - 0.4 * MM_PER_PT).abs() < 1e-12);
}

#[test]
fn quarter_arc_bounds_use_the_arc_not_the_circle() {
    let mut doc = Document::new();
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.0, 0.0),
            Length::new(1.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(FRAC_PI_2),
        ),
    );
    let b = bounds(&scene(&doc, &LabelLines::new()).unwrap()).unwrap();
    let h = pt(0.4) / 2.0;
    // Flat ends at headings 0 and π/2 lie on the axes; the outer edge reaches 1 + h.
    assert_box(b, (0.0, 0.0), (1.0 + h, 1.0 + h));
}

#[test]
fn clockwise_arc_through_an_axis() {
    let part = Part::Stroke {
        path: vec![
            PathSeg::Move(Point2::new(0.0, 1.0)),
            PathSeg::Arc {
                center: Point2::new(0.0, 0.0),
                radius: 1.0,
                start: FRAC_PI_2,
                sweep: -FRAC_PI_2 * 2.0,
            },
        ],
        width: 0.2,
        color: Rgb(0, 0, 0),
        dash: None,
    };
    let b = bounds(&one(part)).unwrap();
    assert_box(b, (0.0, -1.1), (1.1, 1.1));
}

#[test]
fn arrow_head_widens_the_bounds() {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(3.0, 0.0)));
    styled(
        &mut doc,
        seg,
        StyleProps {
            arrow: Some(Arrow::End),
            ..StyleProps::default()
        },
    );
    let b = bounds(&scene(&doc, &LabelLines::new()).unwrap()).unwrap();
    let l = pt(3.0 + 4.5 * 0.4);
    assert!((b.height() - 0.75 * l).abs() < TOL, "{}", b.height());
    assert!((b.max.x - 3.0).abs() < TOL);
}

#[test]
fn cubic_extremum_inside_the_control_box() {
    // Symmetric hump: control points at y = 4 reach y = 3 at t = 1/2.
    let fill = Part::Fill {
        contours: vec![vec![
            PathSeg::Move(Point2::new(0.0, 0.0)),
            PathSeg::Cubic(Point2::new(0.0, 4.0), Point2::new(2.0, 4.0), Point2::new(2.0, 0.0)),
            PathSeg::Close,
        ]],
        color: Rgb(0, 0, 0),
    };
    assert_box(bounds(&one(fill)).unwrap(), (0.0, 0.0), (2.0, 3.0));
    let quad = Part::Fill {
        contours: vec![vec![
            PathSeg::Move(Point2::new(0.0, 0.0)),
            PathSeg::Quad(Point2::new(1.0, 2.0), Point2::new(2.0, 0.0)),
            PathSeg::Close,
        ]],
        color: Rgb(0, 0, 0),
    };
    assert_box(bounds(&one(quad)).unwrap(), (0.0, 0.0), (2.0, 1.0));
}

#[test]
fn outline_rule_is_a_butt_rectangle() {
    let rule = Part::Outline {
        contours: vec![vec![PathSeg::Move(Point2::new(1.0, 1.0)), PathSeg::Line(Point2::new(3.0, 1.0))]],
        width: 0.5,
        color: Rgb(0, 0, 0),
    };
    assert_box(bounds(&one(rule)).unwrap(), (1.0, 0.75), (3.0, 1.25));
}

#[test]
fn empty_scene_has_no_bounds() {
    assert_eq!(bounds(&Scene::default()), None);
    let mut doc = Document::new();
    doc.add_geometry::<Segment>(GeometryRole::Construction, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    assert_eq!(bounds(&scene(&doc, &LabelLines::new()).unwrap()), None);
}
