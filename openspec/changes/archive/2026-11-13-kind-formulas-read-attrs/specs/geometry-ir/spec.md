# Spec Delta: geometry-ir

## MODIFIED Requirements

### Requirement: Kind charts supply initial parameters; formulas use the shared block window

Each geometry kind SHALL define how its initial continuous parameters are packed for structural add (a kind-owned chart init). When those parameters are registered on the Document, the shared geometry metadata SHALL record the contiguous State block placement. Kind chart formulas that evaluate points, free vectors, scalars, exact shape or curvature SHALL read State entries inside that recorded block window; they SHALL NOT rely on a separate authoritative global-index layout owned outside Document. Besides that block, a kind formula MAY read the non-parameter attributes of the same geometry (for example a label's text attributes); it SHALL NOT read another geometry's attributes. Attributes SHALL NOT be State degrees of freedom: they contribute no derivative columns, and which State entries a formula reads SHALL NOT depend on them.

#### Scenario: Segment add packs endpoints into the new block

- **WHEN** a line segment is added with two endpoint locations as chart init
- **THEN** State receives four contiguous scalars in endpoint order for that geometry
- **AND** the geometry record's block layout identifies those scalars
- **AND** evaluating the segment's start and end point expressions at that State yields the init endpoints

#### Scenario: Formula evaluation uses the recorded block

- **WHEN** a kind-owned point or vector formula for a geometry is evaluated against Document and State
- **THEN** it resolves the geometry by stable identity
- **AND** it samples State only within the contiguous parameter window stored on that geometry's shared metadata

#### Scenario: Attributes add no derivative columns

- **WHEN** the derivatives of a formula of a geometry with non-parameter attributes (a label's origin) are evaluated
- **THEN** they refer only to State entries inside that geometry's block window
- **AND** their number is the same as for the same formula with other attribute values

#### Scenario: Kinds whose geometry does not depend on attributes are unaffected by them

- **WHEN** the text attributes of a label are replaced and State is unchanged
- **THEN** the label's origin and direction expressions evaluate to the same values as before

#### Scenario: Replacing attributes keeps committed State canonical

- **WHEN** the attributes of a geometry are replaced while its committed State block holds non-canonical chart values (for example an arc with a negative radius)
- **THEN** the committed block afterwards holds the canonical chart values for the new attributes
- **AND** the geometry evaluates to the same curve as before
