## MODIFIED Requirements

### Requirement: Sketch file envelope

A sketch file SHALL be UTF-8 JSON text holding one object. In version 3 it SHALL have exactly the fields `format` (the string `"illustrator-sketch"`), `version` (an integer), `scale` and `font_size` (numbers), `preamble` (a string), `styles` (an object) and `commands` (an array of commands in the declarative subset of sketch-commands, in their interchange form).

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

Saving SHALL write the drawing's document settings, its style table and its declarative command list (sketch-commands) as a version 3 sketch file. Saving SHALL be deterministic: the same drawing SHALL give identical bytes, and saving, loading and saving again SHALL give identical bytes.

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

## ADDED Requirements

### Requirement: Envelope settings and styles

The three settings fields hold the document settings and SHALL obey their value ranges. `styles` maps each style name to an object of that style's properties (styling) in their interchange shapes; `default` MAY be absent, meaning no property set.

#### Scenario: Default style in the envelope

- **WHEN** a version 3 file has `"styles": {"default": {"width": 0.8}}`
- **THEN** loading succeeds and the style `default` sets width 0.8

#### Scenario: Out-of-range font size is rejected

- **WHEN** a version 3 file has `"font_size": 0`
- **THEN** loading fails because of the invalid value of `font_size`

### Requirement: Saved file layout

Each command SHALL be on a line of its own, so that changing one object changes one line, and each settings field and each style SHALL be on a line of its own. `default` SHALL be written only when it has a property set; styles SHALL be written in the order of their names.

#### Scenario: One changed object changes one line

- **WHEN** a drawing is saved, one segment gets another style through `set_style`, and the drawing is saved again
- **THEN** the two files differ in exactly one line, the `create` of that segment

#### Scenario: Settings fields on their own lines

- **WHEN** a drawing is saved
- **THEN** `scale`, `font_size` and `preamble` each occupy a line of their own
