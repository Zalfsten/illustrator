//! Read-only lookup of document geometries by stable id.
//!
//! Chart unpack uses a **resolved** [`Geometry`] reference (no second id lookup).
//! Call sites resolve via [`Self::expect_geometry`], then
//! [`Self::unpack_chart`] / [`Self::chart_param_block`].

use std::ops::Range;

use crate::geometry::{Chart, Geometry, GeometryId};
use crate::state::State;

/// O(1) geometry resolution during evaluate/solve.
///
/// Implemented by [`crate::document::DocumentStructure`] (id→slot index) and
/// [`crate::document::Document`] (delegates to structure).
pub trait GeometryStore {
    fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry>;

    fn expect_geometry(&self, id: GeometryId) -> &dyn Geometry {
        self.geometry(id)
            .unwrap_or_else(|| panic!("geometry id {id:?} not in document"))
    }

    /// State index window from a resolved geometry record.
    fn chart_param_block(&self, g: &dyn Geometry) -> Range<usize> {
        g.param_block()
    }

    /// Unpack [`Chart::Init`] from State using a resolved geometry's parameter window.
    fn unpack_chart<C: Chart>(&self, g: &dyn Geometry, p: &State) -> C::Init {
        let block = self.chart_param_block(g);
        debug_assert_eq!(
            block.len(),
            C::PARAMETER_COUNT,
            "geometry {:?} block width does not match chart",
            g.id()
        );
        C::unpack(&p.as_slice()[block])
    }
}
