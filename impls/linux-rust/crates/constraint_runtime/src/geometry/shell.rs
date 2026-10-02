//! Document identity shared by every chart instance.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryId(pub usize);

/// Allocates non-colliding [`GeometryId`] values for Document registration.
///
/// Owned by DocumentStructure — kinds never call this during chart construction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct IdGenerator {
    next: usize,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Next fresh id; advances the counter.
    pub fn allocate(&mut self) -> GeometryId {
        let id = GeometryId(self.next);
        self.next += 1;
        id
    }

    /// Record an explicit id (import/restore) so later [`Self::allocate`] values do not collide.
    pub fn observe(&mut self, id: GeometryId) {
        if id.0 >= self.next {
            self.next = id.0 + 1;
        }
    }
}

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

/// Local shell access for chart instances.
///
/// Supertrait of object-safe [`super::Geometry`]. Prefer [`geometry_shell_instance!`]
/// on kinds with a `shell: GeometryShell` field — orphan rules block a blanket
/// `Deref`/`From` over all such types, so the macro expands the only coherent
/// per-kind impls.
pub trait AsShell {
    fn shell(&self) -> &GeometryShell;
    fn shell_mut(&mut self) -> &mut GeometryShell;
}

/// Expand [`AsShell`] + [`From<GeometryShell>`] for a kind whose shell field is
/// named `shell`.
///
/// ```ignore
/// pub struct Segment { pub shell: GeometryShell }
/// geometry_shell_instance!(Segment);
/// ```
macro_rules! geometry_shell_instance {
    ($ty:ty) => {
        impl $crate::geometry::AsShell for $ty {
            fn shell(&self) -> &$crate::geometry::GeometryShell {
                &self.shell
            }

            fn shell_mut(&mut self) -> &mut $crate::geometry::GeometryShell {
                &mut self.shell
            }
        }

        impl From<$crate::geometry::GeometryShell> for $ty {
            fn from(shell: $crate::geometry::GeometryShell) -> Self {
                Self { shell }
            }
        }
    };
}
pub(crate) use geometry_shell_instance;
