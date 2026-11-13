//! Display Mesh: tessellation of Document + Vector into drawable edges.
//!
//! Derived view only — not authoritative for parameters or constraints.
//! Curves are sampled via the curve capability's on-curve expressions at
//! normalized parameters chosen from piece bounds (length and curvature
//! upper bounds per parameter piece, [`Document::curve_piece_bound`]):
//! each edge provably stays within the chord tolerance passed to
//! [`Mesh::from_document`], up to [`MAX_MESH_EDGES`] per curve. Uniform kinds
//! (segment, arc) get equal edges, as few as the tolerance allows; other
//! curves are placed by demand and refined locally where an edge fails.
//! Geometries without the curve capability (labels) contribute nothing.
//! Endpoint hit targets exist only for kinds with the endpoints capability.

use std::f64::consts::FRAC_PI_2;

use crate::document::Document;
use crate::expr::PointExpr;
use crate::geom::{CurveParam, Point2};
use crate::geometry::{Caps, Geometry, GeometryId, GeometryRole, PieceBound, PointFormula};
use crate::geometry_kinds::with_kind;
use crate::vector::Vector;

/// Cap on polyline edges per curve; beyond it the chord tolerance is not
/// guaranteed (display-mesh: at least 256).
pub const MAX_MESH_EDGES: usize = 1024;

/// Equal cells of the normalized parameter whose edge demand places the
/// initial vertices of a non-uniform curve.
const GRID_INTERVALS: usize = 64;

/// Sub-pieces whose bounds check one edge.
const CHECK_PIECES: usize = 4;

