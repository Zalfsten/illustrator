# Spec Delta

## ADDED Requirements

### Requirement: Geometries have a result or construction role

Each geometry in DocumentStructure SHALL have a role that is either **result** or **construction**. The role SHALL be part of DocumentStructure (persistent sketch data), not solely transient UI state. Construction geometries SHALL remain parameterized geometries that participate in relationships, expression evaluation, and assembly like result geometries. The role SHALL NOT by itself remove a geometry from DocumentStructure or from the State parameter vector.

#### Scenario: Construction geometry still constrains

- **WHEN** a construction line segment and a result line segment are related by a structural constraint
- **AND** the equation system is assembled and solved
- **THEN** the construction geometry's parameters participate in residuals like any other geometry
- **AND** the construction role does not exempt it from hard structural constraints

#### Scenario: Role is inspectable without evaluating State

- **WHEN** DocumentStructure is inspected without reading State \(p\)
- **THEN** each geometry's result vs construction role is determined by DocumentStructure alone

## MODIFIED Requirements

### Requirement: Geometry set includes curves with segment as proving kind

The system SHALL treat parameterized curves as the primary geometry kind for path-like primitives. A line segment SHALL be a curve in this sense and SHALL remain supported. DocumentStructure SHALL store geometries as a sum of kinds (of which line segment is the only required kind in this slice). Shared geometry operations (parameter count, connectivity, reindex after removal, optional curve capability, and result/construction role) SHALL apply uniformly; kind-specific behavior SHALL be obtained by matching on kind or by querying a capability (for example curve), not by a privileged segment-only store API in the core evaluate/solve path. Additional curve kinds (circles, arcs, splines, function graphs) MAY be introduced in later changes that conform to the curve contract; this capability does not require them to be implemented yet. First-class point geometry entities and face/region geometries remain outside this requirement.

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
