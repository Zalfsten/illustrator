//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`chart_atoms!`](crate::geometry::chart::chart_atoms) naming
//!   [`crate::geometry::StateAtom`] slots + [`geometry_shell_instance!`] →
//!   [`crate::geometry::GeometryKind`] / [`crate::geometry::Geometry`]
//! - [`crate::geometry::CurveExprs`] via `impl_curve_formula_key!` (and
//!   [`crate::geometry::EndpointExprs`] via `impl_endpoint_formula_key!` or
//!   a default `impl EndpointExprs for Ty {}` if join endpoints apply)
//! - one entry in [`register_expr_kinds!`](crate::geometry::expr) (same variant
//!   names as the formula-key macros)

pub mod segment;

pub use segment::Segment;
