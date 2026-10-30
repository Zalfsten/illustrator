# Spec Delta

## MODIFIED Requirements

### Requirement: Sketch file envelope

A sketch file SHALL be UTF-8 JSON text holding one object. In version 2 it SHALL have exactly the fields `format` (the string `"illustrator-sketch"`), `version` (an integer), `scale` and `font_size` (numbers), `preamble` (a string) and `commands` (an array of commands in the declarative subset of sketch-commands, in their interchange form). The three settings fields hold the document settings and SHALL obey their value ranges.

#### Scenario: Minimal file

- **WHEN** the file `{"format": "illustrator-sketch", "version": 2, "scale": 10, "font_size": 10, "preamble": "", "commands": []}` is loaded
- **THEN** loading succeeds with an empty drawing and those settings

#### Scenario: Unknown envelope field is rejected

- **WHEN** a file has the additional field `"author": "x"` next to its envelope fields
- **THEN** loading fails because of the unknown field `author`

#### Scenario: Missing setting is rejected

- **WHEN** a version 2 file has no `preamble` field
- **THEN** loading fails because the field `preamble` is missing

#### Scenario: Out-of-range setting is rejected

- **WHEN** a version 2 file has `"scale": 0`
- **THEN** loading fails because of the invalid value of `scale`

### Requirement: Version 1 files load with default settings

A version 1 file SHALL have exactly the fields `format`, `version` and `commands`. It SHALL load as before, and its drawing SHALL get the default document settings (document-settings). Saving it again SHALL write version 2.

#### Scenario: Old file, new settings

- **WHEN** the file `{"format": "illustrator-sketch", "version": 1, "commands": []}` is loaded
- **THEN** loading succeeds with scale 10, font size 10 and an empty preamble

#### Scenario: Settings fields in version 1 are rejected

- **WHEN** a version 1 file has the additional field `"scale": 5`
- **THEN** loading fails because of the unknown field `scale`

### Requirement: Files are recognized by content and version

Loading SHALL fail with `not a sketch file` for bytes that are not JSON text, with `unknown format` for JSON whose `format` is missing or different, and with `unsupported version` for a version this reader does not know, naming that version. Versions 1 and 2 SHALL be supported. A reader SHALL read every JSON layout of a file, not only the layout it writes. Readers SHALL recognize a file by its content, not by its name; the recommended file name extension SHALL be `.sketch.json`.

#### Scenario: Compressed or binary bytes

- **WHEN** bytes starting with a zip archive signature are loaded
- **THEN** loading fails with `not a sketch file`

#### Scenario: Newer version

- **WHEN** a file with `"version": 3` is loaded by a reader that knows only versions 1 and 2
- **THEN** loading fails with `unsupported version` naming version 3

#### Scenario: Other layout loads

- **WHEN** a version 2 file is written as a single line, or with every field on its own line
- **THEN** it loads to the same drawing as the file saved in the standard layout
