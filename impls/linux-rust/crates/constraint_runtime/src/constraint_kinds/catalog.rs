//! Closed [`ConstraintInst`] catalog (generated from the kind list).

use super::{Congruence, EqualLength, FixPoint, Orthogonal, Parallel};
use crate::constraint::ConstraintKind;
use crate::geometry::GeometryId;
use crate::matrix::MatrixWrite;
use crate::store::GeometryStore;
use crate::vector::{VecWrite, Vector};

/// Build [`ConstraintInst`] + `From` + dispatch from the closed kind list.
macro_rules! constraint_catalog {
    ($($kind:ident),+ $(,)?) => {
        /// Closed constraint catalog stored on [`crate::document::Document`].
        #[derive(Clone, Debug)]
        pub enum ConstraintInst {
            $($kind($kind),)+
        }

        $(
            impl From<$kind> for ConstraintInst {
                fn from(c: $kind) -> Self {
                    Self::$kind(c)
                }
            }
        )+

        impl ConstraintInst {
            /// Congruence payload when this entry is a point-point identification.
            pub fn as_congruence(&self) -> Option<&Congruence> {
                match self {
                    Self::Congruence(c) => Some(c),
                    _ => None,
                }
            }

            pub fn residual_arity(&self) -> usize {
                match self {
                    $(Self::$kind(_) => $kind::RESIDUAL_ARITY,)+
                }
            }

            pub fn param_connectivity<S: GeometryStore + ?Sized>(&self, store: &S) -> Vec<usize> {
                match self {
                    $(Self::$kind(c) => c.param_connectivity(store),)+
                }
            }

            pub fn eval_residual<S: GeometryStore + ?Sized, R: VecWrite>(
                &self,
                store: &S,
                p: &Vector,
                out: &mut R,
            ) {
                match self {
                    $(Self::$kind(c) => c.eval_residual(store, p, out),)+
                }
            }

            pub fn accumulate_jacobian<S: GeometryStore + ?Sized, J: MatrixWrite>(
                &self,
                store: &S,
                p: &Vector,
                j: &mut J,
            ) {
                match self {
                    $(Self::$kind(c) => c.accumulate_jacobian(store, p, j),)+
                }
            }

            pub fn referenced_geometries(&self) -> Vec<GeometryId> {
                match self {
                    $(Self::$kind(c) => c.referenced_geometries(),)+
                }
            }

            pub fn references_geometry(&self, id: GeometryId) -> bool {
                match self {
                    $(Self::$kind(c) => c.references_geometry(id),)+
                }
            }
        }
    };
}

constraint_catalog! {
    Congruence,
    Parallel,
    Orthogonal,
    FixPoint,
    EqualLength,
}
