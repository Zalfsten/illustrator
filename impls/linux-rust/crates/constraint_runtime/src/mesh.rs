//! Display Mesh: tessellation of DocumentStructure + State into drawable edges.
//!
//! Derived view only — not authoritative for parameters or constraints.

use crate::document::DocumentStructure;
use crate::geom::{GeometryId, GeometryRole, Point2};
use crate::geometries::{GeometryBody, PointExpr, Segment};

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
    /// Start endpoint for segment edges.
    pub a: Point2,
    /// End endpoint for segment edges.
    pub b: Point2,
}

/// View-layer edge list for paint and Mesh-backed hit-testing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub edges: Vec<MeshEdge>,
}

impl Mesh {
    pub fn from_structure(
        structure: &DocumentStructure,
        p: &[f64],
        filter: MeshFilter,
    ) -> Self {
        let mut edges = Vec::with_capacity(structure.geometries().len());
        for g in structure.geometries() {
            if !filter.include_construction && g.role() == GeometryRole::Construction {
                continue;
            }
            match g.body {
                GeometryBody::Segment => {
                    let id = g.id();
                    edges.push(MeshEdge {
                        geometry: id,
                        role: g.role(),
                        a: Segment::start_expr(id).evaluate(structure, p),
                        b: Segment::end_expr(id).evaluate(structure, p),
                    });
                }
            }
        }
        Self { edges }
    }

    /// Nearest Mesh endpoint within `radius`, or `None`.
    /// For segment edges, `a` is start and `b` is end.
    pub fn hit_endpoint(&self, query: Point2, radius: f64) -> Option<PointExpr> {
        let mut best: Option<PointExpr> = None;
        let mut best_d = radius;
        for e in &self.edges {
            for (expr, pt) in [
                (Segment::start_expr(e.geometry), e.a),
                (Segment::end_expr(e.geometry), e.b),
            ] {
                let d = (pt.x - query.x).hypot(pt.y - query.y);
                if d < best_d {
                    best_d = d;
                    best = Some(expr);
                }
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
