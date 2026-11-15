# Tasks

Implementation: impls/rust, crate `sketch_ui` (see design.md). Run tests from `impls/rust` with `cargo test -p sketch_ui`.

## 1. View state (system-architecture, design decision 1)

- [x] 1.1 Add `ViewState { center, scale }` with `viewport(canvas)`, `zoom(canvas, anchor, factor, bounds)`, `pan(delta)`, `fit(canvas, min, max, margin, bounds)`, `percent(print_scale)`, plus `print_scale(settings)` and `zoom_bounds(settings)`, to `viewport.rs`. Verify in `tests_viewport.rs`:
  - "Zoom in at the pointer": factor 2 doubles the scale, and the anchor's world image is unchanged;
  - "Pan right";
  - "Wider window shows more": same center and scale for two canvas widths, and a point just outside the narrow view is inside the wide one;
  - fit puts a box inside the canvas;
  - "Zooming in stops": repeated ×10 stops at the upper bound with the anchor fixed, and likewise at the lower bound;
  - "One hundred percent": scale 10 mm gives ≈ 37.8 px per unit and 100 %.

## 2. App navigation (system-architecture, design decisions 2–6)

- [x] 2.1 Replace the per-frame refit in `app.rs` with `view: ViewState` and `pending_fit`. Set the flag at start, on New / Open (`replaced`) and for Fit, and fit at the paint step. Re-apply the bounds each frame. Keep `world_box` as the fit target. Verify that the existing `tests_app.rs` tests pass, and add:
  - an edit leaves `view` unchanged ("Edits do not move the view");
  - Fit after a zoom brings every visible geometry back inside the canvas ("Fit after zooming away").
- [x] 2.2 Add `WheelMode`, `NavIntent` and `wheel_intent(unit, delta, modifiers, mode)` per design decision 3. Apply wheel, `Event::Zoom` and middle-button drag intents to `view`, anchored at the hover position. Verify with a table test in `tests_app.rs` covering:
  - Ctrl+wheel in each mode;
  - Shift+wheel;
  - Line / Page / Point in Automatic, Zoom and Pan;
  - the zoom factor of one line (1.2).
- [x] 2.3 Restrict point drags to the primary button. When the view changed during a drag, re-target the drag to the pointer's new world image (design decisions 4 and 5). Verify with a `tests_app.rs` test driven through the session and view (no egui frame needed): `drag_begin` on a segment end, zoom around another point, `drag_to(view.viewport(canvas).to_world(pointer))`, `drag_end`. The end follows the pointer's world image under the new scale, and exactly one undo step results ("Zoom during a drag").
- [x] 2.4 Add the View menu (Fit drawing, Print size, Wheel / scroll setting), the `F` key (only without keyboard focus), and `Zoom N %` in the status row. Verify with a test of the percentage formatting (`37 %`, `100 %`, `1250 %`, `12 500 %`, in `tests_navigation.rs` next to the new `navigation.rs` module instead of `tests_app.rs`), a test that the print-size zoom reads 100 %, and by building the app (`cargo build -p sketch_ui`).
- [x] 2.5 Check the native and web builds manually (split off from 2.4; needs a person at a browser with mouse and touchpad): wheel zooms around the pointer, two-finger scroll pans, pinch zooms, middle-button drag pans without dragging a point, `F` fits, and Ctrl+wheel over the canvas does not zoom the page.
  - Checked natively, in Firefox and in Chromium with a mouse. The check found Firefox's 3-line notches and Chromium's pixel wheel, which led to the clamp and the notch rule in design decision 3 (tests in `tests_navigation.rs`).
  - Touchpad gestures (two-finger pan, pinch) are untested: no touchpad was available.
  - Firefox first loaded a stale cached worker file; that is #105.

## 3. Documentation and checks

- [x] 3.1 Update the `viewport` / `app.rs` lines in `openspec/config.yaml` (ViewState, pending fit, wheel classification, middle-button pan, primary-only point drags). Verify that `openspec validate viewport-navigation --strict` passes.
- [x] 3.2 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `scripts/check-wasm.sh` from `impls/rust`. Verify that all three succeed.

## Workflow follow-up

- Archive the change after review; sync the `system-architecture` delta.
- Check off #67 in #98 and #90.
