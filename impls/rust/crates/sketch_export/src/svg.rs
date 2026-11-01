//! A scene as one standalone SVG document in millimetres (drawing-export:
//! SVG form). Fixed layout and number format, so equal scenes give equal
//! bytes.

use std::f64::consts::PI;
use std::fmt::Write;

use constraint_runtime::{DocumentSettings, Point2, Rgb};

use crate::bounds::Bounds;
use crate::scene::{on_circle, DashArray, Part, PathSeg, Scene};

/// Page coordinates in mm: x right and y down from the top left of `bounds`.
struct Page {
    min_x: f64,
    max_y: f64,
    /// Millimetres per world unit.
    scale: f64,
}

impl Page {
    fn len(&self, v: f64) -> String {
        num(v * self.scale)
    }

    fn point(&self, p: Point2) -> String {
        format!("{} {}", num((p.x - self.min_x) * self.scale), num((self.max_y - p.y) * self.scale))
    }
}

/// `v` with 3 decimals, without trailing zeros, and `-0` as `0`.
fn num(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_owned()
    } else {
        s.to_owned()
    }
}

fn color(c: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}

/// Path data of `path`; arcs are split into parts of at most π.
fn data(page: &Page, path: &[PathSeg]) -> String {
    let mut out = Vec::new();
    for seg in path {
        match *seg {
            PathSeg::Move(p) => out.push(format!("M {}", page.point(p))),
            PathSeg::Line(p) => out.push(format!("L {}", page.point(p))),
            PathSeg::Quad(c, p) => out.push(format!("Q {} {}", page.point(c), page.point(p))),
            PathSeg::Cubic(c1, c2, p) => {
                out.push(format!("C {} {} {}", page.point(c1), page.point(c2), page.point(p)))
            }
            PathSeg::Arc {
                center,
                radius,
                start,
                sweep,
            } => {
                let n = (sweep.abs() / PI).ceil().max(1.0);
                let step = sweep / n;
                // The page's y axis points down: a counterclockwise world sweep
                // is a negative-angle sweep there.
                let flag = if sweep > 0.0 { 0 } else { 1 };
                let r = page.len(radius);
                for k in 1..=n as usize {
                    let end = on_circle(center, radius, start + step * k as f64);
                    out.push(format!("A {r} {r} 0 0 {flag} {}", page.point(end)));
                }
            }
            PathSeg::Close => out.push("Z".to_owned()),
        }
    }
    out.join(" ")
}

fn contours(page: &Page, contours: &[Vec<PathSeg>]) -> String {
    contours.iter().map(|c| data(page, c)).collect::<Vec<_>>().join(" ")
}

fn dash_attrs(page: &Page, dash: Option<&DashArray>) -> String {
    match dash {
        None => String::new(),
        Some(d) => format!(
            r#" stroke-dasharray="{}" stroke-dashoffset="{}""#,
            d.pattern.iter().map(|&v| page.len(v)).collect::<Vec<_>>().join(" "),
            page.len(d.phase)
        ),
    }
}

/// `scene` as SVG text, its view box exactly `bounds`, at the document scale.
pub fn svg(scene: &Scene, bounds: &Bounds, settings: &DocumentSettings) -> String {
    let page = Page {
        min_x: bounds.min.x,
        max_y: bounds.max.y,
        scale: settings.scale,
    };
    let (w, h) = (page.len(bounds.width()), page.len(bounds.height()));
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{w}mm" height="{h}mm" viewBox="0 0 {w} {h}">"#
    );
    for object in &scene.objects {
        if object.name.is_empty() {
            out.push_str("<g>\n");
        } else {
            let _ = writeln!(out, r#"<g id="{}">"#, object.name);
        }
        for part in &object.parts {
            let _ = match part {
                Part::Stroke {
                    path,
                    width,
                    color: c,
                    dash,
                } => writeln!(
                    out,
                    r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="butt"{}/>"#,
                    data(&page, path),
                    color(*c),
                    page.len(*width),
                    dash_attrs(&page, dash.as_ref())
                ),
                Part::Fill { contours: cs, color: c } => writeln!(
                    out,
                    r#"<path d="{}" fill="{}" fill-rule="nonzero" stroke="none"/>"#,
                    contours(&page, cs),
                    color(*c)
                ),
                Part::Outline {
                    contours: cs,
                    width,
                    color: c,
                } => writeln!(
                    out,
                    r#"<path d="{}" fill="none" stroke="{}" stroke-width="{}" stroke-linecap="butt" stroke-linejoin="miter"/>"#,
                    contours(&page, cs),
                    color(*c),
                    page.len(*width)
                ),
            };
        }
        out.push_str("</g>\n");
    }
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
#[path = "tests_svg.rs"]
mod tests_svg;
