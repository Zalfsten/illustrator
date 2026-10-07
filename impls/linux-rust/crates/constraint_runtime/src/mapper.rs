//! Vector column maps and chart value access.
//!
//! **Columns of \(p\):** every participant (geometry block or constraint
//! contribution) answers “which global indices?” as `Vec<usize>` (scatter
//! list). Contiguous geometry windows are the dense special case — still
//! emitted as an index list via [`Mapper::geometry_indices`]. Build all such
//! maps here ([`indices`](Mapper::indices) / [`local_indices`](Mapper::local_indices) /
//! [`join_indices`](Mapper::join_indices)).
//!
//! **Values in a block:** unpack / drain still use a contiguous
//! [`chart_param_block`](Mapper::chart_param_block) `Range` for slice access;
//! that is layout sugar, not a second answer to the column question.
//!
//! [`GeometryStore`] resolves identity; this type reads `param_offset` / width.

use std::ops::Range;

use crate::geometry::{Chart, Geometry};
use crate::vector::Vector;

/// Layout + column-map authority (stateless namespace).
///
/// Document owns offsets on each shell; Mapper reads them to unpack charts and
/// to build global index lists. Pass [`Vector`] per call when values are needed.
pub struct Mapper;

impl Mapper {
    /// Contiguous Vector window for slice ops (unpack, structural drain).
    ///
    /// Prefer [`geometry_indices`](Self::geometry_indices) when the question is
    /// only which columns are touched.
    pub fn chart_param_block(g: &dyn Geometry) -> Range<usize> {
        g.param_block()
    }

    /// All global columns owned by geometry `g` (dense block as index list).
    pub fn geometry_indices(g: &dyn Geometry) -> Vec<usize> {
        Self::indices(Self::chart_param_block(g))
    }

    /// Unpack [`Chart::Init`] from Vector using the geometry's parameter window.
    pub fn unpack_chart<C: Chart>(g: &dyn Geometry, p: &Vector) -> C::Init {
        let block = Self::chart_param_block(g);
        debug_assert_eq!(
            block.len(),
            C::PARAMETER_COUNT,
            "geometry {:?} block width does not match chart",
            g.id()
        );
        C::unpack(&p.as_slice()[block])
    }

    /// Lift formula-local parameter indices into global Vector indices.
    pub fn globalize_indices(
        g: &dyn Geometry,
        local: impl IntoIterator<Item = usize>,
    ) -> Vec<usize> {
        let offset = g.shell().param_offset;
        local.into_iter().map(|i| offset + i).collect()
    }

    /// Collect already-global Vector indices (sparse or dense).
    pub fn indices(global_indices: impl IntoIterator<Item = usize>) -> Vec<usize> {
        global_indices.into_iter().collect()
    }

    /// Lift local chart indices through `g`'s offset into global indices.
    pub fn local_indices(
        g: &dyn Geometry,
        local: impl IntoIterator<Item = usize>,
    ) -> Vec<usize> {
        Self::globalize_indices(g, local)
    }

    /// Concatenate several global index lists (constraint contribution order).
    pub fn join_indices(
        parts: impl IntoIterator<Item = impl IntoIterator<Item = usize>>,
    ) -> Vec<usize> {
        Self::indices(parts.into_iter().flatten())
    }
}

#[cfg(test)]
#[path = "tests_mapper.rs"]
mod tests_mapper;
