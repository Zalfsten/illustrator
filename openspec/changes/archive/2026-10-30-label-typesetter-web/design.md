# Design

## Context

See proposal.md for the motivation and the specs for the requirements. This design targets `impls/rust`.

- **#59 is in place:**
  - `label_layout` with a **synchronous** `Typesetter::typeset_line`, and `label_typst` implementing it natively;
  - in `sketch_ui`:
    - `labels.rs` holds a per-line `LabelCache` that owns `Option<Box<dyn Typesetter>>` and is swept every frame;
    - `label_paint` returns `PaintItem::Typeset` or `PaintItem::Fallback`;
    - `App::new(Option<Box<dyn Typesetter>>)`;
    - `main.rs` passes `TypstTypesetter` natively and `None` in the browser.
- **Browser build:** Trunk 0.22 builds `sketch_ui` from `index.html` (`<link data-trunk rel="rust" data-bin="sketch_ui">`, `wasm-opt -Oz` in release). CI runs `scripts/check-wasm.sh`, and the Pages job builds and deploys `dist/`.
- **Spike #11 measured Typst as WASM:** 30 MB raw / 12.2 MB gzip, compiled by V8 in 36 ms; first typeset 87 ms, then about 1 ms per line (worst 3 ms). Under Node, rayon fell back to one thread.
- **What #11 did not test:** a second module loaded from a Trunk app, Web Workers, and a real browser. The container has no browser.
- **The host boundary** (`system-architecture`) is specified but not implemented. #8 and #60 agreed that whichever comes first introduces it.

## Goals / Non-Goals

**Goals:**
- One asynchronous port for all hosts. The native path completes immediately, so it behaves exactly as today.
- Typst runs in the browser without touching the UI thread, and is fetched only when a label line is requested.
- Everything except the browser glue is testable natively.

**Non-Goals:**
- A shared-memory or threaded worker pool.
- Streaming partial results.
- Service-worker caching.

## Decisions

### 1. Two traits: engine and port (label_layout)

```rust
/// What an engine does: one line, synchronously.  (was `Typesetter` in #59)
pub trait LineTypesetter {
    fn typeset_line(&self, line: &str, markup: Markup, size_pt: f64, preamble: &str)
        -> Result<TypesetLine, TypesetError>;
}

/// What the app talks to: requests never block; results arrive later.
pub trait Typesetter {
    fn request(&self, req: LineRequest);
    fn take_completed(&self) -> Vec<(LineRequest, Result<TypesetLine, TypesetError>)>;
    fn state(&self) -> TypesetterState;          // Loading | Ready | Failed(String)
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LineRequest { pub line: String, pub markup: Markup, pub size_bits: u64, pub preamble: String }

/// Port over an engine that answers at once (native).
pub struct Immediate<E: LineTypesetter> { engine: E, done: RefCell<Vec<…>> }   // state() == Ready
```

- `&self` with interior mutability: the app is single-threaded, and a worker client needs shared state with its message callback anyway.
- `label_typst` implements `LineTypesetter`. Its tests are unchanged apart from the trait name.
- The `LineRequest` is the cache key: no second key type, and a result names the request it answers.

*Alternative:* `async fn` / futures. Rejected: there is no executor in the egui loop, and polling once per frame is all that is needed.

### 2. Cache and pending state (sketch_ui `labels.rs`)

- **Line entries:** `HashMap<LineRequest, Entry>` with `Entry = Pending | Done(Result<(LineMetrics, Triangles), TypesetError>)`.
- **Per frame**, `label_paint`:
  1. drains `take_completed()` into the entries;
  2. for each line of each visible label, requests and inserts `Pending` if it is missing (`Immediate` completes inside `request`, so the next drain within the same `label_paint` call finds it, and the native path never shows a pending form);
  3. sweeps unused entries at the end, as in #59.
- **Last complete rendering:** `HashMap<GeometryId, Vec<(LineMetrics, Triangles)>>`, updated whenever all lines of a label are `Done(Ok)`. While any line is `Pending`, this rendering is placed with the current frame map (spec "Label moves while pending"). Entries of labels that no longer exist are dropped.
- **Fallback reasons:** `PaintItem::Fallback` gets `reason: FallbackReason`:
  - `Pending` → grey;
  - `Failed(TypesetError)`, `NoTypesetter`, `LoadFailed(String)` → orange.
  
  This replaces `error: Option<TypesetError>`.
- **Status line:**
  - `Loading` with any pending label → "loading typesetter…";
  - `Failed(msg)` → "typesetter could not be loaded: msg";
  - otherwise the first failing label, as in #59.
- **Typesetter state `Failed`:** no more requests are sent, and every label without a complete rendering shows `LoadFailed`.

### 3. Platform (sketch_ui `platform.rs`)

