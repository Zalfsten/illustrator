//! Concrete geometry kinds (charts) and the closed document catalog.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory with a unit-struct marker
//! - `chart_atoms!` for the layout
//! - `impl` [`Geometry`](crate::geometry::Geometry) (+ opt-in factory traits
//!   [`CurveExprs`](crate::geometry::CurveExprs) /
//!   [`EndpointExprs`](crate::geometry::EndpointExprs))
//! - `pub use` + one variant in [`GeometryKind`] and one arm in `with_kind!`

pub mod arc;
pub mod catalog;
pub mod segment;

pub use arc::Arc;
pub(crate) use catalog::with_kind;
pub use catalog::{GeometryInst, GeometryKind};
pub use segment::Segment;
