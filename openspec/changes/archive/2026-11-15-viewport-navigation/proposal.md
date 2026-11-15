# Proposal: Pan and zoom the Canvas

## Why

The view is refitted to a world box on every frame (#67). The user cannot zoom into a detail, cannot follow a drawing that grows past the box, and a larger window only enlarges the drawing instead of showing more. Function curves (#81) and labels make detail work routine, so navigation is now the most noticeable gap of the app. This is the first issue of the UI plan (#90). The decisions were taken in the UI discussion and recorded on #67.

## What Changes

- **View state:** the view is a world center plus a scale (screen pixels per world unit), kept across frames. Paint, hit tests and pointer input use it, as they use the Viewport today.
- **Zoom** by a factor around an anchor screen position: the world point under the anchor stays under it.
- **Pan** by a screen displacement.
- **Resizing the Canvas keeps center and scale:** a larger window shows more.
- **Fit on demand** to the drawing, with the box rule used on New / Open. Fit to the selection follows with the selection model (#91).
- **Bounded zoom:** the scale stays within a fixed range relative to print size.
- **Navigation during a drag:** zoom and pan do not end a drag; the dragged point follows the pointer in the new world coordinates.
- **Print size:**
  - a zoom step "100 %" shows the drawing at its physical print size, assuming 96 logical pixels per inch;
  - the zoom is displayed as a percentage of print size.
- **Desktop bindings** (front end, impls/rust):
  - mouse wheel zooms around the pointer, and a two-finger touchpad scroll pans, distinguished by the wheel event's unit;
  - Ctrl+wheel and pinch zoom; Shift+wheel pans horizontally; middle-button drag pans;
  - key `F` fits;
  - a session setting "wheel / scroll" offers Automatic / Zoom / Pan, because the unit is unreliable in some browsers and on X11.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `system-architecture`: the App's Viewport gains zoom around an anchor, pan, resize behaviour, fit on demand, zoom bounds, navigation during a drag, and a print-size zoom with a percentage display. The existing requirements (world/screen mapping, pointer input through the Viewport, the Viewport not part of Document) stay as they are.

## Non-goals

- **Saving the view in the file:** the view is per session, and a file opens fitted (file-commands, unchanged).
- **Fit to the selection:** it needs the selection model (#91).
- **Persistent preferences:** the wheel setting lives for the session; #96 makes it persistent.
- **Touch gestures beyond pinch / two-finger scroll**, view rotation, grids and rulers.
- **Keyboard zoom with `+` / `-` / `0`:** these collide with the browser's own shortcuts in the web build.

The specs stay shared across implementations. The input bindings, the zoom bounds, the step factor and the wheel classification are design decisions of the desktop front end.

## Impact

- **impls/rust `sketch_ui`:**
  - `viewport.rs`: a toolkit-independent `ViewState` (center, scale, zoom, pan, resize, fit, bounds, print-size percentage) next to `Viewport`;
  - `app.rs`: view state instead of the per-frame refit, input handling for wheel / pinch / middle drag / `F`, a View menu (Fit, 100 %, wheel setting), and the zoom percentage in the status area.
- **Tests:** `tests_viewport.rs` (view math), `tests_app.rs` (wheel classification, navigation during a drag).
- **Docs:** `openspec/config.yaml` (viewport line).
- No change to the core, the session, the file format or the export.
