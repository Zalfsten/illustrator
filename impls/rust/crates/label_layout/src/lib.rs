//! Labels between the drawing model and a typesetting engine.
//!
//! - [`typeset`]: the engine trait [`LineTypesetter`] and its results (lines as exact
//!   outlines in pt, in the label frame).
//! - [`port`]: the asynchronous [`Typesetter`] port the app talks to, and
//!   [`Immediate`] over an engine that answers at once.
//! - [`split`]: a label source split into lines at its explicit breaks.
//! - [`place`](mod@place): lines stacked by the LaTeX spacing rule and mapped from pt in
//!   the label frame to world coordinates.
//! - [`tessellate`](mod@tessellate): outlines as triangles for display.
//! - [`wire`]: JSON messages to and from a typesetting worker.
//!
//! No engine and no GUI: the app, exports and tests share these pieces.

pub mod place;
pub mod port;
pub mod split;
pub mod tessellate;
pub mod typeset;
pub mod wire;

pub use place::{
    baselines, frame_map, label_size, line_origins, place, world_per_pt, Affine2, LineMetrics,
};
pub use port::{Immediate, LineRequest, LineResult, Typesetter, TypesetterState};
pub use split::split_lines;
pub use tessellate::{tessellate, Triangles};
pub use typeset::{Contour, LineTypesetter, Paint, Segment, Shape, TypesetError, TypesetLine};
