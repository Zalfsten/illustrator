//! Command-layer traits implemented by each geometry and constraint kind.
//!
//! A kind supplies its interchange tag, its parameter type (fields of the
//! `create` / `constrain` object) and the mappings between parameters and the
//! core value; [`crate::catalog`] dispatches over the kind list.

use std::fmt::Debug;

use constraint_runtime::{
    Address, ConstraintInst, ConstraintKind, Document, Geometry, KindAttrs, LengthExpr, Point2,
    PointExpr, VecExpr,
};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::command::{CommandError, Family};
use crate::wire::{LenAddr, PointAddr, VecAddr};

mod arc;
mod equal_length;
mod fix;
mod join;
mod label;
mod parallel;
mod perpendicular;
mod segment;

pub use arc::ArcParams;
pub use equal_length::LenPair;
pub use fix::FixParams;
pub use join::PointPair;
pub use label::{LabelParams, LabelSet};
pub use parallel::VecPair;
pub use segment::SegmentParams;

/// Parameters of one kind in interchange form.
pub trait Params: Clone + Debug + PartialEq + Serialize + DeserializeOwned + JsonSchema {}

impl<T: Clone + Debug + PartialEq + Serialize + DeserializeOwned + JsonSchema> Params for T {}

/// A geometry kind that can be created by command.
pub trait GeometryCommand: Geometry {
    /// Interchange tag (`"segment"`); independent of [`Geometry::NAME_PREFIX`].
    const TAG: &'static str;
    type Params: Params;

    /// Chart values for `p`, or `InvalidValue` (non-finite or out of range).
    fn init(p: &Self::Params) -> Result<Self::Init, CommandError>;

    /// Non-parameter attributes for `p` (kinds without attributes keep the
    /// default), or `InvalidValue`.
    fn attrs(p: &Self::Params) -> Result<KindAttrs, CommandError> {
        let _ = p;
        Ok(Self::default_attrs())
    }

    /// Parameters that re-create `init` with `attrs` (export).
    fn params(init: Self::Init, attrs: &KindAttrs) -> Self::Params;
}

/// A geometry kind whose attributes can be changed by `set`.
pub trait AttributeCommand: GeometryCommand {
    /// Fields of `set` (all optional, `deny_unknown_fields`).
    type SetParams: Params;

    /// Whether `p` carries no attribute (rejected when read).
    fn is_empty(p: &Self::SetParams) -> bool;

    /// Change the attributes `p` carries, or `InvalidValue` (then `attrs` may
    /// be partly changed; callers work on a copy).
    fn apply(p: &Self::SetParams, attrs: &mut KindAttrs) -> Result<(), CommandError>;
}

/// A constraint kind that can be added by command.
pub trait ConstraintCommand: ConstraintKind + Into<ConstraintInst> + Sized {
    /// Interchange tag (`"parallel"`); independent of [`ConstraintKind::NAME_PREFIX`].
    const TAG: &'static str;
    type Params: Params;

    /// Constraint for `p` on the drawing behind `r`; includes kind-specific
    /// checks (e.g. `AlreadyJoined`).
    fn resolve(p: &Self::Params, r: &Resolver<'_>) -> Result<Self, CommandError>;

    /// Parameters that re-create `self`, or `None` if an operand has no address.
    fn params(&self, doc: &Document) -> Option<Self::Params>;
}

/// Address resolution with family check against one drawing.
pub struct Resolver<'a> {
    doc: &'a Document,
}

impl<'a> Resolver<'a> {
    pub fn new(doc: &'a Document) -> Self {
        Self { doc }
    }

    pub fn document(&self) -> &'a Document {
        self.doc
    }

    pub fn point(&self, a: &PointAddr) -> Result<PointExpr, CommandError> {
        match self.doc.resolve_address(a.as_str())? {
            Address::Point(e) => Ok(e),
            _ => Err(wrong_family(a.as_str(), Family::Point)),
        }
    }

    pub fn vector(&self, a: &VecAddr) -> Result<VecExpr, CommandError> {
        match self.doc.resolve_address(a.as_str())? {
            Address::Vector(e) => Ok(e),
            _ => Err(wrong_family(a.as_str(), Family::Vector)),
        }
    }

    pub fn length(&self, a: &LenAddr) -> Result<LengthExpr, CommandError> {
        match self.doc.resolve_address(a.as_str())? {
            Address::Length(e) => Ok(e),
            _ => Err(wrong_family(a.as_str(), Family::Length)),
        }
    }
}

fn wrong_family(address: &str, expected: Family) -> CommandError {
    CommandError::WrongFamily {
        address: address.to_owned(),
        expected,
    }
}

/// `v`, or `InvalidValue` for `field` if it is NaN or infinite.
pub fn finite(field: &'static str, v: f64) -> Result<f64, CommandError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(CommandError::InvalidValue {
            field,
            reason: "not a finite number",
        })
    }
}

/// `v`, or `InvalidValue` for `field` unless it is finite and greater than zero.
pub fn positive(field: &'static str, v: f64) -> Result<f64, CommandError> {
    if finite(field, v)? > 0.0 {
        Ok(v)
    } else {
        Err(CommandError::InvalidValue {
            field,
            reason: "not greater than zero",
        })
    }
}

/// `p`, or `InvalidValue` for `field` if a coordinate is NaN or infinite.
pub fn finite_point(field: &'static str, p: Point2) -> Result<Point2, CommandError> {
    finite(field, p.x)?;
    finite(field, p.y)?;
    Ok(p)
}

#[cfg(test)]
#[path = "tests_kinds.rs"]
mod tests_kinds;
