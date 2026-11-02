use std::f64::consts::{FRAC_PI_2, PI};

use constraint_runtime::{
    Arc, Arrow, Dash, Document, GeometryId, GeometryRole, HeadingAngle, Length, Rgb, Segment, StyleProps,
    StyleRef, TurnAngle, MM_PER_PT,
};
use lopdf::content::Content as Ops;

use super::*;
use crate::{export, export_svg, ExportError, Format, LabelLines};

/// Largest distance of the cubic `c` from the circle (`center`, `r`), by dense sampling.
fn deviation(c: [Point2; 4], center: Point2, r: f64) -> f64 {
    (0..=400)
        .map(|i| {
            let t = i as f64 / 400.0;
            let u = 1.0 - t;
            let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            let p = Point2::new(
                w.iter().zip(&c).map(|(w, p)| w * p.x).sum(),
                w.iter().zip(&c).map(|(w, p)| w * p.y).sum(),
            );
            ((p - center).length() - r).abs()
        })
        .fold(0.0, f64::max)
}

fn max_deviation_mm(radius: f64, sweep: f64, mm_per_unit: f64) -> (usize, f64) {
    let center = Point2::new(1.0, -2.0);
    let parts = arc_cubics(center, radius, 0.3, sweep, mm_per_unit);
    let dev = parts.iter().map(|&c| deviation(c, center, radius)).fold(0.0, f64::max);
    (parts.len(), dev * mm_per_unit)
}

#[test]
fn arcs_stay_within_one_micrometre() {
    // r = 100 mm on paper, three quarters of a turn.
    let (n, dev) = max_deviation_mm(10.0, 1.5 * PI, 10.0);
    assert!(dev <= 1e-3, "{dev} mm with {n} parts");
    // r = 1 m on paper, clockwise.
    let (n, dev) = max_deviation_mm(100.0, -2.0, 10.0);
    assert!(dev <= 1e-3, "{dev} mm with {n} parts");
    // Full circle, small scale.
    let (_, dev) = max_deviation_mm(3.0, 2.0 * PI, 1.0);
    assert!(dev <= 1e-3);
}

#[test]
fn small_arc_is_one_piece() {
    // Quarter arc of radius 0.1 at scale 10: 1 mm on paper.
    assert_eq!(arc_parts(1.0, FRAC_PI_2), 1);
    let (n, dev) = max_deviation_mm(0.1, FRAC_PI_2, 10.0);
    assert_eq!(n, 1);
    assert!(dev <= 1e-3);
    // More than a quarter turn needs more than one piece, however small.
    assert_eq!(arc_parts(1e-3, PI), 2);
}

#[test]
fn parts_join_and_end_exactly() {
    let center = Point2::new(0.5, 0.5);
    let (start, sweep) = (0.7, -4.1);
    let parts = arc_cubics(center, 20.0, start, sweep, 10.0);
    assert!(parts.len() > 3);
    for w in parts.windows(2) {
        assert_eq!(w[0][3], w[1][0], "no gap between parts");
    }
    assert_eq!(parts[0][0], on_circle(center, 20.0, start));
    assert_eq!(parts.last().unwrap()[3], on_circle(center, 20.0, start + sweep));
}

#[test]
fn quadratic_becomes_an_equal_cubic() {
    let (p0, c, p1) = (Point2::new(0.0, 0.0), Point2::new(1.0, 3.0), Point2::new(4.0, 1.0));
    let (c1, c2) = quad_to_cubic(p0, c, p1);
    for i in 0..=10 {
        let t = i as f64 / 10.0;
        let u = 1.0 - t;
        let q = Point2::new(
            u * u * p0.x + 2.0 * u * t * c.x + t * t * p1.x,
            u * u * p0.y + 2.0 * u * t * c.y + t * t * p1.y,
        );
        let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
        let pts = [p0, c1, c2, p1];
        let k = Point2::new(
            w.iter().zip(&pts).map(|(w, p)| w * p.x).sum(),
            w.iter().zip(&pts).map(|(w, p)| w * p.y).sum(),
        );
        assert!((q - k).length() < 1e-12, "t={t}");
    }
}

// --- the PDF file (task 1.4) ---

fn seg(doc: &mut Document, a: (f64, f64), b: (f64, f64)) -> GeometryId {
    doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(a.0, a.1), Point2::new(b.0, b.1)))
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

fn pdf_of(doc: &Document) -> Vec<u8> {
    export(doc, &LabelLines::new(), Format::Pdf).unwrap()
}

/// The single page's MediaBox and decoded content operations.
fn parse(bytes: &[u8]) -> (Vec<f32>, Vec<lopdf::content::Operation>) {
    let doc = lopdf::Document::load_mem(bytes).unwrap();
    let pages = doc.get_pages();
    assert_eq!(pages.len(), 1, "exactly one page");
    let page_id = pages[&1];
    let page = doc.get_dictionary(page_id).unwrap();
    let media: Vec<f32> = page
        .get(b"MediaBox")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o.as_float().unwrap())
        .collect();
    let data = doc.get_page_content(page_id);
    (media, Ops::decode(&data).unwrap().operations)
}

fn operators(ops: &[lopdf::content::Operation]) -> Vec<&str> {
    ops.iter().map(|o| o.operator.as_str()).collect()
}

