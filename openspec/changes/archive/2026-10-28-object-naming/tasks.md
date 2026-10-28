# Tasks

Paths relative to `impls/rust/crates/constraint_runtime` unless noted. Tests: `cargo test --workspace` from `impls/rust`.

## 1. Kind prefixes

- [x] 1.1 Add `const NAME_PREFIX: &'static str` to `Geometry` (`src/geometry/formula.rs`, no default) and to `ConstraintKind` (`src/constraint.rs`). Set `seg`, `arc`, `join`, `par`, `orth`, `fix`, `eqlen` (D2). Add `GeometryKind::name_prefix()` via `with_kind!` and `ConstraintInst::name_prefix()` in `constraint_catalog!`. Update the "Adding a kind" docs in `geometry/formula.rs`, `geometry_kinds/mod.rs` and `constraint_kinds/mod.rs`. Verify `cargo test -p constraint_runtime` is green.
- [x] 1.2 Test in `src/geometry_kinds/tests_catalog.rs` (plus a constraint-side test in the existing constraint test file): every prefix is a valid name (name syntax from 2.1), does not end in a digit, and is unique across all geometry and constraint kinds (spec "Kinds supply name prefixes"). Verify that the test passes after 2.1.

## 2. Name table

- [x] 2.1 New `src/names.rs` (D1, D3): `ObjectId`, `NameError`, `is_valid_name`, `Names` with insert / remove / rename / lookup both ways, the stem → numbers index, and `next_auto(prefix)` (max + 1, `u64` overflow fallback). Register the module in `src/lib.rs` and re-export the public types. Verify with `cargo doc -p constraint_runtime --no-deps` (0 warnings).
- [x] 2.2 Tests in `src/tests_names.rs`, one per spec scenario of "Name syntax" and "Auto-naming numbers continue after the highest in use": valid/invalid names, case-sensitivity, `seg1 seg2 seg5` → `seg6`, independent prefixes, top name reused after removal, user name `seg7` counts, `seg01` not counted and no collision, overflow fallback. Rename: free, taken, invalid, same name. Verify that the tests pass.

## 3. Document integration

- [x] 3.1 `src/document.rs` (D4): private `names` field; `add_geometry` / `add_constraint` assign `next_auto(prefix)`; `name`, `geometry_name`, `constraint_name`, `lookup`, `rename`; `Removal`; `remove_geometry` / `remove_constraint` return `Option<Removal>` and drop names, including the cascade. Update the four `assert!(doc.remove_*(..))` test calls to `.is_some()`. Verify `cargo test --workspace` is green.
- [x] 3.2 Tests in `src/tests_document.rs`: geometry and constraint names never collide; lookup round trip; adding a parallel then a perpendicular constraint yields `par1` and `orth1`; removing `seg2` keeps `seg1`/`seg3`; geometry removal reports and drops `seg1`, `par1`, `join1`; rename keeps parameter values and constraints; a cloned Document (snapshot) restores names. Verify that the tests pass.

## 4. Addresses

- [x] 4.1 New `src/address.rs` (D5): `Address`, `AddressError`, `Document::resolve_address`, `point_address`, `vec_address`, `length_address`, built on the id-based capability factories. Re-export from `src/lib.rs`. Verify `cargo doc -p constraint_runtime --no-deps` (0 warnings).
- [x] 4.2 Tests in `src/tests_address.rs`: each suffix on a segment and an arc evaluates like the static factory; `seg1.direction` is a vector; errors for malformed (`seg1`, `seg1.`, `seg1.at(`, `seg1.foo`), unknown name, constraint name, `at(1.5)` / `at(-0.1)` / `at(NaN)`, and missing capability (`arc1.direction`, `seg1.center`, `seg1.radius`); print → resolve round trip for every addressable formula; `Uniform` prints as `None`. Verify that the tests pass.

## 5. GUI

- [x] 5.1 `crates/sketch_ui/src/app.rs` (D6): `report` takes the success message; the Segment, Join, Parallel/Orthogonal and FixPoint tools show names and addresses on pick and after adding. Verify `cargo build -p sketch_ui` and `scripts/check-wasm.sh`. Manual smoke test by the user: drawing a segment shows `Added seg…`, a join shows `Join: seg….end → pick second point`, and then `Added join…`.

## 6. Docs

- [x] 6.1 Update the `openspec/config.yaml` context (module list: `names.rs`, `address.rs`; kind prefixes) and `impls/rust/README.md` if it lists modules. Verify `openspec validate object-naming --strict` still passes and `openspec list --json` parses the config.

## 7. Integration

- [x] 7.1 From `impls/rust` run `cargo test --workspace`, `cargo clippy --workspace --all-targets` (0 warnings), `cargo doc -p constraint_runtime --no-deps` (0 warnings), `cargo bench -p constraint_runtime --bench solve --no-run` and `scripts/check-wasm.sh`. Do not run a crate-wide `cargo fmt`; format only touched files. Verify that all pass.

## Workflow follow-up

- Archive the change after merge, then close #15 and tick it in #14.
