//! Labels between the drawing model and a typesetting engine.
//!
//! - [`typeset`]: the [`Typesetter`] port and its results (lines as exact
//!   outlines in pt, in the label frame).
//! - [`split`]: a label source split into lines at its explicit breaks.
//! - [`place`](mod@place): lines stacked by the LaTeX spacing rule and mapped from pt in
//!   the label frame to world coordinates.
//! - [`tessellate`](mod@tessellate): outlines as triangles for display.
//!
//! No engine and no GUI: the app, exports and tests share these pieces.

pub mod place;
pub mod split;
pub mod tessellate;
pub mod typeset;

pub use place::{
    baselines, frame_map, label_size, line_origins, place, world_per_pt, Affine2, LineMetrics,
};
pub use split::split_lines;
pub use tessellate::{tessellate, Triangles};
pub use typeset::{Contour, Paint, Segment, Shape, TypesetError, TypesetLine, Typesetter};
