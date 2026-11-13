//! Expression language (spec: `openspec/specs/expression-language`).
//!
//! A strict subset of MATLAB syntax for real functions of one variable:
//! every accepted expression is valid MATLAB and has the same real value
//! there; whatever MATLAB evaluates as complex is a domain error. The layer
//! knows nothing about geometry.

mod domain;
mod dual;
mod error;
mod interval;
mod lexer;
mod number;
mod parser;
mod tree;

pub use dual::Dual2;
pub use error::{DomainCheckError, DomainError, ParseError, ParseErrorKind};
pub use interval::Interval;
pub use number::Number;

use tree::Node;

/// Value and first and second derivative with respect to the variable.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jet {
    pub value: f64,
    pub d1: f64,
    pub d2: f64,
}

/// A parsed expression in one variable.
///
/// The source text is authoritative and kept verbatim; the node array is
/// derived from it. Equality compares source text and variable name only.
#[derive(Clone, Debug)]
pub struct Expression {
    source: String,
    variable: String,
    nodes: Vec<Node>,
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.variable == other.variable
    }
}

impl Expression {
    /// Parses `source` with the single variable `variable` (e.g. `x`).
    ///
    /// # Panics
    /// If `variable` is not an identifier.
    pub fn parse(source: &str, variable: &str) -> Result<Self, ParseError> {
        let is_ident = variable
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && variable.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        assert!(is_ident, "variable name {variable:?} is not an identifier");
        let nodes = parser::parse(source, variable)?;
        Ok(Self {
            source: source.to_string(),
            variable: variable.to_string(),
            nodes,
        })
    }

    /// The source text, verbatim.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The variable name given to [`Self::parse`].
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Value and first and second derivative at `x`.
    ///
    /// A domain error names the first function or operation whose result is
    /// outside its open domain or not finite.
    ///
    /// Allocates a scratch buffer per call; see [`Self::eval_in`] for hot paths.
    pub fn eval(&self, x: f64) -> Result<Jet, DomainError> {
        self.eval_in(x, &mut Vec::new())
    }

    /// [`Self::eval`] with a caller-owned scratch buffer: allocation-free once
    /// `scratch` has room for [`Self::node_count`] entries (it is cleared and
    /// reused, never shrunk).
    pub fn eval_in(&self, x: f64, scratch: &mut Vec<Dual2<f64>>) -> Result<Jet, DomainError> {
        let (r, bad) = tree::eval(&self.nodes, Dual2::variable(x), scratch);
        match bad {
            None => Ok(Jet {
                value: r.v,
                d1: r.d1,
                d2: r.d2,
            }),
            Some(i) => Err(DomainError {
                function: self.nodes[i].name(),
            }),
        }
    }

    /// Proves that value, first and second derivative are finite on the whole
    /// closed range `[x0, x1]` (conservative: may reject a valid expression,
    /// never accepts an invalid one).
    pub fn check_domain(&self, x0: f64, x1: f64) -> Result<(), DomainCheckError> {
        domain::check_domain(&self.nodes, x0, x1)
    }

    /// Number of nodes: the scratch size [`Self::eval_in`] needs.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Evaluates the expression with the variable set to `x`, in any number
    /// type (allocates a scratch buffer per call).
    pub fn eval_with<N: Number>(&self, x: N) -> N {
        tree::eval(&self.nodes, x, &mut Vec::with_capacity(self.nodes.len())).0
    }
}

#[cfg(test)]
#[path = "tests_lib.rs"]
mod tests_lib;
