# Proposal

## Why

Figures for publications and teaching will be built and edited not only by mouse, but by scripts, a command API and AI clients (#17, #18, #19), saved to files (#8) and annotated with labels anchored to objects (#12). All of these need a way to refer to "that segment" or "the end of that arc" that a human can read and write. Today objects only have dense internal ids, which are not meaningful to users and will be renumbered when a file is loaded.

This change (GitHub #15) gives every geometry and every constraint a user-visible name, unique at any time, and a small address grammar for points and quantities on named geometries. It is the first building block of the command API (#32) and is independent of it.

## What Changes

- **Every object has a name.** Geometries and constraints share one namespace; a name is unique among all live objects at any time.
- **Auto-naming on creation:** each geometry kind and each constraint kind supplies a short prefix (`seg`, `arc`, `join`, `par`, `orth`, `fix`, `eqlen`). The new name is the prefix plus the highest number currently in use with that prefix plus one. The next name is derived from the current names only; no counter is stored. A name that is no longer in use may be given out again.
- **Rename** to any valid, unused name. Invalid or taken names are rejected with a reason. The system never renames an object on its own.
- **Name syntax:** `[A-Za-z_][A-Za-z0-9_]*`, case-sensitive.
- **Lookup in both directions** (name → object, object → name) without scanning all objects.
- **Removal drops names**, including the names of constraints removed together with a geometry, and reports which names were removed.
- **Address grammar** for quantities on named geometries: `seg1.start`, `seg1.end`, `arc1.center`, `seg1.at(0.5)`, `seg1.tangent(0.5)`, `seg1.normal(0.5)`, `seg1.direction`, `seg1.length`, `arc1.radius`. Resolution uses the capability queries from the previous change, so a missing capability is a resolution error (for example `arc1.direction`), not a crash. Expressions can also be printed back as addresses.
- **GUI:** the status bar names the picked objects and the objects just created (`Join: seg1.end → pick second point`, `Added par1`). No canvas labels.
- **BREAKING (Rust API only):** removing a geometry or constraint returns what was removed (names included) instead of `bool`.

## Capabilities

### New Capabilities

- `object-naming`: user-visible object names (syntax, uniqueness, auto-naming, renaming, lookup, removal) and the address grammar for points, vectors and lengths on named geometries.

### Modified Capabilities

- (none) The existing identity requirements (`system-architecture` "Constraints are resolved by stable identity", `geometry-ir` "Geometries are resolved by stable identity …") stay as they are: internal identities remain the solve-path handles; names are an additional, user-facing layer.

## Non-goals

- Command API, undo/redo of renames, structured command errors (#32, #33). Rename here is a Document operation; turning it into an undoable command is #32.
- File format and load path (#8): names will come from the file; ids are compacted on load. Nothing is persisted in this change.
- Geometry tree view, rename via GUI, labels on the canvas (#12). The tree view is not specified yet.
- Reserved words of scripting front-ends (Rhai keywords); handled by the front-end (#18).
- Uniform (arc-length) curve parameters in addresses; `at(t)` uses the normalized parameter. Can be added later without changing existing addresses.

Specs stay shared across implementations; how names are stored and how addresses are parsed is a design decision of each implementation (here `impls/rust`).

## Impact

- `impls/rust/crates/constraint_runtime`: new modules for names and addresses (with sibling test files); `Document` owns the name table and keeps it consistent on add/remove; `Geometry` and `ConstraintKind` gain a name prefix; `ConstraintInst` exposes it; `remove_geometry` / `remove_constraint` return a removal report.
- `impls/rust/crates/sketch_ui`: status messages use names and addresses.
- Solve path unchanged: names live outside the geometry and constraint stores used by evaluation and assembly.
- Specs: new `openspec/specs/object-naming/spec.md` on archive.
