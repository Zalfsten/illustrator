# Tasks

## 1. Host boundary for files

- [ ] 1.1 In `sketch_ui/src/platform.rs`, add the `Files` trait, `Done`, `SaveTarget`, `Opened` and opaque `FileHandle` (with `name()`), and `Platform::files()`. Extend `Host::new` with a boxed `Files`. Verify with `cargo build -p sketch_ui` and `scripts/check-wasm.sh`. Until 2.x and 4.x land, the existing call sites use a placeholder host that cancels every request.
- [ ] 1.2 Add `FakeFiles` (test only) and `Host` construction for tests. It records `open`/`save` requests and `show_document` calls and lets a test complete a request with cancel, error or success. Verify with a test in `src/tests_platform.rs` that completes one request of each kind.

## 2. File state and flow (no egui, no host)

- [ ] 2.1 Create `src/files.rs` with `FileState` (handle, baseline bytes, unsaved, pending action, request in flight) and `refresh(&Session)`, which compares `session.save()` with the baseline; a save error counts as unsaved. Verify with `src/tests_files.rs`:
  - an edit makes the state unsaved;
  - undo back to the baseline clears it;
  - a different edit after undo sets it again;
  - a fresh session on the sample is not unsaved.
- [ ] 2.2 Implement the flow for New, Open and Close: ask when unsaved; save, discard or cancel; Save without a handle or on a host without overwrite becomes Save As; the pending action continues only after a successful save; a request in flight and an active drag block file commands. Verify with `tests_files.rs` against `FakeFiles`, covering each spec scenario of "Unsaved changes are not discarded silently", "Save writes the drawing's file", "Save As asks for a target", "Hosts that can only download" and "A failed save keeps the unsaved state".
- [ ] 2.3 Implement the completion of Open: `Session::load` succeeds → new session, handle, baseline = the file's bytes, then `refresh` (so a drawing adjusted on load is unsaved); a failure leaves everything unchanged. Return the status text from D9 (adjusted names with max displacement, frozen count) or the `LoadError` message. Verify with tests that load:
  - a saved sample (no report);
  - a file with an off-tolerance value (adjusted, unsaved);
  - a file with an unresolvable address at position 3 (error names 3, previous session kept);
  - a cancelled open (nothing changes).
- [ ] 2.4 Compute the title (`<name>` or `untitled`, with `*` when unsaved). Call `Files::show_document` whenever the title or the unsaved state changes. Verify with a test that `FakeFiles` sees `a.sketch.json*` after an edit and `a.sketch.json` after undo.

## 3. App integration

- [ ] 3.1 Wire `FileState` into `App`. Add the event channel drained at frame start; callbacks send `FileEvent` and request a repaint. Call `refresh` after every successful command, undo, redo and drag end. On New and Open, reset `logical_points`, picks, segment clicks and `LabelCache`. Verify with `tests_app.rs`: an App on `FakeFiles` that opens a file shows the file's geometry, has an empty history and an empty pick state.
- [ ] 3.2 Add the File menu (New, Open…, Save, Save As…), the shortcuts from D8, the modal "Unsaved changes" window, and the disabled state during drags and requests. Verify by running the native app: each menu entry and shortcut triggers its flow (with the fake or a real dialog), and the dialog's three buttons behave as in 2.2.
- [ ] 3.3 Implement the view fit from D7: an App field `world_box`, set after New and Open from mesh points and visible label origins, with a margin and a minimum size; the old box for an empty drawing and at start. Verify with `tests_app.rs` (or `tests_viewport.rs`): a drawing between (40, 20) and (60, 30) lies inside the canvas after Open; New gives the default box.
- [ ] 3.4 Implement close handling (D6): on `close_requested` with unsaved changes, send `CancelClose`, run the flow with `Pending::Close`, then `Close`. Verify by running the native app: closing with unsaved changes asks, Cancel keeps the window, Discard closes, and Save closes only after saving.

## 4. Hosts

- [ ] 4.1 Native host `src/host_native.rs`, declared in `main.rs` behind `cfg(not(wasm32))`: `rfd` 0.17 (default features) and `pollster` in the native target table; dialogs and file I/O on a thread; filters "Sketch" (`*.sketch.json`) and "All files"; `show_document` sets the window title. Verify by running the native app on a desktop with a portal: open the file saved by Save As, edit it, save, and check that the file on disk changed and that the title changes with the unsaved state.
- [ ] 4.2 Browser host `src/host_web.rs`, declared in `main.rs` behind `cfg(wasm32)`: a hidden file input with `change`/`cancel` for Open; a Blob download for Save (`can_overwrite() == false`); `document.title`; a `beforeunload` warning while unsaved. Add the `web-sys` features and `js-sys` to the wasm table. Verify with `scripts/check-wasm.sh` and a Playwright check with `trunk serve`:
  - Save As yields a download with the drawing's bytes;
  - Open with `setInputFiles` loads that file;
  - the title shows the `*` after an edit;
  - closing the page while unsaved triggers a `beforeunload` dialog.
- [ ] 4.3 Wire both hosts in `main.rs` into `Host::new` and remove the placeholder from 1.1. Verify with `cargo build -p sketch_ui`, `scripts/check-wasm.sh` and `cargo clippy --workspace --all-targets`.

## 5. Documentation and integration checks

- [ ] 5.1 Update `impls/rust/README.md`: the file menu and shortcuts, the `.sketch.json` files, native dialogs through xdg-desktop-portal on Linux, and Save as a download in the browser. Update the `openspec/config.yaml` context notes for the `Files` boundary, `files.rs` and the host modules. Verify that the README describes what 3.2 and 4.x deliver.
- [ ] 5.2 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `scripts/check-wasm.sh` and `openspec validate file-open-save --strict`, and verify that all pass.

## Workflow follow-up

- Create the GitHub issue for pan and zoom (decided in explore), and link it under #14.
- Archive the change with spec sync once the PR is reviewed.
