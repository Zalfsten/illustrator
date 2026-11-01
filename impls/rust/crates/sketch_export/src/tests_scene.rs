use std::f64::consts::FRAC_PI_2;

use constraint_runtime::{
    Arc, Arrow, Dash, DocumentSettings, HeadingAngle, KindAttrs, Label, LabelAttrs, Length, Segment as SegmentKind,
    StyleProps, StyleRef, TurnAngle, MM_PER_PT,
};
use label_layout::{Shape, TypesetLine};

use super::*;

const TOL: f64 = 1e-12;

fn pt(v: f64) -> f64 {
    v * MM_PER_PT / DocumentSettings::default().scale
}

fn close(a: Point2, b: Point2) -> bool {
    (a - b).length() < TOL
}

fn segment(doc: &mut Document, role: GeometryRole, a: (f64, f64), b: (f64, f64)) -> GeometryId {
    doc.add_geometry::<SegmentKind>(role, (Point2::new(a.0, a.1), Point2::new(b.0, b.1)))
}

fn style(doc: &mut Document, id: GeometryId, overrides: StyleProps) {
    doc.set_geometry_style(
        id,
        StyleRef {
            overrides,
            ..StyleRef::default()
        },
    )
    .unwrap();
}

fn only(scene: &Scene) -> &[Part] {
    assert_eq!(scene.objects.len(), 1, "{scene:?}");
    &scene.objects[0].parts
}

fn stroke(part: &Part) -> (&[PathSeg], f64, Rgb, Option<&DashArray>) {
    match part {
        Part::Stroke {
            path,
            width,
            color,
            dash,
        } => (path, *width, *color, dash.as_ref()),
        other => panic!("stroke expected, got {other:?}"),
    }
}

#[test]
fn construction_geometry_is_left_out() {
    let mut doc = Document::new();
    let seg = segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (3.0, 0.0));
    segment(&mut doc, GeometryRole::Construction, (0.0, 1.0), (3.0, 1.0));
    let s = scene(&doc, &LabelLines::new()).unwrap();
    assert_eq!(s.objects.len(), 1);
    assert_eq!(s.objects[0].id, seg);
    assert_eq!(s.objects[0].name, "seg1");
}

#[test]
fn segment_is_one_straight_piece() {
    let mut doc = Document::new();
    segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (3.0, 0.0));
    let s = scene(&doc, &LabelLines::new()).unwrap();
    let parts = only(&s);
    assert_eq!(parts.len(), 1);
    let (path, width, color, dash) = stroke(&parts[0]);
    assert_eq!(
        path,
        [
            PathSeg::Move(Point2::new(0.0, 0.0)),
            PathSeg::Line(Point2::new(3.0, 0.0))
        ]
    );
    assert!((width - pt(0.4)).abs() < TOL);
    assert_eq!(color, Rgb(0, 0, 0));
    assert_eq!(dash, None);
}

#[test]
fn arc_is_written_as_arc_pieces() {
    let mut doc = Document::new();
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.0, 0.0),
            Length::new(2.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(FRAC_PI_2),
        ),
    );
    let s = scene(&doc, &LabelLines::new()).unwrap();
    let (path, ..) = stroke(&only(&s)[0]);
    assert_eq!(path.len(), 2);
    assert!(matches!(path[0], PathSeg::Move(p) if close(p, Point2::new(2.0, 0.0))));
    assert_eq!(
        path[1],
        PathSeg::Arc {
            center: Point2::new(0.0, 0.0),
            radius: 2.0,
            start: 0.0,
            sweep: FRAC_PI_2
        }
    );
    assert!(close(path[1].end().unwrap(), Point2::new(0.0, 2.0)));
}

#[test]
fn width_and_colour_at_physical_size() {
    let mut doc = Document::new();
    let seg = segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (3.0, 0.0));
    style(
        &mut doc,
        seg,
        StyleProps {
            width: Some(0.8),
            color: Some(Rgb(0xc0, 0, 0)),
            ..StyleProps::default()
        },
    );
    let s = scene(&doc, &LabelLines::new()).unwrap();
    let (_, width, color, _) = stroke(&only(&s)[0]);
    assert!((width - pt(0.8)).abs() < TOL);
    assert_eq!(color, Rgb(0xc0, 0, 0));
}

