//! Object-safe document geometry: shared shell ops without a kind sum.

use std::fmt::Debug;
use std::ops::Range;

use super::chart::Chart;
use super::shell::{AsShell, GeometryId, GeometryRole, GeometryShell};

/// Concrete geometry kind ready for Document storage and structural add.
///
/// Not object-safe ([`Chart`] has associated types; [`From`] constructs instances).
/// Implement [`Chart`] and invoke [`geometry_shell_instance!`] — this marker and
/// [`Geometry`] follow via blankets.
pub trait GeometryKind: Chart + AsShell + From<GeometryShell> + Clone + Debug + 'static {}

impl<T> GeometryKind for T where T: Chart + AsShell + From<GeometryShell> + Clone + Debug + 'static {}

/// Document-held geometry instance (object-safe).
///
/// Extends [`AsShell`] for identity / State-block fields. Supplied automatically
/// for every [`GeometryKind`] so Document can store `Box<dyn Geometry>` without
/// a kind enum.
pub trait Geometry: AsShell + Debug {
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

impl<T: GeometryKind> Geometry for T {
    fn parameter_count(&self) -> usize {
        T::PARAMETER_COUNT
    }

    fn clone_box(&self) -> Box<dyn Geometry> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Geometry> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
