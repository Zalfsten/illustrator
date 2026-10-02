//! Document: DocumentStructure + committed State \(p\).

use std::collections::HashMap;

use crate::constraint::Constraint;
use crate::geometries::{
    Geometry, GeometryId, GeometryKind, GeometryRole, GeometryShell, IdGenerator,
};
use crate::state::State;
use crate::store::GeometryStore;

/// Discrete sketch: geometries, structural constraints, connectivity schema.
#[derive(Debug, Default)]
pub struct DocumentStructure {
    geometries: Vec<Box<dyn Geometry>>,
    /// `GeometryId` → index into `geometries` (O(1) lookup during evaluate/solve).
    geometry_index: HashMap<GeometryId, usize>,
    constraints: Vec<Constraint>,
    ids: IdGenerator,
}

impl Clone for DocumentStructure {
    fn clone(&self) -> Self {
        Self {
            geometries: self.geometries.iter().map(|g| g.clone_box()).collect(),
            geometry_index: self.geometry_index.clone(),
            constraints: self.constraints.clone(),
            ids: self.ids,
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

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn parameter_count(&self) -> usize {
        self.geometries.iter().map(|g| g.parameter_count()).sum()
    }

    pub fn add_constraint(&mut self, c: Constraint) {
        self.constraints.push(c);
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

    fn constraint_references_geometry(c: &Constraint, id: GeometryId) -> bool {
        match c {
            Constraint::Congruence { a, b } => {
                a.referenced_geometry() == id || b.referenced_geometry() == id
            }
            Constraint::Parallel { a, b } | Constraint::Orthogonal { a, b } => {
                a.referenced_geometry() == id || b.referenced_geometry() == id
            }
            Constraint::FixPoint { point, .. } => point.referenced_geometry() == id,
            Constraint::EqualLength { a, b } => {
                a.referenced_geometry() == id || b.referenced_geometry() == id
            }
        }
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

    pub fn constraints(&self) -> &[Constraint] {
        self.structure.constraints()
    }

    /// Pack chart init → extend State → register a geometry instance.
    ///
    /// Chart pack receives only kind init (no identity). Id and State offset are
    /// assigned here via [`IdGenerator`] and the current State length.
    pub fn add_geometry<C: GeometryKind>(&mut self, role: GeometryRole, init: C::Init) -> GeometryId {
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
            .retain(|c| !DocumentStructure::constraint_references_geometry(c, id));

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