#[test]
fn arrow_at_the_end_and_stroke_ends_at_its_base() {
    let mut doc = Document::new();
    let seg = segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (3.0, 0.0));
    style(
        &mut doc,
        seg,
        StyleProps {
            arrow: Some(Arrow::End),
            ..StyleProps::default()
        },
    );
    let s = scene(&doc, &LabelLines::new()).unwrap();
    let parts = only(&s);
    assert_eq!(parts.len(), 2);
    let len = pt(3.0 + 4.5 * 0.4);
    let (path, ..) = stroke(&parts[0]);
    assert!(matches!(path[1], PathSeg::Line(p) if close(p, Point2::new(3.0 - len, 0.0))));
    let Part::Fill { contours, .. } = &parts[1] else {
        panic!("arrow head expected")
    };
    let c = &contours[0];
    assert!(matches!(c[0], PathSeg::Move(p) if close(p, Point2::new(3.0, 0.0))), "tip at the end");
    for corner in [c[1], c[2]] {
        let PathSeg::Line(q) = corner else { panic!() };
        assert!((q.x - (3.0 - len)).abs() < TOL, "base where the stroke ends");
        assert!((q.y.abs() - 0.375 * len).abs() < TOL, "base width 0.75 L");
    }
    assert_eq!(c[3], PathSeg::Close);
}

#[test]
fn dash_phase_starts_at_the_curve_start() {
    let mut doc = Document::new();
    let seg = segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (3.0, 0.0));
    style(
        &mut doc,
        seg,
        StyleProps {
            dash: Some(Dash::Dashed),
            arrow: Some(Arrow::Start),
            ..StyleProps::default()
        },
    );
    let s = scene(&doc, &LabelLines::new()).unwrap();
    let (path, _, _, dash) = stroke(&only(&s)[0]);
    let len = pt(3.0 + 4.5 * 0.4);
    assert!(matches!(path[0], PathSeg::Move(p) if close(p, Point2::new(len, 0.0))));
    let dash = dash.expect("dashed");
    assert!((dash.phase - len).abs() < TOL, "pattern counted from the curve start");
    assert_eq!(dash.pattern.len(), 2);
    assert!((dash.pattern[0] - pt(3.0)).abs() < TOL && (dash.pattern[1] - pt(3.0)).abs() < TOL);
}

#[test]
fn short_curve_gets_heads_only() {
    let mut doc = Document::new();
    let seg = segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (0.01, 0.0));
    style(
        &mut doc,
        seg,
        StyleProps {
            arrow: Some(Arrow::Both),
            ..StyleProps::default()
        },
    );
    let s = scene(&doc, &LabelLines::new()).unwrap();
    let parts = only(&s);
    assert_eq!(parts.len(), 2);
    assert!(parts.iter().all(|p| matches!(p, Part::Fill { .. })));
}

#[test]
fn arc_trim_keeps_an_arc() {
    let quarter = ShapePiece::Arc {
        center: Point2::new(0.0, 0.0),
        radius: 1.0,
        start: HeadingAngle::new(0.0),
        sweep: TurnAngle::new(FRAC_PI_2),
    };
    let line = ShapePiece::Line {
        start: Point2::new(0.0, 1.0),
        end: Point2::new(-1.0, 1.0),
    };
    let out = trim(&[quarter, line], 0.5, 0.25);
    assert_eq!(out.len(), 2);
    let ShapePiece::Arc { start, sweep, .. } = out[0] else {
        panic!("arc expected")
    };
    assert!((start.radians() - 0.5).abs() < TOL);
    assert!((sweep.radians() - (FRAC_PI_2 - 0.5)).abs() < TOL);
    assert!(close(out[1].end_point(), Point2::new(-0.75, 1.0)));
    assert!(trim(&[line], 0.6, 0.4).is_empty());
}

// --- labels (task 2.4) ---

