//! Concrete geometry kinds (charts) and the closed document catalog.
//!
//! Extend when adding a geometry:
//! - new `mod` + file under this directory
//! - [`chart_atoms!`](crate::geometry::chart::chart_atoms) + [`geometry_shell_instance!`]
//! - `pub use` + one name in [`catalog`]
//! - per formula family: `eval_*` + `accumulate_*_jacobian`
//! - one entry in [`register_expr_kinds!`](crate::geometry::expr)

pub mod catalog;
pub mod segment;

pub use catalog::{GeometryInst, InGeometryCatalog};
pub use segment::Segment;
