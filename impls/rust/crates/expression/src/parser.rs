//! Precedence-climbing parser for the MATLAB subset.
//!
//! ```text
//! sum      := product (('+'|'-') product)*
//! product  := unary (('*'|'/'|'.*'|'./') unary)*
//! unary    := ('+'|'-') unary | power
//! power    := primary (('^'|'.^') exponent)*
//! exponent := ('+'|'-')+ primary      -- must not be followed by '^'
//!           | primary
//! primary  := number | 'pi' | variable | name '(' args ')' | '(' sum ')'
//! ```

use std::ops::Range;

use crate::error::{ParseError, ParseErrorKind};
use crate::lexer::{lex, Tok, Token};
use crate::tree::{eval, BinOp, Func1, Func2, Node, NodeId};

/// Deepest nesting of operands (parentheses, unary signs, call arguments,
/// exponents). Every recursion of the parser passes through
/// [`Parser::unary`], so this bounds its stack use (wasm has a small stack).
const MAX_NESTING: usize = 128;

/// Names MATLAB knows that this subset rejects on purpose.
const REJECTED: [(&str, Option<&str>); 12] = [
    ("e", Some("write `exp(x)` for the exponential")),
    ("i", Some("complex numbers are not supported")),
    ("j", Some("complex numbers are not supported")),
    ("ln", Some("write `log(x)` for the natural logarithm")),
    ("sign", Some("discontinuous functions are not supported")),
    ("floor", Some("discontinuous functions are not supported")),
    ("ceil", Some("discontinuous functions are not supported")),
    ("round", Some("discontinuous functions are not supported")),
    ("fix", Some("discontinuous functions are not supported")),
    ("mod", Some("discontinuous functions are not supported")),
    ("rem", Some("discontinuous functions are not supported")),
    ("Inf", None),
];

struct Parser<'a> {
    src: &'a str,
    variable: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    nodes: Vec<Node>,
    /// Whether node `i`'s subtree contains the variable.
    has_var: Vec<bool>,
    /// Current recursion depth of [`Self::unary`].
    depth: usize,
}

fn err<T>(kind: ParseErrorKind, span: Range<usize>) -> Result<T, ParseError> {
    Err(ParseError { kind, span })
}

/// Parses `src` with the single variable `variable` into a post-order node array.
pub(crate) fn parse(src: &str, variable: &str) -> Result<Vec<Node>, ParseError> {
    let mut p = Parser {
        src,
        variable,
        tokens: lex(src)?,
        pos: 0,
        nodes: Vec::new(),
        has_var: Vec::new(),
        depth: 0,
    };
    p.sum()?;
    let t = p.peek().clone();
    match t.tok {
        Tok::End => Ok(p.nodes),
        _ => err(ParseErrorKind::UnexpectedToken, t.span),
    }
}

