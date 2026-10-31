# Spec Delta

## MODIFIED Requirements

### Requirement: Sketch file envelope

A sketch file SHALL be UTF-8 JSON text holding one object. In version 3 it SHALL have exactly the fields `format` (the string `"illustrator-sketch"`), `version` (an integer), `scale` and `font_size` (numbers), `preamble` (a string), `styles` (an object) and `commands` (an array of commands in the declarative subset of sketch-commands, in their interchange form). The three settings fields hold the document settings and SHALL obey their value ranges. `styles` maps each style name to an object of that style's properties (styling) in their interchange shapes; `default` MAY be absent, meaning no property set.

#### Scenario: Minimal file

- **WHEN** the file `{"format": "illustrator-sketch", "version": 3, "scale": 10, "font_size": 10, "preamble": "", "styles": {}, "commands": []}` is loaded
- **THEN** loading succeeds with an empty drawing, those settings and only the style `default`

#### Scenario: Unknown envelope field is rejected

- **WHEN** a file has the additional field `"author": "x"` next to its envelope fields
- **THEN** loading fails because of the unknown field `author`

#### Scenario: Missing setting is rejected

- **WHEN** a version 3 file has no `preamble` field
- **THEN** loading fails because the field `preamble` is missing

#### Scenario: Out-of-range setting is rejected

- **WHEN** a version 3 file has `"scale": 0`
- **THEN** loading fails because of the invalid value of `scale`

#### Scenario: Styles and a styled command

- **WHEN** a version 3 file has `"styles": {"axis": {"width": 0.8, "arrow": "end"}}` and a command creating segment `x` with `"style": "axis"`
- **THEN** loading succeeds and `x` refers to style `axis` with width 0.8 and arrow `end`

#### Scenario: Invalid style in the envelope is rejected

- **WHEN** a version 3 file has `"styles": {"axis": {"width": -1}}`
- **THEN** loading fails because of the invalid value of `width` in style `axis`

### Requirement: Saving writes the drawing as a sketch file

Saving SHALL write the drawing's document settings, its style table and its declarative command list (sketch-commands) as a version 3 sketch file. Each command SHALL be on a line of its own, so that changing one object changes one line, and each settings field and each style SHALL be on a line of its own. `default` SHALL be written only when it has a property set; styles SHALL be written in the order of their names. Saving SHALL be deterministic: the same drawing SHALL give identical bytes, and saving, loading and saving again SHALL give identical bytes.

#### Scenario: One command per line

- **WHEN** a drawing with segments `seg1`, `seg2` and a join `join1` is saved
- **THEN** the file has exactly one line for each of the three commands
- **AND** no command spans more than one line

#### Scenario: Save, load, save is byte-identical

- **WHEN** a drawing is saved, the file is loaded, and the loaded drawing is saved again
- **THEN** both files are identical byte for byte

#### Scenario: Multi-line preamble and label stay on one line

- **WHEN** a drawing whose preamble and one label source each contain line breaks is saved
- **THEN** the preamble field and the label's command each occupy one line of the file
- **AND** loading the file restores both texts exactly

#### Scenario: Unaddressable drawing cannot be saved

- **WHEN** a drawing has a constraint operand without an address
- **THEN** saving fails, naming that constraint, and no file content is produced

#### Scenario: One style per line

- **WHEN** a drawing with styles `vector` and `axis` and an unchanged `default` is saved
- **THEN** the file has one line for `axis`, then one line for `vector`, and none for `default`

### Requirement: Files are recognized by content and version

Loading SHALL fail with `not a sketch file` for bytes that are not JSON text, with `unknown format` for JSON whose `format` is missing or different, and with `unsupported version` for a version this reader does not know, naming that version. Versions 1, 2 and 3 SHALL be supported. A reader SHALL read every JSON layout of a file, not only the layout it writes. Readers SHALL recognize a file by its content, not by its name; the recommended file name extension SHALL be `.sketch.json`.

#### Scenario: Compressed or binary bytes

- **WHEN** bytes starting with a zip archive signature are loaded
- **THEN** loading fails with `not a sketch file`

#### Scenario: Newer version

- **WHEN** a file with `"version": 4` is loaded by a reader that knows only versions 1 to 3
- **THEN** loading fails with `unsupported version` naming version 4

#### Scenario: Other layout loads

- **WHEN** a version 3 file is written as a single line, or with every field on its own line
- **THEN** it loads to the same drawing as the file saved in the standard layout

### Requirement: Version 1 files load with default settings

A version 1 file SHALL have exactly the fields `format`, `version` and `commands`. It SHALL load as before, and its drawing SHALL get the default document settings (document-settings) and only the style `default` without properties. Saving it again SHALL write version 3.

#### Scenario: Old file, new settings

- **WHEN** the file `{"format": "illustrator-sketch", "version": 1, "commands": []}` is loaded
- **THEN** loading succeeds with scale 10, font size 10 and an empty preamble

#### Scenario: Settings fields in version 1 are rejected

- **WHEN** a version 1 file has the additional field `"scale": 5`
- **THEN** loading fails because of the unknown field `scale`

## ADDED Requirements

### Requirement: Version 2 files load without styles

A version 2 file SHALL have exactly the fields of version 3 except `styles`. It SHALL load as before, with only the style `default` without properties. Style fields in its commands and a `styles` field SHALL be rejected. Saving it again SHALL write version 3.

#### Scenario: Version 2 file

- **WHEN** the file `{"format": "illustrator-sketch", "version": 2, "scale": 10, "font_size": 10, "preamble": "", "commands": []}` is loaded
- **THEN** loading succeeds with only the style `default`

#### Scenario: Styles in version 2 are rejected

- **WHEN** a version 2 file has the field `"styles": {}`
- **THEN** loading fails because of the unknown field `styles`
