# Spec Delta

## Purpose

Gives every geometry and constraint a user-visible name that is unique at any time, and defines how points, free vectors and lengths on named geometries are addressed, so scripts, the command API, files and labels can refer to objects without internal identities.

## ADDED Requirements

### Requirement: Every object has a unique name

Every geometry and every structural constraint in Document SHALL have exactly one name. Geometries and constraints SHALL share one namespace: at any time, no two live objects SHALL have the same name. Names are part of the document structure; restoring a previous document state SHALL restore its names. The internal identities used by evaluation and solving SHALL remain unchanged by naming.

#### Scenario: Geometry and constraint names do not collide

- **WHEN** a document contains a geometry and a constraint
- **THEN** each has a name
- **AND** the two names differ

#### Scenario: Name lookup in both directions

- **WHEN** an object is looked up by its name
- **THEN** that object is found
- **AND** looking up the name of that object yields the same name

### Requirement: Name syntax

A valid name SHALL start with an ASCII letter or underscore, followed by zero or more ASCII letters, digits or underscores. Names SHALL be case-sensitive. The dot SHALL NOT occur in a name, because it separates a geometry name from an address suffix. Reserved words of particular scripting front-ends are outside this requirement.

#### Scenario: Valid and invalid names

- **WHEN** names are checked for validity
- **THEN** `seg1`, `x_axis` and `_tmp` are valid
- **AND** `1seg`, `seg.1`, `seg-1` and the empty name are invalid

#### Scenario: Case distinguishes names

- **WHEN** one object is named `Seg1` and another `seg1`
- **THEN** both names are accepted and refer to different objects

### Requirement: Kinds supply name prefixes

Each geometry kind and each constraint kind SHALL have its own name prefix. A prefix SHALL be a valid name that does not end in a digit. Creating an object without an explicit name SHALL name it with its kind's prefix followed by a decimal number.

#### Scenario: Constraint kinds have their own prefixes

- **WHEN** a parallel and then a perpendicular constraint are added to an empty document's segments
- **THEN** they are named with the parallel prefix and number 1, and the perpendicular prefix and number 1

### Requirement: Auto-naming numbers continue after the highest in use

The number of an automatic name SHALL be one more than the highest number currently used with that prefix by any live object (user-chosen names included), or 1 if there is none. The next name SHALL depend only on the names currently in use; a name no longer in use MAY be given out again.

#### Scenario: Numbers continue after the highest in use

- **WHEN** segments named `seg1`, `seg2` and `seg5` exist
- **AND** a new segment is created
- **THEN** it is named `seg6`

#### Scenario: Prefixes are counted independently

- **WHEN** a document contains `seg1`, `seg2` and no arcs
- **AND** an arc is created
- **THEN** it is named `arc1`

#### Scenario: Top name is reused after removal

- **WHEN** `seg1` and `seg2` exist and `seg2` is removed
- **AND** a new segment is created
- **THEN** it is named `seg2`

#### Scenario: User-chosen names count

- **WHEN** a user has renamed an object to `seg7` while the highest automatic segment name is `seg3`
- **AND** a new segment is created
- **THEN** it is named `seg8`

### Requirement: Renaming

The system SHALL rename an object to a new name if that name is valid and not used by another live object, and SHALL otherwise reject the rename with the reason (invalid syntax or name taken), leaving all names unchanged. Renaming an object to its current name SHALL succeed without effect.

#### Scenario: Rename to a free name

- **WHEN** `seg1` is renamed to `x_axis`
- **THEN** the object is found under `x_axis`
- **AND** `seg1` no longer resolves
- **AND** constraints on that geometry are unchanged

#### Scenario: Rename to a taken name is rejected

- **WHEN** `seg1` is renamed to `seg2` while `seg2` exists
- **THEN** the rename is rejected because the name is taken
- **AND** both objects keep their names

#### Scenario: Rename to an invalid name is rejected

- **WHEN** an object is renamed to `my.axis`
- **THEN** the rename is rejected because the name is invalid

### Requirement: Names change only by explicit rename

Renaming SHALL NOT change the drawing: geometry, constraints and values stay as they are, and constraints keep referring to the same geometries. The system SHALL NOT change an existing object's name except through an explicit rename; in particular, adding or removing other objects SHALL NOT rename it.

#### Scenario: Other edits keep names

- **WHEN** `seg1` and `seg3` exist and `seg2` is removed
- **THEN** `seg1` and `seg3` keep their names

#### Scenario: Rename keeps the drawing

- **WHEN** a geometry with constraints on it is renamed
- **THEN** its parameter values and the constraints on it are unchanged

### Requirement: Removal drops names and reports them

Removing an object SHALL drop its name. Removing a geometry SHALL also drop the names of the constraints removed with it. The removal SHALL report the names of all removed objects.

#### Scenario: Geometry removal cascades names

- **WHEN** `seg1` has constraints `par1` and `join1` on it
- **AND** `seg1` is removed
- **THEN** the removal reports `seg1`, `par1` and `join1` as removed
- **AND** none of these names resolves any more

### Requirement: Addresses on named geometries

The system SHALL resolve an address of the form `<name>.<suffix>` on a named geometry to the corresponding opaque expression. It SHALL support at least: `start`, `end`, `center`, `at(<t>)` (points); `direction`, `tangent(<t>)`, `normal(<t>)` (free vectors); `length`, `radius` (lengths). `<t>` is a normalized curve parameter in \([0, 1]\). Points and free vectors SHALL remain distinct result kinds.

#### Scenario: Endpoint address

- **WHEN** `seg1.end` is resolved
- **THEN** the result is a point expression that evaluates to the end point of `seg1`

#### Scenario: Vector address is not a point

- **WHEN** `seg1.direction` is resolved
- **THEN** the result is a free-vector expression, not a point expression

### Requirement: Address resolution reports why it fails

Resolving an address SHALL fail, returning no expression and a distinct reason, for: a malformed address, an unknown name, a name that denotes a constraint, a curve parameter outside \([0, 1]\), and a geometry that lacks the capability the suffix needs.

#### Scenario: Missing capability is reported

- **WHEN** `arc1.direction` is resolved
- **THEN** resolution fails because `arc1` has no line capability
- **AND** no expression is returned

#### Scenario: Unknown name and constraint name

- **WHEN** `nope.start` is resolved while no object is named `nope`
- **THEN** resolution fails because the name is unknown
- **AND** resolving `par1.start` for a constraint `par1` fails because the name does not denote a geometry

#### Scenario: Curve parameter out of range

- **WHEN** `seg1.at(1.5)` is resolved
- **THEN** resolution fails because the parameter is outside \([0, 1]\)

### Requirement: Expressions print as addresses

For a point, free-vector or length expression on a named geometry that corresponds to an address suffix, the system SHALL produce that address, such that resolving it yields an equal expression.

#### Scenario: Round trip of an endpoint

- **WHEN** the start point expression of the geometry named `seg1` is printed
- **THEN** the result is `seg1.start`
- **AND** resolving `seg1.start` yields an equal expression

### Requirement: Presentation identifies objects by name

The application SHALL identify picked and newly created objects to the user by their names (and points by their addresses), for example in status feedback. This capability SHALL NOT require names to be drawn on the canvas.

#### Scenario: Picking a point shows its address

- **WHEN** the user picks the end point of `seg1` as the first point of a join
- **THEN** the feedback names `seg1.end`

#### Scenario: Created constraint is named

- **WHEN** the user adds a parallel relation between two segments
- **THEN** the feedback names the new constraint
