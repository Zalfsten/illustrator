//! Read-only lookup of document geometries by stable id.

use std::ops::Range;

use crate::geom::{Geometry, GeometryId};
use crate::geometries::Chart;
use crate::state::State;

/// O(1) geometry resolution during evaluate/solve.
///
/// Implemented by [`crate::document::DocumentStructure`] (id→slot index) and
/// [`crate::document::Document`] (delegates to structure).
pub trait GeometryStore {
    fn geometry(&self, id: GeometryId) -> Option<&Geometry>;

    fn expect_geometry(&self, id: GeometryId) -> &Geometry {
        self.geometry(id)
            .unwrap_or_else(|| panic!("geometry id {id:?} not in document"))
    }

    /// Resolve `id` and assert the stored body matches [`Chart::BODY`].
    fn expect_chart<C: Chart>(&self, id: GeometryId) -> &Geometry {
        let g = self.expect_geometry(id);
        assert_eq!(
            g.body,
            C::BODY,
            "geometry {id:?} is {:?}, expected {:?}",
            g.body,
            C::BODY
        );
        g
    }

    /// State index window for the chart instance identified by `id`.
    fn chart_param_block<C: Chart>(&self, id: GeometryId) -> Range<usize> {
        self.expect_chart::<C>(id).param_block()
    }

    /// Unpack [`Chart::Init`] from the State window of chart instance `id`.
    fn unpack_chart<C: Chart>(&self, id: GeometryId, p: &State) -> C::Init {
        C::unpack(&p.as_slice()[self.chart_param_block::<C>(id)])
    }
}