impl Parser<'_> {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn next(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        if t.tok != Tok::End {
            self.pos += 1;
        }
        t
    }

    fn push(&mut self, node: Node, has_var: bool) -> NodeId {
        self.nodes.push(node);
        self.has_var.push(has_var);
        self.nodes.len() - 1
    }

    fn var_in(&self, ids: &[NodeId]) -> bool {
        ids.iter().any(|&i| self.has_var[i])
    }

    fn sum(&mut self) -> Result<NodeId, ParseError> {
        let mut a = self.product()?;
        loop {
            let op = match self.peek().tok {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => return Ok(a),
            };
            self.next();
            let b = self.product()?;
            let v = self.var_in(&[a, b]);
            a = self.push(Node::Bin(op, a, b), v);
        }
    }

    fn product(&mut self) -> Result<NodeId, ParseError> {
        let mut a = self.unary()?;
        loop {
            let op = match self.peek().tok {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                _ => return Ok(a),
            };
            self.next();
            let b = self.unary()?;
            let v = self.var_in(&[a, b]);
            a = self.push(Node::Bin(op, a, b), v);
        }
    }

    fn unary(&mut self) -> Result<NodeId, ParseError> {
        if self.depth == MAX_NESTING {
            return err(ParseErrorKind::TooDeep, self.peek().span.clone());
        }
        self.depth += 1;
        let r = self.unary_inner();
        self.depth -= 1;
        r
    }

    fn unary_inner(&mut self) -> Result<NodeId, ParseError> {
        match self.peek().tok {
            Tok::Plus => {
                self.next();
                self.unary()
            }
            Tok::Minus => {
                self.next();
                let a = self.unary()?;
                let v = self.has_var[a];
                Ok(self.push(Node::Neg(a), v))
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<NodeId, ParseError> {
        let mut base = self.primary()?;
        while self.peek().tok == Tok::Caret {
            self.next();
            let start = self.nodes.len();
            let mut negate = false;
            let mut signed = false;
            while matches!(self.peek().tok, Tok::Plus | Tok::Minus) {
                signed = true;
                negate ^= self.next().tok == Tok::Minus;
            }
            let mut e = self.primary()?;
            if negate {
                let v = self.has_var[e];
                e = self.push(Node::Neg(e), v);
            }
            if signed && self.peek().tok == Tok::Caret {
                return err(ParseErrorKind::AmbiguousPower, self.peek().span.clone());
            }
            base = self.power_node(base, e, start);
        }
        Ok(base)
    }

    /// `base ^ e`, where `e`'s subtree occupies `nodes[start..]`. A constant
    /// integer exponent is folded into [`Node::PowInt`].
    fn power_node(&mut self, base: NodeId, e: NodeId, start: usize) -> NodeId {
        if let Some(n) = self.constant_integer(e) {
            self.nodes.truncate(start);
            self.has_var.truncate(start);
            let v = self.has_var[base];
            return self.push(Node::PowInt(base, n), v);
        }
        let v = self.var_in(&[base, e]);
        self.push(Node::PowReal(base, e), v)
    }

    /// Value of the constant subtree `id` if it is an integer in `i32` range.
    fn constant_integer(&self, id: NodeId) -> Option<i32> {
        if self.has_var[id] {
            return None;
        }
        let (value, _) = eval::<f64>(&self.nodes[..=id], f64::NAN, &mut Vec::new());
        let ok = value.is_finite()
            && value.fract() == 0.0
            && value >= f64::from(i32::MIN)
            && value <= f64::from(i32::MAX);
        ok.then_some(value as i32)
    }

    fn primary(&mut self) -> Result<NodeId, ParseError> {
        let t = self.next();
        let id = match t.tok {
            Tok::Num(value) => self.push(Node::Const(value), false),
            Tok::Ident => self.identifier(t.span.clone())?,
            Tok::LParen => {
                let inner = self.sum()?;
                self.close_paren()?;
                inner
            }
            _ => return err(ParseErrorKind::ExpectedOperand, t.span),
        };
        let after = self.peek().clone();
        if matches!(t.tok, Tok::Num(_)) && after.tok == Tok::Ident && after.span.start == t.span.end {
            // `2i`, `3j`: MATLAB's imaginary literals.
            if let name @ ("i" | "j") = &self.src[after.span.clone()] {
                let hint = REJECTED.iter().find(|(n, _)| n == &name).and_then(|(_, h)| *h);
                return err(
                    ParseErrorKind::RejectedName {
                        name: name.to_string(),
                        hint,
                    },
                    t.span.start..after.span.end,
                );
            }
        }
        if matches!(after.tok, Tok::Num(_) | Tok::Ident | Tok::LParen) {
            return err(ParseErrorKind::ImplicitMultiplication, after.span);
        }
        Ok(id)
    }

    fn close_paren(&mut self) -> Result<(), ParseError> {
        let t = self.peek().clone();
        match t.tok {
            Tok::RParen => {
                self.next();
                Ok(())
            }
            _ => err(ParseErrorKind::MissingCloseParen, t.span),
        }
    }

    fn identifier(&mut self, span: Range<usize>) -> Result<NodeId, ParseError> {
        let name = &self.src[span.clone()];
        let called = self.peek().tok == Tok::LParen;
        if name == self.variable {
            return Ok(self.push(Node::Var, true));
        }
        if name == "pi" {
            if called {
                return err(ParseErrorKind::Unsupported("calling `pi` (write `pi`)"), span);
            }
            return Ok(self.push(Node::Const(std::f64::consts::PI), false));
        }
        let func1 = Func1::ALL.iter().find(|(n, _)| *n == name).map(|(n, f)| (*n, *f));
        let func2 = [("atan2", Func2::Atan2), ("hypot", Func2::Hypot)]
            .into_iter()
            .find(|(n, _)| *n == name);
        let static_name = func1
            .map(|(n, _)| n)
            .or(func2.map(|(n, _)| n))
            .or((name == "nthroot").then_some("nthroot"));
        if let Some(fname) = static_name {
            if !called {
                return err(ParseErrorKind::FunctionWithoutCall(fname), span);
            }
            self.next();
            let args = self.arguments()?;
            let call_span = span.start..self.tokens[self.pos - 1].span.end;
            let expected = if func1.is_some() { 1 } else { 2 };
            if args.len() != expected {
                return err(
                    ParseErrorKind::WrongArity {
                        function: fname,
                        expected,
                        found: args.len(),
                    },
                    call_span,
                );
            }
            if let Some((_, f)) = func1 {
                let (a, _) = args[0];
                let v = self.has_var[a];
                return Ok(self.push(Node::Call1(f, a), v));
            }
            if let Some((_, f)) = func2 {
                let (a, b) = (args[0].0, args[1].0);
                let v = self.var_in(&[a, b]);
                return Ok(self.push(Node::Call2(f, a, b), v));
            }
            // nthroot: the degree is a constant nonzero integer; its nodes are the suffix.
            let (x, _) = args[0];
            let (n_id, (n_start, n_span)) = (args[1].0, args[1].1.clone());
            match self.constant_integer(n_id) {
                Some(n) if n != 0 => {
                    self.nodes.truncate(n_start);
                    self.has_var.truncate(n_start);
                    let v = self.has_var[x];
                    return Ok(self.push(Node::NthRoot(x, n), v));
                }
                _ => return err(ParseErrorKind::NthrootDegree, n_span),
            }
        }
        if let Some((_, hint)) = REJECTED.iter().find(|(n, _)| *n == name) {
            return err(
                ParseErrorKind::RejectedName {
                    name: name.to_string(),
                    hint: *hint,
                },
                span,
            );
        }
        err(ParseErrorKind::UnknownIdentifier(name.to_string()), span)
    }

    /// Arguments after `(` up to and including `)`: each root with its first
    /// node index and source span.
    #[allow(clippy::type_complexity)]
    fn arguments(&mut self) -> Result<Vec<(NodeId, (usize, Range<usize>))>, ParseError> {
        let mut args = Vec::new();
        loop {
            let start_node = self.nodes.len();
            let start_span = self.peek().span.start;
            let a = self.sum()?;
            let end_span = self.tokens[self.pos - 1].span.end;
            args.push((a, (start_node, start_span..end_span)));
            match self.peek().tok {
                Tok::Comma => {
                    self.next();
                }
                _ => {
                    self.close_paren()?;
                    return Ok(args);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_parser.rs"]
mod tests_parser;
