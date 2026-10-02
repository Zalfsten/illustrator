//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`Chart`] + [`geometry_shell_instance!`] → [`GeometryKind`] / [`Geometry`]
//! - [`CurveExprs`] via `impl_curve_exprs!` (and [`EndpointExprs`] / `impl_endpoint_exprs!` if join endpoints apply)
//! - [`expr::PointExpr`] / [`expr::VecExpr`] / [`expr::ScalarExpr`] (kind-keyed wrappers)

pub mod capabilities;
pub mod chart;
pub mod expr;
pub mod geometry;
pub mod segment;
pub mod shell;

pub use capabilities::{CurveExprs, EndpointExprs};
pub use chart::Chart;
pub use expr::{PointExpr, ScalarExpr, VecExpr};
pub use geometry::{Geometry, GeometryKind};
pub use segment::Segment;
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell, IdGenerator};
