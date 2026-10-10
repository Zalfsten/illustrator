//! Display Mesh: tessellation of Document + Vector into drawable edges.
//!
//! Derived view only — not authoritative for parameters or constraints.

use crate::document::Document;
use crate::expr::PointExpr;
use crate::geom::{CurveParam, Point2};
use crate::geometry::{CurveExprs, EndpointExprs, GeometryId, GeometryRole};
use crate::geometry_kinds::{Arc, GeometryInst, Segment};
use crate::vector::Vector;

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
            let id = g.id();
            let role = g.role();
            match g {
                GeometryInst::Segment(_) => {
                    let a = Segment::start_expr(id).evaluate(doc, p);
                    let b = Segment::end_expr(id).evaluate(doc, p);
                    edges.push(MeshEdge {
                        geometry: id,
                        role,
                        a,
                        b,
                    });
                    endpoints.push((Segment::start_expr(id), a));
                    endpoints.push((Segment::end_expr(id), b));
                }
                GeometryInst::Arc(_) => {
                    let start_expr = Arc::start_expr(id);
                    let end_expr = Arc::end_expr(id);
                    let start_pt = start_expr.evaluate(doc, p);
                    let end_pt = end_expr.evaluate(doc, p);
                    let chart = crate::mapper::Mapper::unpack_chart(
                        <Arc as crate::geometry_kinds::InGeometryCatalog>::as_catalog_ref(g),
                        p,
                    );
                    let n = Arc::mesh_sample_count(chart.3);
                    let mut prev = start_pt;
                    for i in 1..n {
                        let t = i as f64 / (n - 1) as f64;
                        let pt = if i + 1 == n {
                            end_pt
                        } else {
                            Arc::on_curve_expr(id, CurveParam::Normalized(t)).evaluate(doc, p)
                        };
                        edges.push(MeshEdge {
                            geometry: id,
                            role,
                            a: prev,
                            b: pt,
                        });
                        prev = pt;
                    }
                    endpoints.push((start_expr, start_pt));
                    endpoints.push((end_expr, end_pt));
                }
            }
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
