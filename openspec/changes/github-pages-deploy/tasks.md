# Tasks

## 1. Release build with wasm-opt (design D3, D5)

- [ ] 1.1 Add `data-wasm-opt="z"` to the Trunk rust link in `impls/rust/crates/sketch_ui/index.html`. Verify that `trunk serve` still starts on port 8737 and serves the app at `/` (debug build, no wasm-opt step in its log).
- [ ] 1.2 Find the `wasm-opt` version Trunk 0.22.0 uses by default and pin it in `Trunk.toml` under `[tools]` (`wasm_opt = "version_<n>"`). Verify that `trunk build --release` logs that version.
- [ ] 1.3 Run `trunk build --release --public-url /illustrator/` with `z`, then once with `s`. Record the `.wasm` size (raw and gzip -9) for both in design.md D5, plus a quick drag smoke test of the `z` build (`trunk serve --release`). Keep `z` unless the smoke test shows noticeable drag lag. Verify that `dist/index.html` references its `.js` and `.wasm` under `/illustrator/`.

## 2. CI and Pages workflow (design D1–D4, D6)

- [ ] 2.1 Create `.github/workflows/pages.yml` with the jobs `test` (system libraries from D2, `cargo test --workspace` in `impls/rust`) and `wasm-check` (`impls/rust/scripts/check-wasm.sh`). Both run on push, pull_request and workflow_dispatch, with `dtolnay/rust-toolchain@stable` (target `wasm32-unknown-unknown`) and `Swatinem/rust-cache` (`workspaces: impls/rust`), and workflow-level `permissions: contents: read`. Verify that the file passes `actionlint` (or at least parses as YAML) locally.
- [ ] 2.2 Add `build-pages` (`needs: [test, wasm-check]`, only on `refs/heads/main`): install the pinned Trunk 0.22.0 release binary, run `trunk build --release --public-url "/${{ github.event.repository.name }}/"` in `impls/rust/crates/sketch_ui`, and upload `dist` with `actions/upload-pages-artifact`. Add `deploy` (`needs: build-pages`, `environment: github-pages`, `pages: write`, `id-token: write`, concurrency group `pages` without cancel-in-progress) with `actions/deploy-pages`. Verify with `actionlint` that the `if:` conditions and `needs` graph are valid.
- [ ] 2.3 Push the branch and verify on GitHub Actions that `test` and `wasm-check` are green and that `build-pages` and `deploy` are skipped (not on `main`).

## 3. Documentation (design D7)

- [ ] 3.1 Update the "Browser build" section in `impls/rust/README.md`: hosted demo URL, workflow behaviour (checks on every push and PR, deploy from `main` after green checks), the local release command with `--public-url /illustrator/`, the one-time setting Pages → Source "GitHub Actions", and that the page is a demo without load or save. Verify that the documented local command runs as written.
- [ ] 3.2 Add a one-line link to the demo in the root `README.md` (German). Verify that the link target is `https://zalfsten.github.io/illustrator/`.

## 4. Deployment check (after merge to `main`)

- [ ] 4.1 Set Pages → Source "GitHub Actions" in the repository settings (owner). After the merge, verify that the workflow run on `main` completes all four jobs and that the `github-pages` environment shows the deployment URL.
- [ ] 4.2 Open `https://zalfsten.github.io/illustrator/` in a browser. Verify that the page loads without console errors, that the `.wasm` response has `Content-Type: application/wasm` (`curl -sI`), and that drawing a segment and dragging it work.

## Workflow follow-up

- Close GitHub issue #4 with a link to the hosted page.
- Archive the change after review; no spec sync is needed (`skip_specs: true`).
