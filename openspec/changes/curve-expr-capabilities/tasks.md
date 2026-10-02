# Tasks

## 1. Specs and factories

- [x] 1.1 Add `CurveExprs` and `EndpointExprs` in geometries; `Segment` implements both (trait methods only)
- [x] 1.2 Remove `direction_expr` and `VecFormula::Direction`; use `tangent_expr` at call sites; verify tests compile

## 2. Verify

- [x] 2.1 Update docs mentioning `direction_expr`; run `cargo test -p constraint_runtime` and `cargo check -p sketch_ui`
- [x] 2.2 Run `openspec validate curve-expr-capabilities`
