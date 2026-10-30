//! Typst's view for one line, and its layout.

use std::collections::HashMap;
use std::sync::OnceLock;

use label_layout::TypesetError;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_layout::PagedDocument;

const MATH_FONT: &[u8] = include_bytes!("../assets/fonts/NewCMMath-Regular.otf");
const TEXT_FONT: &[u8] = include_bytes!("../assets/fonts/NewCM10-Regular.otf");

/// mitex's Typst definitions (Apache-2.0), with xarrow replaced.
const MITEX_FILES: &[(&str, &str)] = &[
    ("/mitex/mod.typ", include_str!("../assets/mitex/mod.typ")),
    (
        "/mitex/prelude.typ",
        include_str!("../assets/mitex/prelude.typ"),
    ),
    (
        "/mitex/latex/standard.typ",
        include_str!("../assets/mitex/latex/standard.typ"),
    ),
];

/// Size of the invisible marker set right after the line: the page frame is
/// flat, so the marker's text position is the line's baseline.
pub const MARKER_PT: f64 = 0.125;

/// Lay out one line of Typst markup at `size_pt`, followed by the marker.
pub fn layout(markup: &str, size_pt: f64) -> Result<PagedDocument, TypesetError> {
    let world = LineWorld::new(markup, size_pt);
    typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|errors| TypesetError::Invalid {
            message: errors
                .iter()
                .map(|e| e.message.to_string())
                .collect::<Vec<_>>()
                .join("; "),
        })
}

struct LineWorld {
    main: Source,
    shared: &'static Shared,
}

/// Loaded once per process: library, fonts and mitex files.
struct Shared {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    files: HashMap<FileId, Source>,
}

fn shared() -> &'static Shared {
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED.get_or_init(|| {
        let fonts: Vec<Font> = [MATH_FONT, TEXT_FONT]
            .into_iter()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();
        let files = MITEX_FILES
            .iter()
            .map(|(path, text)| {
                let id = project_file(path);
                (id, Source::new(id, (*text).to_owned()))
            })
            .collect();
        Shared {
            library: LazyHash::new(Library::builder().build()),
            book: LazyHash::new(FontBook::from_fonts(&fonts)),
            fonts,
            files,
        }
    })
}

fn project_file(path: &str) -> FileId {
    RootedPath::new(VirtualRoot::Project, VirtualPath::new(path).unwrap()).intern()
}

/// `s` as the body of a Typst string literal.
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

impl LineWorld {
    fn new(markup: &str, size_pt: f64) -> Self {
        let text = format!(
            "#set page(width: auto, height: auto, margin: 0pt, fill: none)\n\
             #set text(font: \"New Computer Modern\", size: {size_pt}pt)\n\
             #import \"/mitex/mod.typ\": mitex-scope\n\
             #box(eval(\"{}\", mode: \"markup\", scope: mitex-scope))#box(width: 0pt, text(size: {MARKER_PT}pt)[x])\n",
            escape(markup)
        );
        Self {
            main: Source::new(project_file("/main.typ"), text),
            shared: shared(),
        }
    }
}

impl World for LineWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.shared.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.shared.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        self.shared
            .files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.get().vpath().get_without_slash().into()))
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.source(id)
            .map(|s| Bytes::from_string(s.text().to_owned()))
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.shared.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        None
    }
}
