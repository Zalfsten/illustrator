//! Document: DocumentStructure + committed State \(p\).

use std::collections::HashMap;

use crate::constraint::{Constraint, EndpointRef};
use crate::geom::{
    Geometry, Point2, PointExpr, Segment, SegmentId, SegmentStore, SEGMENT_STRIDE,
};

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
    /// `SegmentId` → index into `geometries` (O(1) lookup during evaluate/solve).
    segment_index: HashMap<SegmentId, usize>,
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

    pub fn join_endpoints(&mut self, a: EndpointRef, b: EndpointRef) {
        self.constraints.push(Constraint::Congruence {
            a: a.as_expr(),
            b: b.as_expr(),
        });
    }

    /// Register a geometry whose parameters already live in an external State.
    pub fn push_geometry(&mut self, g: Geometry) {
        match &g {
            Geometry::Segment(s) => {
                debug_assert!(
                    !self.segment_index.contains_key(&s.id),
                    "duplicate segment id {:?}",
                    s.id
                );
                self.segment_index.insert(s.id, self.geometries.len());
                if s.id.0 >= self.next_id {
                    self.next_id = s.id.0 + 1;
                }
            }
        }
        self.geometries.push(g);
    }

    fn constraint_references_segment(c: &Constraint, id: SegmentId) -> bool {
        match c {
            Constraint::Congruence { a, b } => {
                point_expr_segment(a) == Some(id) || point_expr_segment(b) == Some(id)
            }
            Constraint::Parallel { a, b } | Constraint::Orthogonal { a, b } => *a == id || *b == id,
            Constraint::FixPoint { point, .. } => point_expr_segment(point) == Some(id),
        }
    }
}

impl SegmentStore for DocumentStructure {
    fn segment(&self, id: SegmentId) -> Option<&Segment> {
        let &slot = self.segment_index.get(&id)?;
        match self.geometries.get(slot)? {
            Geometry::Segment(s) => Some(s),
        }
    }
}

fn point_expr_segment(expr: &PointExpr) -> Option<SegmentId> {
    Some(expr.referenced_segment())
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

    /// Convenience: all line-segment geometries in document order.
    pub fn segments(&self) -> impl Iterator<Item = &Segment> {
        self.structure.geometries.iter().filter_map(|g| match g {
            Geometry::Segment(s) => Some(s),
        })
    }

    pub fn constraints(&self) -> &[Constraint] {
        self.structure.constraints()
    }

    pub fn add_segment(&mut self, a: Point2, b: Point2) -> SegmentId {
        let id = SegmentId(self.structure.next_id);
        self.structure.next_id += 1;
        let param_offset = self.state.len();
        self.state.0.extend_from_slice(&[a.x, a.y, b.x, b.y]);
        debug_assert_eq!(self.state.len() - param_offset, SEGMENT_STRIDE);
        self.structure
            .push_geometry(Geometry::Segment(Segment { id, param_offset }));
        id
    }

    pub fn add_constraint(&mut self, c: Constraint) {
        self.structure.add_constraint(c);
    }

    pub fn join_endpoints(&mut self, a: EndpointRef, b: EndpointRef) {
        self.structure.join_endpoints(a, b);
    }

    pub fn segment(&self, id: SegmentId) -> Option<&Segment> {
        self.structure.segment(id)
    }

    /// Remove a segment: compact State, remap offsets, drop orphan constraints.
    pub fn remove_segment(&mut self, id: SegmentId) -> bool {
        let Some(pos) = self.structure.segment_index.remove(&id) else {
            return false;
        };
        let removed = match self.structure.geometries.remove(pos) {
            Geometry::Segment(s) => s,
        };
        for slot in self.structure.segment_index.values_mut() {
            if *slot > pos {
                *slot -= 1;
            }
        }

        let start = removed.param_offset;
        let removed_len = SEGMENT_STRIDE;
        let end = start + removed_len;
        assert!(end <= self.state.len());
        self.state.0.drain(start..end);

        for g in &mut self.structure.geometries {
            g.reindex_after_removal(start, removed_len);
        }

        self.structure
            .constraints
            .retain(|c| !DocumentStructure::constraint_references_segment(c, id));

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

impl SegmentStore for Document {
    fn segment(&self, id: SegmentId) -> Option<&Segment> {
        self.structure.segment(id)
    }
}

#[cfg(test)]
mod tests;
