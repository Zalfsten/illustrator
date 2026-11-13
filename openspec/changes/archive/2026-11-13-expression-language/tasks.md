# Tasks

Implementation: impls/rust, new crate `crates/expression` (see design.md). Unit tests live in `src/tests_<module>.rs` beside each module. Run tests from `impls/rust` with `cargo test -p expression`.

## 1. Crate

- [x] 1.1 Create `crates/expression` (`Cargo.toml` with no dependencies, `src/lib.rs` with crate docs naming the spec and the MATLAB-subset rule). Add it to `members` and `default-members` in `impls/rust/Cargo.toml` and to `scripts/check-wasm.sh`. Verify with `cargo build -p expression` and `scripts/check-wasm.sh`.

## 2. Parsing

- [x] 2.1 Lexer (`src/lexer.rs`): tokens with byte spans, number grammar with the dot-before-operator rule, identifiers, the eight operators, parentheses, comma, and dedicated rejection tokens (`'`, `\`, `**`, comparison and logical operators, `[`, `]`). Verify with tests in `src/tests_lexer.rs`:
  - each number form from the spec (`2`, `2.5`, `.5`, `5.`, `1e-3`, `1.5E+3`);
  - `5.*x` lexes as number, `.*`, identifier;
  - each rejected character yields its dedicated error with the right span.
- [x] 2.2 Parser and tree (`src/parser.rs`, `src/tree.rs`): precedence climbing per design decision 2; identifier classification; arity checks; the power rule (constant-exponent folding into `PowInt` / `PowReal`); `nthroot` with a constant nonzero integer `n`; the flat post-order node array; `ParseError` with kind, span and message. Verify with tests in `src/tests_parser.rs`:
  - every precedence scenario of the spec (`2^3^2`, `-x^2`, `2^-1`, `2*-3`, `2--3`, and `2^-2^2` rejected at the second `^`);
  - every rejection scenario (`2x`, `e^x` with the `exp` hint, `i`, `j`, `sign(x)`, `x'`, `x\2`, `x**2`, `x==1`, `[x]`, an unknown function, `atan2(x)` arity, `sin` without a call, `sin(x` missing `)`, unknown identifier `y`), each asserting kind and span;
  - `t^2` with variable `t`;
  - `nthroot(x, 1.5)` and `nthroot(x, x)` rejected.
- [x] 2.3 `Expression` in `src/lib.rs`: `parse(source, variable)`, verbatim source, `PartialEq` on source and variable only, `Clone + Debug`. Verify with tests in `src/tests_lib.rs`:
  - source kept verbatim (`x .*  sin( x )`);
  - equal sources give equal expressions; different spacing gives unequal ones.

## 3. Number types and point evaluation

- [x] 3.1 `Number` trait and the `f64` implementation (`src/number.rs`); generic evaluator `eval_with` over the node array (`src/tree.rs`), recording the first node with a non-finite result. Verify with tests in `src/tests_number.rs`: the value of every catalog function and operator at an interior point against `std` results, including:
  - the `nthroot` odd root of −8;
  - `PowInt` on a negative base;
  - `PowReal` on a negative base giving NaN.
- [x] 3.2 `Dual2<T: Number>` (`src/dual.rs`) with second-order rules for every trait method, and `Expression::eval(x) -> Result<Jet, DomainError>`. Verify with tests in `src/tests_dual.rs`:
  - for every catalog function, f′ and f″ at two interior points against hand-derived formulas (1e-12 relative) and against central differences of the value (loose tolerance);
  - the spec scenario `x*sin(x)` at 2;
  - domain errors for `log(x)` at 0, `sqrt(x)` at −1 and `(-8)^(1/3)`, each naming the failing function.

## 4. Intervals and the domain check

- [x] 4.1 `Interval` (`src/interval.rs`) per design decision 5:
  - outward rounding (1 ulp for IEEE operations, 4 ulps for library functions);
  - range clamps;
  - critical points for `sin` / `cos`;
  - poison and every domain guard from the spec's catalog;
  - `Interval` implementing `Number`, so `Dual2<Interval>` works.

  Verify with tests in `src/tests_interval.rs`:
  - for every catalog function, 200 deterministic pseudo-random sub-intervals inside the open domain (fixed seed, simple LCG, no dependency), each containing the `f64` values at 20 sample points;
  - every guard produces poison on an argument touching or crossing its boundary (`sqrt([0,1])`, `log([-1,1])`, `asin([0,1])`, `tan([1,2])`, `abs([-1,1])`, division by `[-1,1]`, `atan2([-1,1], [-1,0])`, `hypot([−1,1],[−1,1])`, `PowReal` with base `[0,1]`, `nthroot` even `n` on `[-1,1]`);
  - poison propagates through every operation.
- [x] 4.2 `Expression::check_domain(x0, x1)` (`src/domain.rs`) with bisection per design decision 6 and `DomainCheckError { InvalidRange | NotProven { range, function } }`. Verify with tests in `src/tests_domain.rs`:
  - every domain-check scenario of the spec (`x .* sin(x)` on [0, 2π] accepted; `sqrt(x)` on [0, 1], `1/x` on [−1, 1] and `abs(x)` on [−1, 1] rejected with location and function; invalid ranges);
  - acceptance of `exp(-x^2)`, `tan(x)` on [−1, 1], `log(x)` on [0.1, 10], `x - x` on [−1, 1] (dependency effect resolved by bisection) and `atan2(x, 1)` on [−5, 5];
  - rejection of `tan(x)` on [1, 2] and `log(x)` on [−1, 1];
  - a narrow pole `1/(x - 0.3333)` on [0, 1] rejected (soundness).

## 5. Reference values

- [x] 5.1 Add `octave-cli` to `.devcontainer/Dockerfile` (no GUI). Write `crates/expression/tests/reference/generate.m`: about 60 rows covering every function, every operator, the precedence cases and some domain-error rows. Generate `reference.tsv` with `octave-cli` and commit both files. Verify that the script runs in the container and the TSV has one row per case. **Needs Octave in the container:** if rebuilding the container is not possible during apply, pause and ask the user to run the script.
- [x] 5.2 Integration test `crates/expression/tests/reference.rs`: parse every row with variable `x` and compare values within 2⁻⁵⁰ relative (absolute floor 1e-300). `domain-error` rows must give `DomainError`. Verify with `cargo test -p expression --test reference`.

## 6. Documentation and integration

- [x] 6.1 Add the crate to the project context in `openspec/config.yaml` (one entry: geometry-free expression layer, MATLAB subset, `Number` trait with `f64` / `Dual2` / `Interval`, sound domain check). Verify by reading it against design decisions 1 and 4.
- [x] 6.2 Verify the whole workspace: `cargo test`, `cargo clippy --all-targets` and `scripts/check-wasm.sh` pass in `impls/rust`.
- [x] 6.3 Validate the change: `openspec validate expression-language --strict` passes.

## Review follow-ups (PR #88)

- [x] 7.1 `check_domain` rejects a sub-range when any node is bad, as `eval` does (`log(x)^0` on [-1, 1]); test `zero_power_keeps_the_base_domain`.
- [x] 7.2 Budget of 20 000 interval evaluations; `DomainCheckError::TooComplex` with the part not yet proven; test `work_is_bounded`.
- [x] 7.3 Parser nesting limit 128 (`ParseErrorKind::TooDeep`); tests `deep_nesting_is_an_error_not_a_crash`, `deepest_accepted_nesting_fits_the_wasm_stack`.
- [x] 7.4 `Expression::eval_in` with a caller-owned scratch buffer and `node_count`; test `eval_in_reuses_the_scratch_buffer`.
- [x] 7.5 Overflowing literals are `InvalidNumber`; `2i` / `3j` get the complex-number hint; spec: byte range, new requirements for finite literals, nesting depth and the work bound.

## Workflow follow-up

- Archive the change after review (`/opsx:archive`), then close #78.
