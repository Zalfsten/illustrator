//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`Chart`] impl (`pack` / `unpack`) + [`geometry_shell_instance!`]
//!   (blanket [`Geometry`] covers Document storage)
//! - [`expr::PointExpr`] / [`expr::VecExpr`] outer formula catalogs

pub mod chart;
pub mod expr;
pub mod geometry;
pub mod segment;
pub mod shell;

pub use chart::Chart;
pub use expr::{PointExpr, VecExpr};
pub use geometry::Geometry;
pub use segment::Segment;
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell};
