//! Picking and box selection over what the Canvas shows (ui-model: pick at a
//! world position, box selection). Pure functions of the [`Scene`]; positions
//! and tolerances are world units chosen by the front end.

use std::collections::HashSet;

use constraint_runtime::{Document, GeometryId, LogicalPoint, Mesh, Point2, PointExpr};

use crate::selection::Entry;

/// What the Canvas shows and can pick: the Mesh and logical points, both
/// built by the front end from the session's document under its
/// presentation filter. The document itself is passed separately, so a
/// session can be changed while a scene is held.
#[derive(Clone, Copy)]
pub struct Scene<'a> {
    pub mesh: &'a Mesh,
    pub points: &'a [LogicalPoint],
}

fn distance(a: Point2, b: Point2) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

/// Nearest logical point (its representative) strictly within `tolerance`.
pub fn nearest_point(doc: &Document, scene: &Scene, at: Point2, tolerance: f64) -> Option<PointExpr> {
    let mut best = None;
    let mut best_d = tolerance;
    for lp in scene.points {
        let d = distance(lp.representative.evaluate(doc, doc.vector()), at);
        if d < best_d {
            best_d = d;
            best = Some(lp.representative);
        }
    }
    best
}

/// Nearest geometric curve endpoint strictly within `tolerance`.
pub fn nearest_endpoint(scene: &Scene, at: Point2, tolerance: f64) -> Option<PointExpr> {
    scene.mesh.hit_endpoint(at, tolerance)
}

/// Nearest curve (by its Mesh edges) strictly within `tolerance`.
pub fn nearest_curve(scene: &Scene, at: Point2, tolerance: f64) -> Option<GeometryId> {
    scene.mesh.hit_geometry(at, tolerance)
}

/// Nearest curve with a direction (a line) strictly within `tolerance`;
/// nearer curves without one are skipped.
pub fn nearest_line(doc: &Document, scene: &Scene, at: Point2, tolerance: f64) -> Option<GeometryId> {
    let mut best = None;
    let mut best_d = tolerance;
    for e in &scene.mesh.edges {
        if !doc.caps(e.geometry).is_some_and(|c| c.line) {
            continue;
        }
        let d = distance_to_segment(at, e.a, e.b);
        if d < best_d {
            best_d = d;
            best = Some(e.geometry);
        }
    }
    best
}

fn distance_to_segment(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    distance(p, Point2::new(a.x + t * dx, a.y + t * dy))
}

/// What a pick at `at` selects: the nearest logical point, else the nearest curve.
pub fn pick(doc: &Document, scene: &Scene, at: Point2, tolerance: f64) -> Option<Entry> {
    nearest_point(doc, scene, at, tolerance)
        .map(Entry::Point)
        .or_else(|| nearest_curve(scene, at, tolerance).map(Entry::Geometry))
}

/// The Mesh edges grouped into one polyline per geometry (edges of a curve
/// are consecutive).
pub fn mesh_polylines(mesh: &Mesh) -> Vec<(GeometryId, Vec<Point2>)> {
    let mut out: Vec<(GeometryId, Vec<Point2>)> = Vec::new();
    for e in &mesh.edges {
        match out.last_mut() {
            Some((id, line)) if *id == e.geometry => line.push(e.b),
            _ => out.push((e.geometry, vec![e.a, e.b])),
        }
    }
    out
}

/// Geometries whose whole polyline lies inside the box with opposite corners
/// `a` and `b`, in Mesh order. Geometries without Mesh edges (labels) never.
/// One pass over the edges.
pub fn box_select(scene: &Scene, a: Point2, b: Point2) -> Vec<GeometryId> {
    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
    let inside = |p: &Point2| (x0..=x1).contains(&p.x) && (y0..=y1).contains(&p.y);
    let mut order: Vec<GeometryId> = Vec::new();
    let mut outside: HashSet<GeometryId> = HashSet::new();
    let mut seen: HashSet<GeometryId> = HashSet::new();
    for e in &scene.mesh.edges {
        if seen.insert(e.geometry) {
            order.push(e.geometry);
        }
        if !(inside(&e.a) && inside(&e.b)) {
            outside.insert(e.geometry);
        }
    }
    order.retain(|id| !outside.contains(id));
    order
}

#[cfg(test)]
#[path = "tests_pick.rs"]
mod tests_pick;
