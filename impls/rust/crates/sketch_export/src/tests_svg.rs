use std::f64::consts::{FRAC_PI_2, PI};

use constraint_runtime::{
    Arc, Arrow, Document, GeometryId, GeometryRole, HeadingAngle, KindAttrs, Label, LabelAttrs, Length,
    Point2, Segment as SegmentKind, StyleProps, StyleRef, TurnAngle,
};
use label_layout::{Contour, Paint, Segment, Shape, TypesetLine};

use crate::{export_svg, ExportError, LabelLines};

fn seg(doc: &mut Document, a: (f64, f64), b: (f64, f64)) -> GeometryId {
    doc.add_geometry::<SegmentKind>(GeometryRole::Result, (Point2::new(a.0, a.1), Point2::new(b.0, b.1)))
}

fn arrow_end(doc: &mut Document, id: GeometryId) {
    doc.set_geometry_style(
        id,
        StyleRef {
            overrides: StyleProps {
                arrow: Some(Arrow::End),
                ..StyleProps::default()
            },
            ..StyleRef::default()
        },
    )
    .unwrap();
}

fn glyph_line() -> TypesetLine {
    let p = |x, y| Point2::new(x, y);
    TypesetLine {
        shapes: vec![Shape {
            contours: vec![Contour(vec![
                Segment::MoveTo(p(0.0, 0.0)),
                Segment::LineTo(p(4.0, 0.0)),
                Segment::CubicTo(p(4.0, 3.0), p(0.0, 3.0), p(0.0, 0.0)),
                Segment::Close,
            ])],
            paint: Paint::Fill,
        }],
        width: 4.0,
        height: 2.25,
        depth: 0.0,
    }
}

#[test]
fn small_drawing_has_the_expected_text() {
    let mut doc = Document::new();
    seg(&mut doc, (0.0, 0.0), (3.0, 0.0));
    let text = export_svg(&doc, &LabelLines::new()).unwrap();
    assert_eq!(
        text,
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"30mm\" height=\"0.141mm\" viewBox=\"0 0 30 0.141\">\n",
            "<g id=\"seg1\">\n",
            "<path d=\"M 0 0.07 L 30 0.07\" fill=\"none\" stroke=\"#000000\" stroke-width=\"0.141\" stroke-linecap=\"butt\"/>\n",
            "</g>\n",
            "</svg>\n",
        )
    );
}

#[test]
fn up_stays_up() {
    let mut doc = Document::new();
    let id = seg(&mut doc, (0.0, 0.0), (0.0, 1.0));
    arrow_end(&mut doc, id);
    let text = export_svg(&doc, &LabelLines::new()).unwrap();
    let head = text.lines().find(|l| l.contains("fill-rule")).unwrap();
    // The tip is the first point of the head: at the top edge of the page.
    let d = head.split('"').nth(1).unwrap();
    let mut it = d.split(' ');
    assert_eq!(it.next(), Some("M"));
    let (_x, y) = (it.next().unwrap(), it.next().unwrap());
    assert_eq!(y, "0", "{head}");
}

#[test]
fn full_circle_is_split_into_half_arcs() {
    let mut doc = Document::new();
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Length::new(1.0), HeadingAngle::new(0.0), TurnAngle::new(2.0 * PI)),
    );
    let text = export_svg(&doc, &LabelLines::new()).unwrap();
    let path = text.lines().find(|l| l.starts_with("<path")).unwrap();
    assert_eq!(path.matches(" A ").count(), 2, "{path}");
    assert_eq!(path.matches("A 10 10 0 0 0 ").count(), 2, "counterclockwise world sweep: flag 0");
    assert!(!path.contains(" L "));
}

#[test]
fn clockwise_arc_has_sweep_flag_1() {
    let mut doc = Document::new();
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Length::new(1.0), HeadingAngle::new(FRAC_PI_2), TurnAngle::new(-FRAC_PI_2)),
    );
    let text = export_svg(&doc, &LabelLines::new()).unwrap();
    assert!(text.contains("A 10 10 0 0 1 "), "{text}");
}

#[test]
fn exporting_twice_gives_identical_bytes() {
    let mut doc = Document::new();
    let a = seg(&mut doc, (0.0, 0.0), (3.0, 1.0));
    arrow_end(&mut doc, a);
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(1.0, 1.0), Length::new(0.5), HeadingAngle::new(0.2), TurnAngle::new(4.0)),
    );
    assert_eq!(
        export_svg(&doc, &LabelLines::new()).unwrap(),
        export_svg(&doc, &LabelLines::new()).unwrap()
    );
}

#[test]
fn nothing_visible_is_not_exported() {
    let mut doc = Document::new();
    doc.add_geometry::<SegmentKind>(
        GeometryRole::Construction,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    assert_eq!(export_svg(&doc, &LabelLines::new()), Err(ExportError::NothingToExport));
    assert_eq!(export_svg(&Document::new(), &LabelLines::new()), Err(ExportError::NothingToExport));
}

#[test]
fn standalone_file_has_only_paths() {
    let mut doc = Document::new();
    let s = seg(&mut doc, (0.0, 0.0), (3.0, 0.0));
    arrow_end(&mut doc, s);
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(1.0, 1.0), Length::new(1.0), HeadingAngle::new(0.0), TurnAngle::new(PI)),
    );
    let lbl = doc.add_geometry_with::<Label>(
        GeometryRole::Result,
        (Point2::new(0.0, 2.0), HeadingAngle::new(0.0)),
        KindAttrs::Label(LabelAttrs {
            source: "$x$".into(),
            ..LabelAttrs::default()
        }),
    );
    let text = export_svg(&doc, &LabelLines::from([(lbl, vec![glyph_line()])])).unwrap();
    let xml = roxmltree::Document::parse(&text).unwrap();
    let root = xml.root_element();
    assert_eq!(root.tag_name().name(), "svg");
    assert_eq!(root.tag_name().namespace(), Some("http://www.w3.org/2000/svg"));
    assert!(root.attribute("width").unwrap().ends_with("mm"));
    assert!(root.attribute("height").unwrap().ends_with("mm"));
    let mut paths = 0;
    for node in xml.descendants().filter(|n| n.is_element()) {
        let name = node.tag_name().name();
        assert!(matches!(name, "svg" | "g" | "path"), "unexpected element {name}");
        for a in node.attributes() {
            assert!(!a.value().contains("url(") && !a.name().contains("href"), "{a:?}");
        }
        paths += usize::from(name == "path");
    }
    // Stroke + head, arc, label glyph.
    assert_eq!(paths, 4);
    let ids: Vec<_> = xml.descendants().filter_map(|n| n.attribute("id")).collect();
    assert_eq!(ids, ["seg1", "arc1", "lbl1"]);
    assert!(text.contains(" C "), "glyph curves kept");
}
