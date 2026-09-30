//! Document: DocumentStructure + committed State \(p\).

use std::collections::HashMap;

use crate::constraint::Constraint;
use crate::geom::{Geometry, GeometryId, GeometryRole, Point2};
use crate::geometries::{Chart, GeometryBody, Segment};
use crate::state::State;
use crate::store::GeometryStore;

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
    /// Prefer [`Document::add_chart`] so Structure and State stay matched.
    pub(crate) fn push_geometry(&mut self, g: Geometry) {
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

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
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

    /// Append a chart geometry: pack init → extend State → register structure.
    pub fn add_chart<C: Chart>(&mut self, role: GeometryRole, init: C::Init) -> GeometryId {
        let packed = C::pack(init);
        debug_assert_eq!(
            packed.as_ref().len(),
            C::PARAMETER_COUNT,
            "Chart::pack length must equal PARAMETER_COUNT"
        );
        self.append_geometry(role, C::BODY, packed.as_ref())
    }

    /// Convenience: add a line segment from endpoints `(a, b)`.
    pub fn add_segment(&mut self, role: GeometryRole, a: Point2, b: Point2) -> GeometryId {
        self.add_chart::<Segment>(role, (a, b))
    }

    /// Low-level append used by [`Self::add_chart`]. Prefer `add_chart` / `add_segment`.
    pub(crate) fn append_geometry(
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
        self.state.extend_from_slice(params);
        self.structure.push_geometry(Geometry {
            id,
            role,
            param_offset,
            body,
        });
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
    fn geometry(&self, id: GeometryId) -> Option<&Geometry> {
        self.structure.geometry(id)
    }
}

#[cfg(test)]
#[path = "tests_document.rs"]
mod tests_document;
