//! Display Mesh: tessellation of Document + Vector into drawable edges.
//!
//! Derived view only — not authoritative for parameters or constraints.
//! Curves are sampled via the curve capability's on-curve expressions at
//! normalized parameters chosen from the curve's speed (tangent expression)
//! and curvature ([`Document::curve_curvature`]): each edge stays within the
//! chord tolerance passed to [`Mesh::from_document`], with as few edges as
//! that allows, up to [`MAX_MESH_EDGES`] per curve. Geometries without the
//! curve capability (labels) contribute nothing. Endpoint hit targets exist
//! only for kinds with the endpoints capability.

use crate::document::Document;
use crate::expr::{PointExpr, VecExpr};
use crate::geom::{CurveParam, Point2};
use crate::geometry::{Caps, Geometry, GeometryId, GeometryRole, PointFormula, VecFormula};
use crate::geometry_kinds::with_kind;
use crate::vector::Vector;

/// Cap on polyline edges per curve; beyond it the chord tolerance is not
/// guaranteed (display-mesh: at least 256).
pub const MAX_MESH_EDGES: usize = 1024;

/// Equal intervals of the normalized parameter on which the edge density is
/// sampled before refinement.
const GRID_INTERVALS: usize = 64;

/// Max bisection depth of one grid interval.
const MAX_REFINE_DEPTH: u32 = 6;

/// An interval is bisected when the densities at its ends differ by more
/// than this factor …
const REFINE_RATIO: f64 = 2.0;

/// … and it contributes at least this many edges.
const REFINE_MIN_EDGES: f64 = 0.25;

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
    /// Mesh of `doc` at State `p`: geometries passing `filter`, curves within
    /// chord `tolerance` (world units, finite and > 0).
    pub fn from_document(doc: &Document, p: &Vector, filter: MeshFilter, tolerance: f64) -> Self {
        assert!(
            tolerance.is_finite() && tolerance > 0.0,
            "chord tolerance must be finite and > 0, got {tolerance}"
        );
        let mut mesh = Self::default();
        for g in doc.geometries().values() {
            if !filter.include_construction && g.role() == GeometryRole::Construction {
                continue;
            }
            let caps = g.kind().caps();
            append_curve_mesh(caps, doc, p, tolerance, g.id(), g.role(), &mut mesh);
        }
        mesh
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
    tolerance: f64,
    id: GeometryId,
    role: GeometryRole,
    mesh: &mut Mesh,
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

    let speed = |t| tangent(id, t).evaluate(doc, p).length();
    let curvature = |t| doc.curve_curvature(id, p, t).unwrap_or(0.0);
    let uniform = doc.geometry(id).is_some_and(|g| with_kind!(g.kind(), K => K::UNIFORM_CURVE));
    let ts = if uniform {
        uniform_samples(speed(0.5), curvature(0.5), tolerance, MAX_MESH_EDGES)
    } else {
        place_samples(speed, curvature, tolerance, MAX_MESH_EDGES)
    };
    let mut prev = start_pt;
    for (i, &t) in ts.iter().enumerate().skip(1) {
        let pt = if i + 1 == ts.len() {
            end_pt
        } else {
            on_curve(id, t).evaluate(doc, p)
        };
        mesh.edges.push(MeshEdge {
            geometry: id,
            role,
            a: prev,
            b: pt,
        });
        prev = pt;
    }
    if let Some((start, end)) = ends {
        mesh.endpoints.push((start, start_pt));
        mesh.endpoints.push((end, end_pt));
    }
}

/// Same expressions as `CurveExprs`; callers check `caps.curve`.
fn on_curve(id: GeometryId, t: f64) -> PointExpr {
    PointExpr::new(id, PointFormula::OnCurve { param: CurveParam::Normalized(t) })
}

fn tangent(id: GeometryId, t: f64) -> VecExpr {
    VecExpr::new(id, VecFormula::TangentAt { param: CurveParam::Normalized(t) })
}

/// Speed, curvature and edge density of a curve at normalized `t`.
#[derive(Clone, Copy, Debug)]
struct Node {
    t: f64,
    speed: f64,
    curvature: f64,
    density: f64,
}

