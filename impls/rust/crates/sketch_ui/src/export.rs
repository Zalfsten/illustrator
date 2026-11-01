//! An SVG export in progress (drawing-export), free of egui: the drawing as
//! it was when the export was requested, waiting for its labels' typeset
//! lines through the [`LabelCache`], then the file's bytes.

use constraint_runtime::{Document, GeometryId};
use label_layout::{LineRequest, Typesetter, TypesetterState};
use sketch_export::{export_svg, exported_labels, ExportError, LabelLines};

use crate::labels::LabelCache;

/// Status line while an export waits for typesetting.
pub const WAITING: &str = "Export waiting for labels…";

/// What an export job reached in this frame.
#[derive(Debug, PartialEq)]
pub enum JobStep {
    /// Lines of exported labels are still being typeset.
    Waiting,
    /// The file's bytes, ready to hand to the host.
    Ready(Vec<u8>),
    /// No file: the error to show.
    Failed(String),
}

/// One export: a copy of the drawing and its exported labels' requests.
pub struct ExportJob {
    doc: Document,
    labels: Vec<(GeometryId, Vec<LineRequest>)>,
}

impl ExportJob {
    /// Export `doc` as it is now; later edits do not change the file.
    pub fn new(doc: &Document) -> Self {
        Self {
            labels: exported_labels(doc),
            doc: doc.clone(),
        }
    }

    /// Advance the job: request and pin missing lines, report failing
    /// labels by name, or produce the bytes once every line is there.
    pub fn step(&self, cache: &mut LabelCache, typesetter: Option<&dyn Typesetter>) -> JobStep {
        let needed: Vec<_> = self.labels.iter().filter(|(_, reqs)| !reqs.is_empty()).collect();
        let mut lines = LabelLines::new();
        if !needed.is_empty() {
            let ts = match (typesetter, typesetter.map(|t| t.state())) {
                (Some(ts), Some(TypesetterState::Loading | TypesetterState::Ready)) => ts,
                (_, Some(TypesetterState::Failed(msg))) => {
                    return self.failed(&needed, &format!("typesetter could not be loaded: {msg}"))
                }
                _ => return self.failed(&needed, "no typesetter available"),
            };
            let mut errors = Vec::new();
            let mut pending = false;
            for (id, reqs) in &needed {
                match cache.ensure(ts, reqs) {
                    None => pending = true,
                    Some(Ok(l)) => {
                        lines.insert(*id, l.iter().map(|l| (**l).clone()).collect());
                    }
                    Some(Err(e)) => errors.push(format!("{}: {e}", self.name(*id))),
                }
            }
            if !errors.is_empty() {
                return JobStep::Failed(format!("Cannot export: {}", errors.join("; ")));
            }
            if pending {
                return JobStep::Waiting;
            }
        }
        match export_svg(&self.doc, &lines) {
            Ok(text) => JobStep::Ready(text.into_bytes()),
            Err(ExportError::NothingToExport) => JobStep::Failed("Nothing to export".into()),
            Err(e) => JobStep::Failed(format!("Cannot export: {e}")),
        }
    }

    fn name(&self, id: GeometryId) -> &str {
        self.doc.geometry_name(id).unwrap_or("?")
    }

    fn failed(&self, labels: &[&(GeometryId, Vec<LineRequest>)], reason: &str) -> JobStep {
        let names: Vec<String> = labels.iter().map(|(id, _)| format!("{}: {reason}", self.name(*id))).collect();
        JobStep::Failed(format!("Cannot export: {}", names.join("; ")))
    }
}

#[cfg(test)]
#[path = "tests_export.rs"]
mod tests_export;