```rust
pub trait Platform {
    /// Optional capability: label typesetting.
    fn typesetter(&self) -> Option<&dyn Typesetter>;
}
pub struct Host { typesetter: Option<Box<dyn Typesetter>> }   // impl Platform; built in main.rs
```

- `App::new(Box<dyn Platform>)`. The cache no longer owns the typesetter; `label_paint(.., typesetter: Option<&dyn Typesetter>)` gets it from the platform each frame.
- `main.rs`, the only place with `cfg(target_arch)`:
  - native: `Host::new(Some(Box::new(Immediate::new(TypstTypesetter))))`;
  - browser: `Host::new(Some(Box::new(WorkerTypesetter::new(WORKER_URL, wake))))`, where `wake` calls `cc.egui_ctx.request_repaint()`.
- `App::with_sample()` (tests) builds a `Host` without a typesetter.

*Alternative:* one `Platform` struct per host in its own crate. Rejected: `Platform` lives in the binary crate, so other crates cannot implement it. A struct that holds boxed services is enough until #8 adds file services, which may need host-specific methods.

### 4. JSON messages across the worker boundary (label_layout `wire.rs`)

Messages are JSON text, written and read with `serde_json`, through **mirror types** in `label_layout::wire`. `constraint_runtime` stays serde-free, as decided in `command-serde`. These are the same pattern as `sketch_session/wire.rs`.

- **Mirror types:**
  - `WireRequest { id, line, markup, size, preamble }`, with `markup` as `"latex"` / `"text"`;
  - `WireLine { width, height, depth, shapes }`, `WireShape { paint, contours }` (`paint`: `{"fill": {}}` or `{"stroke": {"width": w}}`), and `WireSegment`, a tagged enum (`move` / `line` / `quad` / `cubic` / `close`) with points as `[x, y]`;
  - `WireError { unsupported | invalid }` with its text.
  
  Each has `From` / `TryFrom` conversions to the `label_layout` and `constraint_runtime` types.
- **Worker → app:** `WorkerMessage` is a tagged enum: `{"ready": {}}` once Typst is initialized, and `{"line": {"id": …, "result": {"ok": WireLine} | {"err": WireError}}}` per answered request.
- **Numbers:** `serde_json` with `float_roundtrip`, which the workspace already enables through `sketch_session`, so coordinates arrive bit-exact. Non-finite values cannot occur in Typst output. Writing one is an error, as in `sketch_session::wire`.
- **Dependencies:** `serde` and `serde_json` are already in the main module through `sketch_session`. They are added to `label_layout`, and so also to the worker.

*Alternative:* own byte format. Rejected: it needs a codec of its own, and it can't be read when debugging. Its advantage (compact, about 3–4× smaller than JSON for coordinates) does not matter at a few kB per line and about 1 ms of typesetting. *Alternative:* serde derives on the core types, behind a `serde` feature of `constraint_runtime`. Rejected: the core stays serde-free; format decisions belong to the outer layers.

### 5. Worker (`label_worker`, wasm only)

- A `cdylib` with `wasm-bindgen`, depending on `label_typst` and `label_layout`.
- Its start function sets `onmessage` on the `DedicatedWorkerGlobalScope`, parses a `WireRequest` from the message string, typesets it with `TypstTypesetter`, and posts the `WorkerMessage::Line` as a JSON string. It sends `WorkerMessage::Ready` after warming up: one dummy line to load the font book and library, the 87 ms cold start, inside the worker.
- **Build: Trunk worker asset** in `index.html`: `<link data-trunk rel="rust" href="../label_worker/Cargo.toml" data-type="worker">`, so Trunk builds it with wasm-bindgen (no-modules) for a worker. The spike used `data-loader-shim`; the final build starts the worker with an own boot script (below).
  - Because nothing references the worker module, the browser fetches it only when the client does `new Worker(url)`.
  - **Spike result (task 1.2, Trunk 0.22.0):**
    - **Files:** Trunk emits `label_worker_loader.js` (`importScripts("./label_worker.js"); wasm_bindgen("./label_worker_bg.wasm")`), `label_worker.js` and `label_worker_bg.wasm`, **without** file hashes. The main module keeps its hashed names.
    - **No preloading:** the generated `index.html` has no script, preload or modulepreload entry for the worker. The URL to pass to `new Worker` is `label_worker_loader.js`, relative to the page, so it also works under the Pages base path `/illustrator/`.
    - **Build time:** `trunk build --release`, including `wasm-opt -Oz` (version_131) on both modules, takes 1 min 37 s in the dev container with a warm cargo cache for dependencies.
    - **Sizes (gzip -9):** main module 2.96 MB raw / **1.38 MB gzip** (as before); worker 27.3 MB raw / **10.8 MB gzip**.
  - **Spike result (task 1.3, headless Chromium via Playwright, `trunk serve` debug build):**
    - **Fetched on demand:** the worker files are requested only when `new Worker("label_worker_loader.js")` runs, 2.1 s after page load in the test (the timer delay), not with the page.
    - **Round trip works:** the worker typesets `leg $\ell = \frac{3}{2}$` and posts back, with no panic from rayon or threads. Cold start to first answer took 752 ms, of which 379 ms was typesetting (debug build, unoptimized). The release build is faster.
    - **UI not blocked:** the UI thread kept painting throughout (150 animation frames, largest gap 33 ms in headless Chromium).
    - **Gate passed:** the Trunk worker asset is the build route. The pre-build-hook fallback (build with cargo + `wasm-bindgen --target no-modules` in a `pre_build` hook) is not needed.
