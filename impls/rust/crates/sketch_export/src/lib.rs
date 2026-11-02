//! A drawing as a standalone vector file (drawing-export).
//!
//! Format-neutral print primitives come first, the writers follow:
//! - `scene`: visible result objects as strokes and filled or stroked
//!   outlines in world coordinates, from exact curve shapes and resolved
//!   styles, with typeset label lines placed.
//! - `bounds`: the exact, tight bounds of a scene.
//! - `svg`: a scene as SVG text in millimetres.
//! - `pdf`: a scene as a one-page PDF, arcs as cubic Béziers within 1 µm.
//!
//! No GUI and no host: the app collects typeset lines and saves the bytes.

pub mod bounds;
pub mod pdf;
pub mod scene;
pub mod svg;

use std::fmt;

use constraint_runtime::{Document, GeometryId};

pub use bounds::{bounds, Bounds};
pub use scene::{exported_labels, scene, DashArray, LabelLines, Object, Part, PathSeg, Scene};
pub use pdf::pdf;
pub use svg::svg;

/// A file format of the export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Format {
    Svg,
    Pdf,
}

impl Format {
    /// File name extension, without the dot.
    pub fn extension(self) -> &'static str {
        match self {
            Format::Svg => "svg",
            Format::Pdf => "pdf",
        }
    }

    /// Media type of the file.
    pub fn mime(self) -> &'static str {
        match self {
            Format::Svg => "image/svg+xml",
            Format::Pdf => "application/pdf",
        }
    }
}

/// `doc` at its committed State as a file in `format`, labels from `labels`
/// (drawing-export).
pub fn export(doc: &Document, labels: &LabelLines, format: Format) -> Result<Vec<u8>, ExportError> {
    let scene = scene(doc, labels)?;
    let bounds = bounds(&scene).ok_or(ExportError::NothingToExport)?;
    Ok(match format {
        Format::Svg => svg(&scene, &bounds, doc.settings()).into_bytes(),
        Format::Pdf => pdf(&scene, &bounds, doc.settings()),
    })
}

/// `doc` at its committed State as SVG text, labels from `labels`
/// (drawing-export).
pub fn export_svg(doc: &Document, labels: &LabelLines) -> Result<String, ExportError> {
    let scene = scene(doc, labels)?;
    let bounds = bounds(&scene).ok_or(ExportError::NothingToExport)?;
    Ok(svg(&scene, &bounds, doc.settings()))
}

/// Why an export produced no file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportError {
    /// No exported object draws anything.
    NothingToExport,
    /// The typeset lines of this label were not supplied (all of them, in order).
    MissingLines(GeometryId),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportError::NothingToExport => write!(f, "nothing to export"),
            ExportError::MissingLines(id) => write!(f, "typeset lines missing for label {id:?}"),
        }
    }
}

impl std::error::Error for ExportError {}