/// A typeset line with one filled unit square at the origin and one stroked rule.
fn square_line(width: f64) -> TypesetLine {
    let p = |x, y| Point2::new(x, y);
    TypesetLine {
        shapes: vec![
            Shape {
                contours: vec![Contour(vec![
                    Segment::MoveTo(p(0.0, 0.0)),
                    Segment::LineTo(p(1.0, 0.0)),
                    Segment::QuadTo(p(1.0, 1.0), p(0.0, 1.0)),
                    Segment::Close,
                ])],
                paint: Paint::Fill,
            },
            Shape {
                contours: vec![Contour(vec![Segment::MoveTo(p(0.0, -1.0)), Segment::LineTo(p(width, -1.0))])],
                paint: Paint::Stroke { width: 0.5 },
            },
        ],
        width,
        height: 1.0,
        depth: 1.0,
    }
}

fn label(doc: &mut Document, role: GeometryRole, source: &str) -> GeometryId {
    doc.add_geometry_with::<Label>(
        role,
        (Point2::new(1.0, 2.0), HeadingAngle::new(FRAC_PI_2)),
        KindAttrs::Label(LabelAttrs {
            source: source.into(),
            offset: Vec2::new(2.0, 3.0),
            ..LabelAttrs::default()
        }),
    )
}

#[test]
fn exported_labels_are_result_labels_with_their_requests() {
    let mut doc = Document::new();
    let a = label(&mut doc, GeometryRole::Result, r"$x$ \\ $y$");
    label(&mut doc, GeometryRole::Construction, "c");
    segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let out = exported_labels(&doc);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, a);
    assert_eq!(out[0].1, label_requests(doc.label_attrs(a).unwrap(), doc.settings()));
    assert_eq!(out[0].1.len(), 2);
}

#[test]
fn label_outlines_are_placed_in_style_colour() {
    let mut doc = Document::new();
    let id = label(&mut doc, GeometryRole::Result, "$x$");
    style(
        &mut doc,
        id,
        StyleProps {
            color: Some(Rgb(0, 0, 0xff)),
            ..StyleProps::default()
        },
    );
    let line = square_line(4.0);
    let labels = LabelLines::from([(id, vec![line.clone()])]);
    let s = scene(&doc, &labels).unwrap();
    let parts = only(&s);
    assert_eq!(parts.len(), 2);
    let map = place(
        &[LineMetrics::from(&line)],
        doc.label_attrs(id).unwrap(),
        doc.settings(),
        Point2::new(1.0, 2.0),
        FRAC_PI_2,
    )[0];
    let Part::Fill { contours, color } = &parts[0] else {
        panic!("fill expected")
    };
    assert_eq!(*color, Rgb(0, 0, 0xff));
    assert_eq!(
        contours[0],
        vec![
            PathSeg::Move(map.apply(Point2::new(0.0, 0.0))),
            PathSeg::Line(map.apply(Point2::new(1.0, 0.0))),
            PathSeg::Quad(map.apply(Point2::new(1.0, 1.0)), map.apply(Point2::new(0.0, 1.0))),
            PathSeg::Close,
        ]
    );
    let Part::Outline { width, color, .. } = &parts[1] else {
        panic!("outline expected")
    };
    assert!((width - pt(0.5)).abs() < TOL);
    assert_eq!(*color, Rgb(0, 0, 0xff));
}

#[test]
fn construction_and_empty_labels_add_nothing() {
    let mut doc = Document::new();
    let c = label(&mut doc, GeometryRole::Construction, "$x$");
    label(&mut doc, GeometryRole::Result, "");
    segment(&mut doc, GeometryRole::Result, (0.0, 0.0), (1.0, 0.0));
    let labels = LabelLines::from([(c, vec![square_line(1.0)])]);
    let s = scene(&doc, &labels).unwrap();
    assert_eq!(s.objects.len(), 1);
    assert_eq!(s.objects[0].name, "seg1");
}

#[test]
fn missing_lines_name_the_label() {
    let mut doc = Document::new();
    let id = label(&mut doc, GeometryRole::Result, r"a \\ b");
    assert_eq!(scene(&doc, &LabelLines::new()), Err(ExportError::MissingLines(id)));
    let one = LabelLines::from([(id, vec![square_line(1.0)])]);
    assert_eq!(scene(&doc, &one), Err(ExportError::MissingLines(id)), "all lines needed");
}
