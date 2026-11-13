//! Expression tree: formulas as nodes in a flat post-order array.

use crate::number::Number;

/// Index of a node in the post-order array (children before parents).
pub(crate) type NodeId = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// One-argument catalog functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Func1 {
    Exp,
    Log,
    Log10,
    Log2,
    Sqrt,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Abs,
}

/// Two-argument catalog functions with two operand subtrees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Func2 {
    Atan2,
    Hypot,
}

impl Func1 {
    pub(crate) const ALL: [(&'static str, Func1); 15] = [
        ("exp", Func1::Exp),
        ("log", Func1::Log),
        ("log10", Func1::Log10),
        ("log2", Func1::Log2),
        ("sqrt", Func1::Sqrt),
        ("sin", Func1::Sin),
        ("cos", Func1::Cos),
        ("tan", Func1::Tan),
        ("asin", Func1::Asin),
        ("acos", Func1::Acos),
        ("atan", Func1::Atan),
        ("sinh", Func1::Sinh),
        ("cosh", Func1::Cosh),
        ("tanh", Func1::Tanh),
        ("abs", Func1::Abs),
    ];

    pub(crate) fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(_, f)| *f == self)
            .map(|(n, _)| *n)
            .expect("every function is listed")
    }

    fn apply<N: Number>(self, a: N) -> N {
        match self {
            Func1::Exp => a.exp(),
            Func1::Log => a.log(),
            Func1::Log10 => a.log10(),
            Func1::Log2 => a.log2(),
            Func1::Sqrt => a.sqrt(),
            Func1::Sin => a.sin(),
            Func1::Cos => a.cos(),
            Func1::Tan => a.tan(),
            Func1::Asin => a.asin(),
            Func1::Acos => a.acos(),
            Func1::Atan => a.atan(),
            Func1::Sinh => a.sinh(),
            Func1::Cosh => a.cosh(),
            Func1::Tanh => a.tanh(),
            Func1::Abs => a.abs(),
        }
    }
}

impl Func2 {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Func2::Atan2 => "atan2",
            Func2::Hypot => "hypot",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Node {
    Const(f64),
    Var,
    Neg(NodeId),
    Bin(BinOp, NodeId, NodeId),
    /// Power with a constant integer exponent (any base).
    PowInt(NodeId, i32),
    /// Power with any other exponent (positive base).
    PowReal(NodeId, NodeId),
    Call1(Func1, NodeId),
    Call2(Func2, NodeId, NodeId),
    /// `nthroot(x, n)` with a constant nonzero integer degree.
    NthRoot(NodeId, i32),
}

impl Node {
    /// Name of the function or operation for error messages; `None` for
    /// nodes that have no domain restriction of their own.
    pub(crate) fn name(&self) -> Option<&'static str> {
        match self {
            Node::Bin(BinOp::Div, ..) => Some("division"),
            Node::PowInt(..) | Node::PowReal(..) => Some("power"),
            Node::Call1(f, _) => Some(f.name()),
            Node::Call2(f, ..) => Some(f.name()),
            Node::NthRoot(..) => Some("nthroot"),
            _ => None,
        }
    }
}

/// Evaluates the post-order array at `x` into `scratch`. Returns the value of
/// the last node and the index of the first node whose result is bad.
pub(crate) fn eval<N: Number>(nodes: &[Node], x: N, scratch: &mut Vec<N>) -> (N, Option<NodeId>) {
    scratch.clear();
    let mut first_bad = None;
    for (i, node) in nodes.iter().enumerate() {
        let v = |j: NodeId| scratch[j];
        let r = match *node {
            Node::Const(c) => N::constant(c),
            Node::Var => x,
            Node::Neg(a) => v(a).neg(),
            Node::Bin(op, a, b) => match op {
                BinOp::Add => v(a).add(v(b)),
                BinOp::Sub => v(a).sub(v(b)),
                BinOp::Mul => v(a).mul(v(b)),
                BinOp::Div => v(a).div(v(b)),
            },
            Node::PowInt(a, n) => v(a).powi(n),
            Node::PowReal(a, b) => v(a).powf(v(b)),
            Node::Call1(f, a) => f.apply(v(a)),
            Node::Call2(Func2::Atan2, a, b) => v(a).atan2(v(b)),
            Node::Call2(Func2::Hypot, a, b) => v(a).hypot(v(b)),
            Node::NthRoot(a, n) => v(a).nthroot(n),
        };
        if first_bad.is_none() && r.is_bad() {
            first_bad = Some(i);
        }
        scratch.push(r);
    }
    (*scratch.last().expect("a parsed expression has a node"), first_bad)
}
