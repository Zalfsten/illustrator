# Rust implementation

Runs natively (Linux, and other desktop targets eframe supports) and in the
browser (WebAssembly). Formerly `impls/linux-rust` (renamed 2026-10-20; archived
OpenSpec changes still use the old path).

First implementation of the shared OpenSpec capabilities for the constraint
runtime (geometry IR, catalog, solve interaction, solver port).

## Requirements

- Rust stable (`rustc`, `cargo`) — installed in the Dev Container image, or via [rustup](https://rustup.rs/).
  The solver is pure Rust (sparse linear algebra via [`faer`](https://codeberg.org/sarah-quinones/faer), MIT);
  no native solver library is needed.
- `sketch_ui` needs `pkg-config` and a C compiler for its windowing stack
  (`wayland-sys`, `wayland-backend`); both are in the Dev Container image.

## Solvers

| Backend | Role |
|---------|------|
| `LmSolver` | Default for `InteractiveSolve` / `sketch_ui`: owned sparse Levenberg–Marquardt (normal equations from the contribution-blocked Jacobian, sparse Cholesky) |
| `StubSolver` | Controllable success/failure for Interact tests |

`SolverOptions::lm_lambda0` is `LmSolver`'s initial damping relative to
`diag(JᵀJ)`. The former Ceres reference backend was removed (change
`remove-ceres-backend`); its measured numbers stay in
[`BENCHMARKS.md`](BENCHMARKS.md).

## Build & test

```bash
cd impls/rust
cargo test --workspace
cargo build -p sketch_ui
```

Benchmarks (results in [`BENCHMARKS.md`](BENCHMARKS.md)):

```bash
cargo bench -p constraint_runtime --bench solve
```

After Dockerfile dependency changes, rebuild the Dev Container.

## Run UI

Needs the host display. The Dev Container mounts `/tmp/.X11-unix` and forwards `DISPLAY`
(see `.devcontainer/devcontainer.json`) and installs X11/EGL libs in the image. On the Linux
host, allow local Docker clients once per session:

```bash
xhost +local:docker
```

Then:

```bash
cargo run -p sketch_ui
```

Tools: Select/Drag, Segment (two clicks), Join, Parallel, Orthogonal, FixPoint,
Brush.

## Styles

Every geometry refers to a named style (`default` unless set) and may override
its properties: width in TeX points, colour `#rrggbb`, dash (`solid`, `dashed`,
`dotted`, `dashdot`) and arrow heads (`none`, `start`, `end`, `both`). A
property resolves from the override, the named style, `default`, then the
built-in look (0.4 pt, black, solid, no arrow). Sizes are physical: they map to
world units through the document scale.

- The toolbar's style row holds the current style and four overrides, each
  switched on by its checkbox. New segments get them.
- The Brush tool gives a clicked curve the current style and overrides; an
  override that is switched off is removed. Labels cannot be brushed yet (no
  label pick); they take the colour of their style.
- "Styles…" opens the style table: add a style, set or clear its properties,
  delete an unused one. A width drag or a colour pick is one undo step, made
  when the button is released or the picker closes.
- The Canvas draws result curves in their style, never thinner than one
  pixel; construction geometry keeps its fixed grey look.

The stroke geometry (`constraint_runtime/src/stroke.rs`: pt to world, arrow
heads on the exact curve, trimming and dashing by arc length) is shared with
the SVG export and the future PDF export.

## Files

The File menu has New (Ctrl+N), Open… (Ctrl+O), Save (Ctrl+S) and Save As…
(Ctrl+Shift+S; Cmd on macOS). Drawings are sketch files, by convention named
`*.sketch.json`. The app starts on the sample drawing, which has no file.
Saving writes version 3 (with the style table, one style per line); files of
versions 1 and 2 still open, with only the `default` style.

- The title shows the file name (or `untitled`) and `*` while the drawing has
  unsaved changes: saving it now would give other bytes than the file holds.
  Undoing back to the saved drawing clears the mark. A file whose values were
  adjusted on open counts as changed.
- New, Open and closing the window ask whether to save, discard or cancel
  while there are unsaved changes.
- Opening a file shows the load report in the status line (adjusted
  components with their largest displacement, frozen components); a rejected
  file leaves the drawing open and names the failing command's position.
- After New and Open the view fits the drawing.

Natively the dialogs are the system's, through `rfd`. On Linux they go through
xdg-desktop-portal (no GTK needed); without a portal, as in the Dev Container,
Open and Save report "no file dialog available". In the browser, Open uses the
browser's file picker and every save is a download (Save behaves as Save As);
leaving the page with unsaved changes shows the browser's leave warning.

## Export

File → Export SVG… writes the drawing as a standalone SVG file, suggesting
the drawing's name with `.svg` (`untitled.svg` without a file); in the browser
the file is downloaded. Exporting is not an edit: the drawing, its history,
its file and the unsaved mark stay as they are.

- Only result geometry and result labels are exported; construction geometry,
  handles and highlights are left out.
- Curves come from their exact shape, not from the display mesh: a segment is
  one straight piece, an arc is written with SVG arc commands at its exact
  radius.
- Styles appear at their physical size: width, dash pattern (laid from the
  curve's start), flat ends, arrow heads, colours.
- Labels are glyph outlines in their style colour, so the file needs no fonts
  and looks the same everywhere; the text is not searchable. The export waits
  while labels are still being typeset and fails, naming the label, when one
  cannot be typeset.
- Size is in millimetres through the document scale (1 pt = 25.4/72.27 mm),
  with world +y up on the page. The bounds are tight around everything drawn,
  without a margin. Equal drawings give equal bytes, and every object is a
  `<g>` with its object name as `id`.

The export lives in the crate `sketch_export` (scene of print primitives,
exact bounds, SVG writer), shared with the planned PDF export.

## Labels

Labels are typeset in two crates:

| Crate | Role |
|-------|------|
| `label_layout` | Engine-neutral: the `Typesetter` port, line splitting, LaTeX line spacing, placement in world space, tessellation. Builds for the browser. |
| `label_typst` | The native typesetter: LaTeX via [mitex](https://github.com/mitex-rs/mitex) 0.2.4, layout via [Typst](https://typst.app) 0.15.1, both pinned. `assets/` holds two New Computer Modern fonts (from `typst-assets`) and mitex's Typst files (xarrow replaced), each with a `NOTICE`. Update Typst and mitex together. |

| `label_worker` | Browser only: a Web Worker (`cdylib`) that runs `label_typst`; JSON messages from `label_layout::wire`. |
| `label_web` | Browser only: the client (`WorkerTypesetter`) that starts the worker on the first label line and requests a repaint on every answer. |

Natively the app links `label_typst` directly (behind `label_layout::Immediate`).
In the browser, Trunk builds `label_worker` from `index.html`, and
`label_worker_boot.js` starts it. Nothing of it is fetched with the page: the
initial page stays about 1.4 MB gzip, and the worker (about 11 MB gzip) is
loaded when the first label needs typesetting. Until then labels show their
source in grey, with "loading typesetter…" in the status line. A label that
fails to typeset, or any label when the worker cannot be loaded, shows its
source in a dashed orange frame, and the status line names the failure.

The app reaches the typesetter through `Platform` (`sketch_ui/src/platform.rs`),
the host boundary that `main.rs` builds per target.

Check the browser build headless with Playwright (Chromium's libraries are in the
Dev Container image): `trunk serve` in `crates/sketch_ui`, then a script that
opens `http://127.0.0.1:8737/`, waits for the label, and takes a screenshot.

## Browser build

The same `App` runs in the browser (WebAssembly, eframe `WebRunner` on a canvas).
The Dev Container installs the target and Trunk in `post-create.sh`; elsewhere:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Check that the core and the UI still build for the browser target:

```bash
impls/rust/scripts/check-wasm.sh
```

Serve locally (http://localhost:8737, port set in `crates/sketch_ui/Trunk.toml`;
reachable from the host because the Dev Container uses host networking):

```bash
cd impls/rust/crates/sketch_ui
trunk serve
```

### Hosted demo (GitHub Pages)

The browser build of `main` is online at <https://zalfsten.github.io/illustrator/>.
It opens files from disk and saves them as downloads.

`.github/workflows/pages.yml` runs `cargo test --workspace` and
`scripts/check-wasm.sh` on every pull request and push to `main` (branches
without a PR are not checked; open a draft PR or start the workflow manually). On `main`, after both
pass, it builds a release with Trunk (`wasm-opt -Oz`, version pinned in
`Trunk.toml`) and deploys `dist/` to Pages. The same release build locally:

```bash
cd impls/rust/crates/sketch_ui
trunk build --release --public-url /illustrator/
```

The project page lives under `/illustrator/`, hence `--public-url`; `trunk serve`
keeps serving at `/`. One-time repository setting: Settings → Pages → Source
"GitHub Actions".

### Platform boundary

`constraint_runtime`, `label_layout`, `app.rs`, `files.rs`, `labels.rs`,
`platform.rs` and `viewport.rs` stay platform-neutral: no host services (files,
threads, wall-clock time, browser APIs) and no `cfg(target_arch = ...)`.
`cfg` appears only in `sketch_ui/src/main.rs` (one `main` per host, and the
choice of host module) and in target-specific dependency tables. Host services
go through the `Platform` trait in `platform.rs`: the typesetter, and the file
services (`Files`: open, save, overwrite or download only, document title)
with completion callbacks that the App drains at frame start.
`host_native.rs` (rfd dialogs on a thread, `std::fs`) and `host_web.rs` (file
input, Blob download, page title, `beforeunload`) implement them; tests use
`fake_files.rs`.