- **Own boot script instead of the loader shim** (found in task 4.3).
  - **The problem:** Trunk's `label_worker_loader.js` calls `wasm_bindgen(...)` and drops the returned promise. If the `.wasm` fails to load, the rejection stays inside the worker and the page never learns of it, so the label would stay "pending" forever.
  - **The fix:** `index.html` declares the worker asset **without** `data-loader-shim` and copies `sketch_ui/label_worker_boot.js` (`rel="copy-file"`). That script does `importScripts("./label_worker.js")` and then `wasm_bindgen({ module_or_path: … })`. A failed load is rethrown with `setTimeout(() => { throw … })`. The resulting uncaught error reaches the page as the worker's `error` event, with the message.
  - **The URL** for `new Worker` is `label_worker_boot.js`.
- **The client's `onerror`** takes a plain `Event`. A worker script that fails to load fires a plain `Event`; an uncaught error inside the worker fires an `ErrorEvent`, whose message is reported.
- **Checked in the browser** (task 4.3, Playwright blocking the files):
  - the blocked boot script gives "typesetter could not be loaded: worker script failed to load";
  - the blocked `.wasm` gives "… typesetter module failed to load: TypeError: Failed to fetch";
  - in both cases the labels show orange.

### 6. Worker client (`label_web`, wasm only)

- `WorkerTypesetter { url, wake, core: Rc<RefCell<Core>>, worker: OnceCell<Worker> }` implements `Typesetter`.
- **`Core` is pure Rust and natively testable:**
  - `next_id`, `in_flight: HashMap<u32, LineRequest>`, and `queued: Vec<WireRequest>` for requests made before the worker is ready;
  - `completed: Vec<…>`;
  - `state`.
  
  It handles "request" (buffer while loading), "ready" (flush the queue), "response" (match the id), and "error" (→ `Failed`).
- **The first `request`** creates the worker with `onmessage` / `onerror` closures that feed `Core` and call `wake`. Requests before that only change `state` to `Loading`.
- **Duplicate requests** for the same line while one is in flight are not sent twice: `Core` checks `in_flight`. Messages it cannot read are ignored.

### 7. CI and Pages

- `scripts/check-wasm.sh` adds `-p label_worker -p label_web`.
- The Pages job is unchanged, because Trunk builds both from `index.html`. Its run time grows with Typst's compile and `wasm-opt`.
- The spike measures the sizes of `dist/` after `trunk build --release`: the main module should still be about 1.3 MB gzip, and the worker about 12 MB.

### 8. Verification in a browser

The container has no browser.
- **Automated route:** Playwright (`npx playwright`) with headless Chromium, downloaded on demand, against `trunk serve`. A small script loads the page, waits for the status line or for the canvas, and takes a screenshot.
- **Without network access** for the download, the browser check is listed for the user to do by hand (`trunk serve`, open the page, look at the sample label).
- Native tests carry the logic (cache, `Core`, wire format). The browser check confirms the plumbing.

## Risks / Trade-offs

- **Trunk worker naming or hashing doesn't fit a lazily created worker.** → The spike gate (task group 1) decides between the Trunk worker asset and the pre-build hook, before any other work.
- **Typst under `wasm-opt -Oz`:** the release build may take minutes. → Measured in the spike. If it's too slow, use `data-wasm-opt="s"` or skip `wasm-opt` for the worker only; size matters less for a module loaded on demand.
- **rayon in a worker:** #11 saw it fall back to one thread under Node. In a browser worker it must not try to spawn threads. → Checked in the spike's round trip. If needed, configure rayon's global pool with one thread at worker start.
- **First result after 1–10 s of download:** labels show the grey pending form meanwhile, and the status line explains it (spec). This is accepted.
- **Breaking `Typesetter` rename:** only in-repo users (`label_typst`, `sketch_ui`) are affected.

## Migration Plan

None for users. Files and commands are unchanged. Native behaviour is unchanged, because `Immediate` completes in the same frame.
