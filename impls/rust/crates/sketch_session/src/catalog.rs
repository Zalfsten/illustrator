//! Closed command catalog: per-kind payloads of `create` / `constrain` and
//! their dispatch (generated from the kind list).
//!
//! Adding a kind: implement [`GeometryCommand`] / [`ConstraintCommand`] in
//! `kinds/` and list the type below. A core kind missing here is a compile
//! error in the reverse mapping (`of`).

use constraint_runtime::{
    Arc, Congruence, ConstraintInst, Document, EqualLength, FixPoint, Geometry, GeometryId,
    GeometryInst, GeometryKind, GeometryRole, Label, Segment, StyleRef, VectorsOrthogonal,
    VectorsParallel,
};
use schemars::Schema;
use serde::{de::Error as _, Serialize, Serializer};
use serde_json::Value;

use crate::command::CommandError;
use crate::json::LookFields;
use crate::kinds::{AttributeCommand, ConstraintCommand, GeometryCommand, Resolver};
use crate::schema::inline_schema;
use crate::wire::Role;

/// Flat interchange object: reserved fields, then the kind's parameters.
#[derive(Serialize)]
pub(crate) struct Tagged<'a, P> {
    pub op: &'static str,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(flatten)]
    pub params: &'a P,
    /// Style fields of `create`, after the kind's parameters.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub look: Option<LookFields>,
}

