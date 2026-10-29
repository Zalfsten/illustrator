# Tasks

## 1. Session start with report (`sketch_session`)

- [x] 1.1 Add `LoadReport` and `Adjusted`, and `Session::start(doc) -> (Session, LoadReport)` with the #51 loop, recording the displacement of each component the start-up solve changes (D5: samples `t = k/32` via `on_curve_expr`); make `Session::new` delegate to it; verify with `tests_session.rs` that the existing session-start tests still pass, that a drawing within tolerance reports nothing, that a segment whose end the solve moves by a known amount reports that displacement within solve tolerance, 1e-6 ("Moved end of a segment"), and that a contradicting component appears in `conflicts` and not in `adjusted`

## 2. Load mode (`sketch_session`)

- [x] 2.1 Add the execution `Mode` (`Interactive` | `Load`) to `execute` / `constrain` (D4): `Load` skips the frozen-operand check and the solve; verify with `tests_session.rs` that all existing command tests pass unchanged (they use `Interactive`)

## 3. File reading and writing (`sketch_session`)

- [x] 3.1 Add `src/file.rs` with `FORMAT`, `VERSION`, `SaveError` and `Session::save` (D1, D2: fixed envelope lines, one compact command per line, `,\n` separators, trailing newline, `[]` for an empty drawing); verify with `src/tests_file.rs`: the exact bytes for a drawing with two segments and a join, an empty drawing, the "One command per line" scenario, and "Unaddressable drawing cannot be saved" (a `Uniform` operand gives `SaveError::Export` naming the constraint)
- [x] 3.2 Add `LoadError`, `CommandLoadError`, `migrate` and `Session::load` (D3 check order, D4 load mode, D5 report); verify with `tests_file.rs` covering every scenario of "Sketch file envelope", "Files are recognized by content and version" (zip signature bytes, version 2, single-line and fully expanded layouts), "Load mode executes commands without solving" (bit-identical values, no undo, forward reference fails at its index) and "Load errors reject the whole file" (taken name at 1, `already joined` at 5, rename at 3)
- [x] 3.3 Add the round-trip and report tests: "Save, load, save is byte-identical" for the GUI sample scene and for a drawing with an arc, construction role and every constraint kind; "Imprecise file is adjusted" (adjusted component names `a` and `b`, displacement > 0, perpendicular holds); "Contradicting file is held" (conflicted, values equal the file); verify `cargo test -p sketch_session`
- [x] 3.4 Export `FORMAT`, `VERSION`, `LoadReport`, `Adjusted`, `LoadError`, `CommandLoadError` and `SaveError` from `lib.rs`, and update the `sketch_session` paragraph of the `openspec/config.yaml` context (file module, load mode, session-start report); verify `cargo doc -p sketch_session --no-deps` builds without warnings and the context names `Session::load` / `Session::save`

## 4. Integration

- [x] 4.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets`, `cargo fmt --check -p sketch_session` from `impls/rust`, and `scripts/check-wasm.sh`; verify that all pass, that `constraint_runtime` is unchanged and that `sketch_ui` changes only in `tests_app.rs` (sample-scene file round trip)
- [x] 4.2 Run `openspec validate sketch-file-format --strict`; verify it reports the change as valid

## Workflow follow-up

- After merge: note in #8 that the format and load/save are done and the host/UI part remains; note in #19 that `Session::save` gives the canonical "current drawing".
- Archive the change with `/opsx:archive`.
