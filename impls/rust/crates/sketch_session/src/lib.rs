//! Command layer of a drawing session.
//!
//! Every front-end (GUI, scripts, AI clients) edits a drawing through
//! [`Session::apply`]: commands address objects by name and address, are
//! atomic, keep the drawing structurally consistent, and form one undo/redo
//! history together with drags.

pub mod catalog;
pub mod command;
pub mod file;
pub mod json;
pub mod kinds;
pub mod schema;
pub mod session;
pub mod wire;

pub use catalog::{ConstraintSpec, GeometrySpec};
pub use command::{
    Adjusted, Command, CommandError, Conflict, DeclCommand, ExportError, Family, HistoryError,
    LoadReport, Outcome,
};
pub use file::{CommandLoadError, LoadError, SaveError, FORMAT, VERSION};
pub use json::{ReadError, WriteError};
pub use kinds::{ArcParams, FixParams, LenPair, PointPair, SegmentParams, VecPair};
pub use schema::{command_schema, declarative_schema};
pub use session::Session;
pub use wire::{LenAddr, PointAddr, Role, VecAddr};