/// Edges narrower than this (in the normalized parameter) are not halved.
const MIN_EDGE_WIDTH: f64 = 1e-12;

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

    let piece = |t0, t1| {
        doc.curve_piece_bound(id, p, t0, t1)
            .expect("curve capability implies a piece bound")
    };
    let uniform = doc.geometry(id).is_some_and(|g| with_kind!(g.kind(), K => K::UNIFORM_CURVE));
    let ts = if uniform {
        let whole = piece(0.0, 1.0);
        uniform_samples(whole.length, whole.curvature, tolerance, MAX_MESH_EDGES)
    } else {
        place_samples(piece, tolerance, MAX_MESH_EDGES)
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

/// Normalized parameters \(0 = t_0 < \dots < t_n = 1\) of a curve's polyline
/// vertices, from its piece bounds (`piece(t0, t1)`: length and curvature
/// upper bounds of that piece).
///
/// 1. **Placement:** the edge demand ([`edge_demand`]) of [`GRID_INTERVALS`]
///    equal cells is integrated; the smallest edge count not below it is
///    placed at equal shares of the cumulative demand.
/// 2. **Check:** each edge passes when [`edge_within_tolerance`] proves it
///    stays within `tolerance`.
/// 3. **Local refinement:** a failing edge is halved in `t` and both halves
///    are checked again, until they pass, become narrower than
///    [`MIN_EDGE_WIDTH`], or the curve has `cap` edges (then the remaining
///    edges are kept as they are).
///
/// The demand uses the small-angle sagitta form, which is exactly what the
/// check accepts for constant curvature, so such curves need no halving.
fn place_samples(piece: impl Fn(f64, f64) -> PieceBound, tolerance: f64, cap: usize) -> Vec<f64> {
    let cap = cap.max(1);
    let mut cumulative = Vec::with_capacity(GRID_INTERVALS + 1);
    cumulative.push(0.0);
    for i in 0..GRID_INTERVALS {
        let t0 = i as f64 / GRID_INTERVALS as f64;
        let t1 = (i + 1) as f64 / GRID_INTERVALS as f64;
        let last = *cumulative.last().expect("non-empty");
        cumulative.push(last + edge_demand(piece(t0, t1), tolerance));
    }
    let total = *cumulative.last().expect("non-empty");
    if !total.is_finite() {
        // An unbounded cell (κ̄ = ∞ on a piece of positive length) leaves no
        // provable placement: cap edges at equal steps, unchecked, which the
        // spec allows at the cap.
        return (0..=cap).map(|j| j as f64 / cap as f64).collect();
    }
    let n = edge_count(total, cap);
    let initial = equal_shares(&cumulative, n);
    let mut budget = cap - n;
    let mut ts = Vec::with_capacity(n + 1);
    ts.push(0.0);
    for w in initial.windows(2) {
        refine(&piece, w[0], w[1], tolerance, &mut budget, &mut ts);
    }
    ts
}

/// Pushes the vertices after `ta` up to and including `tb`, halving the edge
/// while it fails the check and `budget` (edges left below the cap) allows.
fn refine(
    piece: &impl Fn(f64, f64) -> PieceBound,
    ta: f64,
    tb: f64,
    tolerance: f64,
    budget: &mut usize,
    ts: &mut Vec<f64>,
) {
    if *budget == 0 || tb - ta < MIN_EDGE_WIDTH || edge_within_tolerance(piece, ta, tb, tolerance) {
        ts.push(tb);
        return;
    }
    *budget -= 1;
    let mid = 0.5 * (ta + tb);
    refine(piece, ta, mid, tolerance, budget, ts);
    refine(piece, mid, tb, tolerance, budget, ts);
}

/// Edges a piece needs: its length times
/// \(\max(\sqrt{\bar\kappa / 8\varepsilon},\ \bar\kappa / (\pi/2))\), the
/// small-angle sagitta bound and the quarter-turn limit of
/// [`edge_within_tolerance`]; 0 for a straight or zero-length piece.
fn edge_demand(b: PieceBound, tolerance: f64) -> f64 {
    if b.length == 0.0 || b.curvature == 0.0 {
        return 0.0;
    }
    b.length * (b.curvature / (8.0 * tolerance)).sqrt().max(b.curvature / FRAC_PI_2)
}

/// Whether the edge `[ta, tb]` provably stays within `tolerance`, from the
/// bounds of [`CHECK_PIECES`] equal sub-pieces:
/// - total turning \(\sum \bar\kappa_i \ell_i \le \pi/2\), so every tangent is
///   within a quarter turn of the chord and every point projects inside the
///   edge;
/// - the Green-function bound [`deviation_bound`] on the distance from the
///   chord line is at most `tolerance`.
fn edge_within_tolerance(
    piece: &impl Fn(f64, f64) -> PieceBound,
    ta: f64,
    tb: f64,
    tolerance: f64,
) -> bool {
    let mut pieces = [PieceBound {
        length: 0.0,
        curvature: 0.0,
    }; CHECK_PIECES];
    let mut turning = 0.0;
    for (i, slot) in pieces.iter_mut().enumerate() {
        let t0 = ta + (tb - ta) * i as f64 / CHECK_PIECES as f64;
        let t1 = ta + (tb - ta) * (i + 1) as f64 / CHECK_PIECES as f64;
        *slot = piece(t0, t1);
        if slot.length > 0.0 && slot.curvature > 0.0 {
            turning += slot.length * slot.curvature;
        }
    }
    if turning.is_nan() || turning > FRAC_PI_2 {
        return false;
    }
    deviation_bound(&pieces) <= tolerance * (1.0 + 1e-9)
}

/// Upper bound of the distance of a curve from its chord line, for
/// consecutive pieces with length upper bounds \(\ell_i\) and curvature
/// bounds \(\bar\kappa_i\).
///
/// The distance \(d(s)\) at arc length \(s\) has \(d(0) = d(L) = 0\) and
/// \(|d''| \le \kappa\), so \(|d(s)| \le \int_0^L G(s, u)\,\kappa(u)\,du\)
/// with the Green function \(G(s,u) = u(L-s)/L\) for \(u \le s\) and
/// \(s(L-u)/L\) for \(u \ge s\). With \(\kappa \le \bar\kappa_i\) on each
/// piece the integral is concave in \(s\) and quadratic on each piece, so its
/// maximum is exact from the ends and the vertex of every piece. Constant
/// curvature gives \(\kappa L^2/8\); curvature only near an end of the chord
/// contributes little. The bound does not decrease when a piece length is
/// over-estimated (tested), so length upper bounds keep it sound.
fn deviation_bound(pieces: &[PieceBound]) -> f64 {
    let total: f64 = pieces.iter().map(|b| b.length).sum();
    if total <= 0.0 {
        return 0.0;
    }
    let at = |s: f64| deviation_at(pieces, total, s);
    let mut best = 0.0_f64;
    let mut lo = 0.0;
    for b in pieces {
        let hi = lo + b.length;
        if hi > lo {
            let (h, m) = (0.5 * (hi - lo), 0.5 * (lo + hi));
            let (fa, fm, fb) = (at(lo), at(m), at(hi));
            best = best.max(fa).max(fm).max(fb);
            // Vertex of the parabola through (−h, fa), (0, fm), (h, fb).
            let curvature = fa - 2.0 * fm + fb;
            if curvature < 0.0 {
                let x = h * (fa - fb) / (2.0 * curvature);
                if x.abs() < h {
                    best = best.max(at(m + x));
                }
            }
        }
        lo = hi;
    }
    best
}

/// \(\sum_i \bar\kappa_i \int_{\text{piece } i} G(s, u)\,du\) at arc length `s`
/// (see [`deviation_bound`]), `total` the sum of the piece lengths; closed
/// form per piece.
fn deviation_at(pieces: &[PieceBound], total: f64, s: f64) -> f64 {
    let mut u = 0.0;
    let mut sum = 0.0;
    for b in pieces {
        let (lo, hi) = (u, u + b.length);
        u = hi;
        if b.curvature == 0.0 || b.length == 0.0 {
            continue;
        }
        let mut integral = 0.0;
        // u ≤ s: G = u (L − s) / L
        let m1 = hi.min(s);
        if m1 > lo {
            integral += (total - s) / total * 0.5 * (m1 * m1 - lo * lo);
        }
        // u ≥ s: G = s (L − u) / L
        let m2 = lo.max(s);
        if hi > m2 {
            integral += s / total * (total * (hi - m2) - 0.5 * (hi * hi - m2 * m2));
        }
        sum += b.curvature * integral;
    }
    sum
}

/// Equal steps \(t_j = j/n\) for a curve whose speed and curvature do not
/// depend on `t` ([`Geometry::UNIFORM_CURVE`]), from its total `length` and
/// constant `curvature`, with the exact osculating-circle count
/// ([`edge_density`]): equal edges, as few as the tolerance allows.
///
/// [`Geometry::UNIFORM_CURVE`]: crate::geometry::Geometry::UNIFORM_CURVE
fn uniform_samples(length: f64, curvature: f64, tolerance: f64, cap: usize) -> Vec<f64> {
    let cap = cap.max(1);
    let total = edge_density(length, curvature, tolerance);
    let n = if total.is_finite() { edge_count(total, cap) } else { cap };
    (0..=n).map(|j| j as f64 / n as f64).collect()
}

/// Smallest edge count not below the demand integral `total`, in `1..=cap`.
fn edge_count(total: f64, cap: usize) -> usize {
    // The offset keeps an exact integer count (e.g. 4.0000000001) from
    // rounding up to one edge too many.
    ((total - 1e-9).ceil().max(1.0) as usize).min(cap)
}

/// `n` edges splitting the cumulative demand of the equal grid cells into
/// equal shares (linear interpolation inside a cell).
fn equal_shares(cumulative: &[f64], n: usize) -> Vec<f64> {
    let total = *cumulative.last().expect("non-empty");
    let cells = (cumulative.len() - 1) as f64;
    let mut ts = Vec::with_capacity(n + 1);
    ts.push(0.0);
    let mut k = 0;
    for j in 1..n {
        let target = total * j as f64 / n as f64;
        while k + 2 < cumulative.len() && cumulative[k + 1] < target {
            k += 1;
        }
        let (c0, c1) = (cumulative[k], cumulative[k + 1]);
        let s = if c1 > c0 { ((target - c0) / (c1 - c0)).clamp(0.0, 1.0) } else { 0.0 };
        ts.push((k as f64 + s) / cells);
    }
    ts.push(1.0);
    ts
}

/// Edges per unit of normalized parameter for a uniform curve:
/// \(|\gamma'|\,\kappa/\varphi_{\max}(\kappa)\).
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
/// \(\varepsilon\kappa = 1\)), so a near-full circle never becomes one edge.
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
