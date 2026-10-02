//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`Chart`] + [`geometry_shell_instance!`] → [`GeometryKind`] / [`Geometry`]
//! - [`expr::PointExpr`] / [`expr::VecExpr`] outer formula catalogs

pub mod chart;
pub mod expr;
pub mod geometry;
pub mod segment;
pub mod shell;

pub use chart::Chart;
pub use expr::{PointExpr, VecExpr};
pub use geometry::{Geometry, GeometryKind};
pub use segment::Segment;
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell, IdGenerator};
