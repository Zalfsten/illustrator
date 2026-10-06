//! Read-only lookup of document geometries by stable id.
//!
//! Chart unpack and global index lists live on [`crate::mapper::Mapper`].
//! Call sites resolve via [`Self::expect_geometry`], then map/unpack.

use crate::geometry::{Geometry, GeometryId};

/// O(1) geometry resolution during evaluate/solve.
///
/// Implemented by [`crate::document::Document`] (id → geometry HashMap).
pub trait GeometryStore {
    fn geometry(&self, id: GeometryId) -> Option<&dyn Geometry>;

    fn expect_geometry(&self, id: GeometryId) -> &dyn Geometry {
        self.geometry(id)
            .unwrap_or_else(|| panic!("geometry id {id:?} not in document"))
    }
}
