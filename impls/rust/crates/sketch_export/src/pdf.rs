//! A scene as a standalone single-page PDF (drawing-export: PDF form).
//!
//! PDF has no circular-arc operator: arcs become cubic Bézier parts that stay
//! within 1 µm of the circle on paper; quadratic glyph pieces become exact
//! cubics. No fonts, images, metadata or file identifier, so equal scenes
//! give equal bytes.

use std::f64::consts::FRAC_PI_2;

use constraint_runtime::{DocumentSettings, Point2, Rgb, Vec2};
use pdf_writer::types::{LineCapStyle, LineJoinStyle};
use pdf_writer::{Content, Filter, Finish, Pdf, Rect, Ref};

use crate::bounds::Bounds;
use crate::scene::{on_circle, DashArray, Part, PathSeg, Scene};

/// PDF points (bp) per millimetre.
const BP_PER_MM: f64 = 72.0 / 25.4;

/// Largest radial deviation of the standard cubic for a quarter circle,
/// relative to the radius (conservatively rounded up).
const QUARTER_DEVIATION: f64 = 2.73e-4;

/// Design target for the arc deviation on paper: half the 1 µm the spec
/// allows, leaving room for rounding.
const ARC_TARGET_MM: f64 = 0.5e-3;

/// Number of equal parts for an arc of `radius_mm` on paper turning by
/// `sweep` radians: at most a quarter turn each, and within [`ARC_TARGET_MM`].
pub(crate) fn arc_parts(radius_mm: f64, sweep: f64) -> usize {
    let turn = sweep.abs();
    let mut n = ((turn / FRAC_PI_2) - 1e-12).ceil().max(1.0) as usize;
    while QUARTER_DEVIATION * radius_mm * (turn / n as f64 / FRAC_PI_2).powi(6) > ARC_TARGET_MM {
        n += 1;
    }
    n
}

/// Cubic parts `[p0, c1, c2, p3]` of an arc around `center` from heading
/// `start` turning by `sweep`; `mm_per_unit` gives its size on paper. Part
/// ends lie exactly on the circle.
pub(crate) fn arc_cubics(
    center: Point2,
    radius: f64,
    start: f64,
    sweep: f64,
    mm_per_unit: f64,
) -> Vec<[Point2; 4]> {
    let n = arc_parts(radius * mm_per_unit, sweep);
    let step = sweep / n as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan() * radius;
    let tangent = |a: f64| Vec2::new(-a.sin(), a.cos());
    (0..n)
        .map(|i| {
            let a0 = start + step * i as f64;
            // The last part ends exactly at the arc's end.
            let a1 = if i + 1 == n { start + sweep } else { start + step * (i + 1) as f64 };
            let p0 = on_circle(center, radius, a0);
            let p3 = on_circle(center, radius, a1);
            [p0, p0 + tangent(a0) * k, p3 - tangent(a1) * k, p3]
        })
        .collect()
}

/// Control points of the cubic equal to the quadratic `p0, c, p1`.
pub(crate) fn quad_to_cubic(p0: Point2, c: Point2, p1: Point2) -> (Point2, Point2) {
    (p0 + (c - p0) * (2.0 / 3.0), p1 + (c - p1) * (2.0 / 3.0))
}

/// World coordinates to bp on the page, origin at the bounds' lower left.
struct Page {
    min: Point2,
    /// bp per world unit.
    k: f64,
    /// mm per world unit.
    mm: f64,
}

impl Page {
    fn len(&self, v: f64) -> f32 {
        (v * self.k) as f32
    }

    fn xy(&self, p: Point2) -> (f32, f32) {
        (((p.x - self.min.x) * self.k) as f32, ((p.y - self.min.y) * self.k) as f32)
    }
}

fn rgb(c: Rgb) -> (f32, f32, f32) {
    let f = |v: u8| f32::from(v) / 255.0;
    (f(c.0), f(c.1), f(c.2))
}

