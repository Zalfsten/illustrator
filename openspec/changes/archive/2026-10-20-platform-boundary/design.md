# Design

## Context

Target implementation: `impls/linux-rust`. Motivation: see proposal.md (Why).

Observed state:

- `constraint_runtime` depends only on `faer` (`std`, `sparse-linalg`). There is no build script and no FFI since `remove-ceres-backend`. No `std::fs`, `std::thread` or `std::time::Instant` in production code.
- `sketch_ui` depends on `eframe 0.29.1` (default features) and `egui 0.29`. `app.rs` and `viewport.rs` use only egui types and `impl eframe::App`. Only `main.rs` is native-specific (`NativeOptions`, `run_native`).
- No CI workflow exists in the repository. Checks run locally in the Dev Container (`mcr.microsoft.com/devcontainers/rust:1-trixie`, host networking). Only `x86_64-unknown-linux-gnu` is installed today.

## Goals / Non-Goals

**Goals:**

- `sketch_ui` runs natively and in the browser from the same `App` code.
- A single command shows whether the core crates and the UI still build for `wasm32-unknown-unknown`.
- The host boundary type and its rules are written down so the first host-service feature follows them.

**Non-Goals:**

- Implementing the `Platform` trait or any host service now.
- Separate `sketch_native` / `sketch_web` crates.
- SIMD tuning of faer for wasm, Web Workers, deployment.

## Decisions

### D1: eframe stays the UI toolkit on both targets

eframe 0.29 supports `wasm32-unknown-unknown` through `eframe::WebRunner` (glow/WebGL by default), so `App` and `Viewport` stay as they are.
*Alternatives:* a separate web front end (TypeScript + canvas) calling the core through wasm-bindgen: two UIs to maintain, which goes against "one App". Switching to another toolkit (iced, Slint): no benefit, since eframe already covers both targets.

### D2: Platform selection by `cfg(target_arch = "wasm32")` only at the entry point

`main.rs` holds two `main` functions:

- `#[cfg(not(target_arch = "wasm32"))]`: as today.
- `#[cfg(target_arch = "wasm32")]`: looks up `<canvas id="the_canvas_id">`, then `wasm_bindgen_futures::spawn_local(WebRunner::new().start(canvas, WebOptions::default(), …))`. The exact `start` signature (canvas element vs. id) is checked against eframe 0.29.1 during implementation.

Web-only dependencies go under `[target.'cfg(target_arch = "wasm32")'.dependencies]` (`wasm-bindgen-futures 0.4`, `web-sys 0.3` with `Window`, `Document`, `HtmlCanvasElement`). Native-only dependencies (later: `rfd` sync dialogs, file watchers) go under the inverse target table. `cfg` attributes are not allowed in `constraint_runtime`, `app.rs` or `viewport.rs`.
*Alternatives:* separate crates now. They are cleaner once the two entry points diverge, but premature for two short `main` functions. Revisit when either entry point grows platform services of its own.

### D3: Host boundary = `Platform` trait, introduced with the first host service

Recorded shape (not implemented in this change):

```rust
pub trait Platform {
    fn open_file(&self, done: Box<dyn FnOnce(Option<Vec<u8>>)>);
    fn save_file(&self, suggested_name: &str, data: Vec<u8>);
    fn now_seconds(&self) -> f64;
    fn share(&self) -> Option<&dyn Share> { None }   // optional capability
}
```

- `App` owns `Box<dyn Platform>`, which is passed in by the entry point (`App::new(platform)`).
- Completion is delivered via callback. A native implementation may call `done` synchronously, a web implementation calls it from `spawn_local`. Callbacks that change App state send a message into an `mpsc` channel (or `Rc<RefCell<Option<_>>>`), and `App::update` drains it at frame start. This keeps borrows simple and matches the spec requirement not to block the loop.
- Optional capabilities return `Option<&dyn Capability>`. The UI shows a command only if the result is `Some`.
- Time: core code never calls `Instant::now()`. UI code that needs time uses `ctx.input(|i| i.time)` or `Platform::now_seconds`.

*Alternatives:* `async fn` in the trait. This needs an executor on native and boxing for object safety, and gains nothing over callbacks at this size. Compile-time selection via `cfg` modules with identical function names: no runtime capability query, and tests cannot inject a fake host. With the trait, unit tests can use a `FakePlatform`.

### D4: Cargo features for heavy optional native parts

If a native-only backend comes back (for example a reference solver with C++ dependencies), it lives behind a crate feature in `constraint_runtime` that only the native entry point enables. Features cover compile-time inclusion, the trait covers runtime availability.

### D5: wasm build check as a script, not CI

No CI exists, so the check is `impls/linux-rust/scripts/check-wasm.sh`:

```bash
cargo build --target wasm32-unknown-unknown -p constraint_runtime -p sketch_ui
```

The README documents it. When CI is added later, it calls the same script.
*Alternative:* a GitHub workflow now. There is no CI to extend, so this is out of scope.

### D6: Trunk for local browser serving

`crates/sketch_ui/index.html` with `<link data-trunk rel="rust" />` and a full-window canvas `the_canvas_id`. Run with `trunk serve` (port 8737 from `crates/sketch_ui/Trunk.toml`, reachable from the host thanks to `--network=host`; an uncommon port because 8080 was already taken on the development host). The Dev Container installs the `wasm32-unknown-unknown` target and `trunk` (via `cargo binstall trunk`, or `cargo install --locked trunk` as fallback) in `.devcontainer/post-create.sh`.

## Risks / Trade-offs

- [faer on wasm32: `std` feature or a transitive dependency (e.g. `rayon`, `getrandom`) may not compile for `wasm32-unknown-unknown`] → the first task is the build check. If it fails, adjust faer features (`default-features = false` is already set) or enable `getrandom/js`; record the outcome here. **Outcome (apply, 2026-10-20):** `constraint_runtime` builds for `wasm32-unknown-unknown` unchanged — faer 0.24.4 with `default-features = false` + `std`, `sparse-linalg` needs no extra features.
- [Solver runs on the browser main thread; large sketches could drop frames] → acceptable for current sketch sizes (drag step at N = 100 is about 1.5 ms natively, maybe 2–5× slower in wasm without SIMD). Web Workers stay out of scope.
- [eframe default features pull in winit/wgpu parts not needed on web, bigger binary] → acceptable for now; trimming features is a later optimization.
- [`cfg` creeps into App code over time] → D2 rule plus review. The wasm check only catches compile breakage, not style.

## Migration Plan

Native behavior does not change: `cargo run -p sketch_ui` works as before. Rollback means removing the wasm entry point, the target dependencies, `index.html` and the script.

## Open Questions

- Whether to enable `+simd128` for wasm builds. Decide later with a browser benchmark. This does not affect specs or tasks.
