# document-settings Specification

## Purpose

Defines the settings that apply to a whole drawing rather than to one object: the physical scale that maps world units to print size, the default font size and the LaTeX preamble. Labels and later styling rely on these settings for their typographic sizes.

## Requirements

### Requirement: A drawing has document settings

Every drawing SHALL have exactly one set of document settings: `scale` (millimetres per world unit), `font_size` (pt), and `preamble` (LaTeX source text). They SHALL be part of the drawing, not of a view or a session.

#### Scenario: Settings belong to the drawing

- **WHEN** a drawing with scale 5 is saved and loaded again
- **THEN** the loaded drawing has scale 5

### Requirement: Document settings defaults

A new, empty drawing SHALL have scale 10 (1 world unit is 1 cm), font size 10 and an empty preamble.

#### Scenario: Fresh drawing

- **WHEN** a session starts on an empty drawing
- **THEN** its settings are scale 10, font size 10 and preamble ``

### Requirement: Document settings value ranges

`scale` and `font_size` SHALL be finite and greater than zero. `preamble` SHALL be any text, including line breaks. A change that would leave a setting out of range SHALL be rejected.

#### Scenario: Zero scale is rejected

- **WHEN** the scale is set to 0
- **THEN** the change is rejected and the scale is unchanged

### Requirement: Document settings changes are history steps

Changing document settings SHALL be a step in the session's history like any command: undo SHALL restore the previous settings together with the rest of the drawing, and redo SHALL restore the change.

#### Scenario: Undo a scale change

- **WHEN** the scale is changed from 10 to 20 and then undone
- **THEN** the scale is 10

### Requirement: The preamble applies to every label

The preamble SHALL define macros (for example with `\newcommand`) for every label in `latex` markup mode of the drawing. Changing the preamble SHALL apply to all such labels and SHALL NOT change any label's source.

#### Scenario: Macro from the preamble

- **WHEN** the preamble is `\newcommand{\R}{\mathbb{R}}` and a label in `latex` mode has source `$x \in \R$`
- **THEN** the label means the same as source `$x \in \mathbb{R}$`