/// Normalized parameters \(0 = t_0 < \dots < t_n = 1\) of a curve's polyline
/// vertices, from its speed \(|\gamma'(t)|\) and unsigned curvature
/// \(\kappa(t)\).
///
/// Edges per unit of `t` follow the osculating circle (see [`edge_density`]);
/// the density is integrated (trapezoid rule) on a grid of [`GRID_INTERVALS`]
/// intervals, refined where it varies strongly. The edge count starts at the
/// smallest integer not below the integral, and the vertices split the
/// integral into equal shares, so constant density gives \(t_j = j/n\).
/// Where curvature varies along an edge, equal shares can exceed the
/// tolerance (by about 15 % next to an inflection), so the count grows by
/// single edges until every edge passes [`edges_within_tolerance`]; for
/// constant curvature the first count already passes. The count is clamped
/// to `1..=cap`.
///
/// Limitation: growing the count for the whole curve and the conservative
/// check \(L\,\kappa_{\max}\) make curves with straight or nearly straight
/// stretches next to curved parts far too dense (a line followed by a
/// circular arc reaches `cap`; a curvature ramp from 0 gets about 17× the
/// density integral). Only [`uniform_samples`] kinds (segment, arc) exist
/// today; function curves (#81) need per-edge refinement first (#86).
fn place_samples(
    speed: impl Fn(f64) -> f64,
    curvature: impl Fn(f64) -> f64,
    tolerance: f64,
    cap: usize,
) -> Vec<f64> {
    let node = |t: f64| {
        let (speed, curvature) = (speed(t), curvature(t));
        Node {
            t,
            speed,
            curvature,
            density: edge_density(speed, curvature, tolerance),
        }
    };

    let mut nodes = vec![node(0.0)];
    for i in 1..=GRID_INTERVALS {
        let b = node(i as f64 / GRID_INTERVALS as f64);
        let a = *nodes.last().expect("grid starts with t = 0");
        refine_grid(&node, a, b, MAX_REFINE_DEPTH, &mut nodes);
    }

    let mut cumulative = Vec::with_capacity(nodes.len());
    cumulative.push(0.0);
    for w in nodes.windows(2) {
        let last = *cumulative.last().expect("non-empty");
        cumulative.push(last + 0.5 * (w[1].t - w[0].t) * (w[0].density + w[1].density));
    }
    let total = *cumulative.last().expect("non-empty");

    let cap = cap.max(1);
    if !total.is_finite() {
        return (0..=cap).map(|j| j as f64 / cap as f64).collect();
    }
    let mut n = edge_count(total, cap);
    loop {
        let ts = equal_shares(&nodes, &cumulative, n);
        if n == cap || edges_within_tolerance(&nodes, &ts, tolerance) {
            return ts;
        }
        n += 1;
    }
}

/// Equal steps \(t_j = j/n\) for a curve whose speed and curvature do not
/// depend on `t` ([`Geometry::UNIFORM_CURVE`]): the result of
/// [`place_samples`] for constant density, from one evaluation instead of a
/// grid.
///
/// [`Geometry::UNIFORM_CURVE`]: crate::geometry::Geometry::UNIFORM_CURVE
fn uniform_samples(speed: f64, curvature: f64, tolerance: f64, cap: usize) -> Vec<f64> {
    let cap = cap.max(1);
    let total = edge_density(speed, curvature, tolerance);
    let n = if total.is_finite() { edge_count(total, cap) } else { cap };
    (0..=n).map(|j| j as f64 / n as f64).collect()
}

/// Smallest edge count not below the density integral `total`, in `1..=cap`.
fn edge_count(total: f64, cap: usize) -> usize {
    // The offset keeps an exact integer count (e.g. 4.0000000001) from
    // rounding up to one edge too many.
    ((total - 1e-9).ceil().max(1.0) as usize).min(cap)
}

/// Pushes the nodes after `a` up to and including `b`, bisecting while the
/// densities at the ends differ by more than [`REFINE_RATIO`] and the interval
/// contributes at least [`REFINE_MIN_EDGES`].
fn refine_grid(node: &impl Fn(f64) -> Node, a: Node, b: Node, depth: u32, nodes: &mut Vec<Node>) {
    let contribution = 0.5 * (b.t - a.t) * (a.density + b.density);
    let (lo, hi) = (a.density.min(b.density), a.density.max(b.density));
    if depth > 0 && contribution >= REFINE_MIN_EDGES && hi > REFINE_RATIO * lo {
        let mid = node(0.5 * (a.t + b.t));
        refine_grid(node, a, mid, depth - 1, nodes);
        refine_grid(node, mid, b, depth - 1, nodes);
    } else {
        nodes.push(b);
    }
}

