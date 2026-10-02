//! Shared geometry infrastructure (not kind-specific).
//!
//! Concrete kinds live under [`crate::geometry_kinds`].

pub mod capabilities;
pub mod chart;
pub mod expr;
pub mod object;
pub mod shell;

pub use capabilities::{CurveExprs, EndpointExprs};
pub use chart::Chart;
pub use expr::{PointExpr, ScalarExpr, VecExpr};
pub use object::{Geometry, GeometryKind};
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell, IdGenerator};