macro_rules! command_catalog {
    (create: $($g:ident),+ ; constrain: $($c:ident),+ ; set: $($a:ident),+ $(;)?) => {
        /// Kind and parameters of a `create` command.
        #[derive(Clone, Debug, PartialEq)]
        pub enum GeometrySpec {
            $($g(<$g as GeometryCommand>::Params),)+
        }

        /// Kind and attribute fields of a `set` command.
        #[derive(Clone, Debug, PartialEq)]
        pub enum AttrSpec {
            $($a(<$a as AttributeCommand>::SetParams),)+
        }

        impl AttrSpec {
            /// Interchange tags of the kinds with attributes.
            pub const TAGS: &'static [&'static str] = &[$(<$a as GeometryCommand>::TAG),+];

            pub fn tag(&self) -> &'static str {
                match self {
                    $(Self::$a(_) => <$a as GeometryCommand>::TAG,)+
                }
            }

            /// Change the attributes of geometry `id` (named `name`); the
            /// geometry must be of this kind.
            pub(crate) fn apply_to(
                &self,
                doc: &mut Document,
                id: GeometryId,
                name: &str,
            ) -> Result<(), CommandError> {
                let g = doc.expect_geometry(id);
                match self {
                    $(Self::$a(p) => {
                        if g.kind() != <$a as Geometry>::KIND {
                            return Err(CommandError::WrongObjectType {
                                name: name.to_owned(),
                                expected: <$a as GeometryCommand>::TAG,
                            });
                        }
                        let mut attrs = g.attrs().clone();
                        <$a as AttributeCommand>::apply(p, &mut attrs)?;
                        doc.set_geometry_attrs(id, attrs);
                        Ok(())
                    })+
                }
            }

            pub(crate) fn from_fields(kind: &str, fields: Value) -> Result<Self, serde_json::Error> {
                $(if kind == <$a as GeometryCommand>::TAG {
                    let p: <$a as AttributeCommand>::SetParams = serde_json::from_value(fields)?;
                    if <$a as AttributeCommand>::is_empty(&p) {
                        return Err(serde_json::Error::custom("no attribute given"));
                    }
                    return Ok(Self::$a(p));
                })+
                Err(serde_json::Error::unknown_variant(kind, Self::TAGS))
            }

            pub(crate) fn serialize_tagged<S: Serializer>(
                &self,
                name: &str,
                s: S,
            ) -> Result<S::Ok, S::Error> {
                match self {
                    $(Self::$a(params) => Tagged {
                        op: "set",
                        kind: <$a as GeometryCommand>::TAG,
                        name: Some(name),
                        role: None,
                        params,
                        look: None,
                    }
                    .serialize(s),)+
                }
            }

            /// `(tag, inline attribute schema)` per kind.
            pub(crate) fn kind_schemas() -> Vec<(&'static str, Schema)> {
                vec![$((
                    <$a as GeometryCommand>::TAG,
                    inline_schema::<<$a as AttributeCommand>::SetParams>(),
                ),)+]
            }
        }

        /// Kind and parameters of a `constrain` command.
        #[derive(Clone, Debug, PartialEq)]
        pub enum ConstraintSpec {
            $($c(<$c as ConstraintCommand>::Params),)+
        }

        impl GeometrySpec {
            /// Interchange tags of all geometry kinds.
            pub const TAGS: &'static [&'static str] = &[$(<$g as GeometryCommand>::TAG),+];

            pub fn tag(&self) -> &'static str {
                match self {
                    $(Self::$g(_) => <$g as GeometryCommand>::TAG,)+
                }
            }

            /// Validate the parameters and add the geometry.
            pub(crate) fn add_to(
                &self,
                doc: &mut Document,
                role: GeometryRole,
            ) -> Result<GeometryId, CommandError> {
                match self {
                    $(Self::$g(p) => {
                        let init = <$g as GeometryCommand>::init(p)?;
                        let attrs = <$g as GeometryCommand>::attrs(p)?;
                        Ok(doc.add_geometry_with::<$g>(role, init, attrs))
                    })+
                }
            }

            /// Parameters that re-create `g` with its current values.
            pub(crate) fn of(doc: &Document, g: &GeometryInst) -> Self {
                match g.kind() {
                    $(GeometryKind::$g => {
                        Self::$g(<$g as GeometryCommand>::params(
                            g.unpack::<$g>(doc.vector()),
                            g.attrs(),
                        ))
                    })+
                }
            }

            pub(crate) fn from_fields(kind: &str, fields: Value) -> Result<Self, serde_json::Error> {
                $(if kind == <$g as GeometryCommand>::TAG {
                    return Ok(Self::$g(serde_json::from_value(fields)?));
                })+
                Err(serde_json::Error::unknown_variant(kind, Self::TAGS))
            }

            pub(crate) fn serialize_tagged<S: Serializer>(
                &self,
                op: &'static str,
                name: Option<&str>,
                role: Option<Role>,
                look: &StyleRef,
                s: S,
            ) -> Result<S::Ok, S::Error> {
                match self {
                    $(Self::$g(params) => Tagged {
                        op,
                        kind: <$g as GeometryCommand>::TAG,
                        name,
                        role,
                        params,
                        look: Some(look.into()),
                    }
                    .serialize(s),)+
                }
            }

            /// `(tag, inline parameter schema)` per kind.
            pub(crate) fn kind_schemas() -> Vec<(&'static str, Schema)> {
                vec![$((
                    <$g as GeometryCommand>::TAG,
                    inline_schema::<<$g as GeometryCommand>::Params>(),
                ),)+]
            }
        }

        impl ConstraintSpec {
            /// Interchange tags of all constraint kinds.
            pub const TAGS: &'static [&'static str] = &[$(<$c as ConstraintCommand>::TAG),+];

            pub fn tag(&self) -> &'static str {
                match self {
                    $(Self::$c(_) => <$c as ConstraintCommand>::TAG,)+
                }
            }

            /// Resolve the operands against the drawing behind `r`.
            pub(crate) fn build(&self, r: &Resolver<'_>) -> Result<ConstraintInst, CommandError> {
                match self {
                    $(Self::$c(p) => Ok(<$c as ConstraintCommand>::resolve(p, r)?.into()),)+
                }
            }

            /// Parameters that re-create `c`, or `None` if an operand has no address.
            pub(crate) fn of(doc: &Document, c: &ConstraintInst) -> Option<Self> {
                match c {
                    $(ConstraintInst::$c(k) => k.params(doc).map(Self::$c),)+
                }
            }

            pub(crate) fn from_fields(kind: &str, fields: Value) -> Result<Self, serde_json::Error> {
                $(if kind == <$c as ConstraintCommand>::TAG {
                    return Ok(Self::$c(serde_json::from_value(fields)?));
                })+
                Err(serde_json::Error::unknown_variant(kind, Self::TAGS))
            }

            pub(crate) fn serialize_tagged<S: Serializer>(
                &self,
                op: &'static str,
                name: Option<&str>,
                s: S,
            ) -> Result<S::Ok, S::Error> {
                match self {
                    $(Self::$c(params) => Tagged {
                        op,
                        kind: <$c as ConstraintCommand>::TAG,
                        name,
                        role: None,
                        params,
                        look: None,
                    }
                    .serialize(s),)+
                }
            }

            /// `(tag, inline parameter schema)` per kind.
            pub(crate) fn kind_schemas() -> Vec<(&'static str, Schema)> {
                vec![$((
                    <$c as ConstraintCommand>::TAG,
                    inline_schema::<<$c as ConstraintCommand>::Params>(),
                ),)+]
            }
        }
    };
}

command_catalog! {
    create: Segment, Arc, Label;
    constrain: Congruence, VectorsParallel, VectorsOrthogonal, FixPoint, EqualLength;
    set: Label;
}
