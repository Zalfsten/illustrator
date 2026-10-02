//! Concrete geometry kinds (charts). One file per kind.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`Chart`] impl (`pack` / `unpack`) + [`Deref`] to [`GeometryShell`]
//!   (blanket [`Geometry`] covers Document storage)
//! - [`expr::PointExpr`] / [`expr::VecExpr`] outer formula catalogs

pub mod chart;
pub mod expr;
pub mod geometry;
pub mod segment;
pub mod shell;

pub use chart::Chart;
pub use expr::{
    PointExpr, PointJacobian, VecExpr, VecJacobian, MAX_POINT_JACOBIAN_PARAMS,
};
pub use geometry::Geometry;
pub use segment::Segment;
pub use shell::{GeometryId, GeometryRole, GeometryShell};
