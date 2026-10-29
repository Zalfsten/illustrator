//! Commands, results and errors.
//!
//! Commands refer to objects by name and to points, free vectors and lengths by
//! address (`seg1.end`, `seg1.direction`, `arc1.radius`), never by internal id.

use std::fmt;

use constraint_runtime::{AddressError, GeometryRole, NameError};

use crate::catalog::{ConstraintSpec, GeometrySpec};

/// One edit of the drawing. Optional `name`s replace the automatic name.
///
/// Interchange form: see [`crate::json`]; kinds and their parameters: see
/// [`crate::catalog`].
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// New geometry of the kind in `geometry`.
    Create {
        name: Option<String>,
        role: GeometryRole,
        geometry: GeometrySpec,
    },
    /// New constraint of the kind in `constraint`.
    Constrain {
        name: Option<String>,
        constraint: ConstraintSpec,
    },
    /// Delete a geometry (with its constraints) or a constraint.
    Delete {
        name: String,
    },
    Rename {
        name: String,
        new_name: String,
    },
    /// Result or construction role of a geometry.
    SetRole {
        name: String,
        role: GeometryRole,
    },
}

/// A command of the declarative subset (files): creates and constraints,
/// each with its final name.
#[derive(Clone, Debug, PartialEq)]
pub enum DeclCommand {
    Create {
        name: String,
        role: GeometryRole,
        geometry: GeometrySpec,
    },
    Constrain {
        name: String,
        constraint: ConstraintSpec,
    },
}

impl From<DeclCommand> for Command {
    fn from(c: DeclCommand) -> Self {
        match c {
            DeclCommand::Create {
                name,
                role,
                geometry,
            } => Command::Create {
                name: Some(name),
                role,
                geometry,
            },
            DeclCommand::Constrain { name, constraint } => Command::Constrain {
                name: Some(name),
                constraint,
            },
        }
    }
}

/// What a successful command did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    /// Name of the created object, if the command created one.
    pub created: Option<String>,
    /// Names of deleted objects (cascade included).
    pub removed: Vec<String>,
}

/// Result kind of an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Point,
    Vector,
    Length,
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Family::Point => "point",
            Family::Vector => "free vector",
            Family::Length => "length",
        })
    }
}

/// Why a command failed. The drawing is unchanged after any error.
#[derive(Clone, Debug, PartialEq)]
pub enum CommandError {
    /// No object has this name.
    UnknownName(String),
    /// A new name is invalid or taken.
    Name(NameError),
    /// An address does not resolve (includes a missing capability).
    Address(AddressError),
    WrongFamily {
        address: String,
        expected: Family,
    },
    /// The named object is of the wrong type for the command.
    WrongObjectType {
        name: String,
        expected: &'static str,
    },
    /// The structural constraints cannot be satisfied after the command.
    Unsatisfiable {
        residual: f64,
    },
    /// A drag is in progress.
    Busy,
    /// A join of two points that are already one congruence class.
    AlreadyJoined {
        a: String,
        b: String,
    },
    /// A number is not finite or outside the range of its field.
    InvalidValue {
        field: &'static str,
        reason: &'static str,
    },
    /// An operand lies in a conflicted (frozen) component; `name` is one of
    /// its geometries.
    Frozen {
        name: String,
    },
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::UnknownName(n) => write!(f, "no object named {n}"),
            CommandError::Name(e) => write!(f, "{e}"),
            CommandError::Address(e) => write!(f, "{e}"),
            CommandError::WrongFamily { address, expected } => {
                write!(f, "{address} is not a {expected}")
            }
            CommandError::WrongObjectType { name, expected } => {
                write!(f, "{name} is not a {expected}")
            }
            CommandError::Unsatisfiable { residual } => {
                write!(
                    f,
                    "unsatisfiable: constraints not met (residual {residual:.3e})"
                )
            }
            CommandError::Busy => f.write_str("busy: a drag is in progress"),
            CommandError::AlreadyJoined { a, b } => {
                write!(f, "already joined: {a} and {b} are one point")
            }
            CommandError::InvalidValue { field, reason } => {
                write!(f, "invalid value for {field}: {reason}")
            }
            CommandError::Frozen { name } => {
                write!(f, "frozen: {name} is in a conflicted component")
            }
        }
    }
}

impl std::error::Error for CommandError {}

impl From<NameError> for CommandError {
    fn from(e: NameError) -> Self {
        CommandError::Name(e)
    }
}

impl From<AddressError> for CommandError {
    fn from(e: AddressError) -> Self {
        CommandError::Address(e)
    }
}

/// A component whose constraints are not met at its values; it is frozen
/// until a delete resolves it. Names are in creation order.
#[derive(Clone, Debug, PartialEq)]
pub struct Conflict {
    pub geometries: Vec<String>,
    pub constraints: Vec<String>,
    /// Norm of the component's structural residual.
    pub residual: f64,
}

/// A component the session-start solve changed: its geometries (creation
/// order) and the largest distance a curve point moved, compared at the same
/// curve position, in drawing units.
#[derive(Clone, Debug, PartialEq)]
pub struct Adjusted {
    pub geometries: Vec<String>,
    pub displacement: f64,
}

/// What starting a session on an existing drawing did to it: components it
/// solved, and components left frozen. Both empty when the drawing's values
/// already met all constraints.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoadReport {
    pub adjusted: Vec<Adjusted>,
    pub conflicts: Vec<Conflict>,
}

/// Why the drawing cannot be written as declarative commands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportError {
    /// An operand of the constraint has no address (e.g. a `Uniform` curve
    /// parameter); only drawings built outside the command layer have these.
    NotAddressable { constraint: String },
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::NotAddressable { constraint } => {
                write!(f, "{constraint} has an operand without address")
            }
        }
    }
}

impl std::error::Error for ExportError {}

/// Why undo or redo did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryError {
    NothingToUndo,
    NothingToRedo,
    /// A drag is in progress.
    Busy,
}

impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            HistoryError::NothingToUndo => "nothing to undo",
            HistoryError::NothingToRedo => "nothing to redo",
            HistoryError::Busy => "busy: a drag is in progress",
        })
    }
}

impl std::error::Error for HistoryError {}
