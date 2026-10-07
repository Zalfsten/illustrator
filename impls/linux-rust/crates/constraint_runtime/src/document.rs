//! Document: discrete sketch graph + committed Vector \(p\).
//!
//! Solve/assemble take `&Document` as geometry resolution plus an explicit
//! Vector (committed or a working copy). Structure and Vector stay separable
//! at call sites even though both live on Document.

use std::collections::HashMap;

use crate::constraint::{Constraint, ConstraintEntry, ConstraintId, ConstraintKind};
use crate::constraint_kinds::Congruence;
use crate::geometry::{
    Geometry, GeometryId, GeometryKind, GeometryRole, GeometryShell, PointExpr,
};
use crate::id::IdGenerator;
use crate::vector::Vector;
use crate::store::GeometryStore;

/// One Congruence edge, indexed under each participating geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
struct IdentificationEdge {
    constraint: ConstraintId,
    a: PointExpr,
    b: PointExpr,
}

impl IdentificationEdge {
    fn partner_of(self, point: &PointExpr) -> Option<PointExpr> {
        if &self.a == point {
            Some(self.b)
        } else if &self.b == point {
            Some(self.a)
        } else {
            None
        }
    }
}

/// Document = discrete sketch (geometries, constraints, incidence) + committed Vector.
#[derive(Clone, Debug, Default)]
pub struct Document {
    /// Geometry store (id → instance).
    geometries: HashMap<GeometryId, Box<dyn Geometry>>,
    /// Constraint store (id → entry).
    constraints: HashMap<ConstraintId, ConstraintEntry>,
    /// `GeometryId` → constraints that reference it (incidence).
    constraints_on_geometry: HashMap<GeometryId, Vec<ConstraintId>>,
    /// `GeometryId` → Congruence edges touching that geometry (PointExpr partners).
    identification_by_geometry: HashMap<GeometryId, Vec<IdentificationEdge>>,
    ids: IdGenerator<GeometryId>,
    constraint_ids: IdGenerator<ConstraintId>,
    /// Committed continuous parameters \(p\).
    pub vector: Vector,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vector(&self) -> &Vector {
        &self.vector
    }

    pub fn vector_mut(&mut self) -> &mut Vector {
        &mut self.vector
    }

    pub fn committed_vector(&self) -> &Vector {
        &self.vector
    }

    pub fn geometries(&self) -> &HashMap<GeometryId, Box<dyn Geometry>> {
        &self.geometries
    }

    pub fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry> {
        self.geometries.get(&id).map(|g| g.as_ref())
    }

    pub fn constraints(&self) -> &HashMap<ConstraintId, ConstraintEntry> {
        &self.constraints
    }

    pub fn constraint(&self, id: ConstraintId) -> Option<&dyn Constraint> {
        self.constraints.get(&id).map(|e| e.body())
    }

