//! Closed geometry catalog: kind tag, stored instance, and the kind dispatch.

use std::ops::Range;

use crate::geometry::{
    Caps, Chart, Geometry, GeometryCaps, GeometryId, GeometryRole, GeometryShell,
};
use crate::geometry_kinds::LabelAttrs;
use crate::vector::Vector;

/// Closed set of geometry kinds (discriminant only; kinds carry no data).
///
/// Adding a kind: one variant here and one arm in `with_kind!`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GeometryKind {
    Segment,
    Arc,
    Label,
}

/// Run `$body` with `$K` bound to the [`Geometry`] type of `$kind`.
///
/// The only place that matches on [`GeometryKind`]; everything else is
/// generic over `K: Geometry` and monomorphizes per arm.
macro_rules! with_kind {
    ($kind:expr, $K:ident => $body:expr) => {
        match $kind {
            $crate::geometry_kinds::GeometryKind::Segment => {
                type $K = $crate::geometry_kinds::Segment;
                $body
            }
            $crate::geometry_kinds::GeometryKind::Arc => {
                type $K = $crate::geometry_kinds::Arc;
                $body
            }
            $crate::geometry_kinds::GeometryKind::Label => {
                type $K = $crate::geometry_kinds::Label;
                $body
            }
        }
    };
}
pub(crate) use with_kind;

impl GeometryKind {
    /// Capability set of this kind (declared by the kind via `geometry_caps!`).
    ///
    /// For code working on dynamic ids; statically typed code uses the traits.
    pub fn caps(self) -> Caps {
        with_kind!(self, K => K::CAPS)
    }

    /// Prefix of automatic names for this kind ([`Geometry::NAME_PREFIX`]).
    pub fn name_prefix(self) -> &'static str {
        with_kind!(self, K => K::NAME_PREFIX)
    }
}

/// Non-parameter attributes of a geometry, by kind (see [`Geometry::default_attrs`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub enum KindAttrs {
    /// Kinds without attributes.
    #[default]
    None,
    Label(LabelAttrs),
}

/// Document-held geometry: shared shell, kind tag and kind attributes.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryInst {
    shell: GeometryShell,
    kind: GeometryKind,
    attrs: KindAttrs,
}

impl GeometryInst {
    /// Instance with the kind's default attributes.
    pub fn new(kind: GeometryKind, shell: GeometryShell) -> Self {
        let attrs = with_kind!(kind, K => K::default_attrs());
        Self { shell, kind, attrs }
    }

    /// Instance of kind `K` (tag taken from [`Geometry::KIND`]).
    pub fn of<K: Geometry>(shell: GeometryShell) -> Self {
        Self::new(K::KIND, shell)
    }

    pub fn kind(&self) -> GeometryKind {
        self.kind
    }

    pub fn shell(&self) -> &GeometryShell {
        &self.shell
    }

    pub fn id(&self) -> GeometryId {
        self.shell.id
    }

    pub fn role(&self) -> GeometryRole {
        self.shell.role
    }

    pub fn set_role(&mut self, role: GeometryRole) {
        self.shell.role = role;
    }

    pub fn attrs(&self) -> &KindAttrs {
        &self.attrs
    }

    /// Replace the attributes; `false` (unchanged) if `attrs` is of another kind.
    pub fn set_attrs(&mut self, attrs: KindAttrs) -> bool {
        if std::mem::discriminant(&attrs) != std::mem::discriminant(&self.attrs) {
            return false;
        }
        self.attrs = attrs;
        true
    }

    pub fn parameter_count(&self) -> usize {
        with_kind!(self.kind, K => K::PARAMETER_COUNT)
    }

    /// Contiguous global Vector window owned by this geometry.
    pub fn param_block(&self) -> Range<usize> {
        let o = self.shell.param_offset;
        o..o + self.parameter_count()
    }

    pub fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
        if self.shell.param_offset > removed_start {
            self.shell.param_offset -= removed_len;
        }
    }

    /// Unpack this geometry's chart init from `p`. `K` must match [`Self::kind`].
    #[inline]
    pub fn unpack<K: Geometry>(&self, p: &Vector) -> K::Init {
        debug_assert_eq!(
            self.kind,
            K::KIND,
            "geometry {:?} unpacked as wrong kind",
            self.id()
        );
        let o = self.shell.param_offset;
        K::unpack(&p[o..o + K::PARAMETER_COUNT])
    }

    /// Lift chart-local columns into global Vector indices.
    pub fn global_indices(&self, local: Range<usize>) -> Vec<usize> {
        let o = self.shell.param_offset;
        (o + local.start..o + local.end).collect()
    }
}

#[cfg(test)]
#[path = "tests_catalog.rs"]
mod tests_catalog;
