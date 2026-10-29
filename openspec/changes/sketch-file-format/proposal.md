# Proposal

## Why

Drawings cannot be kept yet: a session lives only as long as the program runs. Figures for publications and teaching must be saved, reopened, kept next to a paper in version control, and written or read by scripts and AI clients. #50 provided the declarative command subset and #51 the ability to hold contradictions, so the native file format (#8) can now be defined on top of them. This change covers the format and the load/save semantics; the host side (file dialogs, Open/Save in the GUI, unsaved-changes state) follows in a separate change.

## What Changes

- A **sketch file**: a JSON envelope `{"format": "illustrator-sketch", "version": 1, "commands": [...]}` whose commands are the declarative subset (sketch-commands) in creation order, with final values and explicit names. Recommended file extension `.sketch.json`; the format is recognized by content, not by name.
- **Saving** produces the file as bytes in a fixed layout: one command per line. Saving, loading and saving again yields identical bytes.
- **Loading** reads bytes, checks envelope and version, and executes the commands in a load mode: names and addresses are validated as for any command, but nothing is solved and no history step is created. The drawing then starts as a new session: values within tolerance are kept exactly, components whose values are off get one solve, contradicting components stay frozen (#51).
- **Load errors** (not JSON, unknown format, unsupported version, a command outside the subset or malformed, an invalid value, an invalid or taken name, an unresolvable address, `already joined`) reject the whole file with the index of the failing command and its error class. Nothing is created.
- **Load report**: per adjusted component the names of its geometries and the largest displacement of a curve point at equal curve position; the frozen components as listed by the session.
- Versions: only version 1 exists; a newer version is rejected as unsupported. The loader has one place where older versions would be migrated.

## Non-goals

- Host services and UI: the `Platform` boundary, file dialogs (native and browser), Open / Save / Save As, unsaved-changes tracking, warning before closing (follow-up change for #8).
- Compressed (zip) files. The byte-based API and content detection leave room for them; they will get their own extension.
- Dropping exact duplicates and redundant joins (#53): duplicates in a file load as redundant constraints, and a join within one congruence class rejects the file as for a command.
- Styles, labels, export formats; they extend this format later.

The file format is shared across implementations: any implementation reads and writes the same files. Byte layout and load semantics are therefore part of the spec, not of one implementation's design.

## Capabilities

### New Capabilities

- `sketch-file`: the native file of a drawing: envelope, versions, byte layout, save, load mode, load errors and load report.

### Modified Capabilities

(none; the file builds on the declarative subset of `sketch-commands` and the session start of #51 without changing them)

## Impact

- `impls/rust/crates/sketch_session`: new file module (envelope, writer, reader, version check), a load mode in the session (commands without solve, frozen check and history), a load report computed at session start, error types for load and save.
- No new dependencies (serde_json is already used). `constraint_runtime` and `sketch_ui` unchanged.
- Unblocks the host/UI change for #8, the MCP resource "current drawing" (#19), and the example gallery (#5).