/// `n` edges splitting the density integral into equal shares (linear
/// interpolation of the cumulative count inside each grid interval).
fn equal_shares(nodes: &[Node], cumulative: &[f64], n: usize) -> Vec<f64> {
    let total = *cumulative.last().expect("non-empty");
    let mut ts = Vec::with_capacity(n + 1);
    ts.push(0.0);
    let mut k = 0;
    for j in 1..n {
        let target = total * j as f64 / n as f64;
        while k + 2 < cumulative.len() && cumulative[k + 1] < target {
            k += 1;
        }
        let (c0, c1) = (cumulative[k], cumulative[k + 1]);
        let (t0, t1) = (nodes[k].t, nodes[k + 1].t);
        let s = if c1 > c0 { ((target - c0) / (c1 - c0)).clamp(0.0, 1.0) } else { 0.0 };
        ts.push(t0 + s * (t1 - t0));
    }
    ts.push(1.0);
    ts
}

/// Whether every edge `[t_j, t_{j+1}]` turns at most \(\varphi_{\max}(\kappa_{\max})\)
/// for its largest curvature: a curve of length \(L\) with curvature at most
/// \(\kappa_{\max}\) and \(L\,\kappa_{\max} \le \pi\) deviates from its chord
/// no more than a circular arc of that length and curvature, so
/// \(L\,\kappa_{\max} \le \varphi_{\max}\) keeps the edge within `tolerance`
/// ([`phi_max`] is at most \(\pi\) for that reason). Speed and curvature
/// between grid nodes are interpolated linearly, so a curvature spike narrower
/// than a grid interval can go unseen.
fn edges_within_tolerance(nodes: &[Node], ts: &[f64], tolerance: f64) -> bool {
    let at = |k: usize, t: f64| {
        let (a, b) = (nodes[k], nodes[k + 1]);
        let s = if b.t > a.t { ((t - a.t) / (b.t - a.t)).clamp(0.0, 1.0) } else { 0.0 };
        (a.speed + s * (b.speed - a.speed), a.curvature + s * (b.curvature - a.curvature))
    };
    let mut k = 0;
    for w in ts.windows(2) {
        let (ta, tb) = (w[0], w[1]);
        while k + 2 < nodes.len() && nodes[k + 1].t <= ta {
            k += 1;
        }
        let (mut speed, mut kappa_max) = at(k, ta);
        let (mut t, mut length) = (ta, 0.0);
        let mut i = k + 1;
        while i + 1 < nodes.len() && nodes[i].t < tb {
            length += 0.5 * (nodes[i].t - t) * (speed + nodes[i].speed);
            (t, speed) = (nodes[i].t, nodes[i].speed);
            kappa_max = kappa_max.max(nodes[i].curvature);
            i += 1;
        }
        let (end_speed, end_kappa) = at(i - 1, tb);
        length += 0.5 * (tb - t) * (speed + end_speed);
        kappa_max = kappa_max.max(end_kappa);
        if length > 0.0 && kappa_max > 0.0 {
            let phi = phi_max(kappa_max, tolerance);
            if length * kappa_max > phi * (1.0 + 1e-9) {
                return false;
            }
        }
    }
    true
}

/// Edges per unit of normalized parameter: \(|\gamma'|\,\kappa/\varphi_{\max}(\kappa)\).
///
/// Zero where the curve is straight or has zero speed (a zero-radius arc:
/// speed 0, \(\kappa = \infty\)).
fn edge_density(speed: f64, curvature: f64, tolerance: f64) -> f64 {
    if speed == 0.0 || curvature == 0.0 {
        return 0.0;
    }
    speed * curvature / phi_max(curvature, tolerance)
}

/// Largest turn of a chord on a circle of curvature \(\kappa\) whose sagitta
/// stays within `tolerance`: \(2\arccos(1-\varepsilon\kappa) =
/// 4\arcsin\sqrt{\varepsilon\kappa/2}\); the arcsine form stays accurate when
/// \(\varepsilon\kappa\) is tiny. Capped at \(\pi\) (reached at
/// \(\varepsilon\kappa = 1\)): beyond a half turn the comparison bound of
/// [`edges_within_tolerance`] no longer holds.
fn phi_max(curvature: f64, tolerance: f64) -> f64 {
    4.0 * (0.5 * tolerance * curvature).sqrt().min(std::f64::consts::FRAC_1_SQRT_2).asin()
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
