# Tasks

Spec-only change: no code, no tests of an implementation change. Commands run from the repository root.

## 1. Delta specs

- [x] 1.1 Check every delta mechanically:
  - each MODIFIED requirement keeps its heading and all of its scenarios verbatim;
  - each description is at most 500 characters;
  - each requirement has at least one scenario;
  - every sentence of the original description appears in the delta, either verbatim or as a recorded rewording.

  Recorded rewordings, where a moved sentence needed an explicit subject because its antecedent stayed behind:
  - drawing-export: "such a piece" → "a circular-arc path piece";
  - solve-interaction: "each such curve" → "each curve under the soft length floor";
  - state-vector: "that mapping" → "the local→global mapping";
  - geometry-eval: "Those expressions" / "They" / "that information" → the named expression families;
  - geometry-ir: "These expressions" → "These on-curve, tangent and normal expressions".

  Requirements whose moved part was MAY-only got a SHALL sentence stating what the MAY sentence already implied. That is needed because strict validation requires SHALL/MUST. Verify that `openspec validate split-long-requirements --strict` passes.

## 2. Sync and verification

- [x] 2.1 Sync the 12 deltas into `openspec/specs/` (MODIFIED blocks replaced in place, ADDED requirements placed after the requirement they were split from). Verify that `openspec validate --specs --strict` passes for all specs, and that no main spec contains delta section headers.

## Review follow-ups (PR #101)

- [x] 3.1 Eight added requirements began with a reference whose antecedent stayed in the requirement they were split from; their subjects are now spelled out:
  - constraint-catalog: "This applies to VectorsParallel and VectorsOrthogonal" folded into "For parallel or perpendicular lines (VectorsParallel, VectorsOrthogonal)"; "The residual" → "The residual of VectorsParallel or VectorsOrthogonal";
  - geometry-eval: "those analytical derivatives" → "the analytical derivatives of point, vector and scalar expressions";
  - geometry-ir: "those formulas" → "the kind-owned chart formulas"; "that recorded block window" → "the State block window recorded in the geometry's shared metadata"; "Besides that block" → "Besides its geometry's State block window"; "those expressions" → "the opaque point, vector and scalar expressions"; "SHALL NOT expose it" → "SHALL NOT expose the line capability";
  - sketch-commands: "these" / "above" → "the classes listed in *Command errors are classified*" (description and scenario).
- [x] 3.2 constraint-catalog *Direct partners need not follow Congruence chains*: the untestable "SHALL remain correct" became "SHALL contain every point expression related to it by a single Congruence"; the scenario checks that C is not required.
- [x] 3.3 geometry-ir *Intersection semantics may come later*: the clause "this capability requires only that such constructions are not modeled as mandatory first-class point geometry entities" is dropped; the following SHALL sentence states the same.

## Workflow follow-up

- Archive the change; close #62 when the PR is merged.
