//! Document: owns global state vector `p` and discrete sketch structure.

use crate::constraint::{Constraint, EndpointRef};
use crate::geom::{Point2, Segment, SegmentId, SEGMENT_STRIDE};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    p: Vec<f64>,
    segments: Vec<Segment>,
    constraints: Vec<Constraint>,
    next_id: usize,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> &[f64] {
        &self.p
    }

    pub fn state_mut(&mut self) -> &mut [f64] {
        &mut self.p
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    pub fn add_segment(&mut self, a: Point2, b: Point2) -> SegmentId {
        let id = SegmentId(self.next_id);
        self.next_id += 1;
        let param_offset = self.p.len();
        self.p
            .extend_from_slice(&[a.x, a.y, b.x, b.y]);
        debug_assert_eq!(self.p.len() - param_offset, SEGMENT_STRIDE);
        self.segments.push(Segment { id, param_offset });
        id
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

    pub fn segment(&self, id: SegmentId) -> Option<&Segment> {
        self.segments.iter().find(|s| s.id == id)
    }

    /// Snapshot of state for undo / drag.
    pub fn clone_state(&self) -> Vec<f64> {
        self.p.clone()
    }

    pub fn restore_state(&mut self, p: &[f64]) {
        assert_eq!(p.len(), self.p.len());
        self.p.copy_from_slice(p);
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
        // s0 end -> [2,3], s1 start -> [4,5]
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
}
