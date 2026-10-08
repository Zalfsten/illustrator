//! Shared geometry infrastructure (not kind-specific).
//!
//! Concrete kinds live under [`crate::geometry_kinds`].

pub mod capabilities;
pub mod chart;
pub mod expr;
pub mod geometry;
pub mod shell;

pub use capabilities::{CurveExprs, EndpointExprs};
pub use chart::{Chart, ChartSlot, StateAtom};
pub use expr::{LengthExpr, PointExpr, VecExpr, MAX_FORMULA_LOCAL_DOF};
pub use geometry::{Geometry, GeometryKind};
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell};
