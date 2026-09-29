//! Concrete geometry kinds (charts). One file per kind.
//!
//! [`GeometryBody`] is the kind catalog: extend it (and `parameter_count`) when
//! adding a new geometry file under this module.

pub mod segment;

pub use segment::Segment;

/// Kind-specific payload (charts). Shared fields live on [`crate::geom::Geometry`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GeometryBody {
    Segment,
}

impl GeometryBody {
    pub fn parameter_count(self) -> usize {
        match self {
            GeometryBody::Segment => Segment::STRIDE,
        }
    }
}
