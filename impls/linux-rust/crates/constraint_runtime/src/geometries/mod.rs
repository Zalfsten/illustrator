//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`GeometryBody`] variant + `parameter_count`
//! - [`expr::PointExpr`] / [`expr::VecExpr`] outer formula catalogs

pub mod expr;
pub mod segment;

pub use expr::{PointExpr, PointJacobian, VecExpr, MAX_POINT_JACOBIAN_PARAMS};
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
