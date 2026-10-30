# Tasks

## 1. Spike gate: second module as a lazily started worker

- [ ] 1.1 Create `crates/label_worker` (`cdylib`, wasm-bindgen pinned like `sketch_ui`'s lock, depends on `label_typst`): at start, typeset one fixed line and post its byte length back. Verify that `cargo build --target wasm32-unknown-unknown -p label_worker` succeeds and that `rayon` / `std::thread` are not hit at runtime (no panic in step 1.3).
- [ ] 1.2 Declare the worker in `index.html` as a Trunk worker asset (`data-type="worker"`, loader shim). Run `trunk build --release` and record the following in design.md decision 5:
  - the emitted file names, and whether their URL is predictable (file hashes);
  - the time `wasm-opt` takes;
  - the gzip sizes of the main module and the worker.
  
  Verify that the initial page (`index.html` plus the main module) references no worker file, so the worker is not fetched up front.
- [ ] 1.3 Check the round trip in a browser. Serve with `trunk serve`, and from a temporary hook in `main.rs` create the worker on a key press, then log the message and its time. Use Playwright with headless Chromium if it can be downloaded; otherwise hand this step to the user with exact instructions. Verify:
  - the worker answers;
  - the time to the first answer is recorded;
  - the UI thread keeps painting while it starts.
  
  **Gate:** if the Trunk worker asset cannot give a predictable URL or does not run, update design decision 5 to the pre-build-hook route before continuing. Remove the temporary hook afterwards.

## 2. label_layout: asynchronous port and JSON messages

- [ ] 2.1 Rename the synchronous trait to `LineTypesetter`. Add `LineRequest`, `TypesetterState`, the asynchronous `Typesetter` (`request`, `take_completed`, `state`) and `Immediate<E>`. `label_typst` implements `LineTypesetter`. Verify:
  - `cargo test -p label_layout -p label_typst` passes;
  - a new test shows that `Immediate` is `Ready` and delivers a requested line in the following `take_completed` (spec "Immediate typesetter").
- [ ] 2.2 Add `wire.rs` (`serde` and `serde_json` as new dependencies of `label_layout`): the mirror types `WireRequest`, `WireLine`, `WireShape`, `WireSegment`, `WireError` and `WorkerMessage`, with conversions to the `label_layout` and `constraint_runtime` types, per design decision 4. `constraint_runtime` gets no serde. Verify in `tests_wire.rs`:
  - JSON round trips of a request with a multi-line preamble and Unicode;
  - a `TypesetLine` with fill and stroke shapes and all segment kinds, bit-exact (negative zero and `1/3` included);
  - both error kinds, and the ready message;
  - malformed or truncated JSON, an unknown tag and a point with three coordinates are rejected with an error rather than a panic;
  - `cargo tree -p constraint_runtime` shows no `serde`.

## 3. sketch_ui: Platform, pending state, display

- [ ] 3.1 Add `platform.rs` (`Platform`, `Host`) and change `App::new(Box<dyn Platform>)`. The cache no longer owns the typesetter; `label_paint` takes `Option<&dyn Typesetter>`. Natively, `main.rs` builds `Host` with `Immediate<TypstTypesetter>`; the browser keeps `None` until group 4. Verify that `cargo test -p sketch_ui` passes and the native app still shows the sample label typeset.
- [ ] 3.2 Add the cache states `Pending` / `Done`, the drain of `take_completed`, the last complete rendering per label, and `FallbackReason`. Verify in `tests_labels.rs`, with a fake asynchronous typesetter that completes on command:
  - "Typesetter still loading" (pending fallback, reason `Pending`);
  - "Changed line keeps the last rendering";
  - "Label moves while pending";
  - "Immediate typesetter" (no pending item with `Immediate`);
  - "Typesetter fails to load" (`LoadFailed` for every label);
  - a line requested twice while pending is requested once.
- [ ] 3.3 Paint: draw `Pending` in grey and the other reasons in orange. The status line shows "loading typesetter…", "typesetter could not be loaded: …" or the first failure. Request a repaint on completion through the platform's wake callback. Verify that `cargo test -p sketch_ui` passes, and that the native app shows no change in behaviour (screenshot as in #59).

## 4. Browser: worker and client

- [ ] 4.1 Turn `label_worker` into the real worker:
  - parse `WireRequest` JSON and typeset with `TypstTypesetter`;
  - post `WorkerMessage::Line` as a JSON string;
  - post `WorkerMessage::Ready` after one warm-up line.
  
  Verify that `cargo build --target wasm32-unknown-unknown -p label_worker` succeeds and the browser check (4.3) gets typeset lines.
- [ ] 4.2 Create `crates/label_web`. The pure `Core` (ids, queue until ready, in-flight set, response matching, error → `Failed`) is unit-tested natively in `tests_core.rs`. `WorkerTypesetter` (web_sys `Worker`, `onmessage` / `onerror`, `wake`) creates the worker on the first `request`. The wasm `main.rs` builds `Host` with it. Verify:
  - `cargo test -p label_web` passes;
  - `scripts/check-wasm.sh` (now with `label_worker` and `label_web`) succeeds.
- [ ] 4.3 Run the browser check:
  - `trunk serve` and the sample page;
  - before the first label request, no worker module is fetched (network log);
  - the sample label appears grey, then typeset;
  - dragging during loading stays fluid;
  - with the worker file removed from `dist/`, the status line says it could not be loaded and the label stays orange.
  
  Use Playwright if available; otherwise hand this to the user with exact steps and record the result.

## 5. Docs and integration

- [ ] 5.1 Update `impls/rust/README.md` (labels in the browser, the worker, sizes, how to check) and the impls/rust notes in the `openspec/config.yaml` context (`Platform`, `LineTypesetter` vs `Typesetter`, `label_worker`, `label_web`, cache states). Verify that the names match the code.
- [ ] 5.2 Run the full check:
  - `cargo test --workspace`, `cargo clippy --workspace --all-targets` with no warnings, `scripts/check-wasm.sh`, and `openspec validate label-typesetter-web --strict` all pass;
  - `trunk build --release` succeeds, and the sizes are recorded in the PR.

## Workflow follow-up

- Archive the change after review. Close #60.
