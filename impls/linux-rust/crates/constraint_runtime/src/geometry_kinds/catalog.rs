//! Closed [`GeometryInst`] catalog (generated from the kind list).

use super::Segment;
use crate::geometry::shell::{AsShell, GeometryId, GeometryRole, GeometryShell};
use crate::geometry::{Chart, GeometryKind};
use std::ops::Range;

/// Build [`GeometryInst`] + shell access + [`InGeometryCatalog`] from the kind list.
macro_rules! geometry_catalog {
    ($($kind:ident),+ $(,)?) => {
        /// Closed geometry catalog stored on [`crate::document::Document`].
        ///
        /// Parallel to [`crate::constraint_kinds::ConstraintInst`].
        #[derive(Clone, Debug)]
        pub enum GeometryInst {
            $($kind($kind),)+
        }

        $(
            impl From<$kind> for GeometryInst {
                fn from(g: $kind) -> Self {
                    Self::$kind(g)
                }
            }
        )+

        impl AsShell for GeometryInst {
            fn shell(&self) -> &GeometryShell {
                match self {
                    $(Self::$kind(g) => g.shell(),)+
                }
            }

            fn shell_mut(&mut self) -> &mut GeometryShell {
                match self {
                    $(Self::$kind(g) => g.shell_mut(),)+
                }
            }
        }

        impl GeometryInst {
            pub fn parameter_count(&self) -> usize {
                match self {
                    $(Self::$kind(_) => $kind::PARAMETER_COUNT,)+
                }
            }

            pub fn param_block(&self) -> Range<usize> {
                let o = self.shell().param_offset;
                o..o + self.parameter_count()
            }

            pub fn reindex_after_removal(&mut self, removed_start: usize, removed_len: usize) {
                if self.shell().param_offset > removed_start {
                    self.shell_mut().param_offset -= removed_len;
                }
            }

            pub fn id(&self) -> GeometryId {
                self.shell().id
            }

            pub fn role(&self) -> GeometryRole {
                self.shell().role
            }

            pub fn set_role(&mut self, role: GeometryRole) {
                self.shell_mut().role = role;
            }
        }

        /// Extract a concrete chart kind from [`GeometryInst`] (catalog membership).
        pub trait InGeometryCatalog: GeometryKind {
            fn as_catalog_ref(inst: &GeometryInst) -> &Self;
        }

        $(
            impl InGeometryCatalog for $kind {
                fn as_catalog_ref(inst: &GeometryInst) -> &Self {
                    match inst {
                        GeometryInst::$kind(g) => g,
                        #[allow(unreachable_patterns)]
                        _ => panic!(
                            "geometry catalog: expected {} , got other kind",
                            stringify!($kind)
                        ),
                    }
                }
            }
        )+
    };
}

geometry_catalog! {
    Segment,
}
