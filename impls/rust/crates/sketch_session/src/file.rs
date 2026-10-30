//! The native sketch file: a versioned envelope around the drawing's
//! document settings and declarative command list.
//!
//! Saving writes a fixed layout (one command per line, so a change to one
//! object changes one line); loading reads any JSON layout, executes the
//! commands in load mode and starts a new session on the result. Both work on
//! bytes only; hosts move the bytes.

use std::fmt;

use constraint_runtime::{Document, DocumentSettings};
use serde_json::{Map, Value};

use crate::command::{CommandError, DeclCommand, ExportError, LoadReport};
use crate::json::{ReadError, WriteError};
use crate::session::{Mode, Session};

/// Value of the envelope's `format` field.
pub const FORMAT: &str = "illustrator-sketch";
/// The file version this implementation writes; it reads 1 to this one.
pub const VERSION: u64 = 2;

/// Envelope fields of the document settings (since version 2).
const SETTINGS_FIELDS: [&str; 3] = ["scale", "font_size", "preamble"];

/// Why the drawing cannot be saved.
#[derive(Debug)]
pub enum SaveError {
    Export(ExportError),
    Write(WriteError),
}

/// Why a file was not loaded. No drawing results from a failed load.
#[derive(Debug)]
pub enum LoadError {
    /// The bytes are not JSON text (with the parser's message, line and column).
    NotASketchFile(String),
    /// JSON, but `format` is missing or not [`FORMAT`].
    UnknownFormat,
    /// A `version` this reader does not know, as written in the file.
    UnsupportedVersion(String),
    /// The envelope has an unknown field, or a field is missing or misshapen.
    Envelope(String),
    /// The command at `index` (counting from 0) failed.
    Command {
        index: usize,
        error: CommandLoadError,
    },
}

/// Why one command of a file failed.
#[derive(Debug)]
pub enum CommandLoadError {
    /// Not a command of the declarative subset in interchange form.
    Read(ReadError),
    /// Readable, but executing it failed.
    Execute(CommandError),
}

impl Session {
    /// The drawing as a sketch file in the standard layout.
    pub fn save(&self) -> Result<Vec<u8>, SaveError> {
        let commands = self.declarative_commands().map_err(SaveError::Export)?;
        let settings = self.settings();
        let json = |v: Value| serde_json::to_string(&v).expect("JSON values serialize");
        let mut out = format!("{{\n  \"format\": \"{FORMAT}\",\n  \"version\": {VERSION},\n");
        for (field, value) in [
            ("scale", Value::from(settings.scale)),
            ("font_size", Value::from(settings.font_size)),
            ("preamble", Value::from(settings.preamble.as_str())),
        ] {
            out.push_str(&format!("  \"{field}\": {},\n", json(value)));
        }
        if commands.is_empty() {
            out.push_str("  \"commands\": []\n}\n");
            return Ok(out.into_bytes());
        }
        out.push_str("  \"commands\": [\n");
        for (i, c) in commands.iter().enumerate() {
            if i > 0 {
                out.push_str(",\n");
            }
            out.push_str("    ");
            out.push_str(&c.to_json().map_err(SaveError::Write)?);
        }
        out.push_str("\n  ]\n}\n");
        Ok(out.into_bytes())
    }

