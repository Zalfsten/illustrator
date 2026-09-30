# Spec Delta: geometry-ir

## ADDED Requirements

### Requirement: Kind charts supply initial parameters; formulas use the shared block window

Each geometry kind SHALL define how its initial continuous parameters are packed for structural add (a kind-owned chart init). When those parameters are registered on the Document, the shared geometry metadata SHALL record the contiguous State block placement. Kind chart formulas that evaluate points or free vectors SHALL read State entries inside that recorded block window; they SHALL NOT rely on a separate authoritative global-index layout owned outside DocumentStructure.

#### Scenario: Segment add packs endpoints into the new block

- **WHEN** a line segment is added with two endpoint locations as chart init
- **THEN** State receives four contiguous scalars in endpoint order for that geometry
- **AND** the geometry record's block layout identifies those scalars
- **AND** evaluating the segment's start and end point expressions at that State yields the init endpoints

#### Scenario: Formula evaluation uses the recorded block

- **WHEN** a kind-owned point or vector formula for a geometry is evaluated against DocumentStructure and State
- **THEN** it resolves the geometry by stable identity
- **AND** it samples State only within the contiguous parameter window stored on that geometry's shared metadata
