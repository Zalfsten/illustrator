# Design

## Context

Target implementation: `impls/rust`, crate `sketch_session` only. See proposal.md for motivation and specs/sketch-file for the contract.

- `DeclCommand` (#50) already reads and writes one command strictly (`from_json` / `to_json`, `ReadError` / `WriteError` over serde_json errors). `serde_json::from_value::<DeclCommand>` works through the same custom `Deserialize`.
- `Session::declarative_commands()` gives the drawing as a `Vec<DeclCommand>` (geometries, then constraints, creation order) or `ExportError::NotAddressable`.
- `Session::new(doc)` (#51) gives each inconsistent component one `structural_scoped` solve and keeps the failures frozen; it reports nothing. `Session::conflicts()` lists frozen components.
- `Session::execute` runs the five command arms. Its constrain path does `build` (resolve, value checks, `already joined`), the name check, the frozen-operand check, `add_constraint`, then the scoped solve.
- `Document::on_curve_expr(id, CurveParam::Normalized(t))` evaluates a curve point at a normalized position for every kind with the curve capability (all kinds today).

## Goals / Non-Goals

**Goals:**
- Load and save as pure functions on bytes, with no I/O, so the later host change only moves bytes.
- Load mode as a variant of the existing command path, not a second interpreter: the same `build` / `add_to` / name checks run.
- One session-start routine that both `Session::new` and loading use, and that produces the report.

**Non-Goals:**
- Streaming or very large files. A file is read whole; a drawing has at most thousands of commands.
- Cross-implementation byte identity. Each implementation is deterministic on its own (spec); number spelling such as `3.0` vs `3` may differ between implementations.

## Decisions

### D1 Module `file.rs`, API on `Session`

```rust
pub const FORMAT: &str = "illustrator-sketch";
pub const VERSION: u64 = 1;

impl Session {
    pub fn save(&self) -> Result<Vec<u8>, SaveError>;
    pub fn load(bytes: &[u8]) -> Result<(Session, LoadReport), LoadError>;
}
```

`load` returns a new session and never touches an open one, so "an open drawing stays unchanged" holds by construction: the caller replaces its session only on `Ok`.

`SaveError { Export(ExportError), Write(WriteError) }`. `Write` covers a non-finite value, which a document passed to `Session::new` could contain.

### D2 Fixed writer for the envelope, serde for each command

```
{
  "format": "illustrator-sketch",
  "version": 1,
  "commands": [
    {"op":"create","kind":"segment","name":"seg1","role":"result","start":[0.0,0.0],"end":[1.5,0.0]},
    {"op":"constrain","kind":"join","name":"join1","a":"seg1.end","b":"seg2.start"}
  ]
}
```

The writer emits these lines literally, with each command from `DeclCommand::to_json` (compact; field order fixed by `Tagged` and the params structs). Commands are separated by `,\n`, and the file ends with a newline. An empty drawing writes `"commands": []` on one line. Determinism follows from the fixed export order, the fixed field order and serde_json's number formatting (`float_roundtrip`), so save → load → save is byte-identical whenever load keeps the values, which it does within tolerance.

*Alternative:* `serde_json::to_string_pretty`. Rejected: it breaks points over four lines (spec: one command per line).

### D3 Reader: bytes → envelope → commands, in a fixed check order

1. `serde_json::from_slice::<Value>`. A failure, including invalid UTF-8 or zip bytes, gives `LoadError::NotASketchFile(ReadError)`. The serde message keeps line and column for a hand-edited file with a syntax error.
2. The value must be an object whose `format` equals `FORMAT`; otherwise `UnknownFormat`. This is checked first, so a JSON file of some other kind is named as such instead of being reported as missing fields.
3. `version` must be an integer: `1` passes through `migrate(version, map)`, which is identity today and the one place for future migrations. Any other value gives `UnsupportedVersion(v)`.
4. After removing `format`, `version` and `commands`, any remaining key gives `Envelope(…)` naming the unknown field. `commands` must be an array.
5. Each element `i`: `serde_json::from_value::<DeclCommand>` fails with `Command { index: i, error: CommandLoadError::Read(ReadError) }`, otherwise it is executed in load mode (D4). An execution error gives `CommandLoadError::Execute(CommandError)`.

`LoadError` and `CommandLoadError` implement `Display`, with the index counted from 0 as in the spec.

### D4 Load mode as an execution mode of the session

`execute` and `constrain` get a `Mode` (`Interactive` | `Load`). In `Load`, constrain skips the frozen-operand check and the solve. Everything else (`build`, `add_to`, the name checks, `created`) is shared. `load` builds a `Session` around an empty `Document`, executes every command in `Load` mode without snapshot or history, and hands the document to the session start (D5).

*Alternative:* execute the commands directly on a `Document` in `file.rs`. Rejected: it would duplicate name handling, resolution and `already joined`.

### D5 Session start with report

```rust
pub struct LoadReport { pub adjusted: Vec<Adjusted>, pub conflicts: Vec<Conflict> }
pub struct Adjusted { pub geometries: Vec<String>, pub displacement: f64 }

impl Session {
    fn start(doc: Document) -> (Session, LoadReport);   // shared by new and load
    pub fn new(doc: Document) -> Session { Self::start(doc).0 }
}
```

`start` runs the #51 loop. For each inconsistent component it keeps the vector from before the solve; if the solve succeeds, it records `Adjusted` with the component's geometry names and its displacement. Afterwards `conflicts` is `Session::conflicts()`.

Displacement (spec "Displacement of a component"): for each geometry of the component, take the maximum over the samples `t = k/32`, `k = 0..=32`, of `|P(t)_before − P(t)_after|`, using `on_curve_expr(id, Normalized(t))`. Geometries without the curve capability are skipped (none today). 33 samples include both ends and give a fine enough approximation for curves that are lines or circular arcs.

## Risks / Trade-offs

- [A file with a syntax error is reported as `not a sketch file`, which sounds harsher than "JSON error at line 12"] → The serde message with line and column is part of the error and its `Display`.
- [Sampling misses the exact maximum on a curved geometry] → The error is bounded by the arc between samples; the report is a notice, not a gate.
- [Load mode skips the frozen check, so constraints may join components that are temporarily inconsistent during loading] → Intended. Only the final state matters, and the session start classifies it.
- [Reading a whole file into a `Value` first costs memory] → Proportional to the file. serde_json limits nesting depth; ids are allocated fresh by the commands, never taken from the file.

## Migration Plan

No existing files. `Session::new` keeps its signature and behaviour. Rollback is a revert.
