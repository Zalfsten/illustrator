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

## Workflow follow-up

- Archive the change; close #62 when the PR is merged.
