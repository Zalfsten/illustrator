//! Kind-owned chart: State layout (`pack` / `unpack`).

use std::ops::Range;

/// Geometry kind as a chart: packs init into State.
///
/// Document identity and shared shell ops live on [`super::Geometry`] (object-safe).
/// Concrete kinds also call [`crate::geometries::shell::geometry_shell_instance`]
/// (expands [`super::AsShell`] + [`From<GeometryShell>`](From)) so the blanket
/// `Geometry` impl applies without hand-written shell boilerplate.
pub trait Chart {
    /// Kind-specific construction inputs (e.g. segment endpoints).
    type Init;

    /// Number of contiguous State scalars for one instance of this kind.
    const PARAMETER_COUNT: usize;

    /// Contiguous initial parameters (length = [`Self::PARAMETER_COUNT`]).
    fn pack(init: Self::Init) -> impl AsRef<[f64]>;

    /// Inverse of [`Self::pack`].
    fn unpack(params: &[f64]) -> Self::Init;

    /// Global index window for a block that would start at `offset`.
    fn param_block_at(offset: usize) -> Range<usize> {
        offset..offset + Self::PARAMETER_COUNT
    }
}
