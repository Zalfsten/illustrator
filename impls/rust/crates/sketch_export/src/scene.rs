//! Print primitives of a drawing: what an export draws, in world coordinates
//! (drawing-export). Format writers only map coordinates and serialize.

use std::collections::HashMap;

use constraint_runtime::{
    arrow_head, arrow_length_pt, arrow_tips, pt_to_world, Document, GeometryId, GeometryRole,
    Point2, Rgb, ShapePiece, Vec2,
};
use label_layout::{label_requests, place, Contour, LineMetrics, LineRequest, Paint, Segment, TypesetLine};

use crate::ExportError;

/// One command of a path in world coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathSeg {
    Move(Point2),
    Line(Point2),
    Quad(Point2, Point2),
    Cubic(Point2, Point2, Point2),
    /// Circular arc from the current point (which lies at heading `start`)
    /// turning by `sweep` radians around `center`.
    Arc {
        center: Point2,
        radius: f64,
        start: f64,
        sweep: f64,
    },
    Close,
}

impl PathSeg {
    /// The point this command ends at (`None` for [`PathSeg::Close`]).
    pub fn end(&self) -> Option<Point2> {
        match *self {
            PathSeg::Move(p) | PathSeg::Line(p) | PathSeg::Quad(_, p) | PathSeg::Cubic(_, _, p) => Some(p),
            PathSeg::Arc {
                center,
                radius,
                start,
                sweep,
            } => Some(on_circle(center, radius, start + sweep)),
            PathSeg::Close => None,
        }
    }
}

pub(crate) fn on_circle(center: Point2, radius: f64, heading: f64) -> Point2 {
    center + Vec2::new(radius * heading.cos(), radius * heading.sin())
}

/// A dash pattern (alternating dash and gap lengths, starting with a dash)
/// and the arc length of the pattern at the path's start, in world units.
#[derive(Clone, Debug, PartialEq)]
pub struct DashArray {
    pub pattern: Vec<f64>,
    pub phase: f64,
}

/// One drawn part of an object; lengths in world units.
#[derive(Clone, Debug, PartialEq)]
pub enum Part {
    /// An open curve stroked with flat ends.
    Stroke {
        path: Vec<PathSeg>,
        width: f64,
        color: Rgb,
        dash: Option<DashArray>,
    },
    /// Closed contours filled with the non-zero rule (arrow heads, glyphs).
    Fill { contours: Vec<Vec<PathSeg>>, color: Rgb },
    /// Contours stroked with flat ends (rules inside labels).
    Outline {
        contours: Vec<Vec<PathSeg>>,
        width: f64,
        color: Rgb,
    },
}

/// The drawn parts of one object, named by its object name.
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub id: GeometryId,
    pub name: String,
    pub parts: Vec<Part>,
}

/// Everything an export draws, objects in creation order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub objects: Vec<Object>,
}

/// Typeset lines of the exported labels, in line order.
pub type LabelLines = HashMap<GeometryId, Vec<TypesetLine>>;

/// The exported labels (result role) with the requests for their lines.
pub fn exported_labels(doc: &Document) -> Vec<(GeometryId, Vec<LineRequest>)> {
    doc.geometries()
        .values()
        .filter(|g| g.role() == GeometryRole::Result)
        .filter_map(|g| Some((g.id(), label_requests(doc.label_attrs(g.id())?, doc.settings()))))
        .collect()
}

/// The scene of `doc` at its committed State; label outlines from `labels`.
pub fn scene(doc: &Document, labels: &LabelLines) -> Result<Scene, ExportError> {
    let mut objects = Vec::new();
    for g in doc.geometries().values() {
        if g.role() != GeometryRole::Result {
            continue;
        }
        let parts = if doc.label_attrs(g.id()).is_some() {
            label_parts(doc, g.id(), labels)?
        } else if doc.caps(g.id()).is_some_and(|c| c.curve) {
            curve_parts(doc, g.id())
        } else {
            Vec::new()
        };
        if !parts.is_empty() {
            objects.push(Object {
                id: g.id(),
                name: doc.geometry_name(g.id()).unwrap_or_default().to_owned(),
                parts,
            });
        }
    }
    Ok(Scene { objects })
}