#[test]
fn page_size_is_the_physical_size() {
    let mut doc = Document::new();
    seg(&mut doc, (0.0, 0.0), (3.0, 0.0));
    let (media, _) = parse(&pdf_of(&doc));
    let bp = |mm: f64| (mm * 72.0 / 25.4) as f32;
    assert_eq!(media[..2], [0.0, 0.0]);
    assert!((media[2] - bp(30.0)).abs() < 1e-3, "{media:?}");
    assert!((media[3] - bp(0.4 * MM_PER_PT)).abs() < 1e-4, "{media:?}");
}

#[test]
fn file_is_self_contained_without_metadata() {
    let mut doc = Document::new();
    seg(&mut doc, (0.0, 0.0), (3.0, 0.0));
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(1.0, 1.0), Length::new(1.0), HeadingAngle::new(0.0), TurnAngle::new(PI)),
    );
    let bytes = pdf_of(&doc);
    assert!(bytes.starts_with(b"%PDF-"));
    let raw = String::from_utf8_lossy(&bytes);
    for key in ["/Font", "/XObject", "/Image", "/Info", "/ID", "CreationDate", "ModDate"] {
        assert!(!raw.contains(key), "{key}");
    }
    assert!(raw.contains("/FlateDecode"));
    let lp = lopdf::Document::load_mem(&bytes).unwrap();
    assert!(lp.trailer.get(b"Info").is_err() && lp.trailer.get(b"ID").is_err());
}

#[test]
fn content_has_style_dashes_and_fills() {
    let mut doc = Document::new();
    let s = seg(&mut doc, (0.0, 0.0), (3.0, 0.0));
    styled(
        &mut doc,
        s,
        StyleProps {
            color: Some(Rgb(0xc0, 0, 0)),
            dash: Some(Dash::Dashed),
            arrow: Some(Arrow::End),
            ..StyleProps::default()
        },
    );
    let (_, ops) = parse(&pdf_of(&doc));
    assert_eq!(
        operators(&ops),
        ["RG", "w", "J", "j", "d", "m", "l", "S", "rg", "m", "l", "l", "h", "f"]
    );
    let num = |o: &lopdf::Object| o.as_float().unwrap();
    let rg = &ops[0].operands;
    assert!((num(&rg[0]) - 192.0 / 255.0).abs() < 1e-4 && num(&rg[1]) == 0.0);
    assert_eq!(num(&ops[2].operands[0]), 0.0, "butt caps");
    assert_eq!(num(&ops[3].operands[0]), 0.0, "miter joins");
    let dash = ops[4].operands[0].as_array().unwrap();
    let bp_per_pt = 72.0 / 72.27;
    assert!((num(&dash[0]) - 3.0 * bp_per_pt as f32).abs() < 1e-4);
    assert_eq!(num(&ops[4].operands[1]), 0.0, "no arrow at the start: phase 0");
    assert!((num(&ops[1].operands[0]) - 0.4 * bp_per_pt as f32).abs() < 1e-4, "width");
}

#[test]
fn arc_pieces_stay_within_one_micrometre_in_the_file() {
    let mut doc = Document::new();
    doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Length::new(10.0), HeadingAngle::new(0.0), TurnAngle::new(1.5 * PI)),
    );
    let bytes = pdf_of(&doc);
    let (media, ops) = parse(&bytes);
    assert!(operators(&ops).iter().all(|o| !["l"].contains(o)), "no straight pieces");
    let k = 10.0 * 72.0 / 25.4; // bp per world unit
    let h = 0.2 * MM_PER_PT / 10.0; // half the stroke width in world units
    // The box's lower left lies at (-r - h, -r - h): the centre on the page.
    let center = Point2::new((10.0 + h) * k, (10.0 + h) * k);
    let r = 10.0 * k;
    let mut current = None;
    let mut pieces = 0;
    for op in &ops {
        let p: Vec<f64> = op.operands.iter().filter_map(|o| o.as_float().ok()).map(f64::from).collect();
        match op.operator.as_str() {
            "m" => current = Some(Point2::new(p[0], p[1])),
            "c" => {
                let c = [
                    current.unwrap(),
                    Point2::new(p[0], p[1]),
                    Point2::new(p[2], p[3]),
                    Point2::new(p[4], p[5]),
                ];
                let dev_mm = deviation(c, center, r) / (72.0 / 25.4);
                assert!(dev_mm <= 1e-3, "{dev_mm} mm");
                current = Some(c[3]);
                pieces += 1;
            }
            _ => {}
        }
    }
    assert_eq!(pieces, arc_parts(100.0, 1.5 * PI));
    assert!(media[2] > 0.0);
}

#[test]
fn equal_drawings_give_equal_bytes() {
    let mut doc = Document::new();
    let s = seg(&mut doc, (0.0, 0.0), (2.0, 1.0));
    styled(
        &mut doc,
        s,
        StyleProps {
            arrow: Some(Arrow::Both),
            ..StyleProps::default()
        },
    );
    assert_eq!(pdf_of(&doc), pdf_of(&doc));
}

#[test]
fn nothing_visible_is_not_exported() {
    assert_eq!(
        export(&Document::new(), &LabelLines::new(), Format::Pdf),
        Err(ExportError::NothingToExport)
    );
}

#[test]
fn svg_format_matches_export_svg() {
    let mut doc = Document::new();
    seg(&mut doc, (0.0, 0.0), (3.0, 1.0));
    assert_eq!(
        export(&doc, &LabelLines::new(), Format::Svg).unwrap(),
        export_svg(&doc, &LabelLines::new()).unwrap().into_bytes()
    );
    assert_eq!((Format::Svg.extension(), Format::Pdf.extension()), ("svg", "pdf"));
    assert_eq!((Format::Svg.mime(), Format::Pdf.mime()), ("image/svg+xml", "application/pdf"));
}
