# Proposal

## Why

The product goal is a constraint-based illustration tool that students and teachers can use without installing anything. The browser build of `sketch_ui` exists since `platform-boundary`, but only people who clone the repository and run `trunk serve` can see it. The build output is static (`index.html`, one `.js`, one `.wasm`), so GitHub Pages can host it for free (the repository is public). The repository also has no CI: `platform-boundary` (design D5) kept the wasm check as a local script and said that CI, once added, should call that script. Hosting needs a workflow anyway, so this change adds CI and deployment together (GitHub issue #4).

## What Changes

- New GitHub Actions workflow in the repository root: on every push and pull request it runs the Rust tests of `impls/rust` and `impls/rust/scripts/check-wasm.sh`. This replaces D5 of `platform-boundary` (script only, no CI): the script stays and CI calls it.
- On push to `main`, the same workflow builds `sketch_ui` with Trunk in release mode under the project-page base path `/illustrator/` and deploys `dist/` to GitHub Pages at `https://zalfsten.github.io/illustrator/`. This happens only after tests and the wasm check pass.
- Smaller download: `wasm-opt` is enabled for release builds in `sketch_ui/index.html`. Local `trunk serve` (debug, root path, port 8737) does not change.
- README: link to the hosted demo, how deployment works, and the one-time repository setting (Pages source "GitHub Actions").

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- (none)

Hosting and CI are delivery concerns of the Rust implementation. No observable behavior of the system changes, so the change sets `skip_specs: true`. Specs stay shared across implementations. Workflow, Trunk and Pages are design choices of `impls/rust` only.

## Non-goals

- Load/save, sharing or any other host service. The hosted page is a "try it" demo until the `Platform` trait (design D3 of `platform-boundary`) arrives with load/save.
- Preview deployments for pull requests or branches other than `main`.
- Custom domain, release versioning, or native binary packaging.
- Size work beyond `wasm-opt` (eframe feature trimming, custom Cargo profiles, `+simd128`). The size is measured and recorded, and further work is a later decision.
- Lints (`clippy`, `rustfmt`) as CI gates. They can be added to the same workflow later.

## Impact

- New `.github/workflows/pages.yml` (first CI in the repository).
- `impls/rust/crates/sketch_ui/index.html`: `data-wasm-opt` on the Trunk rust link.
- `impls/rust/README.md` and the root `README.md`: demo link and deployment section.
- Repository settings: Pages → Source "GitHub Actions" (one-time manual step by the owner).
- No change to `constraint_runtime`, App code or specs.
