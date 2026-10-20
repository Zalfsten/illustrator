# Tasks

## 1. Toolchain and wasm build check

- [x] 1.1 Add `rustup target add wasm32-unknown-unknown` and trunk installation (`cargo binstall -y trunk`, falling back to `cargo install --locked trunk`) to `.devcontainer/post-create.sh`; verify `rustup target list --installed` lists `wasm32-unknown-unknown` and `trunk --version` succeeds
- [x] 1.2 Create `impls/linux-rust/scripts/check-wasm.sh` running `cargo build --target wasm32-unknown-unknown -p constraint_runtime`; verify it exits 0, or fix faer/transitive features (see design.md Risks) and record what was needed in design.md

## 2. Dual entry point for sketch_ui

- [x] 2.1 Split `crates/sketch_ui/src/main.rs` into a `cfg(not(target_arch = "wasm32"))` native `main` (unchanged behavior) and a `cfg(target_arch = "wasm32")` `main` that starts `eframe::WebRunner` on canvas `the_canvas_id`, checking the `start` signature against eframe 0.29.1; verify `cargo build -p sketch_ui` and `cargo test -p sketch_ui` still pass natively
- [x] 2.2 Add `[target.'cfg(target_arch = "wasm32")'.dependencies]` (`wasm-bindgen-futures`, `web-sys` with `Window`, `Document`, `HtmlCanvasElement`; apply note: also `wasm-bindgen` for `JsCast::dyn_into` and web-sys `Element`, versions matched to eframe's lockfile entries) to `crates/sketch_ui/Cargo.toml` and extend `check-wasm.sh` with `-p sketch_ui`; verify the script exits 0
- [x] 2.3 Verify no `cfg(target_arch` occurs outside `crates/sketch_ui/src/main.rs` (`grep -rn "target_arch" impls/linux-rust/crates` shows only main.rs)

## 3. Browser serving

- [x] 3.1 Add `crates/sketch_ui/index.html` for Trunk (rust link, full-window canvas `the_canvas_id`); verify `trunk build` in `crates/sketch_ui` produces `dist/` with `.wasm` and `.js`, and add `dist/` to `.gitignore`
- [ ] 3.2 Run `trunk serve` and open `http://localhost:8737` on the host (apply note: port set in `Trunk.toml`; 8080 was taken on this host); verify the sample sketch renders and a point drag updates the sketch (system-architecture: same behavior on both hosts)

## 4. Documentation

- [x] 4.1 Add a "Browser build" section to `impls/linux-rust/README.md` (target setup, `scripts/check-wasm.sh`, `trunk serve`) and the rule "no host services or `cfg` in core/App code; host services via the `Platform` trait (design.md D3)"; verify the documented commands run as written
- [x] 4.2 Add a short "Platform boundary" paragraph to the `openspec/config.yaml` project context (core is platform-neutral, host services via one boundary, `cfg` only at the entry point); verify `openspec validate platform-boundary --strict` passes

## Workflow follow-up

- Introduce the `Platform` trait (design.md D3) in the change that adds load/save.
- Archive this change after review.
