# Spec Delta

## RENAMED Requirements

- FROM: `### Requirement: Parallel constrains two free-vector expressions`
- TO: `### Requirement: VectorsParallel constrains two free-vector expressions`
- FROM: `### Requirement: Orthogonal constrains two free-vector expressions`
- TO: `### Requirement: VectorsOrthogonal constrains two free-vector expressions`

## ADDED Requirements

### Requirement: Constraints have a pointwise, existential, or universal scope

Each relationship SHALL be classified by where it holds on the curves it involves: **pointwise** (at stated curve locations), **existential** (at some location to be found by the solve), or **universal** (along the whole curve). All catalog constraints SHALL be pointwise. A universal relation between curves SHALL be expressed as pointwise catalog constraints on curve invariants (expressions independent of curve location), not as a new evaluation mechanism.

#### Scenario: Tangent at a stated location is pointwise

- **WHEN** VectorsParallel is applied to an arc tangent expression at normalized \(t = 0\) and a segment direction expression
- **THEN** only the arc direction at \(t = 0\) is constrained
- **AND** other locations of the arc are not constrained by it

#### Scenario: Concentric arcs are a universal relation on invariants

- **WHEN** Congruence is applied to the center expressions of two arcs
- **AND** the document is edited through a successful solve
- **THEN** both arcs share one center within solve tolerance, so every point of one arc keeps a constant distance from the other arc's circle

#### Scenario: Parallel lines are a universal relation on invariants

- **WHEN** VectorsParallel is applied to the direction expressions of two segments
- **THEN** the segments are parallel along their whole length, independent of any curve parameter

## MODIFIED Requirements

### Requirement: Direct Congruence partners of a point expression are enumerable

Given a point expression \(P\) and the set of structural Congruence relationships in Document, the system SHALL enumerate the point expressions that are **directly** identified with \(P\) (the other side of each Congruence that names \(P\)). That enumeration SHALL NOT require scanning unrelated constraint kinds for correctness of the partner set. Transitive closure of Congruence (longer identification chains) is NOT required by this catalog; Interact MAY use the direct-partner set when applying soft mouse targets to a congruence class.

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

The system SHALL provide a VectorsParallel relationship between two **opaque free-vector expressions**. VectorsParallel SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are linearly dependent (parallel or anti-parallel), including the degenerate case where either vector has zero length. For parallel or perpendicular **lines**, tools SHALL supply the direction expressions of the line capability (see geometry-ir); tangent expressions at a stated curve parameter remain valid inputs and then constrain the curves only at that location (pointwise). The residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Parallel segment directions stay aligned under edit

- **WHEN** VectorsParallel is applied to the direction expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those direction vectors remain linearly dependent within solve tolerance

### Requirement: VectorsOrthogonal constrains two free-vector expressions

The system SHALL provide a VectorsOrthogonal relationship between two **opaque free-vector expressions**. VectorsOrthogonal SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are perpendicular (dot product zero), including the degenerate case where either vector has zero length. For parallel or perpendicular **lines**, tools SHALL supply the direction expressions of the line capability (see geometry-ir); tangent expressions at a stated curve parameter remain valid inputs and then constrain the curves only at that location (pointwise). The residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Orthogonal segment directions keep a right angle under edit

- **WHEN** VectorsOrthogonal is applied to the direction expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those direction vectors remain perpendicular within solve tolerance

### Requirement: Each catalog constraint has a fixed residual arity

Each structural constraint kind in this catalog SHALL contribute a fixed number of residual components (Congruence and FixPoint: two; VectorsParallel, VectorsOrthogonal, and EqualLength: one). Assembly SHALL size the structural residual block as the sum of those arities over the constraints present in Document.

#### Scenario: Residual length matches sum of arities

- **WHEN** a document contains one Congruence and one VectorsOrthogonal
- **AND** the structural residual (without soft terms) is assembled
- **THEN** the residual length is three

### Requirement: Structural constraints are hard invariants

Congruence, VectorsParallel, VectorsOrthogonal, FixPoint, and EqualLength SHALL be structural constraints. After every successful solve that commits a new document state, all structural residuals SHALL be satisfied within solve tolerance. The system SHALL NOT commit a document state from a solve that leaves structural residuals unsatisfied beyond that tolerance.

#### Scenario: Successful commit preserves structural satisfaction

- **WHEN** a solve succeeds and its result is committed as the document state
- **THEN** every structural constraint in the document is satisfied within solve tolerance

### Requirement: Degenerate zero vectors do not crash the catalog

If either free-vector expression involved in VectorsParallel or VectorsOrthogonal evaluates to the zero vector (e.g. a zero-length segment whose tangent vanishes), the residual MAY be algebraically satisfied without conveying a meaningful angle. The system SHALL still accept such configurations without undefined behavior; improving or rejecting degeneracy is outside this catalog's normative behavior.

#### Scenario: Zero-length segment direction does not make assembly fail by itself

- **WHEN** a segment has coincident endpoints
- **AND** VectorsParallel or VectorsOrthogonal is applied to vector expressions that include that segment's tangent
- **THEN** equation assembly completes without runtime failure attributable solely to that degeneracy
