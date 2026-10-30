# Spec Delta

## Purpose

Defines the label as an object of the drawing. A label is anchored by a baseline frame whose position and orientation take part in constraints and solving, and it carries the text attributes that every typesetting and export path interprets the same way.

## ADDED Requirements

### Requirement: A label is a geometry anchored by a baseline frame

A label SHALL be a geometry whose continuous parameters are an origin location and a heading angle, and nothing else. Its direction SHALL be the unit free vector of that heading. The extent of its text SHALL NOT be a parameter, and typesetting SHALL NOT change parameter values. A label SHALL have a result or construction role like any geometry.

#### Scenario: Label parameters

- **WHEN** a label is created with origin (1, 2) and heading π/2
- **THEN** the drawing gains exactly three parameters for it
- **AND** its origin evaluates to (1, 2) and its direction to (0, 1)

#### Scenario: Text does not change the parameters

- **WHEN** the source of a label changes from `x` to a long multi-line text
- **THEN** the label's parameter values are unchanged

### Requirement: Labels follow constraints on their frame

A label's origin and direction SHALL take part in constraints like any point and free vector, and a label SHALL move and turn only through its parameters. A label joined or fixed to geometry SHALL follow that geometry when the drawing is solved.

#### Scenario: Label follows a dragged endpoint

- **WHEN** `lbl1.origin` is joined with `seg1.end` and `lbl1.direction` is parallel to `seg1.direction`
- **AND** `seg1.end` is dragged to a new position
- **THEN** after the drag `lbl1.origin` coincides with `seg1.end` within solve tolerance
- **AND** `lbl1.direction` is parallel to `seg1.direction` within solve tolerance

### Requirement: Label frame and reference point

A label's frame SHALL have its x axis along the direction and its y axis along the direction turned a quarter turn counterclockwise. The reference point SHALL be the origin plus the offset `[dx, dy]` taken in that frame and converted from pt to world units through the document scale. Text SHALL be laid out along the frame's x axis. It SHALL NOT be flipped automatically when the direction points left.

#### Scenario: Offset in a turned frame

- **WHEN** the document scale is 10 mm per world unit and a label has origin (0, 0), heading π/2 and offset [0, 72.27]
- **THEN** its reference point is (−2.54, 0) in world units

#### Scenario: Leftward direction is not flipped

- **WHEN** a label's heading is π
- **THEN** its text runs in the direction (−1, 0) and appears upside down

### Requirement: Label lengths are in TeX points

Label offsets and sizes SHALL be in pt, where 1 pt is 1/72.27 inch (25.4/72.27 mm). They SHALL be converted to world units only through the document scale. Zooming a view SHALL NOT change a label's size relative to the geometry.

#### Scenario: One inch of offset

- **WHEN** the document scale is 25.4 mm per world unit and a label has offset [72.27, 0]
- **THEN** the reference point lies 1 world unit from the origin along the direction

### Requirement: Label alignment on the reference point

`halign` SHALL place the text block at the reference point by its left end, centre or right end. Within the block, lines SHALL be aligned the same way. `valign` SHALL select whose baseline passes through the reference point: the `first` line's or the `last` line's. Further lines SHALL extend away from that baseline: below it for `first`, above it for `last`.

#### Scenario: Last-line anchor stacks upward

- **WHEN** a label with three lines has `valign` `last`
- **THEN** the baseline of its third line passes through the reference point
- **AND** its first two lines lie on the positive y side of the frame

#### Scenario: Centred block

- **WHEN** a single-line label has `halign` `center`
- **THEN** the midpoint of its line along the frame's x axis is at the reference point

### Requirement: Label markup modes

`markup` SHALL be `latex` or `text`. In `latex` mode, `source` SHALL be LaTeX in text mode: `$...$` delimits math, `\\` breaks a line, and a newline character counts as a space. In `text` mode, every character of `source` SHALL stand for itself, and a newline character SHALL break a line. Switching modes SHALL keep `source` unchanged.

#### Scenario: Special characters in text mode

- **WHEN** a label in `text` mode has source `50% of $x_1`
- **THEN** its text shows the characters `50% of $x_1` exactly

#### Scenario: Math and line break in latex mode

- **WHEN** a label in `latex` mode has source `velocity\\$\vec v$`
- **THEN** it has two lines: the word `velocity`, and the vector v as math

#### Scenario: Newline in latex mode is a space

- **WHEN** a label in `latex` mode has source `a` + newline + `b`
- **THEN** it has one line

### Requirement: Label lines break only explicitly

A label SHALL break lines only where its source breaks them in its markup mode. It SHALL NOT wrap lines automatically. An empty source SHALL be a valid label with no visible text.

#### Scenario: Long line stays one line

- **WHEN** a label's source is a 200-character sentence without a line break
- **THEN** the label has one line

### Requirement: Label size

A label SHALL either have its own size in pt, or inherit the document's font size (document-settings). An inherited size SHALL follow later changes of the document font size; an own size SHALL NOT.

#### Scenario: Inherited size follows the document

- **WHEN** label `lbl1` inherits its size, `lbl2` has size 8, and the document font size changes from 10 to 12
- **THEN** `lbl1` is set at 12 pt and `lbl2` at 8 pt

### Requirement: Label attribute defaults

A label created without them SHALL get these attribute values: `markup` `latex`, `halign` `left`, `valign` `first`, `offset` [0, 0], and an inherited size. `source` SHALL be given explicitly.

#### Scenario: Minimal label

- **WHEN** a label is created with only origin, heading and source `A`
- **THEN** it has markup `latex`, halign `left`, valign `first`, offset [0, 0] and an inherited size
