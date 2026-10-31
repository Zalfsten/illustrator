//! Document identity shared by every chart instance.

use crate::style::StyleRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryId(pub usize);

impl From<usize> for GeometryId {
    fn from(raw: usize) -> Self {
        Self(raw)
    }
}

impl From<GeometryId> for usize {
    fn from(id: GeometryId) -> Self {
        id.0
    }
}

/// Presentation role: result figure vs construction helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GeometryRole {
    #[default]
    Result,
    Construction,
}

/// Shared document identity, role, style and Vector-block placement for a
/// geometry instance.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryShell {
    pub id: GeometryId,
    pub role: GeometryRole,
    /// Style reference and overrides (styling); never read by the solver.
    pub style: StyleRef,
    /// Start index of this geometry's contiguous parameter block in Vector.
    pub param_offset: usize,
}

impl GeometryShell {
    pub fn new(id: GeometryId, role: GeometryRole, param_offset: usize) -> Self {
        Self {
            id,
            role,
            style: StyleRef::default(),
            param_offset,
        }
    }
}
