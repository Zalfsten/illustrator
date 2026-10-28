//! Command layer of a drawing session.
//!
//! Every front-end (GUI, scripts, AI clients) edits a drawing through
//! [`Session::apply`]: commands address objects by name and address, are
//! atomic, keep the drawing structurally consistent, and form one undo/redo
//! history together with drags.

pub mod command;
pub mod session;

pub use command::{Command, CommandError, Family, HistoryError, Outcome};
pub use session::Session;
