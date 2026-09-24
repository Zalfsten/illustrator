# Tasks

## 1. Viewport (sketch_ui)

- [x] 1.1 Add `viewport` module with `Viewport` (uniform scale, origin_screen, y-flip in `to_screen` / `to_world`, `px_to_world`); verify unit tests: round-trip, increasing world y decreases screen y, `px_to_world(12) == 12/scale`
- [x] 1.2 Add `fit` (or equivalent) from canvas rect + world bounds with margin; verify unit test that fitted scale is positive and maps bounds inside the rect

## 2. App wiring

- [x] 2.1 Rebuild Viewport each paint from canvas size and fixed world bounds; paint Mesh edges and handles via `to_screen`; verify `cargo build -p sketch_ui`
- [x] 2.2 Map pointer → world for segment clicks, drag soft targets, Mesh hits, and logical-point hits; convert pixel radii via `px_to_world`; verify build succeeds
- [x] 2.3 Rewrite sample geometry in world units (y-up right angle near origin); verify app compiles

## 3. Glossary

- [x] 3.1 Add world coordinates (y-up) and Viewport notes to `docs/constraint-mathematics.md`; verify terms match the deltas

## 4. Verification

- [x] 4.1 Run `cargo test -p constraint_runtime` and `cargo test -p sketch_ui` (or build if no UI tests beyond Viewport); verify all pass
