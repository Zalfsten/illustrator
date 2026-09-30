//! Kind-owned chart packing for structural Document adds.

use crate::geometries::GeometryBody;

/// Geometry kind as a chart: packs initial State slots for Document registration.
///
/// Kinds own layout (`pack`); the Document composite owns mutation (extend State +
/// register geometry). Evaluate reads the recorded parameter window from structure.
pub trait Chart {
    /// Kind-specific construction inputs (e.g. segment endpoints).
    type Init;

    /// Discriminant stored on the geometry shell.
    const BODY: GeometryBody;

    /// Contiguous initial parameters for this kind (length = body parameter count).
    fn pack(init: Self::Init) -> impl AsRef<[f64]>;
}
