# Proposal: Expression language

## Why

The product goal names coordinate axes with a function curve and a constrained tangent as a typical teaching figure. Function curves (#74) need the user to type a function such as `x*sin(x)`. The system must parse it, evaluate it with its first and second derivatives, and prove that it is defined and smooth over the drawn range before drawing it. This change is change A of the function-curve split (#78). It is a self-contained layer with no knowledge of geometry, so it can land before the curve kind (#81) that uses it.

## What Changes

- **New expression capability:** a strict subset of MATLAB syntax for real functions of one variable.
  - Operators `+ - * / ^` and the elementwise forms `.* ./ .^`.
  - MATLAB precedence (`^` left-associative, `-x^2 = -(x^2)`, `2^-1`).
  - Numbers such as `1e-3`, `.5`, `5.`, and the constant `pi`.
  - The functions `exp log log10 log2 sqrt nthroot sin cos tan asin acos atan atan2 hypot sinh cosh tanh abs`.
  - The caller names the variable: `x` for the function form.
- **Compatibility rule:** every accepted expression is valid MATLAB and has the same real value there. Whatever MATLAB evaluates as complex is a domain error. Everything outside the subset is rejected with an error that points to its position in the source text. This covers:
  - implicit multiplication, the constant `e`, the imaginary units `i` and `j`;
  - comparisons, discontinuous functions, transpose and `\`.
- **Evaluation:** an expression evaluates to its value, its first derivative and its second derivative at a point, by automatic differentiation rather than by symbolic or numeric differencing.
- **Domain check over a closed range:** proves that value, first and second derivative are finite on the whole range. It is conservative: it may reject a valid expression but never accepts an invalid one. On failure it names where the proof failed. Singularities inside or at the ends of the range are rejected here; handling them is #83.
- **Source text:** stored verbatim. The parsed form is derived from it.
- **Reference values:** the tests check values against a table computed with GNU Octave. MATLAB itself is not available to the project. Octave is used only for values; what the language accepts is decided by this capability, because Octave also accepts syntax MATLAB rejects.

## Capabilities

### New Capabilities

- `expression-language`: grammar and precedence of the MATLAB subset, the function catalog with its open domains, the power rule, the compatibility rule, evaluation of value and derivatives, the domain check over a range, and error reporting with source positions.

### Modified Capabilities

None. Geometry, commands and files do not use expressions yet. That is #81.

## Non-goals

- **Function curve kind:** its attributes, chart, commands, file format and UI (#81, #82, #84).
- **Singularities:** locating and classifying them, and splitting at them (#83). Branches across poles (#76).
- **Expression parameters and further inputs:**
  - parameters as solver degrees of freedom (#20);
  - named or shared expressions as document objects;
  - a second variable for the parametric form (the variable name is a parameter, so this can come later without a spec change).
- **Automatic Jacobians for existing geometry and constraint kinds** (#77). This change builds the number-type foundation but does not migrate the kinds.
- **Further MATLAB functions** beyond the v1 list.

The specs stay shared across implementations. Parser technique, number types and interval arithmetic are design decisions of each implementation.

## Impact

- **impls/rust:**
  - a new crate `expression` with no dependencies on other crates of the workspace and no third-party dependencies, built for native and wasm;
  - the workspace `Cargo.toml` and `scripts/check-wasm.sh` include it;
  - a reference table and the Octave script that generates it live with the crate's tests.
- **openspec/config.yaml:** the project context gains a line for the new crate.
- **Unchanged:** the existing crates, the sketch file, commands and export.
