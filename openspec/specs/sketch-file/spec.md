# sketch-file Specification

## Purpose

Defines the native file of a drawing: a versioned list of declarative commands that saves a drawing losslessly and loads it back, including how files with imprecise or contradicting values are loaded and reported.

## Requirements

### Requirement: Sketch file envelope

A sketch file SHALL be UTF-8 JSON text holding one object with exactly the fields `format` (the string `"illustrator-sketch"`), `version` (an integer) and `commands` (an array of commands in the declarative subset of sketch-commands, in their interchange form). The recommended file name extension SHALL be `.sketch.json`; readers SHALL recognize a file by its content, not by its name.

#### Scenario: Minimal file

- **WHEN** the file `{"format": "illustrator-sketch", "version": 1, "commands": []}` is loaded
- **THEN** loading succeeds with an empty drawing

#### Scenario: Unknown envelope field is rejected

- **WHEN** a file has the additional field `"author": "x"` next to `format`, `version` and `commands`
- **THEN** loading fails because of the unknown field `author`

### Requirement: Saving writes the drawing as a sketch file

Saving SHALL write the drawing's declarative command list (sketch-commands) as a version 1 sketch file. Each command SHALL be on a line of its own, so that changing one object changes one line. Saving SHALL be deterministic: the same drawing SHALL give identical bytes, and saving, loading and saving again SHALL give identical bytes.

#### Scenario: One command per line

- **WHEN** a drawing with segments `seg1`, `seg2` and a join `join1` is saved
- **THEN** the file has exactly one line for each of the three commands
- **AND** no command spans more than one line

#### Scenario: Save, load, save is byte-identical

- **WHEN** a drawing is saved, the file is loaded, and the loaded drawing is saved again
- **THEN** both files are identical byte for byte

#### Scenario: Unaddressable drawing cannot be saved

- **WHEN** a drawing has a constraint operand without an address
- **THEN** saving fails, naming that constraint, and no file content is produced

### Requirement: Files are recognized by content and version

Loading SHALL fail with `not a sketch file` for bytes that are not JSON text, with `unknown format` for JSON whose `format` is missing or different, and with `unsupported version` for a version this reader does not know, naming that version. Version 1 SHALL be supported. A reader SHALL read every JSON layout of a file, not only the layout it writes.

#### Scenario: Compressed or binary bytes

- **WHEN** bytes starting with a zip archive signature are loaded
- **THEN** loading fails with `not a sketch file`

#### Scenario: Newer version

- **WHEN** a file with `"version": 2` is loaded by a reader that knows only version 1
- **THEN** loading fails with `unsupported version` naming version 2

#### Scenario: Other layout loads

- **WHEN** a version 1 file is written as a single line, or with every field on its own line
- **THEN** it loads to the same drawing as the file saved in the standard layout

### Requirement: Load mode executes commands without solving

Loading SHALL execute the file's commands in file order, validating names, addresses and values as for any command, but without solving, without the frozen-component check and without creating history steps. A command SHALL only refer to objects created by earlier commands. Afterwards the drawing SHALL start as a new session (sketch-commands "Session start from an existing drawing"), without undo history.

#### Scenario: Values within tolerance are kept exactly

- **WHEN** a file whose constraints hold within solve tolerance is loaded
- **THEN** every value of the drawing equals the value in the file bit for bit
- **AND** undo is not available

#### Scenario: Forward reference fails

- **WHEN** a `constrain` command refers to `seg2.start` and the `create` of `seg2` comes later in the file
- **THEN** loading fails at that `constrain` command because the address does not resolve

### Requirement: Load errors reject the whole file

If any part of a file fails to read or execute, loading SHALL fail as a whole and produce no drawing; a drawing that is already open SHALL stay unchanged. A failing command SHALL be reported with its position in the `commands` array (counting from 0) and its error class: a reading error (not in the declarative subset, unknown or missing field, wrong shape, number out of range) or a command error class of sketch-commands, including `already joined`.

#### Scenario: Taken name

- **WHEN** the commands at positions 0 and 1 both create an object named `seg1`
- **THEN** loading fails at position 1 with the taken-name class

#### Scenario: Join within one congruence class

- **WHEN** a file joins `a.end` with `b.start`, `b.start` with `c.start`, and then `c.start` with `a.end` at position 5
- **THEN** loading fails at position 5 with class `already joined`

#### Scenario: Operation outside the subset

- **WHEN** position 3 holds `{"op": "rename", "name": "seg1", "new_name": "x"}`
- **THEN** loading fails at position 3 because rename is not declarative

### Requirement: Load report

Loading SHALL report each component whose values were changed by the session-start solve ("adjusted"), with the names of its geometries and its largest displacement, and each component left conflicted, with the names of its geometries and constraints and its structural residual. Both lists SHALL be empty when the file's values satisfy all constraints within solve tolerance.

#### Scenario: Imprecise file is adjusted

- **WHEN** a file has segment `a` from (0, 0) to (2, 0), segment `b` from (2, 0) to (2.1, 3) and a perpendicular constraint on `a.direction` and `b.direction`
- **THEN** loading succeeds and the constraint holds within solve tolerance
- **AND** the report lists one adjusted component naming `a` and `b` with a largest displacement greater than zero

#### Scenario: Contradicting file is held

- **WHEN** a file fixes `seg1.start` to (0, 0) and, in a second constraint, to (0, 1)
- **THEN** loading succeeds
- **AND** the report lists the component of `seg1` as conflicted, and its values equal the file's values

### Requirement: Displacement of a component

The displacement of a component SHALL be the largest distance, in drawing units, between a point of one of its curves before and after the solve, compared at the same curve position. Implementations MAY approximate it by sampling curve positions that include the start and end of open curves.

#### Scenario: Moved end of a segment

- **WHEN** a solve moves the end of a segment by 0.1 and keeps its start
- **THEN** the displacement of its component is 0.1 within solve tolerance
