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

fn cubic_stroke(b: [Point2; 4], width: f64) -> Part {
    Part::Stroke {
        path: vec![PathSeg::Move(b[0]), PathSeg::Cubic(b[1], b[2], b[3])],
        width,
        color: Rgb(0, 0, 0),
        dash: None,
    }
}

fn arch() -> [Point2; 4] {
    [Point2::new(0.0, 0.0), Point2::new(1.0, 2.0), Point2::new(2.0, 2.0), Point2::new(3.0, 0.0)]
}

#[test]
fn cubic_bounds_use_the_curve_not_its_control_points() {
    let w = 0.2;
    let b = bounds(&one(cubic_stroke(arch(), w))).unwrap();
    assert!((b.max.y - (1.5 + 0.5 * w)).abs() < TOL, "top {}", b.max.y);
    assert!(b.max.y < 2.0);
}

#[test]
fn cubic_butt_ends_set_the_side_extents() {
    // End tangents (1, 2)/√5 and (1, −2)/√5: the butt corners stick out
    // by h·2/√5 in x and h/√5 in y.
    let h = 0.1;
    let s5 = 5f64.sqrt();
    let b = bounds(&one(cubic_stroke(arch(), 2.0 * h))).unwrap();
    assert_box(b, (-2.0 * h / s5, -h / s5), (3.0 + 2.0 * h / s5, 1.5 + h));
}

#[test]
fn cusped_offset_cubic_bounds_match_the_dense_outline() {
    // Curvature passes 1/h, so the offset curves have cusps. Found by a random
    // search: the end corners and axis-tangent points alone miss the right
    // side by about 0.66 here.
    let b = [Point2::new(-0.043, 0.659), Point2::new(0.808, 0.739), Point2::new(0.559, 0.068), Point2::new(0.643, 0.080)];
    let h = 0.79;
    let got = bounds(&one(cubic_stroke(b, 2.0 * h))).unwrap();
    // Dense outline: both offset curves, ends included.
    let (mut lo, mut hi) = (Point2::new(f64::INFINITY, f64::INFINITY), Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY));
    let n = 200_000;
    for k in 0..=n {
        let u = k as f64 / n as f64;
        let d = cubic::derivative(&b, u);
        let nrm = d.left_normal() * (h / d.length());
        for q in [cubic::point(&b, u) + nrm, cubic::point(&b, u) - nrm] {
            lo = Point2::new(lo.x.min(q.x), lo.y.min(q.y));
            hi = Point2::new(hi.x.max(q.x), hi.y.max(q.y));
        }
    }
    for (g, d) in [(got.min.x, lo.x), (got.min.y, lo.y), (got.max.x, hi.x), (got.max.y, hi.y)] {
        assert!((g - d).abs() < 1e-9, "bounds {got:?} vs dense {lo:?}..{hi:?}");
    }
}

#[test]
fn stroked_quadratic_bounds_are_tight() {
    // Quadratic (0, 0), (1, 2), (2, 0) peaks at y = 1 (control point at 2).
    let rule = Part::Outline {
        contours: vec![vec![
            PathSeg::Move(Point2::new(0.0, 0.0)),
            PathSeg::Quad(Point2::new(1.0, 2.0), Point2::new(2.0, 0.0)),
        ]],
        width: 0.2,
        color: Rgb(0, 0, 0),
    };
    let b = bounds(&one(rule)).unwrap();
    assert!((b.max.y - 1.1).abs() < TOL, "top {}", b.max.y);
}
