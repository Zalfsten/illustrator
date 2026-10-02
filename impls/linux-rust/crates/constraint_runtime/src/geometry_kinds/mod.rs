//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`crate::geometry::Chart`] + [`geometry_shell_instance!`] →
//!   [`crate::geometry::GeometryKind`] / [`crate::geometry::Geometry`]
//! - [`crate::geometry::CurveExprs`] via `impl_curve_exprs!` (and
//!   [`crate::geometry::EndpointExprs`] / `impl_endpoint_exprs!` if join
//!   endpoints apply)
//! - one entry in [`register_expr_kinds!`](crate::geometry::expr) (same variant
//!   names as the capability macros)

pub mod segment;

pub use segment::Segment;
