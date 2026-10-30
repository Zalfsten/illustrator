# Design

## Context

- **`sketch_session`** already turns a drawing into bytes and back: `Session::save() -> Result<Vec<u8>, SaveError>` and `Session::load(&[u8]) -> Result<(Session, LoadReport), LoadError>`. Saving is deterministic: save, load and save again give identical bytes.
- **`sketch_ui`** has the `Platform` trait (`platform.rs`) with one optional service, `typesetter()`. A `Host` struct holds the boxed services, and `main.rs` builds one per target. `cfg(target_arch)` appears only in `main.rs` and in the Cargo target tables.
- **The App** builds the sample drawing in `App::new`. It fits the `Viewport` every frame to a fixed world box (−0.5 … 2.5). Fields that depend on the drawing: `logical_points`, `pick_points`, `pick_geoms`, `segment_clicks` and the `LabelCache`.
- **The archived change `platform-boundary` (D3)** recorded a callback shape for file services: `open_file(done)` and `save_file(name, data)`. Results go into a channel that `App::update` drains at frame start. This design follows it and adds a file handle.

## Goals / Non-Goals

**Goals:**
- The file logic (handle, baseline bytes, unsaved state, the pending action behind the save/discard/cancel question) lives in a module without egui or host code, so a fake platform can test it.
- The host boundary keeps the same shape when the browser later gains real overwriting (File System Access API).

**Non-Goals:**
- A shared host crate for other front-ends. Only `sketch_ui` uses these services.

## Decisions

### D1: File services as a `Files` trait inside `Platform`

```rust
pub trait Files {
    /// Ask the user for a file; `done(None)` on cancel.
    fn open(&self, done: Done<Option<Result<Opened, String>>>);
    /// Write `bytes` to `target` (overwrite) or ask for a target suggesting `name`.
    fn save(&self, target: SaveTarget, bytes: Vec<u8>, done: Done<Option<Result<FileHandle, String>>>);
    /// Whether `SaveTarget::Overwrite` is supported (false: every save is a download).
    fn can_overwrite(&self) -> bool;
    /// Title and unsaved mark of the drawing (window title, page title, leave warning).
    fn show_document(&self, title: &str, unsaved: bool);
}
pub type Done<T> = Box<dyn FnOnce(T) + Send>;
pub enum SaveTarget { Overwrite(FileHandle), Ask { suggested_name: String } }
pub struct Opened { pub handle: FileHandle, pub bytes: Vec<u8> }
pub struct FileHandle { name: String, host: Arc<dyn Any + Send + Sync> }
```

- `Platform` gains `fn files(&self) -> &dyn Files`. It is a required service, not an `Option`: every host can at least download.
- **`FileHandle` is opaque.** The App reads only `name()`. The host puts what it needs behind `host`: natively a `PathBuf`, in the browser nothing (`()`) for now, later a File System Access handle.
- **`None` means cancelled, `Some(Err(msg))` means the host failed.** The App does not interpret host errors; it shows the message.
- **Completion is a `Send` callback.** The native host runs dialogs and writes on a thread. The App's callbacks only send a `FileEvent` into its `mpsc` channel and call `ctx.request_repaint()`. `update` drains the channel at frame start (as in D3).
- *Alternatives:*
  - `async fn` in the trait: needs an executor natively and boxing for object safety (already rejected in D3).
  - Separate `Platform` methods without a sub-trait: works, but `Host` would grow one field per method. A `Files` object keeps the native and browser implementations together.

### D2: Native host with `rfd` on a thread

- `rfd` 0.17 with its default features (`xdg-portal`, `wayland`). On Linux it talks to xdg-desktop-portal over D-Bus and needs no GTK. On Windows and macOS it uses the system dialogs.
- **Each request spawns a `std::thread`** that runs `pollster::block_on(rfd::AsyncFileDialog::…)`. Then it reads with `std::fs::read`, or writes with `std::fs::write`, and calls `done`.
  - The async dialog also works on macOS, where a dialog must run on the main thread; `rfd` dispatches there itself.
  - Overwrite (`SaveTarget::Overwrite`) writes on a thread as well, so a slow disk never stalls a frame.
- **Filters:** "Sketch" `*.sketch.json` and "All files". The format is recognized by content (sketch-file).
- `show_document` sends `ViewportCommand::Title` through the `egui::Context` that the native host keeps. It ignores the unsaved mark: the App handles closing itself (D6).
- *Alternatives:*
  - `rfd` with the GTK3 backend: needs GTK in the container and on every Linux machine.
  - An egui-internal dialog (`egui-file-dialog`): not native (decided in explore: option a).

### D3: Browser host with a file input and a download

- **Open:**
  - Creates a hidden `<input type="file" accept=".json">` and calls `click()` on it.
  - The `change` event reads `File::array_buffer()` with `wasm_bindgen_futures`. The `cancel` event reports `None`.
  - The handle carries the file's name.
- **Save:**
  - Always `SaveTarget::Ask` (`can_overwrite() == false`).
  - Wraps the bytes in a `Blob`, makes an object URL, clicks an `<a download="name">`, then revokes the URL.
  - The download is reported as success with a handle carrying the suggested name.
- **`show_document`:**
  - sets `document.title`;
  - keeps the unsaved flag in a cell that a `beforeunload` listener reads. When the flag is set, the listener calls `preventDefault()` and the browser shows its generic warning.
