# Proposal

## Why

Since #59 the native app typesets labels, but the browser build has no typesetter: every label there shows its source in the "not typeset" form. The hosted demo is the first thing many readers see, so labels must look the same in the browser.

Typst with its fonts adds about 11 MB gzip (#11). Shipping it in the initial page would make the page about ten times larger for everyone, including people who never use labels. This change (#60) loads the typesetter in the browser only when a label needs it, and runs it beside the app so that the app never waits for it.

## What Changes

- **Typesetting becomes asynchronous.**
  - A request never blocks the input–solve–paint loop. Results arrive by completion and trigger a repaint.
  - The typesetter has a queryable state: loading, ready, or failed.
  - A host whose engine answers at once (the native app) delivers results within the same frame.
- **A new "pending" state for labels:**
  - a label that was typeset before keeps its last complete rendering until all its lines are typeset again;
  - a label without one shows its source in the not-typeset form, marked as pending (grey rather than the orange of a failure);
  - the App indicates that typesetting is loading.
- **A load failure** (offline, missing file, unsupported browser) leaves labels in the not-typeset form, and the App says that the typesetter could not be loaded.
- **The host boundary (`Platform`) is introduced.** It is passed to the App when it starts, and the typesetter is its first optional capability. #8 will add file services to the same boundary.
- **In the browser,** the Typst typesetter runs in a Web Worker. Its module is fetched only when the first label line is requested, so the initial page keeps its size. Natively, Typst stays linked directly.

## Non-goals

- Editing labels (#63), and file dialogs and other host services (#8). They will use the same `Platform` boundary.
- Retrying a failed load automatically, and caching the module across visits beyond what the browser cache does.
- A smaller Typst build (math-only fork) or a different engine in the browser (#11 rejected both).
- Typesetting several lines in parallel.

The asynchronous contract, the pending state and the typesetter as an optional host capability are shared specs. The worker, the message format, the build of the second module and the class names are design decisions of the Rust implementation.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `label-typesetting`:
  - "Typesetting is a replaceable backend": asynchronous requests and a queryable typesetter state;
  - "Labels that cannot be typeset stay visible": also covers a typesetter that failed to load;
  - new requirement for labels whose typesetting is pending.
- `system-architecture`: "Optional host capabilities are discoverable" gains the typesetter as an example of an optional capability that the host boundary provides.

## Impact

- **`label_layout`:**
  - the asynchronous `Typesetter` port, with request, completion and state;
  - the synchronous engine trait `LineTypesetter`, with an adapter that completes immediately;
  - JSON messages for requests and lines across the worker boundary, through serde mirror types in `label_layout` (the core stays serde-free).
- **`label_typst`:** implements `LineTypesetter` instead of the old synchronous `Typesetter`.
- **New crate `label_worker`** (wasm only, `cdylib`): the Web Worker that wraps `label_typst`.
- **New crate `label_web`** (wasm only): the worker client that implements `Typesetter`. It starts the worker lazily and requests a repaint on every result.
- **`sketch_ui`:**
  - `Platform` trait and `App::new(Box<dyn Platform>)`;
  - the cache gains the pending state and keeps the last complete rendering of each label;
  - grey pending form and status texts;
  - `main.rs` builds the platform per host;
  - `index.html` declares the worker for Trunk.
- **Build and CI:**
  - `scripts/check-wasm.sh` also builds `label_worker`, so Typst is compiled for wasm32 in CI, which takes longer.
  - The Pages build gains a second, separately fetched module (about 12 MB gzip). The initial page stays about 1.3 MB gzip.
