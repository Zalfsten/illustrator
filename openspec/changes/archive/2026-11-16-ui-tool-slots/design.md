# Design: Tools as slot forms, select-first, and the desktop layout

## Context

See proposal.md for motivation and the spec delta (`ui-model`) for the contract. The target implementation is impls/rust, on top of branch `issue-91-ui-model-selection` (PR #108).

- **`ui_model` (from #108):**
  - `UiModel { selection }` with `apply(Intent, &mut Session, &Scene) -> Result<Effect, GroupError>`;
  - intents Pick / Box / Clear / Delete;
  - `pick.rs`: `nearest_point` (logical points), `nearest_endpoint`, `nearest_curve`, `pick`;
  - `Entry { Geometry, Constraint, Point }`.
- **`sketch_ui/src/app.rs`:**
  - **`Tool` enum:** Select, Segment, Join, Parallel, Orthogonal, FixPoint, Brush.
  - **`handle_click`** issues commands from three private lists:
    - Segment: two world clicks;
    - Join: two `hit_end` points at 12 px;
    - Parallel / Orthogonal: two `hit_edge` lines at 24 px, checked with `caps(id).line`;
    - FixPoint: one `hit_point` at 12 px, with its current location as the target;
    - Brush: one `hit_edge` at 12 px, then `CurrentStyle::brush(name)`.
  - **Commands** are built with `PointPair` / `VecPair` / `FixParams` addresses (`seg1.end`, `seg1.direction`).
  - **Toolbar:** a top panel with four rows (menus + tools + Undo / Redo + "Clear picks"; Show; style row; status); errors in red.
- **`style_ui.rs`:** `CurrentStyle::look() -> StyleRef` and `brush(name) -> Command::SetStyle`.

## Goals / Non-Goals

**Goals:**

- **Tools in `ui_model`:** every tool's slot logic and command building lives there, tested headless. The App only maps devices to intents and paints.
- **Command-identical tools:** the existing tools issue the same commands as today (same addresses and style fields), so files and undo behave unchanged.

**Non-Goals:**

- Previews of new geometry (#95).
- Snapping (#46).
- Panel contents (#92 / #93).

## Decisions

### 1. Tool kinds and slots as data (`tool.rs`)

```rust
pub enum ToolKind { Select, Segment, Join, Parallel, Orthogonal, Fix, Brush }
pub enum SlotKind { Position, Point, Direction, Curve }
pub enum Filled { Position(Point2), Point(PointExpr), Geometry(GeometryId) }
impl ToolKind {
    pub fn name(self) -> &'static str;
    pub fn slots(self) -> &'static [SlotKind];      // Select: []
    fn command(self, doc: &Document, filled: &[Filled], look: &StyleRef) -> Command;
}
```

| tool | slots | command |
|---|---|---|
| Segment | Position, Position | `create segment` with `look` |
| Join | Point, Point | `constrain congruence(a, b)` |
| Parallel / Orthogonal | Direction, Direction | `constrain vectors_parallel / _orthogonal(a.direction, b.direction)` |
| Fix | Point | `constrain fix_point(p, current location)` |
| Brush | Curve | `set_style` as in today's `CurrentStyle::brush` |

- **Capability test:**
  - `Point` takes `Entry::Point`;
  - `Direction` takes a geometry with `caps.line`;
  - `Curve` takes a geometry with `caps.curve`;
  - `Position` never takes a selection entry.
- **Addresses** are printed through the same document functions as today (`point_address`, `direction_expr` + `vec_address`).

*Alternative:* a `Tool` trait with one object per tool (#24 per-kind tool traits). Rejected for now: six tools with fixed slot lists are clearer as a table. The enum can turn into a trait when kinds bring their own tools (#24, #95).

### 2. Tool state, look and new intents in `UiModel`

```rust
pub struct UiModel { selection, tool: ToolKind, filled: Vec<Filled>, look: StyleRef }
pub enum Intent { Pick{..}, Box{..}, Clear, Delete, ChooseTool(ToolKind), Cancel }
pub enum Effect { Selection, Deleted(Vec<String>), Issued(Outcome), Slot, Refused(String), Nothing }
pub enum UiError { Group(GroupError), Command(CommandError) }   // apply -> Result<Effect, UiError>
```

- **`ChooseTool(t)`:**
  - drops filled slots and sets the tool;
  - walks the selection in order, putting each entry into the first empty slot it fits;
  - clears the selection;
  - if the tool is complete, issues its command (decision 3).
  - Choosing `Select` only ends the current tool.
- **`Pick` with a tool active** (not Select) fills the next empty slot instead of changing the selection:
  - Position: `at`;
  - Point: `nearest_point`;
  - Direction: the nearest curve with `caps.line`;
  - Curve: `nearest_curve`.

  A miss or a duplicate gives `Effect::Refused(reason)` and leaves the slots unchanged. A filled slot that does not complete the tool gives `Effect::Slot`.
- **`Box` with a tool active** is ignored (`Nothing`). Box selection belongs to Select.
- **`Cancel`:** with a tool active, it drops the slots and returns to Select; with Select active, it clears the selection.
- **`Clear` and `Delete`** stay selection intents; while a tool is active the selection is empty, so Delete does nothing.
- **Look:** `set_look(StyleRef)` is called by the front end from `CurrentStyle::look()` before every intent (first drafted as "each frame"; before every intent also holds in tests without a frame). `CurrentStyle::brush` is replaced by `ui_model::brush_command(name, look)`, which `ToolKind::command(Brush)` uses. The Brush detail is the brushed name, so the App can print `Brushed <name>`.
- **Prompt:** `prompt(&Document) -> Option<String>` gives `"<Tool>: <filled labels joined by ', '> → pick <a position | a point | a direction | a curve>"`, or `None` in Select. Filled labels use `Entry::label` / `point_address`, and `(x, y)` for positions.
- **Nearest direction:** `pick.rs` gains `nearest_line(doc, scene, at, tol)`, which filters Mesh edges by `caps.line` before taking the nearest.

### 3. Issue, stay active, report

- When the last slot is filled, `UiModel` builds the command and calls `session.apply`.
  - On success: `Effect::Issued(outcome)`.
  - On error: `Err(UiError::Command(e))`. The session already left the drawing unchanged.
  - Either way the slots are emptied and the tool stays.
- The App shows the outcome in the status line, with the same texts as today: `Added <name> (<details>)`, `Brushed <name>`, `Rejected — drawing unchanged. <e>`.
  - The details come from `UiModel::last_detail()`, the filled labels of the issued command (e.g. ` (seg1.end = seg2.start)`).

### 4. Desktop bindings and layout (`sketch_ui`)

- **Panels, in egui order:**
  - `TopBottomPanel::top("menu")`: File, Edit (Undo, Redo), View (Fit, Print size, Wheel / scroll, Show: Result only / Include construction).
  - `TopBottomPanel::top("ribbon")`: a tab row, then the active tab's groups (each a labelled `ui.group`):
    - SELECT: Select;
    - CREATE: Segment (L);
    - CONSTRAIN: Join (J), Parallel (P), Orthogonal (O), Fix (X);
    - STYLE: Brush (B), Styles….
    - LABELS / FUNCTIONS / DESIGN show "Coming with #63 / #84 / later".
  - `TopBottomPanel::bottom("status")`: zoom · prompt (if a tool) or `Selected: …` · status text. The label status and errors follow on a second line, coloured.
  - `SidePanel::left("browser")`: resizable, default 180 px, a heading and "Coming with #92".
  - `SidePanel::right("inspector")`: resizable, default 220 px. It holds a collapsible section "Style for new geometry" (the current look as a two-column grid: name with override switch | value, plus Styles…) and a note that the selection's properties follow with #93. The style settings were first drafted in the ribbon's STYLE group; the manual check showed that they do not belong in a tool bar and do not fit narrow windows.
  - View → Panels toggles both side panels, so a small window keeps the Canvas.
  - `CentralPanel`: the Canvas.
- **Tool buttons** send `Intent::ChooseTool`. The active tool shows as selected.
- **Shortcuts** (L / J / P / O / X / B → `ChooseTool`, Esc → `Cancel`) apply under `canvas_has_keyboard`, like Delete and F. Without modifiers, so the browser's own shortcuts (Ctrl+L etc.) are untouched.
- **Canvas input:**
  - **Select:** as in #108.
  - **Any other tool:** a primary click sends `Pick` with 12 px. The 24 px line radius of today's Parallel / Orthogonal becomes 12 px; the spec leaves tolerances to the front end, and one radius is simpler to explain.
  - Primary drags do nothing in tools.
- **Feedback:**
  - filled point and position slots are painted as orange dots, like today's segment click;
  - filled direction and curve slots get the selection halo.
- **Removed from `App`:** `pick_points`, `pick_geoms`, `segment_clicks`, `handle_click`, `Tool`, "Clear picks". `replaced` calls `ui.clear()`, which now also ends the tool.

## Risks / Trade-offs

- **[12 px instead of 24 px for lines makes Parallel slightly harder to hit]** → the Canvas halo shows what was hit, and select-first avoids precise picks.
- **[Ribbon and side panels shrink the Canvas in small browser windows]** → the panels are collapsible from the View menu. The group row sits in a horizontal scroll area: egui cannot wrap whole groups reliably (`horizontal_wrapped` knows a group's width only after laying it out, so groups were cut off before wrapping; found in the manual check).
- **[Letters shortcuts and future text fields in panels]** → they are read only while no text field has the keyboard (`canvas_has_keyboard`).
- **[Clearing the selection when a tool is chosen loses it if the user cancels]** → this matches Fusion. The selection is cheap to redo, and keeping it would give Delete two meanings while a tool is active.
