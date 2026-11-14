## MODIFIED Requirements

### Requirement: Direct Congruence partners of a point expression are enumerable

Given a point expression \(P\) and the set of structural Congruence relationships in Document, the system SHALL enumerate the point expressions that are **directly** identified with \(P\) (the other side of each Congruence that names \(P\)). That enumeration SHALL NOT require scanning unrelated constraint kinds for correctness of the partner set.

#### Scenario: Partner of a joined endpoint is the other side

- **WHEN** Congruence relates endpoint expression \(A\) to endpoint expression \(B\)
- **AND** direct partners of \(A\) are requested
- **THEN** \(B\) appears in that partner set
- **AND** requesting partners of \(B\) includes \(A\)

#### Scenario: Unrelated constraints do not invent partners

- **WHEN** a geometry participates in VectorsOrthogonal or FixPoint but not in Congruence with point expression \(P\)
- **AND** direct partners of \(P\) are requested
- **THEN** those VectorsOrthogonal/FixPoint relationships do not contribute partner point expressions for \(P\)

### Requirement: VectorsParallel constrains two free-vector expressions

The system SHALL provide a VectorsParallel relationship between two **opaque free-vector expressions**. VectorsParallel SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are linearly dependent (parallel or anti-parallel), including the degenerate case where either vector has zero length.

#### Scenario: Parallel segment directions stay aligned under edit

- **WHEN** VectorsParallel is applied to the direction expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those direction vectors remain linearly dependent within solve tolerance

### Requirement: VectorsOrthogonal constrains two free-vector expressions

The system SHALL provide a VectorsOrthogonal relationship between two **opaque free-vector expressions**. VectorsOrthogonal SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are perpendicular (dot product zero), including the degenerate case where either vector has zero length.

#### Scenario: Orthogonal segment directions keep a right angle under edit

- **WHEN** VectorsOrthogonal is applied to the direction expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those direction vectors remain perpendicular within solve tolerance

## ADDED Requirements

### Requirement: Direct partners need not follow Congruence chains

Transitive closure of Congruence (longer identification chains) is NOT required by this catalog; Interact MAY use the direct-partner set when applying soft mouse targets to a congruence class. The direct-partner enumeration SHALL remain correct without following chains.

#### Scenario: Chain partner is not a direct partner

- **WHEN** Congruence relates \(A\) to \(B\) and \(B\) to \(C\)
- **AND** direct partners of \(A\) are requested
- **THEN** \(B\) appears in that partner set
- **AND** the set is correct whether or not it also lists \(C\)

### Requirement: Lines are constrained through their direction expressions

This applies to VectorsParallel and VectorsOrthogonal. For parallel or perpendicular **lines**, tools SHALL supply the direction expressions of the line capability (see geometry-ir); tangent expressions at a stated curve parameter remain valid inputs and then constrain the curves only at that location (pointwise). The residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Parallel lines use direction expressions

- **WHEN** a tool makes two segments parallel
- **THEN** the VectorsParallel relationship it adds takes the two segments' direction expressions

#### Scenario: Tangent input constrains one location

- **WHEN** VectorsOrthogonal relates the tangent of an arc at normalized t = 0.5 to a segment's direction
- **THEN** after a successful solve the arc's tangent at t = 0.5 is perpendicular to the segment
- **AND** the relationship states nothing about the arc's tangent at other parameters
