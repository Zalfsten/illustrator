//! Display Mesh: tessellation of Document + Vector into drawable edges.
//!
//! Derived view only — not authoritative for parameters or constraints.
//! Curves are sampled via the curve capability's on-curve expressions;
//! subdivision stops when the unsigned turn between endpoint tangents of each
//! parameter interval is at most [`MAX_CHORD_TURN`]. Geometries without the
//! curve capability (labels) contribute nothing. Endpoint hit targets exist
//! only for kinds with the endpoints capability.

use crate::document::Document;
use crate::expr::{PointExpr, VecExpr};
use crate::geom::{CurveParam, Point2, Vec2};
use crate::geometry::{Caps, GeometryId, GeometryRole, PointFormula, VecFormula};
use crate::vector::Vector;

/// Max unsigned turn (radians) between tangents at the ends of one chord interval.
///
/// Larger curvature (faster tangent change) yields denser samples. Line segments
/// have zero turn and become a single edge.
pub const MAX_CHORD_TURN: f64 = std::f64::consts::PI / 8.0;

/// Hard cap on polyline vertices per geometry (including endpoints).
const MAX_MESH_SAMPLES: usize = 64;

/// Which geometries contribute edges to a Mesh.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeshFilter {
    pub include_construction: bool,
}

impl MeshFilter {
    pub const RESULT_ONLY: Self = Self {
        include_construction: false,
    };

    pub const INCLUDE_CONSTRUCTION: Self = Self {
        include_construction: true,
    };
}

impl Default for MeshFilter {
    fn default() -> Self {
        Self::RESULT_ONLY
    }
}

/// One stroked edge in the plane, tagged with the producing geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshEdge {
    pub geometry: GeometryId,
    pub role: GeometryRole,
    /// Edge start in world coordinates.
    pub a: Point2,
    /// Edge end in world coordinates.
    pub b: Point2,
}

/// View-layer edge list for paint and Mesh-backed hit-testing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub edges: Vec<MeshEdge>,
    /// Geometric curve endpoints (not every polyline vertex) for hit-testing.
    endpoints: Vec<(PointExpr, Point2)>,
}

impl Mesh {
    pub fn from_document(doc: &Document, p: &Vector, filter: MeshFilter) -> Self {
        let mut edges = Vec::new();
        let mut endpoints = Vec::new();
        for g in doc.geometries().values() {
            if !filter.include_construction && g.role() == GeometryRole::Construction {
                continue;
            }
            let caps = g.kind().caps();
            append_curve_mesh(caps, doc, p, g.id(), g.role(), &mut edges, &mut endpoints);
        }
        Self { edges, endpoints }
    }

    /// Geometric curve endpoint expressions (segment start/end; arc start/end).
    pub fn endpoint_exprs(&self) -> impl Iterator<Item = PointExpr> + '_ {
        self.endpoints.iter().map(|(e, _)| *e)
    }

    /// Nearest geometric curve endpoint within `radius`, or `None`.
    pub fn hit_endpoint(&self, query: Point2, radius: f64) -> Option<PointExpr> {
        let mut best: Option<PointExpr> = None;
        let mut best_d = radius;
        for (expr, pt) in &self.endpoints {
            let d = (pt.x - query.x).hypot(pt.y - query.y);
            if d < best_d {
                best_d = d;
                best = Some(*expr);
            }
        }
        best
    }

    /// Nearest Mesh edge (distance to finite segment) within `radius`, or `None`.
    pub fn hit_geometry(&self, query: Point2, radius: f64) -> Option<GeometryId> {
        let mut best: Option<GeometryId> = None;
        let mut best_d = radius;
        for e in &self.edges {
            let d = distance_point_to_segment(query, e.a, e.b);
            if d < best_d {
                best_d = d;
                best = Some(e.geometry);
            }
        }
        best
    }
}

