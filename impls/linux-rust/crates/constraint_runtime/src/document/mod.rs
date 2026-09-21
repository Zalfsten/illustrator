//! Document: DocumentStructure + committed State \(p\).

use crate::constraint::{Constraint, EndpointRef};
use crate::geom::{Point2, PointExpr, Segment, SegmentId, SEGMENT_STRIDE};

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
    segments: Vec<Segment>,
    constraints: Vec<Constraint>,
    next_id: usize,
}

impl DocumentStructure {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn segment(&self, id: SegmentId) -> Option<&Segment> {
        self.segments.iter().find(|s| s.id == id)
    }

    pub fn parameter_count(&self) -> usize {
        self.segments.len() * SEGMENT_STRIDE
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

    pub fn segments(&self) -> &[Segment] {
        self.structure.segments()
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
        self.structure.segments.push(Segment { id, param_offset });
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
        let Some(pos) = self.structure.segments.iter().position(|s| s.id == id) else {
            return false;
        };
        let removed = self.structure.segments.remove(pos);
        let start = removed.param_offset;
        let end = start + SEGMENT_STRIDE;
        assert!(end <= self.state.len());
        self.state.0.drain(start..end);

        for s in &mut self.structure.segments {
            if s.param_offset > start {
                s.param_offset -= SEGMENT_STRIDE;
            }
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

#[cfg(test)]
mod tests;
