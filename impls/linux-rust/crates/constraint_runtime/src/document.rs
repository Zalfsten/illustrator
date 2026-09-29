//! Document: DocumentStructure + committed State \(p\).

use std::collections::HashMap;

use crate::constraint::Constraint;
use crate::geom::{Geometry, GeometryId, GeometryRole};
use crate::geometries::GeometryBody;
use crate::store::GeometryStore;

/// Contiguous continuous parameters for a sketch.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State(pub Vec<f64>);

impl State {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn as_slice(&self) -> &[f64] {
        &self.0
    }

    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn to_vec(&self) -> Vec<f64> {
        self.0.clone()
    }

    pub fn copy_from_slice(&mut self, p: &[f64]) {
        assert_eq!(p.len(), self.0.len());
        self.0.copy_from_slice(p);
    }
}

impl std::ops::Deref for State {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Discrete sketch: geometries, structural constraints, connectivity schema.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentStructure {
    geometries: Vec<Geometry>,
    /// `GeometryId` → index into `geometries` (O(1) lookup during evaluate/solve).
    geometry_index: HashMap<GeometryId, usize>,
    constraints: Vec<Constraint>,
    next_id: usize,
}

impl DocumentStructure {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn geometries(&self) -> &[Geometry] {
        &self.geometries
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn parameter_count(&self) -> usize {
        self.geometries.iter().map(Geometry::parameter_count).sum()
    }

    pub fn add_constraint(&mut self, c: Constraint) {
        self.constraints.push(c);
    }

    /// Register a geometry whose parameters already live in an external State.
    pub fn push_geometry(&mut self, g: Geometry) {
        let id = g.id();
        debug_assert!(
            !self.geometry_index.contains_key(&id),
            "duplicate geometry id {id:?}"
        );
        self.geometry_index.insert(id, self.geometries.len());
        if id.0 >= self.next_id {
            self.next_id = id.0 + 1;
        }
        self.geometries.push(g);
    }

    fn constraint_references_geometry(c: &Constraint, id: GeometryId) -> bool {
        match c {
            Constraint::Congruence { a, b } => {
                a.referenced_geometry() == id || b.referenced_geometry() == id
            }
            Constraint::Parallel { a, b } | Constraint::Orthogonal { a, b } => *a == id || *b == id,
            Constraint::FixPoint { point, .. } => point.referenced_geometry() == id,
        }
    }
}

impl GeometryStore for DocumentStructure {
    fn geometry(&self, id: GeometryId) -> Option<&Geometry> {
        let &slot = self.geometry_index.get(&id)?;
        self.geometries.get(slot)
    }
}

/// Document = DocumentStructure + committed State.
#[derive(Clone, Debug, Default, PartialEq)]
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

    pub fn state(&self) -> &[f64] {
        self.state.as_slice()
    }

    pub fn state_mut(&mut self) -> &mut [f64] {
        self.state.as_mut_slice()
    }

    pub fn committed_state(&self) -> &State {
        &self.state
    }

    pub fn geometries(&self) -> &[Geometry] {
        self.structure.geometries()
    }

    pub fn geometry(&self, id: GeometryId) -> Option<&Geometry> {
        self.structure.geometry(id)
    }

    pub fn constraints(&self) -> &[Constraint] {
        self.structure.constraints()
    }

    /// Append a geometry: extends State by `params` and registers structure.
    ///
    /// `params.len()` MUST equal `body.parameter_count()`.
    pub fn add_geometry(
        &mut self,
        role: GeometryRole,
        body: GeometryBody,
        params: &[f64],
    ) -> GeometryId {
        assert_eq!(
            params.len(),
            body.parameter_count(),
            "params length must match geometry parameter count"
        );
        let id = GeometryId(self.structure.next_id);
        self.structure.next_id += 1;
        let param_offset = self.state.len();
        self.state.0.extend_from_slice(params);
        self.structure.push_geometry(Geometry {
            id,
            role,
            param_offset,
            body,
        });
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

    pub fn add_constraint(&mut self, c: Constraint) {
        self.structure.add_constraint(c);
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

        let (start, removed_len) = (removed.param_offset, removed.body.parameter_count());
        let end = start + removed_len;
        assert!(end <= self.state.len());
        self.state.0.drain(start..end);

        for g in &mut self.structure.geometries {
            g.reindex_after_removal(start, removed_len);
        }

        self.structure
            .constraints
            .retain(|c| !DocumentStructure::constraint_references_geometry(c, id));

        debug_assert_eq!(self.state.len(), self.structure.parameter_count());
        true
    }

    /// Snapshot of committed state for undo / drag.
    pub fn clone_state(&self) -> Vec<f64> {
        self.state.to_vec()
    }

    pub fn restore_state(&mut self, p: &[f64]) {
        self.state.copy_from_slice(p);
    }

    pub fn commit_state(&mut self, p: &[f64]) {
        self.restore_state(p);
    }
}

impl GeometryStore for Document {
    fn geometry(&self, id: GeometryId) -> Option<&Geometry> {
        self.structure.geometry(id)
    }
}

#[cfg(test)]
#[path = "tests_document.rs"]
mod tests_document;
