//! Shared geometry infrastructure (not kind-specific).
//!
//! Concrete kinds live under [`crate::geometry_kinds`].
//! Opaque expressions live under [`crate::expr`].

pub mod capabilities;
pub mod chart;
pub mod expr;
pub mod shell;

pub use capabilities::{CurveExprs, EndpointExprs};
pub use chart::{Chart, ChartSlot, GeometryKind, StateAtom};
pub use shell::{AsShell, GeometryId, GeometryRole, GeometryShell};