    /// A new session on the drawing in `bytes`, with what its start changed.
    pub fn load(bytes: &[u8]) -> Result<(Session, LoadReport), LoadError> {
        let value: Value =
            serde_json::from_slice(bytes).map_err(|e| LoadError::NotASketchFile(e.to_string()))?;
        let Value::Object(mut envelope) = value else {
            return Err(LoadError::UnknownFormat);
        };
        if envelope.remove("format") != Some(Value::String(FORMAT.into())) {
            return Err(LoadError::UnknownFormat);
        }
        let version = match envelope.remove("version") {
            None => return Err(missing("version")),
            Some(v) => v
                .as_u64()
                .filter(|v| (1..=VERSION).contains(v))
                .ok_or_else(|| LoadError::UnsupportedVersion(v.to_string()))?,
        };
        let mut envelope = migrate(version, envelope)?;
        let settings = settings(&mut envelope)?;
        let Some(commands) = envelope.remove("commands") else {
            return Err(missing("commands"));
        };
        if let Some(field) = envelope.keys().next() {
            return Err(LoadError::Envelope(format!("unknown field `{field}`")));
        }
        let Value::Array(commands) = commands else {
            return Err(LoadError::Envelope("`commands` is not an array".into()));
        };

        let mut doc = Document::new();
        doc.set_settings(settings);
        let mut session = Session::new(doc);
        for (index, value) in commands.into_iter().enumerate() {
            let fail = |error| LoadError::Command { index, error };
            let cmd: DeclCommand = serde_json::from_value(value)
                .map_err(|e| fail(CommandLoadError::Read(ReadError(e))))?;
            session
                .execute(cmd.into(), Mode::Load)
                .map_err(|e| fail(CommandLoadError::Execute(e)))?;
        }
        Ok(Session::start(session.into_document()))
    }
}

/// Bring an envelope of `version` to the current version.
///
/// Version 1 has no settings fields; it gets the default settings.
fn migrate(version: u64, mut envelope: Map<String, Value>) -> Result<Map<String, Value>, LoadError> {
    if version == 1 {
        if let Some(field) = SETTINGS_FIELDS.iter().find(|f| envelope.contains_key(**f)) {
            return Err(LoadError::Envelope(format!("unknown field `{field}`")));
        }
        let d = DocumentSettings::default();
        envelope.insert("scale".into(), d.scale.into());
        envelope.insert("font_size".into(), d.font_size.into());
        envelope.insert("preamble".into(), d.preamble.into());
    }
    Ok(envelope)
}

/// Take the settings fields out of a current-version envelope and check them.
fn settings(envelope: &mut Map<String, Value>) -> Result<DocumentSettings, LoadError> {
    let mut take = |field: &str| envelope.remove(field).ok_or_else(|| missing(field));
    let positive = |field: &str, v: Value| match v.as_f64() {
        Some(x) if x.is_finite() && x > 0.0 => Ok(x),
        Some(_) => Err(LoadError::Envelope(format!(
            "invalid value of `{field}`: not greater than zero"
        ))),
        None => Err(LoadError::Envelope(format!("`{field}` is not a number"))),
    };
    let scale = positive("scale", take("scale")?)?;
    let font_size = positive("font_size", take("font_size")?)?;
    let Value::String(preamble) = take("preamble")? else {
        return Err(LoadError::Envelope("`preamble` is not a string".into()));
    };
    Ok(DocumentSettings {
        scale,
        font_size,
        preamble,
    })
}

fn missing(field: &str) -> LoadError {
    LoadError::Envelope(format!("missing field `{field}`"))
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Export(e) => write!(f, "cannot save: {e}"),
            SaveError::Write(e) => write!(f, "cannot save: {e}"),
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::NotASketchFile(e) => write!(f, "not a sketch file: {e}"),
            LoadError::UnknownFormat => {
                write!(f, "unknown format: expected \"format\": \"{FORMAT}\"")
            }
            LoadError::UnsupportedVersion(v) => write!(f, "unsupported version {v}"),
            LoadError::Envelope(e) => write!(f, "invalid sketch file: {e}"),
            LoadError::Command { index, error } => write!(f, "command {index}: {error}"),
        }
    }
}

impl fmt::Display for CommandLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandLoadError::Read(e) => write!(f, "{e}"),
            CommandLoadError::Execute(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SaveError {}
impl std::error::Error for LoadError {}
impl std::error::Error for CommandLoadError {}

#[cfg(test)]
#[path = "tests_file.rs"]
mod tests_file;
