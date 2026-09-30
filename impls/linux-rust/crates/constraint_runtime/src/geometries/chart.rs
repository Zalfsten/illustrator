//! Kind-owned chart packing for structural Document adds.

use std::ops::Range;

use crate::geometries::GeometryBody;

/// Geometry kind as a chart: packs initial State slots for Document registration.
///
/// Kinds own layout (`PARAMETER_COUNT`, `pack`); the Document composite owns
/// mutation (extend State + register geometry). Evaluate reads the recorded
/// parameter window from structure.
pub trait Chart {
    /// Kind-specific construction inputs (e.g. segment endpoints).
    type Init;

    /// Discriminant stored on the geometry shell.
    const BODY: GeometryBody;

    /// Number of contiguous State scalars for one instance of this kind.
    const PARAMETER_COUNT: usize;

    /// Contiguous initial parameters (length = [`Self::PARAMETER_COUNT`]).
    fn pack(init: Self::Init) -> impl AsRef<[f64]>;

    /// Global index window for one instance whose block starts at `offset`.
    /// Prefer [`crate::geom::Geometry::param_block`] when a geometry record exists.
    fn param_block(offset: usize) -> Range<usize> {
        offset..offset + Self::PARAMETER_COUNT
    }
}
