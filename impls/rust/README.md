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

Tools: Select/Drag, Segment (two clicks), Join, Parallel, Orthogonal, FixPoint.

## Labels

Labels are typeset in two crates:

| Crate | Role |
|-------|------|
| `label_layout` | Engine-neutral: the `Typesetter` port, line splitting, LaTeX line spacing, placement in world space, tessellation. Builds for the browser. |
| `label_typst` | The native typesetter: LaTeX via [mitex](https://github.com/mitex-rs/mitex) 0.2.4, layout via [Typst](https://typst.app) 0.15.1, both pinned. `assets/` holds two New Computer Modern fonts (from `typst-assets`) and mitex's Typst files (xarrow replaced), each with a `NOTICE`. Update Typst and mitex together. |

The native app links `label_typst`. The browser build has no typesetter yet
(#60) and shows every label as plain text in a dashed orange frame, as it does
natively for a label that fails to typeset (the status line names the
failure).

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
It is a "try it" demo: the app cannot load or save yet.

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

`constraint_runtime`, `label_layout`, `app.rs`, `labels.rs` and `viewport.rs` stay platform-neutral: no
host services (files, threads, wall-clock time, browser APIs) and no
`cfg(target_arch = ...)`. Platform code lives only in `sketch_ui/src/main.rs`
(one `main` per host) and in target-specific dependency tables. Host services
(load/save, clipboard, sharing, ...) will go through the `Platform` trait
described in the `platform-boundary` change design (D3), introduced with the
first feature that needs one.
