# Design: Pan and zoom the Canvas

## Context

See proposal.md for motivation and the spec delta (`system-architecture`) for the contract. Target implementation: impls/rust, crate `sketch_ui` (egui 0.29). Decisions from the UI discussion are on #67.

- **`viewport.rs`:** `Viewport { scale, origin_screen }` maps world ↔ screen (y flip). `Viewport::fit(canvas, min, max, margin)` fits a box; `world_box(points)` adds a 10 % margin and a minimum size.
- **`app.rs`:**
  - `App::world_box` is set at start (`WORLD_MIN..WORLD_MAX`), on New and on Open (`replaced` → `drawing_box`, which uses the view-independent export tolerance).
  - Every frame runs `self.viewport = Viewport::fit(canvas, world_box, VIEW_MARGIN)`, so the view is refitted and a window resize rescales.
  - Paint, hit tests (`px_to_world`), the Mesh tolerance and pointer input already go through `self.viewport`.
- **Drag** (Select tool): it begins on `response.drag_started()` with **any** button and the press origin, calls `drag_to` on `response.dragged()`, and ends on `drag_stopped()`. A middle-button pan would therefore also start a point drag today.
- **egui input:** `ctx.input(|i| i.events)` contains the raw `Event::MouseWheel { unit: Point | Line | Page, delta, modifiers }` and `Event::Zoom(factor)` (pinch on some platforms). Browsers report pinch as Ctrl+wheel. `MouseWheel` deltas point in the direction the content should move.
- **Existing app tests** check `world_box` after Open / New and that edits keep it.

## Goals / Non-Goals

**Goals:**

- **A toolkit-independent view state** whose math is unit-tested headless. #91 (UI model) can later move it unchanged.
- **The spec behaviours:** zoom around an anchor, pan, resize, fit, bounds, navigation during a drag, print-size zoom and its display.
- **The desktop bindings from #67**, with the session wheel setting.

