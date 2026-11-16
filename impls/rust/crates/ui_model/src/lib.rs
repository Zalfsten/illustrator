//! Interaction model of a front end, independent of GUI toolkit and input
//! device (ui-model spec): one selection shared by every view, picking and
//! box selection over what the Canvas shows, and actions on the selection.
//! Front ends translate their devices into intents.

pub mod model;
pub mod pick;
pub mod selection;
pub mod tool;

pub use model::{BoxMode, Effect, Intent, UiError, UiModel};
pub use pick::{box_select, mesh_polylines, nearest_curve, nearest_endpoint, nearest_line, nearest_point, pick, Scene};
pub use selection::{entry_named, Entry, PickMode, Selection};
pub use tool::{brush_command, Filled, SlotKind, ToolKind};

#[cfg(test)]
mod test_util;
