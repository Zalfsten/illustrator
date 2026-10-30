//! File services of the native host: system file dialogs through `rfd`
//! (xdg-desktop-portal on Linux) and `std::fs`, each request on its own
//! thread so the frame loop never waits.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui;

use crate::platform::{Done, FileHandle, Files, OpenResult, Opened, SaveResult, SaveTarget};

/// Window title after the drawing's title.
const APP_TITLE: &str = "Constraint Sketch";
/// A dialog that returns nothing faster than this did not show (no portal).
const NO_DIALOG: Duration = Duration::from_millis(150);

pub struct NativeFiles {
    ctx: egui::Context,
}

impl NativeFiles {
    pub fn new(ctx: egui::Context) -> Self {
        Self { ctx }
    }
}

fn dialog() -> rfd::AsyncFileDialog {
    rfd::AsyncFileDialog::new()
        .add_filter("Sketch", &["sketch.json"])
        .add_filter("All files", &["*"])
}

/// Run `pick` (a dialog); `None` with an error when no dialog could be shown.
fn pick(
    pick: impl std::future::Future<Output = Option<rfd::FileHandle>>,
) -> Result<Option<PathBuf>, String> {
    let start = Instant::now();
    match pollster::block_on(pick) {
        Some(f) => Ok(Some(f.path().to_path_buf())),
        None if start.elapsed() < NO_DIALOG => {
            Err("no file dialog available (is xdg-desktop-portal running?)".into())
        }
        None => Ok(None),
    }
}

fn handle(path: &Path) -> FileHandle {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    FileHandle::new(name, path.to_path_buf())
}

fn write(path: PathBuf, bytes: &[u8]) -> Result<FileHandle, String> {
    std::fs::write(&path, bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(handle(&path))
}

impl Files for NativeFiles {
    fn open(&self, done: Done<OpenResult>) {
        std::thread::spawn(move || {
            let result = match pick(dialog().set_title("Open drawing").pick_file()) {
                Ok(None) => None,
                Ok(Some(path)) => Some(
                    std::fs::read(&path)
                        .map(|bytes| Opened {
                            handle: handle(&path),
                            bytes,
                        })
                        .map_err(|e| format!("{}: {e}", path.display())),
                ),
                Err(e) => Some(Err(e)),
            };
            done(result);
        });
    }

    fn save(&self, target: SaveTarget, bytes: Vec<u8>, done: Done<SaveResult>) {
        std::thread::spawn(move || {
            let result = match target {
                SaveTarget::Overwrite(h) => match h.host::<PathBuf>() {
                    Some(path) => Some(write(path.clone(), &bytes)),
                    None => Some(Err("not a file of this host".into())),
                },
                SaveTarget::Ask { suggested_name } => {
                    let save = dialog()
                        .set_title("Save drawing")
                        .set_file_name(suggested_name)
                        .save_file();
                    match pick(save) {
                        Ok(None) => None,
                        Ok(Some(path)) => Some(write(path, &bytes)),
                        Err(e) => Some(Err(e)),
                    }
                }
            };
            done(result);
        });
    }

    fn can_overwrite(&self) -> bool {
        true
    }

    fn show_document(&self, title: &str, _unsaved: bool) {
        // The App asks before closing an unsaved window itself.
        self.ctx
            .send_viewport_cmd(egui::ViewportCommand::Title(format!(
                "{title} — {APP_TITLE}"
            )));
    }
}
