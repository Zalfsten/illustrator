# Tasks

## 1. Dependencies and wire adapters (`sketch_session`)

- [x] 1.1 Add `serde` (derive), `serde_json` (feature `float_roundtrip`) and `schemars` 1.x to `sketch_session`, and `jsonschema` as a dev-dependency; verify `cargo build -p sketch_session` and `scripts/check-wasm.sh` from `impls/rust` succeed and `constraint_runtime/Cargo.toml` is unchanged
- [x] 1.2 Add `src/wire.rs` (D5): `point` and `finite` serde adapters with schemas, and the `PointAddr` / `VecAddr` / `LenAddr` newtypes; verify with `src/tests_wire.rs`: `[x, y]` round trip bit for bit for 0.1, 1/3, 1e-300, -0.0 and `f64::MAX`; `[0,0,0]`, `[0]` and `"x"` are rejected; NaN and ±inf fail to serialize; `1e999` fails to parse

## 2. Kind traits and catalog (`sketch_session`)

- [x] 2.1 Define `GeometryCommand`, `ConstraintCommand`, `Resolver` (moving the `point` / `vector` / `length` helpers from `Session`), `CommandError::InvalidValue { field, reason }` with `Display`, and `ExportError` (D1, D5, D7); verify `cargo build -p sketch_session`
- [x] 2.2 Implement the traits in `src/kinds.rs` + `src/kinds/{segment,arc,join,parallel,perpendicular,fix,equal_length}.rs` with tags from the spec, params types with `deny_unknown_fields`, finite checks, the arc `radius >= 0` check (schema `minimum: 0`), and `Congruence::check` for `AlreadyJoined`; verify with `src/tests_kinds.rs`: `init` / `resolve` reject NaN, inf and radius -1 with `InvalidValue` naming the field, accept radius 0, and `params(init(p)) == p` for each geometry kind
- [x] 2.3 Add `src/catalog.rs` with `command_catalog!` (D2) generating `GeometrySpec`, `ConstraintSpec`, create/build dispatch and the reverse mapping; verify `cargo build -p sketch_session` and that `grep -n "Segment\|Arc\|Congruence\|VectorsParallel" src/session.rs` finds no per-kind arm

## 3. Commands, reading and writing (`sketch_session`)

- [x] 3.1 Restructure `Command` into `Create` / `Constrain` / `Delete` / `Rename` / `SetRole`, add `DeclCommand` with `From<DeclCommand> for Command` (D3), and reduce `Session::execute` to five arms; update `tests_session.rs` to the new shape and verify all existing session tests pass (`cargo test -p sketch_session`), including `already joined` and `unsatisfiable`
- [x] 3.2 Implement the two-step `Serialize` / `Deserialize` for `Command` and `DeclCommand` plus `from_json` / `to_json` with `ReadError` / `WriteError` (D4, D8); verify with `src/tests_command.rs` covering every scenario of the requirements "Commands have one interchange form", "Kinds are selected by a kind tag", "Interchange field shapes", "Interchange reading is strict" and "Declarative subset": the literal JSON from the spec reads to the expected command, unknown op/kind/field, `rol`, missing `name` in the subset, and `rename` in the subset are rejected
- [x] 3.3 Add the round-trip tests for "Interchange round trip is lossless": every command variant and kind (including the values 0.1, 1/3, 1e-300, -0.0) is identical after `to_json` → `from_json`, and a NaN coordinate makes `to_json` fail; verify `cargo test -p sketch_session`
- [x] 3.4 Add a test that every reserved name (`op`, `kind`, `name`, `role`) is absent from each kind's params schema, plus a debug assertion on write (D4); verify `cargo test -p sketch_session`

## 4. Export (`sketch_session`)

- [x] 4.1 Implement `Session::declarative_commands` (D7); verify with tests for "Drawing as declarative command list": the export scenario's drawing yields `seg1`, `x_axis` (construction), `fillet`, `join1`, `par1` in that order with those addresses; after rename and delete, `base` is present and `seg3` and its constraints are absent; re-executing the list in `Session::new(Document::new())` and exporting again yields equal names, kinds, roles and addresses with values within solve tolerance
- [x] 4.2 Export the GUI's sample scene (built on `Document`, then `Session::new`) and verify in a test that it exports without `ExportError` and survives JSON round trip and re-execution

## 5. Schemas (`sketch_session`)

- [x] 5.1 Generate the `Command` and `DeclCommand` schemas from the catalog (D6) and check them in under `crates/sketch_session/schema/`; verify with `tests/schema.rs`: generated equals checked-in (with `UPDATE_SCHEMA=1` rewriting), every op and kind tag appears, the subset schema requires `name` and has only `create` / `constrain`
- [x] 5.2 Validate the spec scenario JSON against the schemas with `jsonschema`: the valid examples pass; `rol`, `[0,0,0]`, `"op": "parallel"`, `"kind": "spline"` and radius -1 fail; verify `cargo test -p sketch_session --test schema`

## 6. GUI (`sketch_ui`)

- [x] 6.1 Update command construction in `src/app.rs` to `Command::Create` / `Command::Constrain` with `GeometrySpec` / `ConstraintSpec`; verify `cargo build -p sketch_ui` and `cargo clippy -p sketch_ui` without new warnings
- [x] 6.2 Manual check in the running app: draw a segment, join two ends, add parallel, perpendicular and fix, undo/redo; behaviour is as before

## 7. Integration

- [x] 7.1 Run `cargo test --workspace`, `cargo clippy --workspace` and `scripts/check-wasm.sh` from `impls/rust`; verify all succeed
- [x] 7.2 Update the `sketch_session` paragraph in `openspec/config.yaml` context (command catalog, kind traits, `DeclCommand`, wire form, schemas); verify `openspec list --json` still parses the config

## Workflow follow-up

- Tick the command-API part of #24 and close #50 after merge; note in #8 and #18 that they are unblocked.
- Archive the change with `/opsx:archive`.
