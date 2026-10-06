# Spec Delta: geometry-ir

## MODIFIED Requirements

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. Document SHALL store geometries as a heterogeneous collection of instances under a **uniform instance contract** over **shared shell metadata** (stable identity, result/construction role, and contiguous State parameter-block layout). Shared geometry operations (parameter count, connectivity derived from the block, reindex after removal, role) SHALL apply uniformly across instances without requiring callers on the evaluate/solve path to match on a document-wide kind discriminant. Kind-specific chart behavior and formula catalogs SHALL live with each kind (see *Geometry kinds own chart formulas; shared shell is kind-agnostic*). Callers SHALL obtain kind-specific behavior by querying a capability or matching on kind only where a discriminant is required (for example Mesh tessellation); the core evaluate/solve path SHALL NOT use a privileged segment-only store API and SHALL NOT require Document to be a closed sum-of-kinds enum. Additional curve kinds (circles, arcs, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement. Canvas paint and Mesh-backed pick SHALL use the display Mesh (or equivalent) rather than iterating document geometries by kind as the authoritative paint path.

#### Scenario: Segment remains an admitted curve

- **WHEN** a document under this capability contains geometry
- **THEN** line segments are valid curve geometries
- **AND** each segment exposes normalized position, tangent, and normal as required for curves

#### Scenario: Non-segment curves are not yet mandatory

- **WHEN** a document is constructed with only line segment curves
- **THEN** the document satisfies this capability's geometry-set obligations
- **AND** absence of circle or spline kinds is not a violation

#### Scenario: Construction and result segments share the same kind API

- **WHEN** a document contains both a construction segment and a result segment
- **THEN** both are segment curve geometries under this capability
- **AND** they differ by role, not by a separate geometry kind

#### Scenario: Structure iterates instances without a kind sum

- **WHEN** Document enumerates geometries for role inspection or parameter-count aggregation
- **THEN** each entry exposes shared shell metadata and uniform instance operations
- **AND** that enumeration does not require matching a closed document-wide kind discriminant

### Requirement: Geometry kinds own chart formulas; shared shell is kind-agnostic

Each document geometry SHALL combine **shared shell metadata** (stable identity, result/construction role, contiguous State parameter-block layout) with a **kind-owned chart instance** that supplies parameter count and chart pack/unpack for structural add. Chart formulas that map that kind's local parameters to points or free vectors (endpoints where applicable, on-curve samples, tangents, normals, segment direction, and similar) SHALL be defined with that kind. Opaque point and vector expressions SHALL evaluate those formulas without requiring constraint, assembly, or Interact code to match on geometry kind or on private formula kinds. Extending the admitted kind set SHALL add a new kind-owned chart, shell-backed instance registration, and expression formula catalog entries; it SHALL NOT require changing shared identity or role types, SHALL NOT reintroduce a privileged segment-only identity or store API on the evaluate/solve path, and SHALL NOT require Document storage to be a closed sum-of-kinds enum.

#### Scenario: Adding a kind does not change identity or role types

- **WHEN** a future curve kind is admitted under this capability
- **THEN** geometries of that kind still use the same stable geometry identity type and result/construction role
- **AND** constraints that already consume opaque point or vector expressions continue to evaluate without matching on the new kind's formula variants

#### Scenario: Segment formulas are kind-owned

- **WHEN** a segment endpoint or on-curve sample is needed for constraints or Mesh
- **THEN** the corresponding point expression is constructed from the segment kind's formula catalog
- **AND** evaluation resolves the named geometry identity and applies that segment formula against State

#### Scenario: Shared shell is independent of chart formulas

- **WHEN** Document resolves a geometry by stable identity for role or parameter-window lookup
- **THEN** that resolution uses shared shell metadata
- **AND** it does not require evaluating or matching the kind's private formula catalog
