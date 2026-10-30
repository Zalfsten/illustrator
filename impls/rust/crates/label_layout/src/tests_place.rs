use constraint_runtime::{DocumentSettings, HAlign, LabelAttrs, Markup, Point2, VAlign, Vec2};

use super::*;
use crate::split::split_lines;
use crate::typeset::{LineTypesetter, TypesetError, TypesetLine};

const EPS: f64 = 1e-9;

/// Every character 5 pt wide, 7 pt high, 2 pt deep; `T` is a tall glyph
/// (12 high, 9 deep); an empty line has no extent.
struct Fixed;

impl LineTypesetter for Fixed {
    fn typeset_line(
        &self,
        line: &str,
        _: Markup,
        _: f64,
        _: &str,
    ) -> Result<TypesetLine, TypesetError> {
        let tall = line.contains('T');
        let empty = line.is_empty();
        Ok(TypesetLine {
            shapes: Vec::new(),
            width: 5.0 * line.chars().count() as f64,
            height: if empty {
                0.0
            } else if tall {
                12.0
            } else {
                7.0
            },
            depth: if empty {
                0.0
            } else if tall {
                9.0
            } else {
                2.0
            },
        })
    }
}

fn metrics(source: &str, markup: Markup) -> Vec<LineMetrics> {
    split_lines(source, markup)
        .into_iter()
        .map(|l| LineMetrics::from(&Fixed.typeset_line(l, markup, 10.0, "").unwrap()))
        .collect()
}

/// Scale with 1 world unit per pt.
fn unit_settings() -> DocumentSettings {
    DocumentSettings {
        scale: 25.4 / 72.27,
        ..DocumentSettings::default()
    }
}

fn close(p: Point2, x: f64, y: f64) {
    assert!(
        (p.x - x).abs() < EPS && (p.y - y).abs() < EPS,
        "{p:?} != ({x}, {y})"
    );
}

#[test]
fn plain_lines_are_twelve_points_apart() {
    assert_eq!(
        baselines(&metrics("a\nb", Markup::Text), 10.0),
        [0.0, -12.0]
    );
}

#[test]
fn tall_line_pushes_apart_with_one_point_gap() {
    let m = metrics("T\nx", Markup::Text);
    let b = baselines(&m, 10.0);
    assert!(b[0] - b[1] > 12.0);
    let gap = (b[0] - m[0].depth) - (b[1] + m[1].height);
    assert!((gap - 1.0).abs() < 0.01, "gap {gap}");
}

#[test]
fn empty_middle_line_still_takes_its_spacing() {
    let b = baselines(&metrics(r"a\\\\b", Markup::Latex), 10.0);
    assert_eq!(b, [0.0, -12.0, -24.0]);
}

#[test]
fn placement_without_an_engine_follows_offset_in_a_turned_frame() {
    // labels "Offset in a turned frame": scale 10, heading π/2, offset [0, 72.27].
    let attrs = LabelAttrs {
        offset: Vec2::new(0.0, 72.27),
        ..LabelAttrs::default()
    };
    let maps = place(
        &metrics("A", Markup::Text),
        &attrs,
        &DocumentSettings::default(),
        Point2::new(0.0, 0.0),
        std::f64::consts::FRAC_PI_2,
    );
    close(maps[0].apply(Point2::new(0.0, 0.0)), -2.54, 0.0);
    // One pt along the baseline goes along the direction (0, 1).
    let k = world_per_pt(&DocumentSettings::default());
    close(maps[0].apply(Point2::new(1.0, 0.0)), -2.54, k);
}

#[test]
fn every_alignment_combination() {
    // Lines "aa" (10 pt) and "aaaa" (20 pt); baselines 0 and -12.
    let m = metrics("aa\naaaa", Markup::Text);
    let s = unit_settings();
    for (halign, dx) in [
        (HAlign::Left, [0.0, 0.0]),
        (HAlign::Center, [-5.0, -10.0]),
        (HAlign::Right, [-10.0, -20.0]),
    ] {
        for (valign, dy) in [(VAlign::First, 0.0), (VAlign::Last, 12.0)] {
            let attrs = LabelAttrs {
                halign,
                valign,
                ..LabelAttrs::default()
            };
            let maps = place(&m, &attrs, &s, Point2::new(100.0, 50.0), 0.0);
            close(
                maps[0].apply(Point2::new(0.0, 0.0)),
                100.0 + dx[0],
                50.0 + dy,
            );
            close(
                maps[1].apply(Point2::new(0.0, 0.0)),
                100.0 + dx[1],
                50.0 + dy - 12.0,
            );
        }
    }
}

#[test]
fn own_size_overrides_the_document_size() {
    let s = DocumentSettings::default();
    assert_eq!(label_size(&LabelAttrs::default(), &s), 10.0);
    let own = LabelAttrs {
        size: Some(8.0),
        ..LabelAttrs::default()
    };
    assert_eq!(label_size(&own, &s), 8.0);
    // At 8 pt, 1.2 × size is 9.6 pt, but depth 2 + height 7 + 1 = 10 pt wins.
    let maps = place(
        &metrics("a\nb", Markup::Text),
        &own,
        &unit_settings(),
        Point2::new(0.0, 0.0),
        0.0,
    );
    close(maps[1].apply(Point2::new(0.0, 0.0)), 0.0, -10.0);
    // Empty lines have no extent, so 1.2 × 8 pt holds.
    let maps = place(
        &metrics(r"\\", Markup::Latex),
        &own,
        &unit_settings(),
        Point2::new(0.0, 0.0),
        0.0,
    );
    close(maps[1].apply(Point2::new(0.0, 0.0)), 0.0, -9.6);
}
