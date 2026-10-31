use super::*;
use crate::geom::{HeadingAngle, Length, TurnAngle};
use crate::geometry::GeometryRole;
use crate::geometry_kinds::{Arc, Segment};
use crate::style::Arrow;

const EPS: f64 = 1e-9;

/// Settings where 1 pt is exactly `unit` world units.
fn settings_with_pt(unit: f64) -> DocumentSettings {
    DocumentSettings {
        scale: MM_PER_PT / unit,
        ..DocumentSettings::default()
    }
}

fn close(a: Point2, b: Point2, eps: f64) -> bool {
    (a.x - b.x).abs() < eps && (a.y - b.y).abs() < eps
}

fn pt(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

fn style(dash: Dash, arrow: Arrow) -> Style {
    Style {
        dash,
        arrow,
        ..Style::BUILT_IN
    }
}

#[test]
fn width_in_world_units() {
    let w = pt_to_world(0.4, &DocumentSettings::default());
    assert!((w - 0.4 * 25.4 / 72.27 / 10.0).abs() < 1e-15);
    assert!((w - 0.01406).abs() < 1e-5);
}

#[test]
fn head_on_a_segment() {
    let mut doc = Document::new();
    doc.set_settings(settings_with_pt(0.01));
    let x = doc.add_geometry::<Segment>(GeometryRole::Result, (pt(0.0, 0.0), pt(1.0, 0.0)));
    let (start, end) = arrow_tips(&doc, doc.vector(), x);
    let end = end.unwrap();
    assert!(close(end.point, pt(1.0, 0.0), EPS));
    assert!(close(start.unwrap().point, pt(0.0, 0.0), EPS));
    assert!((start.unwrap().out.x + 1.0).abs() < EPS);
    let [tip, left, right] = arrow_head(end.point, end.out, 0.4, doc.settings());
    assert!(close(tip, pt(1.0, 0.0), EPS));
    assert!(close(left, pt(0.952, 0.018), EPS));
    assert!(close(right, pt(0.952, -0.018), EPS));
}

#[test]
fn head_on_an_arc_follows_the_tangent() {
    let mut doc = Document::new();
    let a = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            pt(0.0, 0.0),
            Length::new(1.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::FRAC_PI_2),
        ),
    );
    let (_, end) = arrow_tips(&doc, doc.vector(), a);
    let end = end.unwrap();
    assert!(close(end.point, pt(0.0, 1.0), EPS));
    assert!((end.out.x + 1.0).abs() < EPS && end.out.y.abs() < EPS);
}

#[test]
fn labels_have_no_tips() {
    let mut doc = Document::new();
    let l = doc.add_geometry::<crate::geometry_kinds::Label>(GeometryRole::Result, (pt(0.0, 0.0), HeadingAngle::new(0.0)));
    assert_eq!(arrow_tips(&doc, doc.vector(), l), (None, None));
}

#[test]
fn shortened_stroke() {
    let s = settings_with_pt(1.0);
    let pieces = stroke_pieces(&[pt(0.0, 0.0), pt(20.0, 0.0)], &style(Dash::Solid, Arrow::End), &s);
    assert_eq!(pieces.len(), 1);
    assert!(close(pieces[0][0], pt(0.0, 0.0), EPS));
    assert!(close(*pieces[0].last().unwrap(), pt(15.2, 0.0), EPS));
}

#[test]
fn very_short_arrow() {
    let s = settings_with_pt(1.0);
    let pieces = stroke_pieces(&[pt(0.0, 0.0), pt(3.0, 0.0)], &style(Dash::Solid, Arrow::Both), &s);
    assert!(pieces.is_empty());
}

#[test]
fn dashed_segment() {
    let s = settings_with_pt(1.0);
    let pieces = stroke_pieces(&[pt(0.0, 0.0), pt(12.0, 0.0)], &style(Dash::Dashed, Arrow::None), &s);
    let spans: Vec<(f64, f64)> = pieces.iter().map(|p| (p[0].x, p.last().unwrap().x)).collect();
    assert_eq!(spans.len(), 2);
    for ((a, b), (ea, eb)) in spans.into_iter().zip([(0.0, 3.0), (6.0, 9.0)]) {
        assert!((a - ea).abs() < EPS && (b - eb).abs() < EPS, "{a}–{b}");
    }
}

#[test]
fn pattern_continues_around_vertices() {
    // 2 + 10 pt: the first dash bends around the corner.
    let pieces = dash(&[pt(0.0, 0.0), pt(2.0, 0.0), pt(2.0, 10.0)], &[3.0, 3.0], 0.0);
    assert_eq!(pieces.len(), 2);
    assert_eq!(pieces[0].len(), 3);
    assert!(close(pieces[0][1], pt(2.0, 0.0), EPS));
    assert!(close(pieces[0][2], pt(2.0, 1.0), EPS));
    assert!(close(pieces[1][0], pt(2.0, 4.0), EPS));
    assert!(close(pieces[1][1], pt(2.0, 7.0), EPS));
}

#[test]
fn pattern_does_not_depend_on_tessellation() {
    let coarse = dash(&[pt(0.0, 0.0), pt(12.0, 0.0)], &[3.0, 3.0], 0.0);
    let fine: Vec<Point2> = (0..=17).map(|i| pt(12.0 * i as f64 / 17.0, 0.0)).collect();
    let fine = dash(&fine, &[3.0, 3.0], 0.0);
    assert_eq!(coarse.len(), fine.len());
    for (c, f) in coarse.iter().zip(&fine) {
        assert!(close(c[0], f[0], EPS));
        assert!(close(*c.last().unwrap(), *f.last().unwrap(), EPS));
    }
}

#[test]
fn trimmed_start_keeps_the_curve_phase() {
    // Arrow at the start covers 4.8 pt: the pattern still counts from 0.
    let s = settings_with_pt(1.0);
    let pieces = stroke_pieces(&[pt(0.0, 0.0), pt(12.0, 0.0)], &style(Dash::Dashed, Arrow::Start), &s);
    let spans: Vec<(f64, f64)> = pieces.iter().map(|p| (p[0].x, p.last().unwrap().x)).collect();
    assert_eq!(spans.len(), 1);
    assert!((spans[0].0 - 6.0).abs() < EPS && (spans[0].1 - 9.0).abs() < EPS);
}

#[test]
fn dash_patterns() {
    assert!(Dash::Solid.pattern(0.4).is_empty());
    assert_eq!(Dash::Dashed.pattern(0.4), vec![3.0, 3.0]);
    assert_eq!(Dash::Dotted.pattern(0.4), vec![0.4, 2.0]);
    assert_eq!(Dash::DashDot.pattern(0.4), vec![3.0, 2.0, 0.4, 2.0]);
}
