//! Concrete geometry kinds (charts) and the closed document catalog.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory with a unit-struct marker
//! - `chart_atoms!` for the layout
//! - `geometry_caps!` for the opt-in capabilities (factory trait impls and
//!   runtime [`Caps`](crate::geometry::Caps))
//! - `impl` [`Geometry`](crate::geometry::Geometry) (including `NAME_PREFIX`)
//! - `pub use` + one variant in [`GeometryKind`] and one arm in `with_kind!`

pub mod arc;
pub mod catalog;
pub mod segment;

pub use arc::Arc;
pub(crate) use catalog::with_kind;
pub use catalog::{GeometryInst, GeometryKind};
pub use segment::Segment;
