//! Parse and evaluation errors.

use std::fmt;
use std::ops::Range;

/// A rejected source text: what is wrong and where (byte range into the source).
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    /// Byte range in the source; empty at the end of the input.
    pub span: Range<usize>,
}

/// What made a source text invalid.
#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    /// A character outside the language.
    UnexpectedChar(char),
    /// Valid MATLAB that this subset rejects (`transpose`, `comparison`, …).
    Unsupported(&'static str),
    /// A malformed number such as `2e`.
    InvalidNumber,
    /// An identifier that is neither the variable, `pi` nor a catalog function.
    UnknownIdentifier(String),
    /// A known name that is deliberately rejected, with an optional hint.
    RejectedName {
        name: String,
        hint: Option<&'static str>,
    },
    /// Two operands without an operator between them (`2x`).
    ImplicitMultiplication,
    /// A power with a signed exponent followed by another power (`2^-2^2`).
    AmbiguousPower,
    /// `(` without its `)`.
    MissingCloseParen,
    /// An operand was expected here.
    ExpectedOperand,
    /// A token that cannot follow what precedes it.
    UnexpectedToken,
    /// A catalog function called with the wrong number of arguments.
    WrongArity {
        function: &'static str,
        expected: usize,
        found: usize,
    },
    /// A catalog function name without a call.
    FunctionWithoutCall(&'static str),
    /// The degree of `nthroot` is not a constant nonzero integer.
    NthrootDegree,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseErrorKind::*;
        match &self.kind {
            UnexpectedChar(c) => write!(f, "unexpected character `{c}`"),
            Unsupported(what) => write!(f, "{what} is not supported"),
            InvalidNumber => write!(f, "invalid number"),
            UnknownIdentifier(name) => write!(f, "unknown name `{name}`"),
            RejectedName { name, hint } => match hint {
                Some(h) => write!(f, "`{name}` is not supported; {h}"),
                None => write!(f, "`{name}` is not supported"),
            },
            ImplicitMultiplication => write!(f, "missing operator (write `*` to multiply)"),
            AmbiguousPower => {
                write!(f, "power after a signed exponent; add parentheses")
            }
            MissingCloseParen => write!(f, "missing `)`"),
            ExpectedOperand => write!(f, "expected a number, a name or `(`"),
            UnexpectedToken => write!(f, "unexpected symbol"),
            WrongArity {
                function,
                expected,
                found,
            } => write!(
                f,
                "`{function}` takes {expected} argument{}, got {found}",
                if *expected == 1 { "" } else { "s" }
            ),
            FunctionWithoutCall(name) => write!(f, "`{name}` must be called, e.g. `{name}(x)`"),
            NthrootDegree => write!(f, "the degree of `nthroot` must be a constant nonzero integer"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Evaluation at a point outside an open domain, or with a non-finite result.
#[derive(Clone, Debug, PartialEq)]
pub struct DomainError {
    /// The function or operation whose result first became invalid, if known.
    pub function: Option<&'static str>,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.function {
            Some(name) => write!(f, "`{name}` is not defined or not smooth here"),
            None => write!(f, "the value is not finite here"),
        }
    }
}

impl std::error::Error for DomainError {}

/// Outcome of a failed domain check over a range.
#[derive(Clone, Debug, PartialEq)]
pub enum DomainCheckError {
    /// The range is not finite or not increasing.
    InvalidRange,
    /// Value and derivatives could not be proven finite on `range`.
    NotProven {
        range: (f64, f64),
        function: Option<&'static str>,
    },
}

impl fmt::Display for DomainCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange => write!(f, "the range must be finite with start < end"),
            Self::NotProven {
                range: (a, b),
                function: Some(name),
            } => write!(f, "`{name}` may be undefined or not smooth in [{a}, {b}]"),
            Self::NotProven {
                range: (a, b),
                function: None,
            } => write!(f, "the function may not be finite in [{a}, {b}]"),
        }
    }
}

impl std::error::Error for DomainCheckError {}
