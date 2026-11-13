# Spec Delta: expression-language

## Purpose

Defines the expression language in which users write real functions of one variable (for example the function of a function curve): its MATLAB-compatible syntax, the function catalog and its domains, evaluation of value and derivatives, and the proof that an expression is defined and smooth over a range. The layer has no knowledge of geometry.

## ADDED Requirements

### Requirement: Expressions are a strict subset of MATLAB syntax

The system SHALL accept an expression only if it is valid MATLAB syntax for a real scalar function of one variable. An accepted expression SHALL evaluate to the same real value as in MATLAB wherever MATLAB's result is real, and SHALL report a domain error wherever MATLAB's result is complex. Expressions SHALL be case-sensitive and MAY contain spaces between tokens.

#### Scenario: Valid MATLAB is accepted

- **WHEN** the expression `x .* sin(x) + 2^-1` is parsed with variable `x`
- **THEN** it is accepted
- **AND** at x = 1 it evaluates to sin(1) + 0.5

#### Scenario: A complex MATLAB result is a domain error

- **WHEN** `(-8)^(1/3)` or `sqrt(x)` at x = -1 is evaluated
- **THEN** the evaluation reports a domain error instead of a value

### Requirement: Expression tokens

An expression SHALL consist of:
- numbers;
- the constant `pi`;
- the variable;
- calls of catalog functions;
- the operators `+ - * / ^ .* ./ .^`;
- parentheses.

A number SHALL be digits with an optional fractional part and an optional exponent (`2`, `2.5`, `.5`, `5.`, `1e-3`, `1.5E+3`). A dot directly followed by `*`, `/` or `^` SHALL belong to the operator, not to the number. The elementwise operators SHALL mean the same as their plain forms.

#### Scenario: Number forms

- **WHEN** `.5 + 5. + 1e-3 + 1.5E+3` is evaluated
- **THEN** the value is 1505.501

#### Scenario: Dot before an operator

- **WHEN** `5.*x` is evaluated at x = 2
- **THEN** the value is 10

### Requirement: Number literals are finite

A number whose value overflows to infinity (`1e999`) SHALL be rejected as an invalid number at its position.

#### Scenario: Overflowing number

- **WHEN** `x + 1e999` is parsed
- **THEN** parsing fails with an invalid-number error at the position of `1e999`

### Requirement: The caller names the variable

Parsing SHALL take the name of the single variable (for example `x`). Any other identifier that is neither `pi` nor a catalog function SHALL be rejected with an error naming it.

#### Scenario: Unknown identifier

- **WHEN** `x + y` is parsed with variable `x`
- **THEN** parsing fails with an error that names `y` and its position

#### Scenario: Variable is chosen by the caller

- **WHEN** `t^2` is parsed with variable `t`
- **THEN** it is accepted
- **AND** at t = 3 it evaluates to 9

### Requirement: Operator precedence follows MATLAB

Precedence SHALL follow MATLAB, from tightest to loosest:
1. parentheses and function calls;
2. power `^ .^`, left-associative, whose right operand MAY carry leading unary signs;
3. unary `+ -`;
4. `* / .* ./`, left-associative;
5. binary `+ -`, left-associative.

A power whose right operand carries a unary sign SHALL NOT be followed directly by another power; such input SHALL be rejected with a hint to add parentheses.

#### Scenario: Power is left-associative

- **WHEN** `2^3^2` is evaluated
- **THEN** the value is 64

#### Scenario: Unary minus binds looser than power

- **WHEN** `-x^2` is evaluated at x = 3
- **THEN** the value is -9

#### Scenario: Signed exponents and signed factors

- **WHEN** `2^-1`, `2*-3` and `2--3` are evaluated
- **THEN** the values are 0.5, -6 and 5

#### Scenario: Signed exponent followed by a power is rejected

- **WHEN** `2^-2^2` is parsed
- **THEN** parsing fails at the second `^` with a hint to add parentheses

### Requirement: Function catalog

The catalog SHALL contain:

| arity | functions |
|---|---|
| one argument | `exp log log10 log2 sqrt sin cos tan asin acos atan sinh cosh tanh abs` |
| two arguments | `nthroot(x, n) atan2(y, x) hypot(a, b)` |

`log` SHALL be the natural logarithm. A call with the wrong number of arguments, or a catalog name used without a call, SHALL be rejected.

#### Scenario: Wrong arity

- **WHEN** `atan2(x)` is parsed
- **THEN** parsing fails with an error naming `atan2` and the expected two arguments

#### Scenario: nthroot gives real odd roots

- **WHEN** `nthroot(x, 3)` is evaluated at x = -8
- **THEN** the value is -2

### Requirement: Open domains of one-argument functions

Each one-argument catalog function and the division SHALL have an **open domain**: the arguments where its value and its first and second derivatives are finite and real.

| function | open domain |
|---|---|
| `log log10 log2 sqrt` | argument > 0 |
| `asin acos` | argument in (−1, 1) |
| `tan` | argument not an odd multiple of π/2 |
| `abs` | argument ≠ 0 |
| division | denominator ≠ 0 |
| `exp sin cos atan sinh cosh tanh` | every real argument |

#### Scenario: Square root at zero

- **WHEN** `sqrt(x)` is evaluated at x = 0
- **THEN** the evaluation reports a domain error, because its derivative is not finite there

### Requirement: Open domains of two-argument functions

The two-argument catalog functions SHALL have these open domains:

| function | open domain |
|---|---|
| `nthroot(x, n)` | `n` a constant nonzero integer; `x ≠ 0` for \|n\| ≥ 2; `x > 0` for even `n` |
| `atan2(y, x)` | not (`y = 0` and `x ≤ 0`) |
| `hypot(a, b)` | not both zero |

