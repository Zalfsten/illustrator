//! The typesetting port: one line of a label in, outlines and metrics out.
//!
//! Units are TeX points (pt). Coordinates are in the label frame: x along the
//! baseline, y a quarter turn counterclockwise (up), origin at the line's left
//! end on its baseline.

use std::fmt;

use constraint_runtime::{Markup, Point2};

/// A typesetting engine. Implementations may differ per host; a host may have
/// none (labels then show in their "not typeset" form).
pub trait Typesetter {
    /// Typeset one line, already split at explicit breaks ([`crate::split_lines`]).
    /// `preamble` applies only to [`Markup::Latex`].
    fn typeset_line(
        &self,
        line: &str,
        markup: Markup,
        size_pt: f64,
        preamble: &str,
    ) -> Result<TypesetLine, TypesetError>;
}

/// One typeset line in the label frame (pt, y up).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TypesetLine {
    pub shapes: Vec<Shape>,
    /// Extent along the baseline as set (advance width), not the ink.
    pub width: f64,
    /// Visible extent above the baseline (0 for a line without outlines).
    pub height: f64,
    /// Visible extent below the baseline (0 for a line without outlines).
    pub depth: f64,
}

/// Outlines painted one way.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    pub contours: Vec<Contour>,
    pub paint: Paint,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    /// Filled with the non-zero rule (glyphs, filled shapes).
    Fill,
    /// Stroked with this line width in pt (fraction bars, rules).
    Stroke { width: f64 },
}

/// One subpath: starts with [`Segment::MoveTo`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Contour(pub Vec<Segment>);

/// Path command with locations in the label frame (pt).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    MoveTo(Point2),
    LineTo(Point2),
    QuadTo(Point2, Point2),
    CubicTo(Point2, Point2, Point2),
    Close,
}

impl Segment {
    /// Locations this command names (control points included).
    pub fn points(&self) -> Vec<Point2> {
        match *self {
            Segment::MoveTo(p) | Segment::LineTo(p) => vec![p],
            Segment::QuadTo(c, p) => vec![c, p],
            Segment::CubicTo(c1, c2, p) => vec![c1, c2, p],
            Segment::Close => Vec::new(),
        }
    }
}

/// Why a line was not typeset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypesetError {
    /// A command or environment the typesetter does not provide (`\foo`).
    Unsupported { construct: String },
    /// The source is not valid in its markup (e.g. an unclosed `$`).
    Invalid { message: String },
}

impl fmt::Display for TypesetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypesetError::Unsupported { construct } => write!(f, "unsupported {construct}"),
            TypesetError::Invalid { message } => write!(f, "invalid: {message}"),
        }
    }
}

impl std::error::Error for TypesetError {}
