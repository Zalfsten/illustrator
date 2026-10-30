//! A [`Typesetter`] backed by Typst, with LaTeX converted by mitex.
//!
//! - [`convert`]: a line's source to Typst markup (mitex for `latex`, an
//!   escaped string for `text`), with unsupported constructs named.
//! - [`world`]: Typst's view for one line: two embedded fonts (New Computer
//!   Modern), mitex's Typst definitions, no packages, no date.
//! - [`frame`]: the laid-out page as outlines in the label frame.
//!
//! Typst and mitex are pinned to one version each; the mitex files under
//! `assets/` belong to that version.

mod convert;
mod frame;
mod world;

use constraint_runtime::Markup;
use label_layout::{TypesetError, TypesetLine, Typesetter};

/// Typesets label lines with Typst; cheap to create (fonts and the library
/// are shared and loaded on first use).
#[derive(Clone, Copy, Debug, Default)]
pub struct TypstTypesetter;

impl TypstTypesetter {
    pub fn new() -> Self {
        Self
    }
}

impl Typesetter for TypstTypesetter {
    fn typeset_line(
        &self,
        line: &str,
        markup: Markup,
        size_pt: f64,
        preamble: &str,
    ) -> Result<TypesetLine, TypesetError> {
        let content = match markup {
            Markup::Latex => convert::latex(preamble, line)?,
            Markup::Text => convert::text(line),
        };
        let doc = world::layout(&content, size_pt)?;
        Ok(frame::line(&doc))
    }
}
