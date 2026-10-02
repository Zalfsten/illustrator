//! Document identity shared by every chart instance.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryId(pub usize);

/// Presentation role: result figure vs construction helper.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GeometryRole {
    #[default]
    Result,
    Construction,
}

/// Shared document identity and State-block placement for a geometry instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryShell {
    pub id: GeometryId,
    pub role: GeometryRole,
    /// Start index of this geometry's contiguous parameter block in State.
    pub param_offset: usize,
}

impl GeometryShell {
    pub fn new(id: GeometryId, role: GeometryRole, param_offset: usize) -> Self {
        Self {
            id,
            role,
            param_offset,
        }
    }
}
