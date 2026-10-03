//! Shared geometry infrastructure (not kind-specific).
//!
//! Concrete kinds live under [`crate::geometry_kinds`].

pub mod capabilities;
pub mod chart;
pub mod expr;
pub mod geometry;
pub mod shell;

pub use capabilities::{CurveExprs, EndpointExprs};
pub use chart::{AtomTuple, Chart, ChartSlot, StateAtom};
pub use expr::{PointExpr, ScalarExpr, VecExpr};
pub use geometry::{Geometry, GeometryKind};
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell, IdGenerator};
