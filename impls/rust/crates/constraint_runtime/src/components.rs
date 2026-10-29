//! Components of the constraint graph and their consistency.
//!
//! Two geometries share a component if a chain of constraints connects them;
//! a constraint belongs to the component of the geometries it references.
//! Like congruence classes, components depend on structure only. Whether a
//! component is consistent depends on the committed values: its structural
//! residual must pass the same gate as a commit ([`COMMIT_TOL`]).

use std::collections::HashSet;

use crate::constraint::ConstraintId;
use crate::document::Document;
use crate::geometry::GeometryId;
use crate::vector::Vector;
use crate::COMMIT_TOL;

/// Geometries and constraints of one component, each in id order. As a solve
/// scope it may also hold a union of components ([`Document::whole`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Component {
    pub geometries: Vec<GeometryId>,
    pub constraints: Vec<ConstraintId>,
}

impl Component {
    pub fn contains(&self, g: GeometryId) -> bool {
        self.geometries.contains(&g)
    }
}

impl Document {
    /// Component containing `g`.
    pub fn component_of(&self, g: GeometryId) -> Component {
        let mut geometries = vec![g];
        let mut constraints: Vec<ConstraintId> = Vec::new();
        let mut seen_g = HashSet::from([g]);
        let mut seen_c = HashSet::new();
        let mut next = 0;
        while next < geometries.len() {
            for &cid in self.constraints_on(geometries[next]) {
                if !seen_c.insert(cid) {
                    continue;
                }
                constraints.push(cid);
                let body = self.constraint(cid).expect("linked constraint exists");
                for h in body.referenced_geometries() {
                    if seen_g.insert(h) {
                        geometries.push(h);
                    }
                }
            }
            next += 1;
        }
        geometries.sort_by_key(|g| g.0);
        constraints.sort_by_key(|c| c.0);
        Component {
            geometries,
            constraints,
        }
    }

    /// Every component, ordered by its smallest geometry id.
    pub fn components(&self) -> Vec<Component> {
        let mut out: Vec<Component> = Vec::new();
        let mut assigned = HashSet::new();
        for g in self.geometries().values() {
            if assigned.contains(&g.id()) {
                continue;
            }
            let c = self.component_of(g.id());
            assigned.extend(c.geometries.iter().copied());
            out.push(c);
        }
        out
    }

    /// Every geometry and constraint: the union of all components.
    pub fn whole(&self) -> Component {
        Component {
            geometries: self.geometries().values().map(|g| g.id()).collect(),
            constraints: self.constraints().iter().map(|(id, _)| id).collect(),
        }
    }

    /// Norm of the structural residual of `c`'s constraints at `p`.
    pub fn component_residual(&self, c: &Component, p: &Vector) -> f64 {
        let mut sum = 0.0;
        let mut out = Vec::new();
        for &cid in &c.constraints {
            let body = self.constraint(cid).expect("component constraint exists");
            out.clear();
            out.resize(body.residual_arity(), 0.0);
            body.eval_residual(self, p, &mut out);
            sum += out.iter().map(|x| x * x).sum::<f64>();
        }
        sum.sqrt()
    }

    /// Whether `c` satisfies its constraints at the committed values.
    pub fn is_consistent(&self, c: &Component) -> bool {
        self.component_residual(c, self.vector()) <= COMMIT_TOL
    }
}

#[cfg(test)]
#[path = "tests_components.rs"]
mod tests_components;
