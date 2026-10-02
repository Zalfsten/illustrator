//! Chart layout mapping: local parameters ↔ global State via [`GeometryShell`].
//!
//! [`GeometryStore`] resolves identity; this type reads `param_offset` / block
//! width and applies chart pack layout. Kinds stay in local indices; call sites
//! that need global State indices go through [`Mapper::globalize_indices`].

use std::ops::Range;

use crate::geometry::{Chart, Geometry};
use crate::state::State;

/// Layout authority for contiguous chart parameter windows.
///
/// Stateless namespace: Document owns offsets on each shell; Mapper only reads
/// them (and unpacks / globalizes). Pass [`State`] per call when values are needed.
pub struct Mapper;

impl Mapper {
    /// Global State index window for a resolved geometry.
    pub fn chart_param_block(g: &dyn Geometry) -> Range<usize> {
        g.param_block()
    }

    /// Unpack [`Chart::Init`] from State using the geometry's parameter window.
    pub fn unpack_chart<C: Chart>(g: &dyn Geometry, p: &State) -> C::Init {
        let block = Self::chart_param_block(g);
        debug_assert_eq!(
            block.len(),
            C::PARAMETER_COUNT,
            "geometry {:?} block width does not match chart",
            g.id()
        );
        C::unpack(&p.as_slice()[block])
    }

    /// Lift formula-local parameter indices into global State indices.
    pub fn globalize_indices(g: &dyn Geometry, local: impl IntoIterator<Item = usize>) -> Vec<usize> {
        let offset = g.shell().param_offset;
        local.into_iter().map(|i| offset + i).collect()
    }
}