/// Path operators of `path`.
fn path(content: &mut Content, page: &Page, path: &[PathSeg]) {
    let mut current = Point2::new(0.0, 0.0);
    let mut first = current;
    for seg in path {
        match *seg {
            PathSeg::Move(p) => {
                let (x, y) = page.xy(p);
                content.move_to(x, y);
                (current, first) = (p, p);
            }
            PathSeg::Line(p) => {
                let (x, y) = page.xy(p);
                content.line_to(x, y);
                current = p;
            }
            PathSeg::Quad(c, p) => {
                let (c1, c2) = quad_to_cubic(current, c, p);
                cubic(content, page, c1, c2, p);
                current = p;
            }
            PathSeg::Cubic(c1, c2, p) => {
                cubic(content, page, c1, c2, p);
                current = p;
            }
            PathSeg::Arc {
                center,
                radius,
                start,
                sweep,
            } => {
                for [_, c1, c2, p] in arc_cubics(center, radius, start, sweep, page.mm) {
                    cubic(content, page, c1, c2, p);
                    current = p;
                }
            }
            PathSeg::Close => {
                content.close_path();
                current = first;
            }
        }
    }
}

fn cubic(content: &mut Content, page: &Page, c1: Point2, c2: Point2, p: Point2) {
    let ((x1, y1), (x2, y2), (x3, y3)) = (page.xy(c1), page.xy(c2), page.xy(p));
    content.cubic_to(x1, y1, x2, y2, x3, y3);
}

/// Stroke state for one part: colour, width, flat ends, miter joins, dashes.
fn stroke_state(content: &mut Content, page: &Page, color: Rgb, width: f64, dash: Option<&DashArray>) {
    let (r, g, b) = rgb(color);
    content.set_stroke_rgb(r, g, b);
    content.set_line_width(page.len(width));
    content.set_line_cap(LineCapStyle::ButtCap);
    content.set_line_join(LineJoinStyle::MiterJoin);
    match dash {
        Some(d) => content.set_dash_pattern(d.pattern.iter().map(|&v| page.len(v)), page.len(d.phase)),
        None => content.set_dash_pattern([], 0.0),
    };
}

/// `scene` as a one-page PDF whose media box is exactly `bounds`, at the
/// document scale.
pub fn pdf(scene: &Scene, bounds: &Bounds, settings: &DocumentSettings) -> Vec<u8> {
    let page = Page {
        min: bounds.min,
        k: settings.scale * BP_PER_MM,
        mm: settings.scale,
    };
    let mut content = Content::new();
    for part in scene.objects.iter().flat_map(|o| &o.parts) {
        match part {
            Part::Stroke {
                path: p,
                width,
                color,
                dash,
            } => {
                stroke_state(&mut content, &page, *color, *width, dash.as_ref());
                path(&mut content, &page, p);
                content.stroke();
            }
            Part::Fill { contours, color } => {
                let (r, g, b) = rgb(*color);
                content.set_fill_rgb(r, g, b);
                for c in contours {
                    path(&mut content, &page, c);
                }
                content.fill_nonzero();
            }
            Part::Outline {
                contours,
                width,
                color,
            } => {
                stroke_state(&mut content, &page, *color, *width, None);
                for c in contours {
                    path(&mut content, &page, c);
                }
                content.stroke();
            }
        }
    }
    let data = miniz_oxide::deflate::compress_to_vec_zlib(&content.finish(), 6);

    let (catalog, pages, page_id, content_id) = (Ref::new(1), Ref::new(2), Ref::new(3), Ref::new(4));
    let mut pdf = Pdf::new();
    pdf.catalog(catalog).pages(pages);
    pdf.pages(pages).kids([page_id]).count(1);
    let mut p = pdf.page(page_id);
    p.media_box(Rect::new(0.0, 0.0, page.len(bounds.width()), page.len(bounds.height())));
    p.parent(pages);
    p.contents(content_id);
    p.resources().finish();
    p.finish();
    pdf.stream(content_id, &data).filter(Filter::FlateDecode);
    pdf.finish()
}

#[cfg(test)]
#[path = "tests_pdf.rs"]
mod tests_pdf;
