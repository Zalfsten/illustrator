//! Document: DocumentStructure + committed State \(p\).

use std::collections::HashMap;

use crate::constraint::{Constraint, ConstraintEntry, ConstraintId, ConstraintKind};
use crate::constraint_kinds::Congruence;
use crate::geometry::{
    Geometry, GeometryId, GeometryKind, GeometryRole, GeometryShell, PointExpr,
};
use crate::id::IdGenerator;
use crate::state::State;
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

/// Discrete sketch: geometries, structural constraints, connectivity schema.
#[derive(Debug, Default)]
pub struct DocumentStructure {
    geometries: Vec<Box<dyn Geometry>>,
    /// `GeometryId` → index into `geometries` (O(1) lookup during evaluate/solve).
    geometry_index: HashMap<GeometryId, usize>,
    constraints: Vec<ConstraintEntry>,
    /// `ConstraintId` → index into `constraints`.
    constraint_index: HashMap<ConstraintId, usize>,
    /// `GeometryId` → constraints that reference it (incidence).
    constraints_on_geometry: HashMap<GeometryId, Vec<ConstraintId>>,
    /// `GeometryId` → Congruence edges touching that geometry (PointExpr partners).
    identification_by_geometry: HashMap<GeometryId, Vec<IdentificationEdge>>,
    ids: IdGenerator<GeometryId>,
    constraint_ids: IdGenerator<ConstraintId>,
}

impl Clone for DocumentStructure {
    fn clone(&self) -> Self {
        Self {
            geometries: self.geometries.iter().map(|g| g.clone_box()).collect(),
            geometry_index: self.geometry_index.clone(),
            constraints: self.constraints.clone(),
            constraint_index: self.constraint_index.clone(),
            constraints_on_geometry: self.constraints_on_geometry.clone(),
            identification_by_geometry: self.identification_by_geometry.clone(),
            ids: self.ids,
            constraint_ids: self.constraint_ids,
        }
    }
}

impl DocumentStructure {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn geometries(&self) -> &[Box<dyn Geometry>] {
        &self.geometries
    }

    pub fn constraints(&self) -> &[ConstraintEntry] {
        &self.constraints
    }

    pub fn constraint(&self, id: ConstraintId) -> Option<&dyn Constraint> {
        let &slot = self.constraint_index.get(&id)?;
        self.constraints.get(slot).map(|e| e.body())
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
        self.geometries.iter().map(|g| g.parameter_count()).sum()
    }

    pub fn add_constraint(&mut self, c: impl ConstraintKind) -> ConstraintId {
        let geos = c.referenced_geometries();
        let id = self.constraint_ids.allocate();
        debug_assert!(
            !self.constraint_index.contains_key(&id),
            "duplicate constraint id {id:?}"
        );
        self.constraint_index.insert(id, self.constraints.len());
        self.constraints.push(ConstraintEntry {
            id,
            body: Box::new(c),
        });
        self.link_constraint(id, &geos);
        if let Some(g) = self.constraints.last().unwrap().body().as_any().downcast_ref::<Congruence>()
        {
            self.link_identification(id, g.a, g.b);
        }
        id
    }

