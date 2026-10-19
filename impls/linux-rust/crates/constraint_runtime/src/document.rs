//! Document: discrete sketch graph + committed Vector \(p\).
//!
//! Solve/assemble take `&Document` as geometry resolution plus an explicit
//! Vector (committed or a working copy). Structure and Vector stay separable
//! at call sites even though both live on Document.

use std::collections::HashMap;

use crate::constraint::{ConstraintEntry, ConstraintId};
use crate::constraint_kinds::ConstraintInst;
use crate::expr::PointExpr;
use crate::geometry::{Geometry, GeometryId, GeometryRole, GeometryShell};
use crate::geometry_kinds::GeometryInst;
use crate::id::IdMap;
use crate::vector::Vector;

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
    /// Geometry store (id → catalog instance; array lookup on the solve path).
    geometries: IdMap<GeometryId, GeometryInst>,
    /// Constraint store (id → entry; id order fixes assembly row order).
    constraints: IdMap<ConstraintId, ConstraintEntry>,
    /// `GeometryId` → constraints that reference it (incidence).
    constraints_on_geometry: HashMap<GeometryId, Vec<ConstraintId>>,
    /// `GeometryId` → Congruence edges touching that geometry (PointExpr partners).
    identification_by_geometry: HashMap<GeometryId, Vec<IdentificationEdge>>,
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

    pub fn geometries(&self) -> &IdMap<GeometryId, GeometryInst> {
        &self.geometries
    }

    pub fn geometry(&self, id: GeometryId) -> Option<&GeometryInst> {
        self.geometries.get(id)
    }

    /// Geometry `id`, panicking if absent (expressions only reference live ids).
    #[inline]
    pub fn expect_geometry(&self, id: GeometryId) -> &GeometryInst {
        match self.geometries.get(id) {
            Some(g) => g,
            None => missing_geometry(id),
        }
    }

    pub fn constraints(&self) -> &IdMap<ConstraintId, ConstraintEntry> {
        &self.constraints
    }

    pub fn constraint(&self, id: ConstraintId) -> Option<&ConstraintInst> {
        self.constraints.get(id).map(|e| e.body())
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
    /// assigned here from [`IdMap::next_id`] and the current Vector length.
    pub fn add_geometry<K: Geometry>(&mut self, role: GeometryRole, init: K::Init) -> GeometryId {
        let packed = K::pack(init);
        debug_assert_eq!(packed.as_ref().len(), K::PARAMETER_COUNT);
        let id = self.geometries.next_id();
        let param_offset = self.vector.len();
        self.vector.extend_from_slice(packed.as_ref());
        self.push_geometry(GeometryInst::of::<K>(GeometryShell::new(id, role, param_offset)));
        debug_assert_eq!(self.vector.len(), self.parameter_count());
        id
    }

    /// Register a geometry whose parameters already live in an external Vector.
    /// Prefer [`Document::add_geometry`] so graph and committed Vector stay matched.
    pub(crate) fn push_geometry(&mut self, g: GeometryInst) {
        self.geometries.insert(g.id(), g);
    }

    /// Set geometry role (result vs construction). Returns false if id missing.
    pub fn set_geometry_role(&mut self, id: GeometryId, role: GeometryRole) -> bool {
        let Some(g) = self.geometries.get_mut(id) else {
            return false;
        };
        g.set_role(role);
        true
    }

    pub fn add_constraint(&mut self, c: impl Into<ConstraintInst>) -> ConstraintId {
        let body = c.into();
        let geos = body.referenced_geometries();
        let id = self.constraints.next_id();
        if let Some(g) = body.as_congruence() {
            self.link_identification(id, g.a, g.b);
        }
        self.constraints.insert(id, ConstraintEntry { id, body });
        self.link_constraint(id, &geos);
        id
    }

    /// Remove a constraint by id. Returns false if missing.
    pub fn remove_constraint(&mut self, id: ConstraintId) -> bool {
        let Some(entry) = self.constraints.remove(id) else {
            return false;
        };
        let geos = entry.body().referenced_geometries();
        let congruence = entry.body().as_congruence().map(|g| (g.a, g.b));
        self.unlink_constraint(id, &geos);
        if let Some((a, b)) = congruence {
            self.unlink_identification(id, a, b);
        }
        true
    }

    /// Remove a geometry: compact Vector, remap offsets, drop orphan constraints.
    pub fn remove_geometry(&mut self, id: GeometryId) -> bool {
        let Some(removed) = self.geometries.remove(id) else {
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
            .retain(|e| !e.body().references_geometry(id));
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
            if let Some(g) = e.body().as_congruence() {
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

#[cold]
#[track_caller]
fn missing_geometry(id: GeometryId) -> ! {
    panic!("geometry id {id:?} not in document")
}

#[cfg(test)]
#[path = "tests_document.rs"]
mod tests_document;
