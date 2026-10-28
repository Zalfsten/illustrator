# Tasks

## 1. Core: congruence classes and logical points (`constraint_runtime`)

- [x] 1.1 Add `src/point_classes.rs` with `Document::point_class` (BFS over `points_identified_with`, D2) and register the module; verify with `src/tests_point_classes.rs`: single point, direct Join, three-endpoint chain (all three report the same set), coincident unjoined endpoints in separate classes, class splits after removing the middle Congruence
- [x] 1.2 Add `LogicalPoint` and `Document::logical_points(filter)` (D3) and re-export `LogicalPoint` from `lib.rs`; verify with tests: coincident unjoined endpoints give two logical points, a Join chain gives one with three members, representative is the lowest-id visible endpoint, construction-only endpoints are absent under `RESULT_ONLY` and present under `INCLUDE_CONSTRUCTION`, a class joining a hidden and a visible endpoint has the visible one as representative, result is unchanged after moving values in the Vector
- [x] 1.3 Use `point_class` for the mouse target in `interact.rs` `soft_for_drag` (D4); verify with a test in `tests_interact.rs`: dragging the first point of a three-endpoint Join chain moves all three to the pointer target within tolerance, and existing interact tests still pass (`cargo test -p constraint_runtime`)

## 2. Session: reject joins within one class (`sketch_session`)

- [x] 2.1 Add `CommandError::AlreadyJoined { a, b }` with its `Display` and the check in `Command::Join` before the constraint is added (D5); verify with tests in `tests_session.rs`: join a point with itself, join across an existing chain (both fail with `AlreadyJoined`, no new constraint, no undo step), join two coincident unjoined endpoints succeeds (`cargo test -p sketch_session`)

## 3. GUI: handles from the core (`sketch_ui`)

- [x] 3.1 Switch `app.rs` to `Document::logical_points` (D6): store `Vec<LogicalPoint>`, paint and hit-test the representative, delete `logical_points_for` and `congruent`, rebuild only after commands, undo/redo and presentation-mode change; verify `cargo build -p sketch_ui`, `cargo clippy -p sketch_ui` without new warnings, and `grep -n "1e-3\|logical_points_for\|congruent(" impls/rust/crates/sketch_ui/src/app.rs` finds nothing
- [x] 3.2 Manual check in the running app: draw two segments with exactly coincident ends (no Join) and drag the corner, only one segment's end moves and two handles remain; join two segment ends and drag, both move; the sample scene's fillet corners still drag as one point

## 4. Integration

- [x] 4.1 Run `cargo test --workspace` and `scripts/check-wasm.sh` from `impls/rust`; verify both succeed
- [x] 4.2 Update the `openspec/config.yaml` context line on Document queries to mention `point_classes.rs` (alongside `names.rs`, `address.rs`); verify `openspec list --json` still parses the config

## Workflow follow-up

- Close #36 and tick item 1 in #35 after merge.
- Archive the change with `/opsx:archive`.
