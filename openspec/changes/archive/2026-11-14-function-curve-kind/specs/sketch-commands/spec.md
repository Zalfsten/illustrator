## MODIFIED Requirements

### Requirement: Create geometry commands

The session SHALL provide commands that create a line segment (start and end point), a circular arc (center, radius, start heading, sweep), a label (origin, heading and its attributes, labels) and a function curve (origin, heading, expression, range and optional scale). Each SHALL accept an optional name, an optional role (result by default), an optional style name (`default` by default) and optional style overrides (styling). On success the result SHALL report the name of the created geometry.

#### Scenario: Create a segment with an automatic name

- **WHEN** a segment is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `seg1`

#### Scenario: Create with an explicit name

- **WHEN** an arc is created with the name `fillet`
- **THEN** the command succeeds and the arc is named `fillet`

#### Scenario: Create a label with an automatic name

- **WHEN** a label is created without a name in an empty drawing
- **THEN** the command succeeds and reports the name `lbl1`

#### Scenario: Create with a style and an override

- **WHEN** a segment is created with style `axis` and colour `#c00000` while `axis` exists
- **THEN** the segment refers to `axis` and overrides its colour

#### Scenario: Create with an unknown style

- **WHEN** a segment is created with style `nope` while no style is named `nope`
- **THEN** the command fails with the `unknown style` class
- **AND** the drawing is unchanged

### Requirement: Set command changes object attributes

The session SHALL provide a `set` command that changes attributes of the named object: properties that are not parameters, such as a label's source or offset. It SHALL change only the attributes it carries and SHALL NOT change names or roles. Unless it changes a curve's geometry (see *Setting geometric attributes solves*), it SHALL NOT change parameter values and SHALL NOT solve, and attributes of a conflicted component MAY be set.

#### Scenario: Change a label's source

- **WHEN** `set` is applied to label `lbl1` with source `$\alpha$`
- **THEN** the command succeeds and `lbl1` has source `$\alpha$`
- **AND** its other attributes and its origin and heading are unchanged

#### Scenario: Several attributes in one step

- **WHEN** `set` is applied to `lbl1` with halign `center` and offset [0, 3]
- **AND** the step is undone
- **THEN** both attributes return to their previous values

#### Scenario: Set on a constraint

- **WHEN** `set` with kind `label` names the constraint `par1`
- **THEN** the command fails with the wrong-object-type class

### Requirement: Declarative list carries attributes and styles, not settings

A label's `create` SHALL carry every attribute explicitly, except `size` when it is inherited. A function curve's `create` SHALL carry `expression`, `range` and `scale` explicitly. A geometry's `create` SHALL carry `style` when it is not `default`, and each of its overrides. The list SHALL NOT contain document settings or style definitions.

#### Scenario: Settings and style definitions stay out

- **WHEN** a drawing with a defined style `axis` and font size 12 pt is exported as a declarative command list
- **THEN** the list contains no command that defines `axis` or sets the font size

#### Scenario: Default style is not written

- **WHEN** a segment with the default style and no overrides is exported as a declarative command list
- **THEN** its `create` has no `style` field and no style property fields

#### Scenario: Inherited label size is not written

- **WHEN** a label that inherits the document font size is exported
- **THEN** its `create` has every label attribute except `size`

## ADDED Requirements

### Requirement: Function interchange fields

Function curve fields SHALL be: `origin` a point; `heading` an angle; `expression` a string in the expression language with variable `x`; `range` a two-element array `[x0, x1]` with \(x_0 < x_1\); `scale` a two-element array `[su, sv]` of numbers greater than zero. On `create`, `origin`, `heading`, `expression` and `range` SHALL be required; an absent `scale` SHALL mean `[1, 1]`.

#### Scenario: Function curve in interchange form

- **WHEN** `{"op": "create", "kind": "function", "origin": [0, 0], "heading": 0, "expression": "sin(x)", "range": [0, 6.283185307179586]}` is read
- **THEN** it is the create command for a sine curve over one period with scales 1 and an automatic name `fn1` when executed in an empty drawing

#### Scenario: Reversed range is rejected

- **WHEN** a function `create` has `"range": [1, 0]`
- **THEN** the command fails with the `invalid value` class

#### Scenario: Zero scale is rejected

- **WHEN** a function `create` has `"scale": [1, 0]`
- **THEN** the command fails with the `invalid value` class

### Requirement: Function set command

A `set` with kind `function` SHALL change any of `expression`, `range` and `scale` of the named function curve, with the same shapes and checks as on `create`.

#### Scenario: New expression

- **WHEN** `set` is applied to function curve `fn1` with expression `cos(x)`
- **THEN** the command succeeds and `fn1` has expression `cos(x)` with its range, scales, origin and heading unchanged

#### Scenario: Rejected expression keeps the curve

- **WHEN** `set` is applied to `fn1` on \([0, 1]\) with expression `log(x)`
- **THEN** the command fails with the `invalid expression` class
- **AND** `fn1` keeps its previous expression

### Requirement: Expression error class

A command with an expression that does not parse, or whose domain check on its range is not proven, SHALL fail with the error class `invalid expression`. The error SHALL state what is wrong: for a parse error its position in the source, for an unproven domain the sub-range where the proof failed, or that the check exceeded its budget.

#### Scenario: Parse error with position

- **WHEN** a function `create` has expression `2x`
- **THEN** the command fails with the `invalid expression` class
- **AND** the error names the position of `x` in the source

#### Scenario: Unproven domain with sub-range

- **WHEN** a function `create` has expression `1/x` and range \([-1, 1]\)
- **THEN** the command fails with the `invalid expression` class
- **AND** the error names a sub-range of \([-1, 1]\) that contains 0

### Requirement: Setting geometric attributes solves

A `set` that changes the expression, range or scale of a function curve changes the curve's geometry. It SHALL then solve the curve's component structurally, as a constraint command does, and SHALL fail with `unsatisfiable` if the component's constraints cannot be satisfied. In a conflicted component it SHALL fail with `frozen`.

#### Scenario: Joined end follows a longer range

- **WHEN** the end of sine curve `fn1` on \([0, \pi]\) is joined to the start of segment `seg1`, and `set` changes the range to \([0, 2\pi]\)
- **THEN** the command succeeds and `fn1.end` and `seg1.start` coincide within solve tolerance

#### Scenario: Frozen component

- **WHEN** `fn1` belongs to a conflicted component and `set` changes its range
- **THEN** the command fails with the `frozen` class and the drawing is unchanged
