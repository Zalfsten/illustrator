//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`chart_atoms!`](crate::geometry::chart::chart_atoms) naming
//!   [`crate::geometry::StateAtom`] slots + [`geometry_shell_instance!`] →
//!   [`crate::geometry::GeometryKind`] / [`crate::geometry::Geometry`]
//! - one `eval_*` → [`FormulaEval`](crate::geometry::expr::FormulaEval) per
//!   formula family (optional sparse `endpoint_point_param_indices`; other
//!   families default connectivity to [`SLOTS`](crate::geometry::Chart))
//! - one entry in [`register_expr_kinds!`](crate::geometry::expr) (installs
//!   Expr dispatch + [`CurveFormulaKey`](crate::geometry::capabilities::CurveFormulaKey)
//!   / optional [`EndpointFormulaKey`](crate::geometry::capabilities::EndpointFormulaKey))

pub mod segment;

pub use segment::Segment;
