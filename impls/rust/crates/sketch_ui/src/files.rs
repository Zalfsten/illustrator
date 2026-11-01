//! The drawing's file and unsaved changes (file-commands), free of egui and
//! host code.
//!
//! [`FileState`] keeps the file a drawing belongs to and the bytes it held
//! when last opened or saved. The drawing has unsaved changes exactly when
//! saving it now would give other bytes. New, Open and Close on an unsaved
//! drawing first ask; a save started from that question continues with the
//! waiting action only once it succeeded.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, OnceLock};

use constraint_runtime::Document;
use sketch_session::{LoadReport, Session};

use crate::platform::{Done, FileHandle, Files, OpenResult, SaveResult, SaveTarget};

/// Suggested name when saving a drawing without a file.
pub const UNTITLED_FILE: &str = "untitled.sketch.json";

/// Suggested name when exporting a drawing without a file.
pub const UNTITLED_EXPORT: &str = "untitled.svg";

/// A file command of the App.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileCommand {
    New,
    Open,
    Save,
    SaveAs,
    /// Close the App (the native window).
    Close,
}

/// The user's answer to the unsaved-changes question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Save,
    Discard,
    Cancel,
}

/// An action that replaces or discards the drawing; it waits behind the
/// question or a save.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pending {
    New,
    Open,
    Close,
}

/// The host request in flight.
enum Request {
    Open,
    /// Writing `bytes`; `then` continues once it succeeded.
    Save {
        bytes: Vec<u8>,
        then: Option<Pending>,
    },
    /// Writing an export; the drawing's file stays as it is.
    Export,
}

/// A completed host request, sent by its callback into the App's channel.
pub enum FileEvent {
    Opened(OpenResult),
    Saved(SaveResult),
    Exported(SaveResult),
}

/// What the App must do after a file step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// The session was replaced (New or Open): reset picks and caches, fit the view.
    Replaced,
    /// Closing may go ahead.
    Close,
}

/// Result of a file step: an effect and a status message (`Err` for the error line).
#[derive(Debug, Default, PartialEq)]
pub struct Step {
    pub effect: Option<Effect>,
    pub message: Option<Result<String, String>>,
}

impl Step {
    fn error(e: String) -> Self {
        Self {
            effect: None,
            message: Some(Err(e)),
        }
    }
}

type Waker = Arc<OnceLock<Box<dyn Fn() + Send + Sync>>>;

/// The drawing's file, its saved bytes and the file commands in progress.
pub struct FileState {
    handle: Option<FileHandle>,
    /// Bytes at the last open or save (at start: the drawing's own bytes);
    /// `None` when the start drawing could not be saved.
    baseline: Option<Vec<u8>>,
    unsaved: bool,
    /// The action behind the unsaved-changes question, while it is shown.
    asking: Option<Pending>,
    request: Option<Request>,
    /// Title last handed to the host.
    shown: Option<String>,
    tx: Sender<FileEvent>,
    rx: Receiver<FileEvent>,
    /// Called after a callback sent its event (the App requests a repaint).
    waker: Waker,
}

impl FileState {
    /// A drawing without a file; `session`'s drawing counts as saved.
    pub fn new(session: &Session) -> Self {
        let (tx, rx) = mpsc::channel();
        let baseline = session.save().ok();
        Self {
            handle: None,
            unsaved: baseline.is_none(),
            baseline,
            asking: None,
            request: None,
            shown: None,
            tx,
            rx,
            waker: Arc::default(),
        }
    }

    /// Set the function called when a host request completes (once).
    pub fn set_waker(&self, wake: impl Fn() + Send + Sync + 'static) {
        let _ = self.waker.set(Box::new(wake));
    }

    /// Name of the drawing's file, if it has one.
    pub fn file_name(&self) -> Option<&str> {
        self.handle.as_ref().map(FileHandle::name)
    }

    pub fn unsaved(&self) -> bool {
        self.unsaved
    }

    /// The file name (or `untitled`), with `*` when unsaved.
    pub fn title(&self) -> String {
        let name = self.file_name().unwrap_or("untitled");
        let mark = if self.unsaved { "*" } else { "" };
        format!("{name}{mark}")
    }

