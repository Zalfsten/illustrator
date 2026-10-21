# Design

## Context

See proposal.md (Why) for motivation. This design covers only the Rust implementation (`impls/rust`).

Current state (2026-10-21):

- The repository is public, its default branch is `main`, and GitHub Pages is not enabled yet. `.github/` does not exist.
- `impls/rust` is a Cargo workspace (`constraint_runtime`, `sketch_ui`) without `rust-toolchain.toml` and without a `[profile.release]` section. The Dev Container uses stable Rust, the `wasm32-unknown-unknown` target and Trunk 0.22.0.
- `sketch_ui/index.html` has `<link data-trunk rel="rust" data-bin="sketch_ui" />` without `data-wasm-opt`. `Trunk.toml` only sets the serve port 8737.
- `impls/rust/scripts/check-wasm.sh` runs `cargo build --target wasm32-unknown-unknown -p constraint_runtime -p sketch_ui` (D5 of `platform-boundary`: a script, no CI).
- `cargo test --workspace` passes locally (92 tests, 1 ignored). The allocation-counting tests use a per-thread counter, so parallel test threads do not disturb them.
- Measured sizes of `sketch_ui`'s wasm: debug Trunk build 13.6 MB. Raw `cargo build --release` 4.6 MB, 1.6 MB with gzip -9. Both are before `wasm-bindgen` and `wasm-opt`.
- Trunk downloads `wasm-bindgen-cli` (matching `wasm-bindgen 0.2.128` from `Cargo.lock`) and `wasm-opt` at build time. A local release build failed once on a download timeout, so the build depends on network access to GitHub releases.

## Goals / Non-Goals

**Goals:**

- One workflow that guards every push and pull request (tests and the wasm build) and deploys to Pages from `main` only after both checks pass.
- Same build commands in CI and in local use: CI calls `check-wasm.sh` and `trunk build`, with no logic of its own beyond the base path.
- Local development stays unchanged: `cargo run -p sketch_ui`, `trunk serve` on port 8737 at the root path, and `check-wasm.sh`.
- Reproducible tool versions: a pinned Trunk version, and wasm-bindgen pinned by `Cargo.lock`.

**Non-Goals:**

- Caching Trunk's tool downloads across runs, a matrix over OS or toolchains, nightly Rust.
- Changing Cargo profiles or eframe features for size (see Open Questions).

## Decisions

### D1: One workflow file, four jobs

`.github/workflows/pages.yml`:

```
on: push (main only), pull_request, workflow_dispatch

  test ----------+
                 +--> build-pages --> deploy
  wasm-check ----+    (if: push to main or workflow_dispatch on main)
```

- `test`: `cargo test --workspace` in `impls/rust`.
- `wasm-check`: `impls/rust/scripts/check-wasm.sh`.
- `build-pages` (`needs: [test, wasm-check]`, only for `refs/heads/main`): Trunk release build, then `actions/upload-pages-artifact` with `impls/rust/crates/sketch_ui/dist`.
- `deploy` (`needs: build-pages`, `environment: github-pages`): `actions/deploy-pages`.

`push` is limited to `main`: with `push` on all branches, a branch with an open pull request ran every check twice (once per event). Trade-off: a push to a branch without a pull request runs no CI. Open a (draft) PR or use `workflow_dispatch` to check such a branch.

Permissions: `contents: read` at workflow level, and `pages: write` plus `id-token: write` only on `deploy`. `concurrency: { group: pages, cancel-in-progress: false }` on the deploy path, so a running deployment is never cut off. `test` and `wasm-check` run in parallel.

*Alternatives:* two workflows (`ci.yml` and `pages.yml`) would need `workflow_run` chaining or duplicated checks. One file keeps the rule "no deploy without green checks" visible in one place. The name `pages.yml` follows the issue. A later split is cheap if lints or more implementations arrive.

### D2: Toolchain and caching

`dtolnay/rust-toolchain@stable` with `targets: wasm32-unknown-unknown`, and `Swatinem/rust-cache` with `workspaces: impls/rust`. Stable matches the Dev Container. A `rust-toolchain.toml` is not added: the workspace does not use one today, and adding it would also change local builds.

On `ubuntu-latest`, eframe (winit, glow) needs X11/Wayland development headers to build natively for `cargo test`. The `test` job installs `libxkbcommon-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libssl-dev` (the list egui documents for Linux). The wasm jobs do not need them.

