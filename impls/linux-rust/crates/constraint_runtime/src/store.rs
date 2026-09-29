//! Read-only lookup of document geometries by stable id.

use crate::geom::{Geometry, GeometryId};

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
}
