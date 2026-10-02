//! Read-only lookup of document geometries by stable id.

use std::ops::Range;

use crate::geometries::{Chart, Geometry, GeometryId};
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

    /// State index window for the geometry identified by `id`.
    fn chart_param_block(&self, id: GeometryId) -> Range<usize> {
        self.expect_geometry(id).param_block()
    }

    /// Unpack [`Chart::Init`] from the State window of geometry `id`.
    fn unpack_chart<C: Chart>(&self, id: GeometryId, p: &State) -> C::Init {
        let block = self.chart_param_block(id);
        debug_assert_eq!(
            block.len(),
            C::PARAMETER_COUNT,
            "geometry {id:?} block width does not match chart"
        );
        C::unpack(&p.as_slice()[block])
    }
}