### D3: Trunk installation from a pinned release binary

Download the `trunk` 0.22.0 Linux x86_64 release archive from `trunk-rs/trunk` GitHub releases and put it on `PATH`. This is the same version as the Dev Container and takes seconds.

*Alternatives:* `cargo install --locked trunk` takes several minutes per run without a dedicated cache. `jetli/trunk-action` adds a third-party action for a one-line download.

Trunk then fetches `wasm-bindgen-cli` 0.2.128 (from `Cargo.lock`) and `wasm-opt` itself. To keep `wasm-opt` reproducible, `Trunk.toml` gets a `[tools]` entry with an explicit `wasm_opt` version (the version Trunk 0.22.0 selects by default, recorded at apply time).

### D4: Base path only in CI

CI runs `trunk build --release --public-url "/${{ github.event.repository.name }}/"`, which is `/illustrator/` for this repository. The path is not put into `Trunk.toml [build] public_url`, because that would also apply to `trunk serve` and break local serving at `/`. Deriving it from the repository name keeps forks working without edits.

*Alternative:* a relative public URL (`./`). That works for the asset links but depends on Trunk's handling of relative paths in the injected loader. The absolute path is what the Pages docs and the issue specify.

### D5: wasm-opt in index.html

`<link data-trunk rel="rust" data-bin="sketch_ui" data-wasm-opt="z" />`. Trunk runs `wasm-opt` only for release builds, so `trunk serve` (debug) stays fast and unchanged. Level `z` (size) is chosen because download size dominates startup for a demo page. A CPU-heavy drag profile would favour `3` or `s`, so the apply step checks `z` with a drag smoke test.

**Outcome (apply, 2026-10-21):** the deployed release build (`main` at `57404dd`, `wasm-opt version_131 -Oz`) has a 2.76 MB `.wasm` (1.31 MB transferred with gzip), down from 13.6 MB debug and 4.6 MB raw release. `z` stays. A comparison with `s` was dropped: at 1.3 MB transferred the difference does not matter for a demo, and local release builds were blocked by slow tool downloads.

### D6: No `.nojekyll`, no 404 page

`actions/deploy-pages` serves the uploaded artifact as is. Jekyll processing does not apply to artifact-based deployments, and Trunk's output has no underscore-prefixed paths. The app is a single page without client-side routes, so no `404.html` fallback is needed.

### D7: Documentation

- `impls/rust/README.md`, section "Browser build": hosted demo URL, what the workflow does (checks on every push, deploy from `main`), the local release equivalent (`trunk build --release --public-url /illustrator/`), and the one-time setting Pages → Source "GitHub Actions". The page is a demo without load or save.
- Root `README.md` (German, like the rest of the file): a one-line link to the demo.

## Risks / Trade-offs

- [Pages source not switched to "GitHub Actions"] → `deploy` fails with a clear error on the first run. The setting is a manual task for the owner, and the README names it.
- [Native `cargo test` on the runner misses system libraries for eframe] → D2 installs the documented list. If a library is still missing, the job fails at link time and the list is extended.
- [Trunk's download of wasm-bindgen or wasm-opt fails (network, as seen locally)] → only `build-pages` is affected. A re-run fixes it, and checks on pull requests do not depend on it.
- [Every push to `main` redeploys, even if only specs or docs changed] → acceptable. A build takes a few minutes and Pages keeps serving the old version until the new one is live. `paths` filters can be added later.
- [The demo shows `main`, while work happens on feature branches such as `refactor2`] → expected. The demo updates when those branches merge.
- [Single-threaded wasm on the browser main thread] → unchanged from `platform-boundary`, acceptable for demo-sized sketches.

## Migration Plan

1. Merge the change to `main`.
2. The owner sets Pages → Source "GitHub Actions" once, before or after the merge. Re-run the workflow if it ran first.
3. Verify `https://zalfsten.github.io/illustrator/`: the page loads, `.wasm` is served as `application/wasm`, and drawing and dragging work.

Rollback: delete the workflow file and disable Pages in the settings. Removing `data-wasm-opt` is optional, because it does not affect debug builds.

## Open Questions

- Whether further size work (a dedicated wasm Cargo profile with `opt-level = "z"` and LTO via `data-cargo-profile`, eframe feature trimming) is worth it. Decide after the measured `wasm-opt` result. It does not change this change's tasks.
