//! Shared geometry infrastructure (not kind-specific).
//!
//! Concrete kinds live under [`crate::geometry_kinds`].
//! Opaque expressions live under [`crate::expr`].

pub mod capabilities;
pub mod chart;
pub mod formula;
pub mod shell;

pub use capabilities::{
    Caps, CircularExprs, CurveExprs, EndpointExprs, GeometryCaps, HeadingFormula, LengthFormula,
    LineExprs, PointFormula, TurnFormula, VecFormula,
};
pub use chart::{Chart, ChartSlot, StateAtom};
pub use formula::{Formula, Geometry};
pub use shell::{GeometryId, GeometryRole, GeometryShell};
