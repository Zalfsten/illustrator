//! Commands, results and errors.
//!
//! Commands refer to objects by name and to points, free vectors and lengths by
//! address (`seg1.end`, `seg1.direction`, `arc1.radius`), never by internal id.

use std::fmt;

use constraint_runtime::{AddressError, GeometryRole, NameError, Point2};

/// One edit of the drawing. Optional `name`s replace the automatic name.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    CreateSegment {
        start: Point2,
        end: Point2,
        role: GeometryRole,
        name: Option<String>,
    },
    /// Angles in radians (start heading from +x, counterclockwise sweep).
    CreateArc {
        center: Point2,
        radius: f64,
        start_heading: f64,
        sweep: f64,
        role: GeometryRole,
        name: Option<String>,
    },
    /// Two point addresses become one location.
    Join {
        a: String,
        b: String,
        name: Option<String>,
    },
    /// Two free-vector addresses are parallel.
    Parallel {
        a: String,
        b: String,
        name: Option<String>,
    },
    /// Two free-vector addresses are perpendicular.
    Perpendicular {
        a: String,
        b: String,
        name: Option<String>,
    },
    /// A point address is pinned to `target`.
    Fix {
        point: String,
        target: Point2,
        name: Option<String>,
    },
    /// Two length addresses are equal.
    EqualLength {
        a: String,
        b: String,
        name: Option<String>,
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