/// Stroke and arrow heads of curve `id` in its resolved style (styling).
fn curve_parts(doc: &Document, id: GeometryId) -> Vec<Part> {
    let p = doc.committed_vector();
    let settings = doc.settings();
    let (Some(shape), Some(style)) = (doc.curve_shape(id, p), doc.resolved_style(id)) else {
        return Vec::new();
    };
    let width = pt_to_world(style.width, settings);
    let head = pt_to_world(arrow_length_pt(style.width), settings);
    let from = if style.arrow.at_start() { head } else { 0.0 };
    let to_end = if style.arrow.at_end() { head } else { 0.0 };

    let mut parts = Vec::new();
    let pieces = trim(&shape, from, to_end);
    if !pieces.is_empty() {
        let pattern: Vec<f64> = style
            .dash
            .pattern(style.width)
            .into_iter()
            .map(|pt| pt_to_world(pt, settings))
            .collect();
        parts.push(Part::Stroke {
            path: path(&pieces),
            width,
            color: style.color,
            dash: (!pattern.is_empty()).then_some(DashArray { pattern, phase: from }),
        });
    }
    let (start, end) = arrow_tips(doc, p, id);
    for tip in [start.filter(|_| style.arrow.at_start()), end.filter(|_| style.arrow.at_end())]
        .into_iter()
        .flatten()
    {
        let [a, b, c] = arrow_head(tip.point, tip.out, style.width, settings);
        parts.push(Part::Fill {
            contours: vec![vec![PathSeg::Move(a), PathSeg::Line(b), PathSeg::Line(c), PathSeg::Close]],
            color: style.color,
        });
    }
    parts
}

/// The part of `pieces` without the first `from` and the last `to_end` of
/// its arc length; empty if nothing is left.
pub(crate) fn trim(pieces: &[ShapePiece], from: f64, to_end: f64) -> Vec<ShapePiece> {
    let total: f64 = pieces.iter().map(ShapePiece::length).sum();
    let to = total - to_end;
    let mut out = Vec::new();
    if to <= from {
        return out;
    }
    let mut s0 = 0.0;
    for piece in pieces {
        let len = piece.length();
        let s1 = s0 + len;
        if len > 0.0 && s1 > from && s0 < to {
            let a = ((from - s0) / len).max(0.0);
            let b = ((to - s0) / len).min(1.0);
            out.push(if a == 0.0 && b == 1.0 { *piece } else { piece.sub(a, b) });
        }
        s0 = s1;
    }
    out
}

/// One open path through consecutive pieces.
fn path(pieces: &[ShapePiece]) -> Vec<PathSeg> {
    let mut out = vec![PathSeg::Move(pieces[0].start_point())];
    for piece in pieces {
        out.push(match *piece {
            ShapePiece::Line { end, .. } => PathSeg::Line(end),
            ShapePiece::Arc {
                center,
                radius,
                start,
                sweep,
            } => PathSeg::Arc {
                center,
                radius,
                start: start.radians(),
                sweep: sweep.radians(),
            },
        });
    }
    out
}

/// Placed outlines of label `id` in its resolved colour (labels, styling).
fn label_parts(doc: &Document, id: GeometryId, labels: &LabelLines) -> Result<Vec<Part>, ExportError> {
    let attrs = doc.label_attrs(id).expect("caller checked the label");
    let settings = doc.settings();
    let expected = label_requests(attrs, settings).len();
    if expected == 0 {
        return Ok(Vec::new());
    }
    let lines = labels
        .get(&id)
        .filter(|l| l.len() == expected)
        .ok_or(ExportError::MissingLines(id))?;
    let p = doc.committed_vector();
    let origin = doc.origin_expr(id).expect("labels have an origin").evaluate(doc, p);
    let dir = doc.direction_expr(id).expect("labels have a direction").evaluate(doc, p);
    let color = doc.resolved_style(id).expect("geometry exists").color;
    let metrics: Vec<LineMetrics> = lines.iter().map(LineMetrics::from).collect();
    let maps = place(&metrics, attrs, settings, origin, dir.y.atan2(dir.x));

    let mut parts = Vec::new();
    for (line, map) in lines.iter().zip(maps) {
        for shape in &line.shapes {
            let contours: Vec<Vec<PathSeg>> = shape
                .contours
                .iter()
                .map(|c| contour(c, |q| map.apply(q)))
                .filter(|c| !c.is_empty())
                .collect();
            if contours.is_empty() {
                continue;
            }
            parts.push(match shape.paint {
                Paint::Fill => Part::Fill { contours, color },
                Paint::Stroke { width } => Part::Outline {
                    contours,
                    width: pt_to_world(width, settings),
                    color,
                },
            });
        }
    }
    Ok(parts)
}

fn contour(c: &Contour, map: impl Fn(Point2) -> Point2) -> Vec<PathSeg> {
    c.0.iter()
        .map(|s| match *s {
            Segment::MoveTo(p) => PathSeg::Move(map(p)),
            Segment::LineTo(p) => PathSeg::Line(map(p)),
            Segment::QuadTo(c, p) => PathSeg::Quad(map(c), map(p)),
            Segment::CubicTo(c1, c2, p) => PathSeg::Cubic(map(c1), map(c2), map(p)),
            Segment::Close => PathSeg::Close,
        })
        .collect()
}

#[cfg(test)]
#[path = "tests_scene.rs"]
mod tests_scene;
