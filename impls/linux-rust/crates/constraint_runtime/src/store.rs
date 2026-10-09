//! Read-only lookup of document geometries by stable id.
//!
//! Chart unpack and global index lists live on [`crate::mapper::Mapper`].
//! Call sites resolve via [`Self::expect_geometry`], then map/unpack.

use crate::geometry::GeometryId;
use crate::geometry_kinds::GeometryInst;

/// O(1) geometry resolution during evaluate/solve.
///
/// Implemented by [`crate::document::Document`] (id → [`GeometryInst`] HashMap).
pub trait GeometryStore {
    fn geometry(&self, id: GeometryId) -> Option<&GeometryInst>;

    fn expect_geometry(&self, id: GeometryId) -> &GeometryInst {
        self.geometry(id)
            .unwrap_or_else(|| panic!("geometry id {id:?} not in document"))
    }
}
