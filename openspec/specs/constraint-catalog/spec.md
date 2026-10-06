# constraint-catalog

## Purpose

Defines the persistent v1 constraint catalog — Congruence, Parallel, Orthogonal, FixPoint, and EqualLength — including residual meaning and the rule that satisfied structural constraints are document invariants.

## Requirements

### Requirement: Congruence identifies two point expressions

The system SHALL provide a Congruence relationship between two point expressions. Congruence SHALL contribute exactly two residual components that are zero if and only if the two expressions evaluate to the same location (within solve tolerance).

#### Scenario: Congruent endpoints share a location after solve

- **WHEN** Congruence is applied to two endpoint expressions
- **AND** a successful solve completes
- **THEN** both expressions evaluate to the same location within solve tolerance

### Requirement: Direct Congruence partners of a point expression are enumerable

Given a point expression \(P\) and the set of structural Congruence relationships in Document, the system SHALL enumerate the point expressions that are **directly** identified with \(P\) (the other side of each Congruence that names \(P\)). That enumeration SHALL NOT require scanning unrelated constraint kinds for correctness of the partner set. Transitive closure of Congruence (longer identification chains) is NOT required by this catalog; Interact MAY use the direct-partner set when applying soft mouse targets to a congruence class.

#### Scenario: Partner of a joined endpoint is the other side

- **WHEN** Congruence relates endpoint expression \(A\) to endpoint expression \(B\)
- **AND** direct partners of \(A\) are requested
- **THEN** \(B\) appears in that partner set
- **AND** requesting partners of \(B\) includes \(A\)

#### Scenario: Unrelated constraints do not invent partners

- **WHEN** a geometry participates in Orthogonal or FixPoint but not in Congruence with point expression \(P\)
- **AND** direct partners of \(P\) are requested
- **THEN** those Orthogonal/FixPoint relationships do not contribute partner point expressions for \(P\)

### Requirement: Parallel constrains two free-vector expressions

The system SHALL provide a Parallel relationship between two **opaque free-vector expressions**. Parallel SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are linearly dependent (parallel or anti-parallel), including the degenerate case where either vector has zero length. In the v1 catalog, tools SHALL typically supply curve tangent vector expressions (for line segments, tangent is constant along the chart); the residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Parallel segment directions stay aligned under edit

- **WHEN** Parallel is applied to the tangent vector expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those tangent vectors remain linearly dependent within solve tolerance

### Requirement: Orthogonal constrains two free-vector expressions

The system SHALL provide an Orthogonal relationship between two **opaque free-vector expressions**. Orthogonal SHALL contribute exactly one residual component that is zero if and only if the two evaluated free vectors are perpendicular (dot product zero), including the degenerate case where either vector has zero length. In the v1 catalog, tools SHALL typically supply curve tangent vector expressions (for line segments, tangent is constant along the chart); the residual SHALL NOT hard-wire segment identity outside those expressions.

#### Scenario: Orthogonal segment directions keep a right angle under edit

- **WHEN** Orthogonal is applied to the tangent vector expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those tangent vectors remain perpendicular within solve tolerance

### Requirement: FixPoint anchors a point expression

The system SHALL provide a FixPoint relationship that anchors a point expression to a fixed target location. FixPoint SHALL contribute exactly two residual components that are zero if and only if the expression evaluates to that target (within solve tolerance).

#### Scenario: Fixed point does not move under successful solve of other edits

- **WHEN** a point expression is constrained with FixPoint to a target location
- **AND** another part of the figure is manipulated through a successful solve
- **THEN** the fixed point expression still evaluates to the target location within solve tolerance

### Requirement: EqualLength constrains two length expressions

The system SHALL provide an EqualLength relationship between two **opaque length expressions**. EqualLength SHALL contribute exactly one residual component that is zero if and only if the two evaluated lengths are equal (within solve tolerance). In the v1 catalog, tools SHALL typically supply curve length expressions; the residual SHALL NOT hard-wire geometry kind outside those expressions.

#### Scenario: Equal segment lengths stay equal under edit

- **WHEN** EqualLength is applied to the length expressions of two segments
- **AND** the document is edited through a successful solve
- **THEN** those lengths remain equal within solve tolerance

### Requirement: Each catalog constraint has a fixed residual arity

Each structural constraint kind in this catalog SHALL contribute a fixed number of residual components (Congruence and FixPoint: two; Parallel, Orthogonal, and EqualLength: one). Assembly SHALL size the structural residual block as the sum of those arities over the constraints present in Document.

#### Scenario: Residual length matches sum of arities

- **WHEN** a document contains one Congruence and one Orthogonal
- **AND** the structural residual (without soft terms) is assembled
- **THEN** the residual length is three

### Requirement: Structural constraints are hard invariants

Congruence, Parallel, Orthogonal, FixPoint, and EqualLength SHALL be structural constraints. After every successful solve that commits a new document state, all structural residuals SHALL be satisfied within solve tolerance. The system SHALL NOT commit a document state from a solve that leaves structural residuals unsatisfied beyond that tolerance.

#### Scenario: Successful commit preserves structural satisfaction

- **WHEN** a solve succeeds and its result is committed as the document state
- **THEN** every structural constraint in the document is satisfied within solve tolerance

### Requirement: Degenerate zero vectors do not crash the catalog

If either free-vector expression involved in Parallel or Orthogonal evaluates to the zero vector (e.g. a zero-length segment whose tangent vanishes), the residual MAY be algebraically satisfied without conveying a meaningful angle. The system SHALL still accept such configurations without undefined behavior; improving or rejecting degeneracy is outside this catalog's normative behavior.

#### Scenario: Zero-length segment direction does not make assembly fail by itself

- **WHEN** a segment has coincident endpoints
- **AND** Parallel or Orthogonal is applied to vector expressions that include that segment's tangent
- **THEN** equation assembly completes without runtime failure attributable solely to that degeneracy