    /// Whether the unsaved-changes question is shown.
    pub fn asking(&self) -> bool {
        self.asking.is_some()
    }

    /// Whether file commands are blocked (a question or a host request is open).
    pub fn busy(&self) -> bool {
        self.asking.is_some() || self.request.is_some()
    }

    /// Suggested export name: the drawing's file name with its sketch
    /// extension replaced by `.svg`, or [`UNTITLED_EXPORT`].
    pub fn export_name(&self) -> String {
        match self.file_name() {
            None => UNTITLED_EXPORT.to_owned(),
            Some(name) => {
                let stem = name
                    .strip_suffix(".sketch.json")
                    .or_else(|| name.strip_suffix(".json"))
                    .unwrap_or(name);
                format!("{stem}.svg")
            }
        }
    }

    /// Hand exported `bytes` to the host under [`export_name`](Self::export_name).
    /// Returns `false`, starting nothing, while [`busy`](Self::busy); the
    /// drawing's file and unsaved state never change (drawing-export).
    pub fn export(&mut self, bytes: Vec<u8>, files: &dyn Files) -> bool {
        if self.busy() {
            return false;
        }
        self.request = Some(Request::Export);
        let target = SaveTarget::Ask {
            suggested_name: self.export_name(),
        };
        files.save(target, bytes, self.done(FileEvent::Exported));
        true
    }

    /// Recompute the unsaved state after a history change and show the title.
    pub fn refresh(&mut self, session: &Session, files: &dyn Files) {
        self.unsaved = match (&self.baseline, session.save()) {
            (Some(base), Ok(now)) => *base != now,
            _ => true,
        };
        let title = self.title();
        if self.shown.as_ref() != Some(&title) {
            files.show_document(&title, self.unsaved);
            self.shown = Some(title);
        }
    }

    /// Start `cmd`. Does nothing while dragging or while [`busy`](Self::busy).
    pub fn command(&mut self, cmd: FileCommand, session: &mut Session, files: &dyn Files) -> Step {
        if session.is_dragging() || self.busy() {
            return Step::default();
        }
        let pending = match cmd {
            FileCommand::Save => return self.save(false, None, session, files),
            FileCommand::SaveAs => return self.save(true, None, session, files),
            FileCommand::New => Pending::New,
            FileCommand::Open => Pending::Open,
            FileCommand::Close => Pending::Close,
        };
        if self.unsaved {
            self.asking = Some(pending);
            Step::default()
        } else {
            self.proceed(pending, session, files)
        }
    }

    /// Answer the unsaved-changes question.
    pub fn answer(&mut self, answer: Answer, session: &mut Session, files: &dyn Files) -> Step {
        let Some(pending) = self.asking.take() else {
            return Step::default();
        };
        match answer {
            Answer::Cancel => Step::default(),
            Answer::Discard => self.proceed(pending, session, files),
            Answer::Save => self.save(false, Some(pending), session, files),
        }
    }

