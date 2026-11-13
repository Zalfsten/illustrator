# Design: Expression language

## Context

See proposal.md for motivation and specs/expression-language/spec.md for the contract. Target implementation: impls/rust.

- **Workspace:** crates `constraint_runtime`, `sketch_session`, `sketch_export`, `label_*`, `sketch_ui`. Every core crate must also build for `wasm32-unknown-unknown` (`scripts/check-wasm.sh`).
- **Existing expression type:** `constraint_runtime` already has `expr.rs` (`Expr<F>`, opaque geometry expressions). The new layer is unrelated and must not be confused with it.
- **Toolchain:** rustc 1.99. `f64::next_up` / `next_down` are stable.
- **First consumer:** the function curve kind (#81) in `constraint_runtime`. It needs:
  - `f`, `f′`, `f″` in `f64` at many points (arc-length chart, tessellation, Bézier pieces);
  - the domain check once per attribute edit;
  - an expression value that can live in `KindAttrs` (`Clone + Debug + PartialEq`, equality on the source text; #79 design decision 3).
- **Octave:** GNU Octave is not installed in the dev container (`.devcontainer/Dockerfile`). MATLAB is not available.

## Goals / Non-Goals

**Goals:**

- A geometry-free crate with no dependencies, usable by `constraint_runtime` and later by UI code for live error highlighting.
- One generic evaluator over a number-type trait, so value, derivatives and interval bounds come from the same tree walk and cannot drift apart. This is the seed of #77.
- A domain check that is sound (never accepts invalid input) and accepts ordinary teaching functions on ordinary ranges.

**Non-Goals:**

- Fast evaluation through compilation, SIMD or JIT. A tree walk is fast enough for the point counts of #81.
- Symbolic simplification or symbolic derivatives.

## Decisions

### 1. A new crate `expression` without dependencies

The crate lives at `impls/rust/crates/expression` and is a workspace member, a default member and part of `check-wasm.sh`. It has no third-party dependencies and depends on no other workspace crate. `constraint_runtime` will depend on it in #81, not in this change.

Public surface:

| item | content |
|---|---|
| `Expression` | source text, variable name and the parsed tree; `PartialEq` compares source and variable only |
| `Expression::parse(source, variable) -> Result<Expression, ParseError>` | parses and validates the source |
| `Expression::eval(x) -> Result<Jet, DomainError>` | `f`, `f′`, `f″` as `f64` |
| `Expression::eval_with::<N: Number>(x: N) -> N` | generic evaluation, also for interval types |
| `Expression::check_domain(x0, x1) -> Result<(), DomainCheckError>` | the domain check |

*Alternatives:*
- **A module inside `constraint_runtime`:** it would couple a geometry-free layer to the solver crate, and the UI could not use it without pulling in the solver.
- **An existing parser crate** (for example `meval`, `evalexpr`): their grammars are not MATLAB, and the exact precedence and rejection rules are the product here. A hand-written parser of a few hundred lines is easier to make exact than to configure.

### 2. Hand-written lexer and precedence-climbing parser

**Lexer.** It produces tokens with byte spans:
- **Numbers:** regex-like `digits [. digits?]? | . digits`, then an optional `[eE][+-]?digits`. A `.` immediately followed by `*`, `/` or `^` ends the number and starts the operator. A literal that overflows to infinity (`1e999`; Rust's `parse` returns `inf`) is an `InvalidNumber` at its span, so a later failure always has a location.
- **Identifiers:** `[A-Za-z_][A-Za-z0-9_]*`.
- **Operators.**
- **Tokens with a dedicated rejection message:** `'`, `\`, `**`, `==`, `<`, `&`, `[`, and similar. A dedicated message beats a generic "unexpected character".

**Parser.** It follows the spec's levels:

```
sum     := product (('+'|'-') product)*
product := unary (('*'|'/'|'.*'|'./') unary)*
unary   := ('+'|'-') unary | power
power   := primary (('^'|'.^') exponent)*
exponent:= ('+'|'-')+ primary   -- signed: must not be followed by '^'
         | primary
primary := number | 'pi' | variable | name '(' args ')' | '(' sum ')'
```

- **Signed exponent followed by a power:** after a signed exponent, another `^` raises `ParseError::AmbiguousPower` at that `^`, with the hint "add parentheses". This keeps the accepted set inside the part of MATLAB's grammar where MATLAB and Octave agree. `2^-2^2` is the only place where their documented precedences are subtle.
- **Implicit multiplication:** a primary directly followed by a number, an identifier or `(` raises `ImplicitMultiplication` at the second token. A number directly followed by `i` or `j` (`2i`, MATLAB's imaginary literal) raises `RejectedName` with the complex-number hint instead.
- **Nesting limit:** every recursion passes through `unary`, which counts its depth; beyond 128 levels it raises `TooDeep`. Without it, 100 000 nested parentheses or minus signs overflowed the stack and aborted the process (review of PR #88); the deepest accepted input is tested on a 1 MiB thread stack, wasm's default.
- **Identifiers are classified at parse time:** variable, `pi`, catalog function (must be followed by `(`), known-but-rejected names (`e` with the hint `exp(x)`, `i`, `j`, `sign`, `floor`, …), or unknown.

`ParseError` is an enum with a `span: Range<usize>` (byte offsets into the UTF-8 source, as the spec states; the UI converts them to characters) and a `Display` text in English.

### 3. Tree as a flat node array

`Vec<Node>` in post-order, with children referenced by index. `Node` is `Const(f64) | Var | Neg(i) | Add/Sub/Mul/Div(i, j) | PowInt(i, n: i32) | PowReal(i, j) | Call1(F1, i) | Call2(F2, i, j) | NthRoot(i, n: i32)`.

- **Power rule at parse time:**
  - a constant exponent subtree (no `Var`) is folded with `f64` arithmetic;
  - if the folded value is an integer within `i32` range, the node is `PowInt`;
  - otherwise it is `PowReal`, which requires base > 0.
- **`nthroot`:** its second argument must fold to a constant nonzero integer. Otherwise the parser rejects it with a pointer to the argument. MATLAB also requires an integer `n`.
- **Evaluation:** one forward loop over the array into a scratch `Vec<N>` (no recursion, no `Box`). `Expression::eval_in(x, &mut scratch)` takes the buffer from the caller and is allocation-free once it has `node_count()` entries; `eval` and `eval_with` allocate one per call. #81 evaluates in the solver's Jacobian path, which `alloc_free.rs` guards, so it uses `eval_in`.

*Alternative:* a boxed recursive enum. It is simpler to write, but allocates per node and recurses. The flat array also makes "which node failed" a plain index for error messages.

### 4. Number-type trait as the formula catalog

```rust
pub trait Number: Copy {
    fn constant(c: f64) -> Self;
    fn add(self, o: Self) -> Self;  /* sub, mul, div, neg */
    fn powi(self, n: i32) -> Self;
    fn powf(self, e: Self) -> Self;   // base > 0
    fn exp(self) -> Self;  /* log, log10, log2, sqrt, sin, cos, tan, asin, acos,
                              atan, sinh, cosh, tanh, abs */
    fn atan2(self, x: Self) -> Self;
    fn hypot(self, b: Self) -> Self;
    fn nthroot(self, n: i32) -> Self;
}
```

There are three families of implementations:

- **`f64`:** plain values. Outside an open domain the result is NaN or ±∞. Point evaluation maps any non-finite result to `DomainError`, naming the first node whose result became non-finite.
- **`Dual2<T: Number>`:** the triple `(v, d1, d2)` with second-order forward rules, for example `sin`: `(sin v, cos v·d1, −sin v·d1² + cos v·d2)`, and `mul`: `(ab, a′b + ab′, a″b + 2a′b′ + ab″)`. The type is generic over the base, so `Dual2<f64>` gives the point jet and `Dual2<Interval>` gives interval bounds on f′ and f″.
- **`Interval`:** see decision 5.

Domain conditions live in the number types (decision 5), not in the evaluator, so all three families apply the same catalog.

*Alternative:* separate hand-written derivative code per function, outside the trait. That duplicates the catalog, and the interval derivative bounds would need a third copy.

### 5. Interval arithmetic: outward rounding, poison outside domains

`Interval { lo, hi }` with `lo ≤ hi`, plus a special value **poison**, the whole line `[−∞, +∞]`.

**Outward rounding** works without changing CPU rounding modes, which wasm does not provide. Each bound is computed in round-to-nearest and then widened:
- correctly rounded IEEE operations (`+ − × ÷ sqrt`): widen by one ulp (`next_down` / `next_up`);
- library functions (`exp log sin …`, whose accuracy is not guaranteed to be correctly rounded): widen by **4 ulps**, and clamp to known ranges (`sin`, `cos` in [−1, 1]; `atan` in (−π/2, π/2) widened; `exp` ≥ 0).

**Bounds per function:**
- **Monotone functions:** from the endpoints.
- **`sin`, `cos`:** from the endpoints plus every interior critical point `kπ/2`. Whether such a point lies inside is tested with a margin of one ulp of `kπ/2` on each side. When in doubt, the extremum is included, which can only widen the result. An interval of width ≥ 2π gives [−1, 1].
- **Even powers, `abs`, `cosh`:** handle sign-straddling arguments.

**Domain guard.** Every operation whose argument interval is not entirely inside the open domain from the spec returns **poison**. This also covers an argument that merely touches the boundary, for example `sqrt([0, 1])`. Poison propagates through all operations, and finiteness tests fail on it.

**Tracking the culprit.** The evaluator records the index of the first node that produced poison, for the error message.

**Additional poison conditions** for the functions whose domain is not an interval:
- `tan`: the argument interval contains an odd multiple of π/2 (tested with margin).
- `atan2(y, x)`: `y` contains 0 and `x` reaches ≤ 0.
- `hypot`: both arguments contain 0.
- Division: the denominator contains 0.

*Alternatives:*
- **An interval crate** (`inari`): it relies on hardware rounding modes or x86 intrinsics and does not build for `wasm32-unknown-unknown`.
- **Directed rounding via soft-float:** exact, but far heavier than what a conservative check needs.

### 6. Domain check by bisection

`check_domain(x0, x1)` runs the following steps:
1. Validate the range: both ends finite, x0 < x1.
2. Evaluate `Dual2<Interval>` with the variable `[x0, x1]` and derivative seed (`d1 = [1, 1]`, `d2 = [0, 0]`).
3. If no node is bad (value, `d1` and `d2` of every node finite: not poison, both bounds finite), the sub-range is proven. Every node, not only the result, as in `eval`: `Dual2::powi(0)` returns the constant 1 and drops a poisoned base, so checking only the result accepted `log(x)^0` on [-1, 1] (review of PR #88).
4. Otherwise bisect and recurse, up to **depth 32**, and stop at a sub-range width below `1e-12·max(1, |x|)`.

A sub-range that fails at the limit is reported with the culprit node's function name. Overestimation, for example the dependency effect in `x - x`, is resolved by bisection on ordinary ranges.

The work is bounded by a budget of 20 000 interval evaluations per check. Depth and width limits alone do not bound it: proven sub-ranges are not revisited, but overestimation that only very narrow sub-ranges resolve (`log(cosh(x) - sinh(x))` on [0, 10]) proves millions of them before a leaf fails, which took minutes (review of PR #88). When the budget is spent, the check returns `DomainCheckError::TooComplex` with the part of the range not yet proven, a conservative rejection the spec allows.

*Alternative:* sampling plus root finding of critical subexpressions. That is not sound: it can miss a narrow pole between samples. It belongs to #83 for locating singularities, not to proving their absence.

### 7. Reference values from GNU Octave, committed as a table

- **The script** `crates/expression/tests/reference/generate.m` lists the test expressions with their evaluation points and writes `reference.tsv` with the columns `source`, `x`, `value`, using `%.17g`. Octave serves for the values only.
- **Derivatives** are checked inside Rust:
  - against hand-derived formulas for every catalog function;
  - against central differences of the value with a loose tolerance, as a second net.
- **The integration test** `tests/reference.rs` parses each row with variable `x`, evaluates it and compares within 4 ulps relative (2⁻⁵⁰), with an absolute floor for values near 0. The tolerance accounts for libm differences between Octave's and Rust's math libraries.
- **Rows where Octave yields a complex or non-finite value** are written as `domain-error` and must produce `DomainError` in Rust.

The table is regenerated manually with `octave-cli generate.m` and committed, so CI needs no Octave. Octave is installed in the dev container for that purpose.

*Alternative:* Julia. Its numeric values are as good, but its precedence differs (`^` right-associative), so typing the same source text there would compute different values for exactly the cases this table is meant to pin down.

## Risks / Trade-offs

- [Library functions may differ from Octave's by more than 4 ulps for extreme arguments] → Keep reference arguments moderate (|x| ≤ 1e3 for trigonometric functions). A larger difference shows up as a test failure that names the row, and can be investigated rather than hidden.
- [4-ulp widening assumes the platform's libm is accurate to a few ulps] → This holds for the glibc and musl-derived implementations used natively and by Rust on wasm, and it is an explicit soundness assumption: the domain guards compare intervals that are themselves built from library results (`sqrt(1 + sin(x))` compares the enclosure of `1 + sin(x)` with 0), so a library error beyond 4 ulps could let a guard pass wrongly.
- [Interval overestimation rejects valid expressions] → The spec allows it, and bisection to depth 32 reduces it. A pathological case shows up as a rejection with a location, not as a wrong curve.
- [Octave accepts a construct our parser rejects, or differs from MATLAB in an accepted construct] → Our parser defines acceptance. Reference rows only contain accepted expressions, and `2^-2^2` is rejected deliberately.
- [Installing Octave in the dev container enlarges the image] → `octave-cli` only (no GUI), added to the Dockerfile. Alternatively it can be installed ad hoc; the committed table keeps CI independent of it.

## Migration Plan

This is a new crate; nothing existing changes. Rollback is a removal.

## Open Questions

- Maximum source length or node count for a UI-entered expression (resource limit). It can be added in #81 or #84 without changing this layer's behaviour for ordinary input. Nesting depth and domain-check work are already bounded here.