- **Programmatic `click()`:** egui handles a button click one frame after the DOM event. Browsers allow `click()` on file inputs and downloads during a few seconds of transient user activation after the click, so this works. `rfd`'s browser backend avoids the question with an overlay card that needs a second click; we take the simpler UI and accept the risk below.
- The module needs `web-sys` features `Blob`, `BlobPropertyBag`, `Url`, `HtmlAnchorElement`, `HtmlInputElement`, `File`, `FileList`, `Event` and `BeforeUnloadEvent`, plus `js-sys`.

### D4: Host modules in `sketch_ui`, selected in `main.rs`

- `src/host_native.rs` and `src/host_web.rs` hold the two `Files` implementations.
- `main.rs` declares each with `#[cfg(...)] mod …;`, so `cfg` stays in `main.rs`. The modules themselves contain none.
- `rfd` and `pollster` go into the native target table; `js-sys` and the extra `web-sys` features into the wasm table.
- *Alternative:* separate crates, as with `label_web`. That pays off for code that other crates use or that builds on its own (the worker). These modules are small and only `main.rs` uses them.

### D5: The file state as its own module

`src/files.rs` holds `FileState`:

- `handle: Option<FileHandle>`;
- `baseline: Vec<u8>`, the bytes at the last open or save (at start: the drawing's own bytes);
- `unsaved: bool`;
- `pending: Option<Pending>`, the action waiting behind the question or a save: `New`, `Open` or `Close`;
- `request: Option<Request>`, the host request in flight, so file commands are disabled until it completes.

The App calls `FileState` at these points:
- `refresh(&Session)` after every successful history change: command, undo, redo, drag end. It sets `unsaved = session.save() != baseline`.
- `saved(handle, bytes)` and `opened(handle, session)` when a request completes.

The control flow (ask → save → continue with the pending action) is a small state machine:

```
 New/Open/Close --unsaved?--no--> do it
        |yes
        v
   ask dialog --cancel--> pending = None
        |discard ------------------------> do it
        |save
        v
   save request --fail/cancel--> pending = None, show error
        |ok
        v
      do it
```

- Unit tests in `src/tests_files.rs` run it with a `FakeFiles`. The fake records requests and lets the test complete them with chosen results.
- **If `session.save()` fails** (`SaveError` for a drawing built outside the command layer), the drawing counts as unsaved, and Save shows the error. The sample drawing is built from addressable expressions, so this does not happen today.
- *Alternative:* a revision counter in the session, rejected in explore. It would treat a drawing adjusted on load as unchanged and needs a session change.

### D6: Closing the native window

- In `update`, the App checks `ctx.input(|i| i.viewport().close_requested())`.
- **If the drawing is unsaved** and closing is not yet confirmed, it sends `ViewportCommand::CancelClose` and starts the flow with `Pending::Close`.
- **When the flow completes,** it sets `close_confirmed` and sends `ViewportCommand::Close`.
- In the browser `close_requested` never arrives; there the leave warning from D3 applies.

### D7: View fit

- The fixed `WORLD_MIN`/`WORLD_MAX` constants become an App field `world_box`. The Viewport is still fitted to it every frame, so window resizes keep working.
- **After New and Open,** `world_box` becomes the bounding box of the mesh edge points and visible label origins.
  - The box grows by 10 % per side.
  - A degenerate box (one point, or a line) grows to at least 1 world unit per side.
  - An empty drawing gets the old default box.
- At start the App keeps the old box for the sample.
- Label text extents are left out (spec: they may still be typesetting).

### D8: Keyboard shortcuts and menu

- A "File" menu button in the top panel: New, Open…, Save, Save As….
- Shortcuts through `consume_shortcut`: `COMMAND+N`, `COMMAND+O`, `COMMAND+S`, `COMMAND+SHIFT+S`.
- File commands are disabled while a drag is in progress or a host request is in flight.
- In the browser some shortcuts belong to the browser (Ctrl+N, Ctrl+O) and never reach egui. The menu covers them.

### D9: Messages

- **Open:**
  - `Opened <name>`;
  - with adjustments: `— adjusted: seg1, arc1 (max 0.003)`;
  - with frozen components: `— frozen: 2 components`.
- **Load errors** go to the error line through `LoadError`'s `Display`, which already names the command index.
- **Save:** `Saved <name>`, or `Downloaded <name>` on a host that cannot overwrite.
- **Dialog:** a modal `egui::Window` titled "Unsaved changes", with the buttons Save, Discard and Cancel.

## Risks / Trade-offs

- [Native dialog without a desktop portal (minimal Linux setups, the dev container)] → `rfd` returns no file, so the request ends as a cancel. The App shows "no file dialog available" if the request returns at once with nothing. Tests use the fake.
- [A browser refuses the programmatic `click()` because transient activation expired (very slow frame)] → nothing happens and the user clicks again. If this turns out to matter, switch to `rfd`'s overlay card for the browser host only. The `Files` trait does not change.
- [The input's `cancel` event is missing in older browsers] → the request stays in flight and file commands stay disabled. Mitigation: a new Open replaces a stale browser request instead of waiting for it.
- [`Arc<dyn Any + Send + Sync>` does not fit a File System Access handle (a `JsValue`, not `Send`)] → wasm is single-threaded, so a later change can wrap it in a `Send` wrapper there. The App-facing API does not change.
- [Comparing bytes after each step costs a full serialization] → it runs once per history step, not per frame or drag move. Drawings are small. Measure if drawings grow large.

## Migration Plan

No migration: files and sessions are unchanged. The app gains commands, and the start scene is the same sample.