**Non-Goals:** fit to the selection (#91), persistence of the setting (#96), keyboard `+` / `-` / `0`.

## Decisions

### 1. `ViewState` beside `Viewport`

```rust
pub struct ViewState { pub center: Point2, pub scale: f64 }   // world units at the Canvas center; px per world unit
impl ViewState {
    pub fn viewport(&self, canvas: Rect) -> Viewport;           // origin = canvas center − scale·(cx, −cy)
    pub fn zoom(&mut self, canvas: Rect, anchor: Pos2, factor: f64, bounds: (f64, f64));
    pub fn pan(&mut self, delta: Vec2 /* screen */);
    pub fn fit(canvas: Rect, min: Point2, max: Point2, margin: f32, bounds: (f64, f64)) -> Self;
    pub fn percent(&self, print_scale: f64) -> f64;
}
pub fn print_scale(settings: &DocumentSettings) -> f64;          // scale_mm · 96 / 25.4 px per world unit
pub fn zoom_bounds(settings: &DocumentSettings) -> (f64, f64);   // 1 % … 100 000 % of print_scale
```

- Only `viewport()`, `zoom()` and `fit()` take egui's `Rect` / `Pos2`. They are plain value types that a later UI-model crate can replace with its own.
- **Resize keeps center and scale for free:** the state stores the center, not the screen origin, so `viewport(canvas)` re-centers on every frame.
- **Zoom with the anchor fixed:** with `w` the anchor's world image before the zoom, set the new scale `s′ = clamp(s·factor)`, then move the center so that `w` maps to the anchor again: `center′ = w + (center − w)·(s / s′)`. With clamping, `s′/s` is the effective factor, and the anchor still stays fixed.
- **Pan:** `center −= (delta.x / s, −delta.y / s)` (y flip).
- **Bounds:** 1 % … 100 000 % of print size. They are applied on zoom and fit, and re-applied each frame, because a `set_document` scale change moves the bounds.

*Alternative:* store `origin_screen` as today. Rejected, because resize would then shift the drawing and the zoom math would need the canvas history.

### 2. Fit is requested, then done when the Canvas is known

- `App` keeps `world_box`, the fit target, set at start, New and Open exactly as today, and a flag `pending_fit`.
- At the paint step the canvas rect is known: if `pending_fit`, then `view = ViewState::fit(canvas, world_box, VIEW_MARGIN, bounds)` and the flag is cleared.
- **Key `F`** and the **View → Fit drawing** menu entry set `world_box = drawing_box()` (or the default box) and `pending_fit = true`.
- Edits change neither the state nor the flag.

The existing app tests on `world_box` stay valid.

### 3. Wheel classification as a pure function

```rust
enum WheelMode { Automatic, Zoom, Pan }                          // session setting, default Automatic
enum NavIntent { Zoom { factor: f64 }, Pan(Vec2) }
fn wheel_intent(unit: MouseWheelUnit, delta: Vec2, mods: Modifiers, mode: WheelMode) -> Option<NavIntent>;
```

| condition (first match wins) | intent |
|---|---|
| Ctrl / Cmd held | zoom by `exp(k·delta.y)` |
| Shift held | pan horizontally by `delta.y + delta.x` |
| mode Zoom, or mode Automatic and unit Line / Page | zoom |
| mode Automatic, unit Point, `delta.x = 0` and `|delta.y| ≥ 40` (a browser's wheel notch) | zoom |
| otherwise (mode Pan, or Automatic and other Point deltas) | pan by `delta` |

- **Zoom per unit:** `k = ln(1.2)` per line, `ln(1.2)/50` per point, and a page counts as 10 lines. The exponent is clamped to ±1 line per event. One mouse-wheel notch is therefore 1.2× everywhere, and a touchpad gesture is smooth.
- **Why the clamp and the notch rule** (found in the manual web check): Firefox reports a notch as 3 lines (1.2³ ≈ 1.73× per notch), and Chromium reports the mouse wheel in pixels (about 53 px on Linux, 100–120 px elsewhere), which the unit rule alone read as a pan.
- **Anchor:** the pointer's hover position, or the Canvas center if there is none.
- `Event::Zoom(f)` (pinch) zooms by `f`.
- Wheel events only count while the pointer is over the Canvas and no modal question is open.

### 4. Middle-button pan; point drags only with the primary button

- `response.dragged_by(PointerButton::Middle)` pans by `response.drag_delta()`.
- The Select tool's point drag starts only on `drag_started_by(PointerButton::Primary)` and continues only while the primary button drags. This removes the accidental point drag during a middle-button pan.

### 5. Navigation during a drag

- Intents are applied before the drag code in the same frame.
- While `session.is_dragging()` and the view changed this frame, the App calls `drag_to(viewport.to_world(pointer))` even without pointer motion. The dragged point then follows the pointer's world image at once.
- The drag is never ended by navigation; `drag_end` still makes one history step.

### 6. View menu and zoom display

- A **View** menu next to **File**: "Fit drawing (F)", "Print size (100 %)", and a "Wheel / scroll" radio group: Automatic / Zoom / Pan.
- "Print size" zooms around the Canvas center to `print_scale`.
- The status row shows `Zoom N %`, rounded to whole percent below 1000 and to three significant digits above.
- `F` is read only when no text field has keyboard focus (`ctx.wants_keyboard_input()`), because labels and future inspector fields take text.

## Risks / Trade-offs

- **[Wheel unit unreliable (Chromium reports mouse wheels as Point; X11 touchpads as Line)]** → the notch rule catches Chromium's mouse wheel. A fast, purely vertical touchpad flick can still read as a notch and zoom. The session setting overrides the classification; #96 will remember it.
- **[Browser zoom shortcuts (Ctrl+wheel zooms the page in the web build)]** → egui's web runner prevents the default for wheel events over its canvas; the native build is unaffected. This is checked manually in the web build.
- **[Pinch arrives as `Zoom` on some platforms and as Ctrl+wheel on others]** → both paths map to the same intent.
- **[96 dpi assumption]** → "100 %" is nominal; the browser does not expose the physical DPI. The percentage is still the right relative measure.
