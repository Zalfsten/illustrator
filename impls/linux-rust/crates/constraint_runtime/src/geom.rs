//! Parameterized geometries: local params are references into the global state vector.

use crate::connectivity::Connectivity;

pub const SEGMENT_STRIDE: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dist2(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SegmentId(pub usize);

/// Line segment: endpoint chart, parameters live at `p[param_offset..param_offset+4]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub id: SegmentId,
    pub param_offset: usize,
}

impl Segment {
    pub fn connectivity(&self) -> Connectivity {
        let o = self.param_offset;
        Connectivity::new(vec![o, o + 1, o + 2, o + 3])
    }

    pub fn start(&self, p: &[f64]) -> Point2 {
        Point2::new(p[self.param_offset], p[self.param_offset + 1])
    }

    pub fn end(&self, p: &[f64]) -> Point2 {
        Point2::new(p[self.param_offset + 2], p[self.param_offset + 3])
    }

    pub fn direction(&self, p: &[f64]) -> Point2 {
        let a = self.start(p);
        let b = self.end(p);
        Point2::new(b.x - a.x, b.y - a.y)
    }

    pub fn endpoints(&self) -> (PointExpr, PointExpr) {
        (
            PointExpr::SegmentEnd {
                segment: self.id,
                end: SegmentEndKind::Start,
            },
            PointExpr::SegmentEnd {
                segment: self.id,
                end: SegmentEndKind::End,
            },
        )
    }

    pub fn endpoint_param_indices(&self, end: SegmentEndKind) -> [usize; 2] {
        match end {
            SegmentEndKind::Start => [self.param_offset, self.param_offset + 1],
            SegmentEndKind::End => [self.param_offset + 2, self.param_offset + 3],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SegmentEndKind {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointExpr {
    SegmentEnd {
        segment: SegmentId,
        end: SegmentEndKind,
    },
}

impl PointExpr {
    pub fn evaluate(&self, segments: &[Segment], p: &[f64]) -> Point2 {
        match *self {
            PointExpr::SegmentEnd { segment, end } => {
                let s = segments
                    .iter()
                    .find(|s| s.id == segment)
                    .expect("segment for PointExpr");
                match end {
                    SegmentEndKind::Start => s.start(p),
                    SegmentEndKind::End => s.end(p),
                }
            }
        }
    }

    pub fn param_indices(&self, segments: &[Segment]) -> [usize; 2] {
        match *self {
            PointExpr::SegmentEnd { segment, end } => {
                let s = segments
                    .iter()
                    .find(|s| s.id == segment)
                    .expect("segment for PointExpr");
                s.endpoint_param_indices(end)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_state_vector_changes_endpoints() {
        let seg = Segment {
            id: SegmentId(0),
            param_offset: 0,
        };
        let mut p = vec![0.0, 0.0, 1.0, 0.0];
        assert_eq!(seg.end(&p), Point2::new(1.0, 0.0));
        p[2] = 3.0;
        p[3] = 4.0;
        assert_eq!(seg.end(&p), Point2::new(3.0, 4.0));
        assert_eq!(seg.connectivity().global_indices, vec![0, 1, 2, 3]);
    }
}
