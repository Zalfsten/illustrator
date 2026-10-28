//! Congruence classes of point expressions and the logical (grabbable) points.
//!
//! A class is the transitive closure of Congruence: everything reachable from a
//! point through a chain of joins. Classes depend on structure only, never on
//! evaluated locations, so coincident but unjoined points stay apart.

use crate::document::Document;
use crate::expr::PointExpr;
use crate::geometry::GeometryRole;
use crate::mesh::MeshFilter;

/// One grabbable point: a congruence class with a visible curve endpoint.
#[derive(Clone, Debug, PartialEq)]
pub struct LogicalPoint {
    /// First visible endpoint of the class (geometry id order, start before end).
    pub representative: PointExpr,
    /// All expressions of the class, representative included.
    pub members: Vec<PointExpr>,
}

impl Document {
    /// `p` and every expression reachable from it through Congruence.
    pub fn point_class(&self, p: &PointExpr) -> Vec<PointExpr> {
        let mut class = vec![*p];
        let mut next = 0;
        while next < class.len() {
            for q in self.points_identified_with(&class[next]) {
                if !class.contains(&q) {
                    class.push(q);
                }
            }
            next += 1;
        }
        class
    }

    /// One entry per class that has a curve endpoint visible under `filter`.
    pub fn logical_points(&self, filter: MeshFilter) -> Vec<LogicalPoint> {
        let mut points: Vec<LogicalPoint> = Vec::new();
        for g in self.geometries().values() {
            if !filter.include_construction && g.role() == GeometryRole::Construction {
                continue;
            }
            let ends = [self.start_expr(g.id()), self.end_expr(g.id())];
            for end in ends.into_iter().flatten() {
                if points.iter().any(|lp| lp.members.contains(&end)) {
                    continue;
                }
                points.push(LogicalPoint {
                    representative: end,
                    members: self.point_class(&end),
                });
            }
        }
        points
    }
}

#[cfg(test)]
#[path = "tests_point_classes.rs"]
mod tests_point_classes;
