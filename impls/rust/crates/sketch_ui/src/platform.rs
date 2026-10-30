//! The host boundary (system-architecture "Host services pass through a
//! single host boundary"): every host service the App uses comes from the
//! [`Platform`] supplied when it starts. Optional capabilities are `Option`s.

use std::any::Any;
use std::sync::Arc;

use label_layout::Typesetter;

/// Host services of the App.
pub trait Platform {
    /// Optional capability: label typesetting (`None` on a host without one).
    fn typesetter(&self) -> Option<&dyn Typesetter>;
    /// File services; every host can at least download.
    fn files(&self) -> &dyn Files;
}

/// Completion of a host request; called once, possibly on another thread.
pub type Done<T> = Box<dyn FnOnce(T) + Send>;
/// Result of [`Files::open`]: `None` when the user cancelled.
pub type OpenResult = Option<Result<Opened, String>>;
/// Result of [`Files::save`]: `None` when the user cancelled.
pub type SaveResult = Option<Result<FileHandle, String>>;

/// Opening and saving files (file-commands). Requests complete later through
/// their `done` callback and never block the frame.
pub trait Files {
    /// Ask the user for a file and read it.
    fn open(&self, done: Done<OpenResult>);
    /// Write `bytes` to `target`; on success `done` gets the file written.
    fn save(&self, target: SaveTarget, bytes: Vec<u8>, done: Done<SaveResult>);
    /// Whether [`SaveTarget::Overwrite`] is supported (false: every save is a download).
    fn can_overwrite(&self) -> bool;
    /// Title and unsaved mark of the drawing (window title, page title, leave warning).
    fn show_document(&self, title: &str, unsaved: bool);
}

/// Where [`Files::save`] writes.
pub enum SaveTarget {
    /// The file the drawing came from.
    Overwrite(FileHandle),
    /// A target the user chooses (or a download), suggesting `suggested_name`.
    Ask { suggested_name: String },
}

/// A file read by [`Files::open`].
pub struct Opened {
    pub handle: FileHandle,
    pub bytes: Vec<u8>,
}

/// A file as the host knows it. The App reads only its name; the host keeps
/// whatever it needs to write the file again behind it.
#[derive(Clone)]
pub struct FileHandle {
    name: String,
    host: Arc<dyn Any + Send + Sync>,
}

impl FileHandle {
    pub fn new(name: impl Into<String>, host: impl Any + Send + Sync) -> Self {
        Self {
            name: name.into(),
            host: Arc::new(host),
        }
    }

    /// The file's name as the host reports it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The host's own data, if it is a `T`.
    #[allow(dead_code)] // the browser host keeps none
    pub fn host<T: Any>(&self) -> Option<&T> {
        self.host.downcast_ref()
    }
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FileHandle").field(&self.name).finish()
    }
}

/// A host as a set of boxed services; `main.rs` builds one per target.
pub struct Host {
    typesetter: Option<Box<dyn Typesetter>>,
    files: Box<dyn Files>,
}

impl Host {
    pub fn new(typesetter: Option<Box<dyn Typesetter>>, files: Box<dyn Files>) -> Self {
        Self { typesetter, files }
    }
}

impl Platform for Host {
    fn typesetter(&self) -> Option<&dyn Typesetter> {
        self.typesetter.as_deref()
    }

    fn files(&self) -> &dyn Files {
        self.files.as_ref()
    }
}

#[cfg(test)]
#[path = "tests_platform.rs"]
mod tests_platform;
