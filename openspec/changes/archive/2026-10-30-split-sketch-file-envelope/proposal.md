# Proposal

## Why

The requirement "Sketch file envelope" in `sketch-file` grew past 500 characters in `label-model` (#58). It now states four behaviours at once: the JSON form, the version 2 fields, the version 1 fields, and recognition by content and file name. `openspec validate sketch-file --strict` fails on it. Splitting it keeps the file spec readable and makes it pass strict validation again, as it did before #58.

## What Changes

- **"Sketch file envelope"** keeps only the JSON form, the exact version 2 fields and the value ranges of the settings fields.
- **"Version 1 files load with default settings"** takes over the exact version 1 field list.
- **"Files are recognized by content and version"** takes over recognition by content and the recommended extension `.sketch.json`.

There is no behaviour change. Every statement and every scenario is kept, and only which requirement states it changes. Implementations and tests are unaffected.

## Non-goals

- The long requirements in the other specs. A separate issue tracks them.
- Any change to the file format.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `sketch-file`: the three requirements above are reworded so that each states one behaviour; the meaning is unchanged.

## Impact

Only `openspec/specs/sketch-file/spec.md`. No code, tests or file format change.
