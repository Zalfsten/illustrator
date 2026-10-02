//! Object-safe document geometry: shared shell ops without a kind sum.

use std::fmt::Debug;
use std::ops::Range;

use super::chart::Chart;
use super::shell::{AsShell, GeometryId, GeometryRole, GeometryShell};

/// Document-held geometry instance (object-safe).
///
/// Concrete kinds implement [`Chart`] and invoke [`geometry_shell_instance!`]
/// ([`AsShell`] + [`From<GeometryShell>`]); the blanket impl below supplies this
/// trait so Document can store `Box<dyn Geometry>` without a kind enum.
pub trait Geometry: Debug {
    fn shell(&self) -> &GeometryShell;
    fn shell_mut(&mut self) -> &mut GeometryShell;
    fn parameter_count(&self) -> usize;
    fn clone_box(&self) -> Box<dyn Geometry>;

    fn id(&self) -> GeometryId {
        self.shell().id
    }

    fn role(&self) -> GeometryRole {
        self.shell().role
    }

    fn set_role(&mut self, role: GeometryRole) {
        self.shell_mut().role = role;
    }

    fn param_block(&self) -> Range<usize> {
        let o = self.shell().param_offset;
        o..o + self.parameter_count()
    }

    /// After a contiguous block was removed from State, shift this instance if
    /// it lay after the hole.
    fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
        if self.shell().param_offset > removed_start {
            self.shell_mut().param_offset -= removed_len;
        }
    }
}

impl<T> Geometry for T
where
    T: Chart + AsShell + Clone + Debug + 'static,
{
    fn shell(&self) -> &GeometryShell {
        AsShell::shell(self)
    }

    fn shell_mut(&mut self) -> &mut GeometryShell {
        AsShell::shell_mut(self)
    }

    fn parameter_count(&self) -> usize {
        T::PARAMETER_COUNT
    }

    fn clone_box(&self) -> Box<dyn Geometry> {
        Box::new(self.clone())
    }
}
