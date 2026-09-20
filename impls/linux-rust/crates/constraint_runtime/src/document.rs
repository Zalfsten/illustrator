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
    match expr {
        PointExpr::SegmentEnd { segment, .. } => Some(*segment),
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
mod tests {
    use super::*;
    use crate::constraint::SegmentEnd;
    use crate::geom::SegmentEndKind;

    #[test]
    fn two_segment_join_connectivity_indices() {
        let mut doc = Document::new();
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let s1 = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(3.0, 1.0));
        doc.join_endpoints(
            EndpointRef::new(s0, SegmentEnd::End),
            EndpointRef::new(s1, SegmentEnd::Start),
        );
        let c = &doc.constraints()[0];
        let conn = c.param_connectivity(doc.segments());
        assert_eq!(conn.global_indices, vec![2, 3, 4, 5]);
    }

    #[test]
    fn join_inserts_congruence_only() {
        let mut doc = Document::new();
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let s1 = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(3.0, 1.0));
        doc.join_endpoints(
            EndpointRef::new(s0, SegmentEnd::End),
            EndpointRef::new(s1, SegmentEnd::Start),
        );
        match &doc.constraints()[0] {
            Constraint::Congruence { a, b } => {
                assert!(matches!(
                    a,
                    crate::geom::PointExpr::SegmentEnd {
                        end: SegmentEndKind::End,
                        ..
                    }
                ));
                assert!(matches!(
                    b,
                    crate::geom::PointExpr::SegmentEnd {
                        end: SegmentEndKind::Start,
                        ..
                    }
                ));
            }
            _ => panic!("expected Congruence"),
        }
    }

    #[test]
    fn add_segment_grows_state_and_connectivity() {
        let mut doc = Document::new();
        assert_eq!(doc.state().len(), 0);
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        assert_eq!(doc.state().len(), SEGMENT_STRIDE);
        assert_eq!(doc.structure.parameter_count(), SEGMENT_STRIDE);
        let s = doc.segment(s0).unwrap();
        assert_eq!(s.connectivity().global_indices, vec![0, 1, 2, 3]);
        let s1 = doc.add_segment(Point2::new(2.0, 0.0), Point2::new(3.0, 1.0));
        assert_eq!(doc.state().len(), 2 * SEGMENT_STRIDE);
        assert_eq!(
            doc.segment(s1).unwrap().connectivity().global_indices,
            vec![4, 5, 6, 7]
        );
    }

    #[test]
    fn remove_segment_compacts_state_and_drops_orphans() {
        let mut doc = Document::new();
        let s0 = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
        let s1 = doc.add_segment(Point2::new(1.0, 0.0), Point2::new(1.0, 1.0));
        let s2 = doc.add_segment(Point2::new(5.0, 5.0), Point2::new(6.0, 5.0));
        doc.join_endpoints(
            EndpointRef::new(s0, SegmentEnd::End),
            EndpointRef::new(s1, SegmentEnd::Start),
        );
        doc.add_constraint(Constraint::Orthogonal { a: s0, b: s1 });
        doc.add_constraint(Constraint::FixPoint {
            point: PointExpr::SegmentEnd {
                segment: s2,
                end: SegmentEndKind::Start,
            },
            target: Point2::new(5.0, 5.0),
        });
        assert_eq!(doc.constraints().len(), 3);
        assert!(doc.remove_segment(s1));
        assert_eq!(doc.segments().len(), 2);
        assert_eq!(doc.state().len(), 2 * SEGMENT_STRIDE);
        // s0 still at 0..4; s2 remapped from 8..12 to 4..8
        assert_eq!(doc.segment(s0).unwrap().param_offset, 0);
        assert_eq!(doc.segment(s2).unwrap().param_offset, SEGMENT_STRIDE);
        assert_eq!(doc.state()[4], 5.0);
        // congruence + orthogonal referenced s1 → dropped; FixPoint on s2 kept
        assert_eq!(doc.constraints().len(), 1);
        assert!(matches!(
            &doc.constraints()[0],
            Constraint::FixPoint { point, .. }
                if matches!(point, PointExpr::SegmentEnd { segment, .. } if *segment == s2)
        ));
    }
}
