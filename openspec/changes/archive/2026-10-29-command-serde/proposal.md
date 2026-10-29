# Proposal

## Why

The JSON form of commands is the shared base of the file format (#8), the JSON command protocol (#18) and the MCP bridge (#19); all three are blocked until it is fixed (#50). Today `Command` is an in-memory Rust enum with no wire form, and every geometry or constraint kind is a hand-written variant matched centrally in the session, so each new kind would touch the command layer, the file format and the protocol again (#24).

This advances the product goal by making drawings persistable and scriptable without losing their relationships: a file is a list of declarative commands that re-create named geometries and constraints, and humans, scripts and AI clients read and write the same language.

## What Changes

- **Wire form of commands** with a closed set of operations: `create`, `constrain`, `delete`, `rename`, `set_role`. Geometry and constraint kinds are selected by a `kind` tag (`segment`, `arc`; `join`, `parallel`, `perpendicular`, `fix`, `equal_length`) in one flat JSON object. Points are `[x, y]`, angles in radians, objects and points / free vectors / lengths are referred to by name and address strings. Unknown fields are rejected; `op`, `kind`, `name` and `role` are reserved field names.
- **Declarative subset** for files: only `create` and `constrain`, each with an explicit name; delete, rename, set role, drags, queries, undo/redo and transactions are not part of a file. A drawing can be turned into its declarative command list.
- **Value validation**: non-finite numbers are rejected, and kinds reject out-of-range parameters (an arc radius below zero). Both fail with a new error class `invalid value`.
- **Kind-supplied command parameters** (#24, command-API part): each geometry and constraint kind supplies its tag, its parameters, how they map to and from the drawing, and its part of the schema; the command layer dispatches over the kind list without a hand-written per-kind match. **BREAKING** (implementation API only): the per-kind `Command` variants (`CreateSegment`, `Join`, …) are replaced by `Create` / `Constrain` with a kind-specific payload; the GUI's call sites change accordingly.
- **Machine-readable schema** for the protocol commands and for the declarative subset, generated from the same source.

## Non-goals

- File envelope (`format`, `version`), load mode without eager solve, residual check on load, `Platform` trait and Open/Save UI (#8). Per-kind schema versions are decided there.
- JSON Lines envelope, request ids, Rhai scripting (#18); MCP bridge (#19).
- Structured errors with cause and objects involved, queries and result shapes (#33). This change adds one error class, not the structured error form.
- Dropping duplicate constraints (#53) and the `already joined` vs. duplicate question.
- Kind traits for GUI tools and Mesh tessellation (rest of #24).
- Styling, labels and other illustration data in commands; they arrive with their own changes.

The wire form and the declarative subset are shared contracts: every implementation reads and writes the same JSON. The Rust crates are one implementation of them.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `sketch-commands`: commands gain a normative interchange form (closed operation set, kind tags, field shapes, strictness), a declarative subset with mandatory names, value validation with the new error class `invalid value` (non-finite numbers, negative arc radius), and conversion of a drawing into its declarative command list.

## Impact

- `sketch_session`: new command catalog (per-kind parameter types and their traits, generated dispatch), serde and JSON Schema for commands, `Command` restructured, `CommandError::InvalidValue`, drawing → declarative commands; new dependencies `serde`, `serde_json` (with exact float round-trip), `schemars`; checked-in generated schemas with a snapshot test.
- `sketch_ui`: command construction in `app.rs` follows the new `Command` shape.
- `constraint_runtime`: unchanged API and no serde dependency; the arc's canonicalization on add stays as the core's safety net.
- No change to solver, assembly or the geometry model.