    /// Constraint ids that reference geometry `id` (empty if none / unknown id).
    pub fn constraints_on(&self, id: GeometryId) -> &[ConstraintId] {
        self.constraints_on_geometry
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Direct Congruence partners of `point` (O(edges on its geometry)).
    pub fn points_identified_with(&self, point: &PointExpr) -> Vec<PointExpr> {
        self.identification_by_geometry
            .get(&point.referenced_geometry())
            .into_iter()
            .flatten()
            .filter_map(|e| e.partner_of(point))
            .collect()
    }

    pub fn parameter_count(&self) -> usize {
        self.geometries.values().map(|g| g.parameter_count()).sum()
    }

    /// Pack chart init → extend Vector → register a geometry instance.
    ///
    /// Chart pack receives only kind init (no identity). Id and Vector offset are
    /// assigned here via [`IdGenerator`] and the current Vector length.
    pub fn add_geometry<C: GeometryKind>(
        &mut self,
        role: GeometryRole,
        init: C::Init,
    ) -> GeometryId {
        let packed = C::pack(init);
        debug_assert_eq!(packed.as_ref().len(), C::PARAMETER_COUNT);
        let id = self.ids.allocate();
        let param_offset = self.vector.len();
        self.vector.extend_from_slice(packed.as_ref());
        self.push_geometry(C::from(GeometryShell::new(id, role, param_offset)));
        debug_assert_eq!(self.vector.len(), self.parameter_count());
        id
    }

    /// Register a geometry whose parameters already live in an external Vector.
    /// Prefer [`Document::add_geometry`] so graph and committed Vector stay matched.
    pub(crate) fn push_geometry(&mut self, g: impl Geometry + 'static) {
        let id = g.id();
        debug_assert!(
            !self.geometries.contains_key(&id),
            "duplicate geometry id {id:?}"
        );
        self.ids.observe(id);
        self.geometries.insert(id, Box::new(g));
    }

    /// Set geometry role (result vs construction). Returns false if id missing.
    pub fn set_geometry_role(&mut self, id: GeometryId, role: GeometryRole) -> bool {
        let Some(g) = self.geometries.get_mut(&id) else {
            return false;
        };
        g.set_role(role);
        true
    }

    pub fn add_constraint(&mut self, c: impl ConstraintKind) -> ConstraintId {
        let geos = c.referenced_geometries();
        let id = self.constraint_ids.allocate();
        debug_assert!(
            !self.constraints.contains_key(&id),
            "duplicate constraint id {id:?}"
        );
        self.constraints.insert(
            id,
            ConstraintEntry {
                id,
                body: Box::new(c),
            },
        );
        self.link_constraint(id, &geos);
        if let Some(g) = self
            .constraints
            .get(&id)
            .unwrap()
            .body()
            .as_any()
            .downcast_ref::<Congruence>()
        {
            self.link_identification(id, g.a, g.b);
        }
        id
    }

    /// Remove a constraint by id. Returns false if missing.
    pub fn remove_constraint(&mut self, id: ConstraintId) -> bool {
        let Some(entry) = self.constraints.remove(&id) else {
            return false;
        };
        let geos = entry.body().referenced_geometries();
        let congruence = entry
            .body()
            .as_any()
            .downcast_ref::<Congruence>()
            .map(|g| (g.a, g.b));
        self.unlink_constraint(id, &geos);
        if let Some((a, b)) = congruence {
            self.unlink_identification(id, a, b);
        }
        true
    }

    /// Remove a geometry: compact Vector, remap offsets, drop orphan constraints.
    pub fn remove_geometry(&mut self, id: GeometryId) -> bool {
        let Some(removed) = self.geometries.remove(&id) else {
            return false;
        };

        let block = removed.param_block();
        let (start, removed_len) = (block.start, block.len());
        let end = start + removed_len;
        assert!(end <= self.vector.len());
        self.vector.drain(start..end);

        for g in self.geometries.values_mut() {
            g.reindex_after_removal(start, removed_len);
        }

        self.constraints
            .retain(|_, e| !e.body().references_geometry(id));
        self.rebuild_incidence();

        debug_assert_eq!(self.vector.len(), self.parameter_count());
        true
    }

    /// Snapshot of committed state for undo / drag.
    pub fn clone_vector(&self) -> Vector {
        self.vector.clone()
    }

    pub fn restore_vector(&mut self, p: &Vector) {
        self.vector.copy_from(p);
    }

    pub fn commit_vector(&mut self, p: &Vector) {
        self.restore_vector(p);
    }

    fn link_constraint(&mut self, cid: ConstraintId, geos: &[GeometryId]) {
        for &gid in geos {
            self.constraints_on_geometry
                .entry(gid)
                .or_default()
                .push(cid);
        }
    }

    fn unlink_constraint(&mut self, cid: ConstraintId, geos: &[GeometryId]) {
        for &gid in geos {
            let Some(list) = self.constraints_on_geometry.get_mut(&gid) else {
                continue;
            };
            list.retain(|&x| x != cid);
            if list.is_empty() {
                self.constraints_on_geometry.remove(&gid);
            }
        }
    }

    fn link_identification(&mut self, cid: ConstraintId, a: PointExpr, b: PointExpr) {
        let edge = IdentificationEdge {
            constraint: cid,
            a,
            b,
        };
        let ga = a.referenced_geometry();
        let gb = b.referenced_geometry();
        self.identification_by_geometry
            .entry(ga)
            .or_default()
            .push(edge);
        if gb != ga {
            self.identification_by_geometry
                .entry(gb)
                .or_default()
                .push(edge);
        }
    }

    fn unlink_identification(&mut self, cid: ConstraintId, a: PointExpr, b: PointExpr) {
        for gid in crate::constraint::unique_geometry_ids([
            a.referenced_geometry(),
            b.referenced_geometry(),
        ]) {
            let Some(list) = self.identification_by_geometry.get_mut(&gid) else {
                continue;
            };
            list.retain(|e| e.constraint != cid);
            if list.is_empty() {
                self.identification_by_geometry.remove(&gid);
            }
        }
    }

    fn rebuild_incidence(&mut self) {
        self.constraints_on_geometry.clear();
        self.identification_by_geometry.clear();
        for e in self.constraints.values() {
            for gid in e.body().referenced_geometries() {
                self.constraints_on_geometry
                    .entry(gid)
                    .or_default()
                    .push(e.id);
            }
            if let Some(g) = e.body().as_any().downcast_ref::<Congruence>() {
                let edge = IdentificationEdge {
                    constraint: e.id,
                    a: g.a,
                    b: g.b,
                };
                let ga = g.a.referenced_geometry();
                let gb = g.b.referenced_geometry();
                self.identification_by_geometry
                    .entry(ga)
                    .or_default()
                    .push(edge);
                if gb != ga {
                    self.identification_by_geometry
                        .entry(gb)
                        .or_default()
                        .push(edge);
                }
            }
        }
    }
}

impl GeometryStore for Document {
    fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry> {
        self.geometries.get(&id).map(|g| g.as_ref())
    }
}

#[cfg(test)]
#[path = "tests_document.rs"]
mod tests_document;