/// Polyline for one curve; nothing without `caps.curve`, endpoint hit
/// targets only with `caps.endpoints`.
///
/// `caps` is the kind's capability set in production; tests pass a reduced set
/// to cover kinds without endpoints.
fn append_curve_mesh(
    caps: Caps,
    doc: &Document,
    p: &Vector,
    id: GeometryId,
    role: GeometryRole,
    edges: &mut Vec<MeshEdge>,
    endpoints: &mut Vec<(PointExpr, Point2)>,
) {
    if !caps.curve {
        return;
    }
    // Same expressions as `EndpointExprs`; `caps.endpoints` says the kind has them.
    let ends = caps.endpoints.then(|| {
        (
            PointExpr::new(id, PointFormula::Start),
            PointExpr::new(id, PointFormula::End),
        )
    });
    let (start_pt, end_pt) = match ends {
        Some((start, end)) => (start.evaluate(doc, p), end.evaluate(doc, p)),
        None => (
            on_curve(id, 0.0).evaluate(doc, p),
            on_curve(id, 1.0).evaluate(doc, p),
        ),
    };

    let ts = sample_normalized_params(doc, p, id);
    let mut prev = start_pt;
    for (i, &t) in ts.iter().enumerate().skip(1) {
        let pt = if i + 1 == ts.len() {
            end_pt
        } else {
            on_curve(id, t).evaluate(doc, p)
        };
        edges.push(MeshEdge {
            geometry: id,
            role,
            a: prev,
            b: pt,
        });
        prev = pt;
    }
    if let Some((start, end)) = ends {
        endpoints.push((start, start_pt));
        endpoints.push((end, end_pt));
    }
}

/// Same expressions as `CurveExprs`; callers check `caps.curve`.
fn on_curve(id: GeometryId, t: f64) -> PointExpr {
    PointExpr::new(id, PointFormula::OnCurve { param: CurveParam::Normalized(t) })
}

fn tangent(id: GeometryId, t: f64) -> VecExpr {
    VecExpr::new(id, VecFormula::TangentAt { param: CurveParam::Normalized(t) })
}

/// Normalized \(t\) samples in \([0,1]\), inclusive, ordered.
fn sample_normalized_params(doc: &Document, p: &Vector, id: GeometryId) -> Vec<f64> {
    let mut ts = vec![0.0, 1.0];
    refine_interval(doc, p, id, 0.0, 1.0, &mut ts);
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ts.dedup_by(|a, b| (*a - *b).abs() < 1e-15);
    ts
}

fn refine_interval(
    doc: &Document,
    p: &Vector,
    id: GeometryId,
    t0: f64,
    t1: f64,
    ts: &mut Vec<f64>,
) {
    if ts.len() >= MAX_MESH_SAMPLES {
        return;
    }
    let ta = tangent(id, t0).evaluate(doc, p);
    let tb = tangent(id, t1).evaluate(doc, p);
    if unsigned_turn(ta, tb) <= MAX_CHORD_TURN {
        return;
    }
    let mid = 0.5 * (t0 + t1);
    if (mid - t0).abs() < 1e-15 || (t1 - mid).abs() < 1e-15 {
        return;
    }
    ts.push(mid);
    refine_interval(doc, p, id, t0, mid, ts);
    refine_interval(doc, p, id, mid, t1, ts);
}

/// Unsigned angle between free vectors in \([0,\pi]\); zero if either is degenerate.
fn unsigned_turn(a: Vec2, b: Vec2) -> f64 {
    let la = a.length();
    let lb = b.length();
    if la < 1e-15 || lb < 1e-15 {
        return 0.0;
    }
    let c = (a.dot(b) / (la * lb)).clamp(-1.0, 1.0);
    c.acos()
}

fn distance_point_to_segment(q: Point2, a: Point2, b: Point2) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = abx * abx + aby * aby;
    if len2 == 0.0 {
        return (q.x - a.x).hypot(q.y - a.y);
    }
    let t = ((q.x - a.x) * abx + (q.y - a.y) * aby) / len2;
    let t = t.clamp(0.0, 1.0);
    let cx = a.x + t * abx;
    let cy = a.y + t * aby;
    (q.x - cx).hypot(q.y - cy)
}

#[cfg(test)]
#[path = "tests_mesh.rs"]
mod tests_mesh;
