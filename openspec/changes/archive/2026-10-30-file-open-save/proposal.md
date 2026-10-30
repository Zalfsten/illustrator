# Proposal

## Why

Figures for publications and teaching must be kept: saved next to a paper, reopened, and edited again. The file format and its load and save semantics exist since the change `sketch-file-format`. The app cannot use them yet: it has no Open or Save, and every session is lost when the window or tab closes. This change completes #8 with the host side. The app gets open and save services through the host boundary, the file commands in the UI, and a protection against losing unsaved work.

## What Changes

- **File commands in the app:** New, Open, Save and Save As.
  - Open reads a sketch file through the host and starts a new session on it.
  - Save writes the drawing to the file it came from. Save As always asks for a target.
  - New starts an empty drawing with default document settings.
  - At start the app still shows the sample drawing; it counts as a drawing without a file.
- **Host boundary:** opening and saving files become host services (system-architecture already names them as examples). They complete later and never block the input–solve–paint loop.
  - On a host that cannot overwrite a chosen file (the browser in this change), every save is a download. Save and Save As then behave the same.
- **Unsaved changes:**
  - A drawing has unsaved changes when saving it now would give other bytes than the file last opened or saved. Undoing back to the saved drawing clears the mark.
  - A file whose values were adjusted when it was loaded counts as changed from the start, because the drawing is no longer the file.
  - Before New, Open and closing the window, the app asks whether to save, discard or cancel. A browser tab can only show the browser's generic leave warning.
  - The title shows the file name and marks unsaved changes.
- **Load feedback:**
  - A rejected file leaves the current drawing open and shows the error, with the position of the failing command where there is one.
  - A loaded file's report (adjusted and frozen components) is shown.
- **View:** after Open and New, the view fits the drawing, so a loaded drawing is never off screen. Pan and zoom are not part of this change; they get their own issue.

## Non-goals

- Overwriting the same file in the browser (File System Access API). It becomes a separate issue. The host boundary is shaped so that it can be added later without changing the app.
- Pan and zoom (separate issue), the example gallery that will replace the sample drawing (#5), recent files, autosave and crash recovery.
- Compressed files, export formats (#9, #13) and styles (#10).

The product goal of durable, re-editable figures needs files that the app can open and save. This change delivers that for the native app and the browser. The behaviour of the file commands, the unsaved-changes rule and the view fit are shared specs. Dialog libraries, the browser download mechanism and keyboard shortcuts are design decisions of the Rust implementation.

## Capabilities

### New Capabilities

- `file-commands`: New, Open, Save and Save As in the app, the file a drawing belongs to, unsaved changes and the warning before losing them, load feedback, the difference between hosts that can overwrite a file and hosts that can only download, and fitting the view to an opened drawing.

### Modified Capabilities

(none) The host-boundary requirements in `system-architecture` already cover file services delivered by completion. The sketch file format and its recommended extension `.sketch.json` (`sketch-file`) stay unchanged.

## Impact

- **`sketch_ui`:**
  - `Platform` gains file services: open and save with an opaque file handle, and completion delivered into the app's frame loop.
  - New, Open, Save and Save As with keyboard shortcuts.
  - Unsaved-changes tracking by comparing bytes, and the save/discard/cancel dialog.
  - Window or page title, status texts for the load report and load errors.
  - The viewport is fitted to the drawing instead of a fixed world box.
  - Close handling: native close request; `beforeunload` in the browser.
- **`main.rs`:** native host with the `rfd` file dialogs (xdg-desktop-portal on Linux); browser host with a file input for Open and a download for Save.
- **New dependency:** `rfd` (native only).
- **`sketch_session`:** unchanged. `Session::save` and `Session::load` are used as they are.
- **Dev container:** no new packages. Native dialogs need a desktop portal on the host, and tests use a fake platform.
