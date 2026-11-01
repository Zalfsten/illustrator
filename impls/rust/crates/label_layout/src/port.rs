//! The typesetting port the app talks to: requests never block, results
//! arrive later (label-typesetting "Typesetting is a replaceable backend").
//!
//! An engine that answers at once ([`LineTypesetter`]) becomes a port through
//! [`Immediate`]; a browser worker client implements [`Typesetter`] itself.

use std::cell::RefCell;

use constraint_runtime::{DocumentSettings, LabelAttrs, Markup};

use crate::place::label_size;
use crate::split::split_lines;

use crate::typeset::{LineTypesetter, TypesetError, TypesetLine};

/// One line to typeset; also the key its result is delivered under.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineRequest {
    pub line: String,
    pub markup: Markup,
    /// Size in pt as bits, so that requests can be compared and hashed.
    pub size_bits: u64,
    /// Applies only to [`Markup::Latex`].
    pub preamble: String,
}

impl LineRequest {
    pub fn new(line: &str, markup: Markup, size_pt: f64, preamble: &str) -> Self {
        Self {
            line: line.to_owned(),
            markup,
            size_bits: size_pt.to_bits(),
            preamble: preamble.to_owned(),
        }
    }

    pub fn size_pt(&self) -> f64 {
        f64::from_bits(self.size_bits)
    }
}

/// The requests for a label's lines: its source split at explicit breaks, at
/// the label's size, with the document preamble. The Canvas and exports build
/// the same keys this way.
pub fn label_requests(attrs: &LabelAttrs, settings: &DocumentSettings) -> Vec<LineRequest> {
    let size = label_size(attrs, settings);
    split_lines(&attrs.source, attrs.markup)
        .into_iter()
        .map(|l| LineRequest::new(l, attrs.markup, size, &settings.preamble))
        .collect()
}

/// The result delivered for a request.
pub type LineResult = Result<TypesetLine, TypesetError>;

/// Whether a typesetter can deliver results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypesetterState {
    /// Not ready yet; requests are kept and answered later.
    Loading,
    Ready,
    /// Cannot be loaded; requests are not answered.
    Failed(String),
}

/// The asynchronous typesetting port.
pub trait Typesetter {
    /// Ask for a line; returns at once.
    fn request(&self, req: LineRequest);
    /// Results completed since the last call, each with its request.
    fn take_completed(&self) -> Vec<(LineRequest, LineResult)>;
    fn state(&self) -> TypesetterState;
}

/// Port over an engine that answers at once: a request is typeset inside
/// [`Typesetter::request`] and delivered by the next [`Typesetter::take_completed`].
pub struct Immediate<E> {
    engine: E,
    done: RefCell<Vec<(LineRequest, LineResult)>>,
}

impl<E: LineTypesetter> Immediate<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine,
            done: RefCell::new(Vec::new()),
        }
    }
}

impl<E: LineTypesetter> Typesetter for Immediate<E> {
    fn request(&self, req: LineRequest) {
        let result = self
            .engine
            .typeset_line(&req.line, req.markup, req.size_pt(), &req.preamble);
        self.done.borrow_mut().push((req, result));
    }

    fn take_completed(&self) -> Vec<(LineRequest, LineResult)> {
        std::mem::take(&mut self.done.borrow_mut())
    }

    fn state(&self) -> TypesetterState {
        TypesetterState::Ready
    }
}

#[cfg(test)]
#[path = "tests_port.rs"]
mod tests_port;