#### Scenario: Even root of a negative number

- **WHEN** `nthroot(x, 2)` is evaluated at x = -4
- **THEN** the evaluation reports a domain error

#### Scenario: atan2 across its branch cut

- **WHEN** `atan2(x, -1)` is checked on [-1, 1]
- **THEN** the domain check rejects it and names `atan2`

### Requirement: Power rule

For `a ^ b`:
- if `b` is a constant (contains no variable) with an integer value, any base SHALL be allowed, with `a ≠ 0` required for negative `b`;
- otherwise `a > 0` SHALL be required.

#### Scenario: Integer exponent on a negative base

- **WHEN** `x^3` is evaluated at x = -2
- **THEN** the value is -8

#### Scenario: Fractional exponent on a negative base

- **WHEN** `x^(1/3)` is evaluated at x = -8
- **THEN** the evaluation reports a domain error

### Requirement: Rejected constructs

The following SHALL be rejected with an error naming the construct and its position, even where MATLAB accepts them:
- implicit multiplication (`2x`);
- the names `e`, `i`, `j`;
- comparisons and logical operators;
- discontinuous functions (`sign floor ceil round mod rem`);
- transpose `'` and `.'`;
- left division `\`;
- `**`;
- matrices and brackets `[ ]`;
- any function outside the catalog.

#### Scenario: Implicit multiplication

- **WHEN** `2x` is parsed with variable `x`
- **THEN** parsing fails at the position of `x`

#### Scenario: Euler's number is not a constant

- **WHEN** `e^x` is parsed with variable `x`
- **THEN** parsing fails with an error naming `e` and suggesting `exp(x)`

### Requirement: Nesting depth is limited

Parentheses, signs, calls and exponents nested more than 128 levels deep SHALL be rejected with an error. The deepest accepted nesting SHALL be parsed within a 1 MiB stack.

#### Scenario: Deep nesting

- **WHEN** an expression with 100 000 nested parentheses or leading minus signs is parsed
- **THEN** parsing fails with an error that the expression is nested too deeply, without exhausting the stack

### Requirement: Parse errors carry source positions

Every parse error SHALL state what is wrong and the position in the source text (a byte range of the UTF-8 source text) where it occurs, so a user interface can point at it.

#### Scenario: Unbalanced parenthesis

- **WHEN** `sin(x` is parsed
- **THEN** parsing fails with an error at the end of the input stating that `)` is missing

### Requirement: The source text is authoritative

An expression SHALL keep its source text verbatim. Its parsed form SHALL be derived from the source text and SHALL NOT replace it. Two expressions SHALL be equal when their source texts and variable names are equal.

#### Scenario: Source is kept verbatim

- **WHEN** `x .*  sin( x )` is parsed
- **THEN** its stored source text is exactly `x .*  sin( x )`

### Requirement: Value and derivatives at a point

The system SHALL evaluate an accepted expression at a variable value to its value and its first and second derivatives with respect to the variable. Derivatives SHALL be exact up to floating-point rounding, not approximations by finite differences. Evaluation SHALL report a domain error when the point lies outside an open domain of any function in the expression, or when the value or a derivative is not finite.

#### Scenario: Derivatives of a product

- **WHEN** `x*sin(x)` is evaluated at x = 2
- **THEN** the value is 2 sin 2, the first derivative is sin 2 + 2 cos 2, and the second derivative is 2 cos 2 − 2 sin 2, each within floating-point tolerance

#### Scenario: Point outside the open domain

- **WHEN** `log(x)` is evaluated at x = 0
- **THEN** the evaluation reports a domain error

### Requirement: Domain check over a closed range

Given an accepted expression and a finite range [x0, x1] with x0 < x1, the system SHALL decide whether the value and the first and second derivatives are finite at every point of the closed range. The check SHALL NOT accept an expression that is not finite there; it MAY reject one that is (conservative). A singularity inside the range or at one of its ends SHALL lead to rejection.

#### Scenario: Smooth function is accepted

- **WHEN** `x .* sin(x)` is checked on [0, 2π]
- **THEN** the check accepts it

#### Scenario: A zero power keeps the domain of its base

- **WHEN** `log(x)^0` is checked on [-1, 1]
- **THEN** the check rejects it and names `log`, as evaluation at x = -0.5 reports a domain error

### Requirement: Domain check failures name a location

When the domain check rejects an expression, it SHALL report a sub-range of [x0, x1] where the proof failed and, where known, the function whose open domain may be violated. A range with x0 ≥ x1 or a non-finite end SHALL be reported as an invalid range.

#### Scenario: Singularity at a range end is rejected

- **WHEN** `sqrt(x)` is checked on [0, 1]
- **THEN** the check rejects it and reports a sub-range containing 0 and the function `sqrt`

#### Scenario: Pole inside the range is rejected

- **WHEN** `1/x` is checked on [-1, 1]
- **THEN** the check rejects it and reports a sub-range containing 0

#### Scenario: Corner is rejected

- **WHEN** `abs(x)` is checked on [-1, 1]
- **THEN** the check rejects it and names `abs`

#### Scenario: Invalid range

- **WHEN** a range with x0 ≥ x1 or a non-finite end is given
- **THEN** the check reports an invalid range

### Requirement: The domain check bounds its work

The domain check SHALL stop after a bounded number of interval evaluations, counting proven sub-ranges as well as failing ones. When the bound is reached before the whole range is proven, the check SHALL reject the expression (conservatively) and report the part of the range not yet proven, distinguishable from a located failure.

#### Scenario: Overestimation does not stall the check

- **WHEN** `log(cosh(x) - sinh(x))` is checked on [0, 10]
- **THEN** the check returns within a bounded amount of work
- **AND** it reports that the expression is too complex to prove on a sub-range ending at 10