    /// Handle a completed host request, if any. Waits while dragging.
    pub fn poll(&mut self, session: &mut Session, files: &dyn Files) -> Step {
        if session.is_dragging() {
            return Step::default();
        }
        let Ok(event) = self.rx.try_recv() else {
            return Step::default();
        };
        let request = self.request.take();
        match event {
            FileEvent::Opened(None) => Step::default(),
            FileEvent::Opened(Some(Err(e))) => Step::error(format!("Open failed: {e}")),
            FileEvent::Opened(Some(Ok(opened))) => match Session::load(&opened.bytes) {
                Ok((loaded, report)) => {
                    let message = open_message(opened.handle.name(), &report);
                    *session = loaded;
                    self.handle = Some(opened.handle);
                    self.baseline = Some(opened.bytes);
                    self.refresh(session, files);
                    Step {
                        effect: Some(Effect::Replaced),
                        message: Some(Ok(message)),
                    }
                }
                Err(e) => Step::error(format!("Not opened — drawing unchanged. {e}")),
            },
            FileEvent::Saved(None) => Step::default(),
            FileEvent::Saved(Some(Err(e))) => Step::error(format!("Save failed: {e}")),
            FileEvent::Saved(Some(Ok(handle))) => {
                let Some(Request::Save { bytes, then }) = request else {
                    return Step::default();
                };
                let verb = if files.can_overwrite() {
                    "Saved"
                } else {
                    "Downloaded"
                };
                let saved = format!("{verb} {}", handle.name());
                self.handle = Some(handle);
                self.baseline = Some(bytes);
                self.refresh(session, files);
                let next = match then {
                    Some(p) => self.proceed(p, session, files),
                    None => Step::default(),
                };
                Step {
                    effect: next.effect,
                    message: next.message.or(Some(Ok(saved))),
                }
            }
            FileEvent::Exported(None) => Step::default(),
            FileEvent::Exported(Some(Err(e))) => Step::error(format!("Export failed: {e}")),
            FileEvent::Exported(Some(Ok(handle))) => {
                let verb = if files.can_overwrite() {
                    "Exported"
                } else {
                    "Downloaded"
                };
                Step {
                    effect: None,
                    message: Some(Ok(format!("{verb} {}", handle.name()))),
                }
            }
        }
    }

    /// Carry out an action that replaces or discards the drawing.
    fn proceed(&mut self, pending: Pending, session: &mut Session, files: &dyn Files) -> Step {
        match pending {
            Pending::New => {
                *session = Session::new(Document::new());
                self.handle = None;
                self.baseline = session.save().ok();
                self.refresh(session, files);
                Step {
                    effect: Some(Effect::Replaced),
                    message: Some(Ok("New drawing".into())),
                }
            }
            Pending::Open => {
                self.request = Some(Request::Open);
                files.open(self.done(FileEvent::Opened));
                Step::default()
            }
            Pending::Close => Step {
                effect: Some(Effect::Close),
                message: None,
            },
        }
    }

    /// Write the drawing: to its file, or (`ask`, no file, or a download-only
    /// host) to a target the host asks for. `then` continues after success.
    fn save(
        &mut self,
        ask: bool,
        then: Option<Pending>,
        session: &Session,
        files: &dyn Files,
    ) -> Step {
        let bytes = match session.save() {
            Ok(b) => b,
            Err(e) => return Step::error(e.to_string()),
        };
        let target = match &self.handle {
            Some(h) if !ask && files.can_overwrite() => SaveTarget::Overwrite(h.clone()),
            h => SaveTarget::Ask {
                suggested_name: h
                    .as_ref()
                    .map_or(UNTITLED_FILE, FileHandle::name)
                    .to_owned(),
            },
        };
        self.request = Some(Request::Save {
            bytes: bytes.clone(),
            then,
        });
        files.save(target, bytes, self.done(FileEvent::Saved));
        Step::default()
    }

    /// Callback that sends its result as an event and wakes the App.
    fn done<T: 'static>(&self, event: fn(T) -> FileEvent) -> Done<T> {
        let tx = self.tx.clone();
        let waker = self.waker.clone();
        Box::new(move |result| {
            let _ = tx.send(event(result));
            if let Some(wake) = waker.get() {
                wake();
            }
        })
    }
}

/// `Opened <name>`, with the load report's adjusted and frozen components.
pub fn open_message(name: &str, report: &LoadReport) -> String {
    let mut s = format!("Opened {name}");
    if !report.adjusted.is_empty() {
        let names: Vec<&str> = report
            .adjusted
            .iter()
            .flat_map(|a| a.geometries.iter().map(String::as_str))
            .collect();
        let max = report
            .adjusted
            .iter()
            .map(|a| a.displacement)
            .fold(0.0, f64::max);
        let max = if max >= 1e-3 {
            format!("{max:.3}")
        } else {
            format!("{max:.1e}")
        };
        s.push_str(&format!(" — adjusted: {} (max {max})", names.join(", ")));
    }
    match report.conflicts.len() {
        0 => {}
        1 => s.push_str(" — frozen: 1 component"),
        n => s.push_str(&format!(" — frozen: {n} components")),
    }
    s
}

#[cfg(test)]
#[path = "tests_files.rs"]
mod tests_files;