    /// Remove a constraint by id. Returns false if missing.
    pub fn remove_constraint(&mut self, id: ConstraintId) -> bool {
        let Some(pos) = self.constraint_index.remove(&id) else {
            return false;
        };
        let geos = self.constraints[pos].body().referenced_geometries();
        let congruence = self.constraints[pos]
            .body()
            .as_any()
            .downcast_ref::<Congruence>()
            .map(|g| (g.a, g.b));
        self.constraints.remove(pos);
        for slot in self.constraint_index.values_mut() {
            if *slot > pos {
                *slot -= 1;
            }
        }
        self.unlink_constraint(id, &geos);
        if let Some((a, b)) = congruence {
            self.unlink_identification(id, a, b);
        }
        true
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

    fn rebuild_constraint_index(&mut self) {
        self.constraint_index.clear();
        for (i, e) in self.constraints.iter().enumerate() {
            self.constraint_index.insert(e.id, i);
        }
    }

    fn rebuild_constraints_on_geometry(&mut self) {
        self.constraints_on_geometry.clear();
        self.identification_by_geometry.clear();
        for e in &self.constraints {
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

    /// Register a geometry whose parameters already live in an external State.
    /// Prefer [`Document::add_geometry`] so Structure and State stay matched.
    pub(crate) fn push_geometry(&mut self, g: impl Geometry + 'static) {
        let id = g.id();
        debug_assert!(
            !self.geometry_index.contains_key(&id),
            "duplicate geometry id {id:?}"
        );
        self.geometry_index.insert(id, self.geometries.len());
        self.ids.observe(id);
        self.geometries.push(Box::new(g));
    }
}

impl GeometryStore for DocumentStructure {
    fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry> {
        let &slot = self.geometry_index.get(&id)?;
        self.geometries.get(slot).map(|g| g.as_ref())
    }
}

/// Document = DocumentStructure + committed State.
#[derive(Clone, Debug, Default)]
pub struct Document {
    pub structure: DocumentStructure,
    pub state: State,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn structure(&self) -> &DocumentStructure {
        &self.structure
    }

    pub fn structure_mut(&mut self) -> &mut DocumentStructure {
        &mut self.structure
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn committed_state(&self) -> &State {
        &self.state
    }

    pub fn geometries(&self) -> &[Box<dyn Geometry>] {
        self.structure.geometries()
    }

    pub fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry> {
        self.structure.geometry(id)
    }

    pub fn constraints(&self) -> &[ConstraintEntry] {
        self.structure.constraints()
    }

    pub fn constraint(&self, id: ConstraintId) -> Option<&dyn Constraint> {
        self.structure.constraint(id)
    }

    /// Constraint ids that reference geometry `id`.
    pub fn constraints_on(&self, id: GeometryId) -> &[ConstraintId] {
        self.structure.constraints_on(id)
    }

    /// Direct Congruence partners of `point`.
    pub fn points_identified_with(&self, point: &PointExpr) -> Vec<PointExpr> {
        self.structure.points_identified_with(point)
    }

    /// Pack chart init → extend State → register a geometry instance.
    ///
    /// Chart pack receives only kind init (no identity). Id and State offset are
    /// assigned here via [`IdGenerator`] and the current State length.
    pub fn add_geometry<C: GeometryKind>(
        &mut self,
        role: GeometryRole,
        init: C::Init,
    ) -> GeometryId {
        let packed = C::pack(init);
        debug_assert_eq!(packed.as_ref().len(), C::PARAMETER_COUNT);
        let id = self.structure.ids.allocate();
        let param_offset = self.state.len();
        self.state.extend_from_slice(packed.as_ref());
        self.structure
            .push_geometry(C::from(GeometryShell::new(id, role, param_offset)));
        debug_assert_eq!(self.state.len(), self.structure.parameter_count());
        id
    }

    /// Set geometry role (result vs construction). Returns false if id missing.
    pub fn set_geometry_role(&mut self, id: GeometryId, role: GeometryRole) -> bool {
        let Some(&slot) = self.structure.geometry_index.get(&id) else {
            return false;
        };
        self.structure.geometries[slot].set_role(role);
        true
    }

    pub fn add_constraint(&mut self, c: impl ConstraintKind) -> ConstraintId {
        self.structure.add_constraint(c)
    }

    pub fn remove_constraint(&mut self, id: ConstraintId) -> bool {
        self.structure.remove_constraint(id)
    }

    /// Remove a geometry: compact State, remap offsets, drop orphan constraints.
    pub fn remove_geometry(&mut self, id: GeometryId) -> bool {
        let Some(pos) = self.structure.geometry_index.remove(&id) else {
            return false;
        };
        let removed = self.structure.geometries.remove(pos);
        for slot in self.structure.geometry_index.values_mut() {
            if *slot > pos {
                *slot -= 1;
            }
        }

        let block = removed.param_block();
        let (start, removed_len) = (block.start, block.len());
        let end = start + removed_len;
        assert!(end <= self.state.len());
        self.state.drain(start..end);

        for g in &mut self.structure.geometries {
            g.reindex_after_removal(start, removed_len);
        }

        self.structure
            .constraints
            .retain(|e| !e.body().references_geometry(id));
        self.structure.rebuild_constraint_index();
        self.structure.rebuild_constraints_on_geometry();

        debug_assert_eq!(self.state.len(), self.structure.parameter_count());
        true
    }

    /// Snapshot of committed state for undo / drag.
    pub fn clone_state(&self) -> State {
        self.state.clone()
    }

    pub fn restore_state(&mut self, p: &State) {
        self.state.copy_from(p);
    }

    pub fn commit_state(&mut self, p: &State) {
        self.restore_state(p);
    }
}

impl GeometryStore for Document {
    fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry> {
        self.structure.geometry(id)
    }
}

#[cfg(test)]
#[path = "tests_document.rs"]
mod tests_document;
