//! Geometry kind marker (no object-safe sum type).
//!
//! Document stores the closed catalog [`crate::geometry_kinds::GeometryInst`].

use std::fmt::Debug;
use std::ops::Range;

use super::chart::Chart;
use super::shell::{AsShell, GeometryId, GeometryRole, GeometryShell};

/// Concrete geometry kind ready for Document storage and structural add.
///
/// Implement [`Chart`] and invoke [`geometry_shell_instance!`] — this marker
/// follows via the blanket. Conversion into the document catalog is
/// [`Into<crate::geometry_kinds::GeometryInst>`].
pub trait GeometryKind: Chart + AsShell + From<GeometryShell> + Clone + Debug + 'static {
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
        o..o + Self::PARAMETER_COUNT
    }

    /// After a contiguous block was removed from Vector, shift this instance if
    /// it lay after the hole.
    fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
        if self.shell().param_offset > removed_start {
            self.shell_mut().param_offset -= removed_len;
        }
    }
}

impl<T> GeometryKind for T where T: Chart + AsShell + From<GeometryShell> + Clone + Debug + 'static {}
