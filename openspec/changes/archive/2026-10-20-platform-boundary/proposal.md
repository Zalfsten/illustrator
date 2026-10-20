# Proposal

## Why

With Ceres removed, every crate on the solve and paint path is pure Rust, and the planned browser (WebAssembly) build of the illustration tool is within reach. A browser build matters for the product goal: students and teachers can open and edit constraint-based figures without installing anything. Later features will need host services that only one platform has: native file system access, file watching, or a heavy native solver on one side; browser downloads, the Web Share API, or storage on the other. Without an agreed boundary, platform conditionals spread through Document, Interact and App, and the browser build quietly breaks. This change records that boundary now, before the first platform-dependent feature (load/save) arrives.

## What Changes

- New shared architecture contract: the core (Document, evaluation, assembly, solver port, Interact, Mesh, App presentation logic) does not depend on host platform services. Host services go through one replaceable host boundary.
- Host services whose result arrives later (file pick, network) are delivered by completion, never by blocking the interactive loop, so the same core works on hosts without blocking I/O or threads.
- Optional host capabilities can be discovered at runtime. If a capability is missing, the App hides or disables the matching UI instead of failing.
- linux-rust: `sketch_ui` gets two entry points, native (`eframe::run_native`) and browser (`eframe::WebRunner` on a canvas), selected by target. The rest of the UI code does not change.
- linux-rust: a scripted check builds the core crates and `sketch_ui` for `wasm32-unknown-unknown`, so losing platform neutrality shows up at once. A Trunk `index.html` lets the browser build be served locally.
- linux-rust: the `Platform` trait (host boundary type) is recorded in design.md but **not** implemented in this change. It is introduced together with the first feature that needs a host service (load/save).

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `system-architecture`: add requirements for a platform-neutral core, a single host boundary with completion-based delivery, and runtime-discoverable optional host capabilities.

## Non-goals

- Load/save, clipboard, sharing, or any other concrete host service (each comes with its own feature change).
- Web Workers or off-main-thread solving in the browser.
- Splitting `sketch_ui` into separate native and web crates (deferred until the two entry points or their dependencies diverge noticeably).
- Deploying or hosting the browser build; packaging for distribution.
- Specs remain shared across implementations; the eframe/Trunk/Cargo choices apply only to linux-rust.

## Impact

- `impls/linux-rust/crates/sketch_ui`: `main.rs` split by `cfg(target_arch = "wasm32")`; target-specific dependencies (`wasm-bindgen-futures`, `web-sys`) in `Cargo.toml`; new `index.html` for Trunk.
- `impls/linux-rust`: new check script for the wasm32 build; README section "Browser build".
- Dev Container: `wasm32-unknown-unknown` target and `trunk` installed in post-create.
- `openspec/specs/system-architecture`: three added requirements.
