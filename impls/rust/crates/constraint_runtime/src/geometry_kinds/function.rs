//! Function curve: the graph of `y = f(x)` over `[x0, x1]` in a frame with
//! origin `O` and heading `θ`, scaled by `su`, `sv` (geometry-ir: Function
//! curve is an admitted curve kind).
//!
//! Only `O` and `θ` are State; the expression, range and scales are
//! attributes. The chart parameter `t` is the arc-length fraction: an
//! [`ArcTable`] built with the attributes inverts the arc length `s(x)` of the
//! scaled graph, and every point is then evaluated exactly on the graph.
//! Solving therefore never re-tabulates and never allocates (`f` is evaluated
//! with a thread-local scratch).

use std::cell::RefCell;
use std::ops::Range;
use std::sync::Arc;

use expression::{DomainCheckError, DomainError, Dual2, Expression, Interval, Jet, Number, ParseError};

use crate::geom::{CurveParam, HeadingAngle, Length, Point2, Vec2};
use crate::geometry::capabilities::geometry_caps;
use crate::geometry::chart::chart_atoms;
use crate::geometry::cubic::{hermite_shape, sampled_deviation};
use crate::geometry::{Geometry, LengthFormula, PieceBound, PointFormula, ShapePiece, VecFormula};
use crate::geometry_kinds::{GeometryKind, KindAttrs};
use crate::matrix::DenseMatrix;

/// Function curve kind (type-level marker).
///
/// Layout via `chart_atoms!`: [`Self::origin`] (Point2) and [`Self::heading`]
/// (heading of the frame's x axis).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Function;

chart_atoms! {
    Function {
        origin: Point2,
        heading: HeadingAngle,
    }
}

geometry_caps!(Function: curve, endpoints, anchor, line, abscissa);

/// The variable of a function curve's expression.
pub const FUNCTION_VARIABLE: &str = "x";

/// Initial equal intervals of an [`ArcTable`].
const TABLE_START_INTERVALS: usize = 16;
/// Most intervals of an [`ArcTable`]; beyond it the attributes are rejected.
pub const MAX_TABLE_INTERVALS: usize = 65_536;
/// Accuracy of the arc-length inverse, relative to the total length.
const TABLE_REL_TOL: f64 = 1e-11;
/// Most speed evaluations one table build may spend; beyond it the
/// attributes are rejected as too complex (the build runs on user input).
/// About twice what the largest table under [`MAX_TABLE_INTERVALS`] needs
/// (`sin(x)` on [0, 900π], 59 752 intervals).
const MAX_TABLE_EVALUATIONS: usize = 15_000_000;
/// A table build is rejected early when the projected interval count
/// exceeds [`MAX_TABLE_INTERVALS`] by this factor (the Hermite inverse's
/// error falls like h⁴, so an interval with error e needs about
/// (e/tol)^¼ parts; e is the first failing point's error, a lower bound of
/// the interval's largest, so the projection does not overshoot).
const TABLE_PROJECTION_SAFETY: f64 = 2.0;
/// Relative accuracy of the adaptive arc-length quadrature of the table.
const QUADRATURE_REL_TOL: f64 = 1e-14;
/// Relative accuracy of piece lengths ([`Geometry::piece_bound`]), well below
/// [`PIECE_LENGTH_MARGIN`]; tighter tolerances made steep flanks recurse deep.
const PIECE_QUADRATURE_REL_TOL: f64 = 1e-11;
/// Relative margin on piece lengths so they stay upper bounds. Relative to
/// the piece, not to the curve: an absolute margin of 1e-9·L dwarfed the
/// tiny pieces at a sharp peak, so the Mesh could never refine them.
const PIECE_LENGTH_MARGIN: f64 = 1e-9;

/// Why function attributes were rejected.
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionAttrsError {
    /// The expression does not parse.
    Parse(ParseError),
    /// `f`, `f′`, `f″` could not be proven finite on the range.
    Domain(DomainCheckError),
    /// The range is not finite or not increasing.
    Range,
    /// A scale is not finite or not greater than zero.
    Scale,
    /// The arc-length table needs more than [`MAX_TABLE_INTERVALS`] intervals
    /// or more than [`MAX_TABLE_EVALUATIONS`] speed evaluations.
    TooComplex,
}

impl std::fmt::Display for FunctionAttrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Domain(e) => write!(f, "{e}"),
            Self::Range => write!(f, "the range must be finite with x0 < x1"),
            Self::Scale => write!(f, "scales must be finite and greater than zero"),
            Self::TooComplex => write!(f, "the curve exceeded its budget; try to simplify it"),
        }
    }
}

impl std::error::Error for FunctionAttrsError {}

/// Attributes of a function curve (never solved) and the arc-length table
/// derived from them.
#[derive(Clone, Debug)]
pub struct FunctionAttrs {
    expression: Expression,
    range: (f64, f64),
    scale: (f64, f64),
    table: Arc<ArcTable>,
}

/// Equality compares the attributes, not the derived table.
impl PartialEq for FunctionAttrs {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression && self.range == other.range && self.scale == other.scale
    }
}

impl FunctionAttrs {
    /// Checked attributes: `source` in the variable `x`, `range = (x0, x1)`
    /// with `x0 < x1`, `scale = (su, sv)` both greater than zero, and `f`,
    /// `f′`, `f″` proven finite on the closed range (geometry-ir: Function
    /// curves accept only smooth functions).
    pub fn new(source: &str, range: (f64, f64), scale: (f64, f64)) -> Result<Self, FunctionAttrsError> {
        let (x0, x1) = range;
        if !(x0.is_finite() && x1.is_finite() && x0 < x1) {
            return Err(FunctionAttrsError::Range);
        }
        let (su, sv) = scale;
        if !(su.is_finite() && sv.is_finite() && su > 0.0 && sv > 0.0) {
            return Err(FunctionAttrsError::Scale);
        }
        let expression = Expression::parse(source, FUNCTION_VARIABLE).map_err(FunctionAttrsError::Parse)?;
        expression.check_domain(x0, x1).map_err(FunctionAttrsError::Domain)?;
        let table = ArcTable::build(&expression, range, scale)?;
        Ok(Self {
            expression,
            range,
            scale,
            table: Arc::new(table),
        })
    }

    /// The expression's source text, verbatim.
    pub fn source(&self) -> &str {
        self.expression.source()
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn range(&self) -> (f64, f64) {
        self.range
    }

    pub fn scale(&self) -> (f64, f64) {
        self.scale
    }

    /// Total arc length of the scaled graph.
    pub fn length(&self) -> f64 {
        self.table.length()
    }

    /// Number of intervals of the arc-length table.
    pub fn table_intervals(&self) -> usize {
        self.table.x.len() - 1
    }

    /// Whether `other` shares this table (a clone, not a rebuild).
    pub fn shares_table(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.table, &other.table)
    }

    /// `x` at arc length `s` from `x0` (clamped to `[0, L]`), and `dx/ds`.
    pub fn x_at_length(&self, s: f64) -> (f64, f64) {
        self.table.inverse(s)
    }

    /// Arc length of the scaled graph between `xa ≤ xb` (adaptive quadrature,
    /// accurate to about 1e-11 relative to that piece's own length).
    pub fn arc_length(&self, xa: f64, xb: f64) -> f64 {
        let speed = |x: f64| self.speed(x);
        let estimate = gauss(&speed, xa, xb);
        integrate(&speed, xa, xb, PIECE_QUADRATURE_REL_TOL * estimate.max(f64::MIN_POSITIVE))
    }

    /// `x` clamped to the range; NaN (a NaN curve parameter) becomes `x0`, so
    /// evaluation never leaves the proven domain and never panics.
    fn in_range(&self, x: f64) -> f64 {
        if x.is_nan() {
            self.range.0
        } else {
            x.clamp(self.range.0, self.range.1)
        }
    }

    /// `f(x)` with derivatives at any `x`, also outside the range (the
    /// continuation used by abscissa parameters).
    pub fn abscissa_jet(&self, x: f64) -> Result<Jet, DomainError> {
        try_jet(&self.expression, x)
    }

    /// Whether `x` is a valid abscissa: finite, `f` defined and `f(x)`, `f′(x)`
    /// finite (geometry-ir: Abscissa requires a defined function with finite
    /// slope).
    pub fn abscissa_valid(&self, x: f64) -> bool {
        x.is_finite() && self.abscissa_jet(x).is_ok_and(|j| j.value.is_finite() && j.d1.is_finite())
    }

    /// Local point and `dγ/dx` at a valid abscissa.
    fn local_at_abscissa(&self, x: f64) -> (Vec2, Vec2) {
        let j = self
            .abscissa_jet(x)
            .expect("abscissa expressions are checked when built and on every expression change");
        let (su, sv) = self.scale;
        (Vec2::new(su * x, sv * j.value), Vec2::new(su, sv * j.d1))
    }

    /// `f(x)` with first and second derivative (`x` clamped to the range).
    fn jet(&self, x: f64) -> Jet {
        jet(&self.expression, self.in_range(x))
    }

    /// `ds/dx` of the scaled graph.
    fn speed(&self, x: f64) -> f64 {
        speed(&self.expression, self.scale, self.in_range(x))
    }

    /// Local point `(su·x, sv·f(x))` (frame coordinates, before `O`, `θ`).
    fn local_point(&self, x: f64) -> Vec2 {
        let (su, sv) = self.scale;
        Vec2::new(su * x, sv * self.jet(x).value)
    }

    /// `x`, local point and local chart derivative `dγ/dt` at normalized `t`.
    fn local_at(&self, t: f64) -> (f64, Vec2, Vec2) {
        let l = self.length();
        let t = if t.is_nan() { 0.0 } else { t };
        let (x, dxds) = self.table.inverse(t * l);
        let j = self.jet(x);
        let (su, sv) = self.scale;
        let k = l * dxds;
        (x, Vec2::new(su * x, sv * j.value), Vec2::new(su * k, sv * j.d1 * k))
    }

    /// Unsigned curvature of the scaled graph at `x`.
    fn curvature_at(&self, x: f64) -> f64 {
        let j = self.jet(x);
        let (su, sv) = self.scale;
        let q = su * su + sv * sv * j.d1 * j.d1;
        (su * sv * j.d2).abs() / (q * q.sqrt())
    }

    /// Upper bound of the curvature on `[xa, xb]` by interval evaluation.
    fn curvature_bound(&self, xa: f64, xb: f64) -> f64 {
        let (su, sv) = self.scale;
        let d = self.expression.eval_with(Dual2::variable(Interval::new(xa, xb)));
        if d.v.is_bad() || d.d1.is_bad() || d.d2.is_bad() {
            return f64::INFINITY;
        }
        let f2 = d.d2.lo().abs().max(d.d2.hi().abs());
        let (lo, hi) = (d.d1.lo(), d.d1.hi());
        let min_f1_sq = if lo <= 0.0 && hi >= 0.0 { 0.0 } else { (lo * lo).min(hi * hi) };
        let q = su * su + sv * sv * min_f1_sq;
        // Rounding margin: the quotient is not computed with outward rounding.
        (su * sv * f2) / (q * q.sqrt()) * (1.0 + 1e-12)
    }
}

impl Default for FunctionAttrs {
    /// `x` on [0, 1] with scales 1: a valid diagonal ([`Geometry::default_attrs`]
    /// must be infallible). Built once and cloned (the table is shared), since
    /// every new function curve starts with it before its own attributes.
    fn default() -> Self {
        static DEFAULT: std::sync::OnceLock<FunctionAttrs> = std::sync::OnceLock::new();
        DEFAULT
            .get_or_init(|| Self::new("x", (0.0, 1.0), (1.0, 1.0)).expect("the identity on [0, 1] is smooth"))
            .clone()
    }
}

thread_local! {
    /// Evaluation scratch reused by every function curve on this thread, so
    /// the solve path does not allocate after the first call.
    static SCRATCH: RefCell<Vec<Dual2<f64>>> = const { RefCell::new(Vec::new()) };
}

fn try_jet(e: &Expression, x: f64) -> Result<Jet, DomainError> {
    SCRATCH.with(|s| e.eval_in(x, &mut s.borrow_mut()))
}

fn jet(e: &Expression, x: f64) -> Jet {
    try_jet(e, x).expect("the domain check proved f, f′ and f″ finite on the range")
}

fn speed(e: &Expression, (su, sv): (f64, f64), x: f64) -> f64 {
    su.hypot(sv * jet(e, x).d1)
}

/// 8-point Gauss–Legendre nodes (positive half) and weights on [−1, 1].
const GL_NODES: [f64; 4] = [
    0.183_434_642_495_649_8,
    0.525_532_409_916_329,
    0.796_666_477_413_626_7,
    0.960_289_856_497_536_3,
];
const GL_WEIGHTS: [f64; 4] = [
    0.362_683_783_378_362,
    0.313_706_645_877_887_3,
    0.222_381_034_453_374_5,
    0.101_228_536_290_376_3,
];

fn gauss(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let (m, h) = (0.5 * (a + b), 0.5 * (b - a));
    GL_NODES
        .iter()
        .zip(GL_WEIGHTS)
        .map(|(x, w)| w * (f(m - h * x) + f(m + h * x)))
        .sum::<f64>()
        * h
}

/// Below this relative difference two quadrature estimates agree within
/// rounding: the halved tolerance of a deep subinterval can fall under the
/// rounding noise of its own value, and the recursion then never converged
/// (`sin(100*x)` on [0, 1000] ran for minutes).
const QUADRATURE_NOISE: f64 = 64.0 * f64::EPSILON;

/// `∫ f` on `[a, b]` by adaptive Gauss–Legendre to absolute `tol`.
fn integrate(f: &impl Fn(f64) -> f64, a: f64, b: f64, tol: f64) -> f64 {
    integrate_until(f, a, b, tol, &|| false)
}

/// [`integrate`] that stops refining as soon as `stop()` returns true (the
/// result is then meaningless; the caller checks `stop` itself).
fn integrate_until(f: &impl Fn(f64) -> f64, a: f64, b: f64, tol: f64, stop: &impl Fn() -> bool) -> f64 {
    fn step(
        f: &impl Fn(f64) -> f64,
        a: f64,
        b: f64,
        whole: f64,
        tol: f64,
        depth: u32,
        stop: &impl Fn() -> bool,
    ) -> f64 {
        let m = 0.5 * (a + b);
        let (l, r) = (gauss(f, a, m), gauss(f, m, b));
        let noise = QUADRATURE_NOISE * (l.abs() + r.abs());
        if depth >= 30 || (l + r - whole).abs() <= tol.max(noise) || stop() {
            return l + r;
        }
        step(f, a, m, l, 0.5 * tol, depth + 1, stop) + step(f, m, b, r, 0.5 * tol, depth + 1, stop)
    }
    if b <= a {
        return 0.0;
    }
    step(f, a, b, gauss(f, a, b), tol, 0, stop)
}

/// Arc length `s(x)` of the scaled graph at table nodes, and its inverse
/// `x(s)` by cubic Hermite interpolation with slopes `dx/ds = 1/σ`.
#[derive(Debug)]
struct ArcTable {
    x: Vec<f64>,
    s: Vec<f64>,
    /// `dx/ds` at the nodes.
    slope: Vec<f64>,
}

impl ArcTable {
    /// Table whose inverse is within [`TABLE_REL_TOL`]·L in arc length at
    /// the checked points (quarter, half and three quarters of each
    /// interval's length), halving failing intervals.
    ///
    /// The check of an interval reads only its own nodes, length and end
    /// slopes, so an interval that passed stays passed: each round integrates
    /// and checks only the new halves. (Re-checking every interval each round
    /// made a rejection at the interval cap take seconds.)
    fn build(e: &Expression, range: (f64, f64), scale: (f64, f64)) -> Result<Self, FunctionAttrsError> {
        let evaluations = std::cell::Cell::new(0usize);
        let speed = |x: f64| {
            evaluations.set(evaluations.get() + 1);
            speed(e, scale, x.clamp(range.0, range.1))
        };
        let spent = || evaluations.get() > MAX_TABLE_EVALUATIONS;
        let integrate = |f: &dyn Fn(f64) -> f64, a: f64, b: f64, tol: f64| integrate_until(&f, a, b, tol, &spent);
        let n = TABLE_START_INTERVALS;
        let mut x: Vec<f64> = (0..=n)
            .map(|i| match i {
                0 => range.0,
                i if i == n => range.1,
                i => range.0 + (range.1 - range.0) * i as f64 / n as f64,
            })
            .collect();
        let crude: f64 = x.windows(2).map(|w| gauss(&speed, w[0], w[1])).sum();
        let qtol = QUADRATURE_REL_TOL * crude;
        let mut len: Vec<f64> = x.windows(2).map(|w| integrate(&speed, w[0], w[1], qtol)).collect();
        if spent() {
            return Err(FunctionAttrsError::TooComplex);
        }
        let mut slope: Vec<f64> = x.iter().map(|&xi| 1.0 / speed(xi)).collect();
        let mut passed = vec![false; n];
        let tol = TABLE_REL_TOL * len.iter().sum::<f64>();
        loop {
            // First error above `tol` of interval i alone (nodes, length, end
            // slopes), or 0 if it passes; stopping at the first failing
            // point keeps wild intervals cheap.
            let error = |i: usize| {
                let single = Self {
                    x: vec![x[i], x[i + 1]],
                    s: vec![0.0, len[i]],
                    slope: vec![slope[i], slope[i + 1]],
                };
                [0.25, 0.5, 0.75]
                    .iter()
                    .map(|&tau| {
                        let target = tau * len[i];
                        let (xm, _) = single.inverse_in(0, target);
                        (integrate(&speed, x[i], xm, qtol) - target).abs()
                    })
                    .find(|&err| err > tol)
                    .unwrap_or(0.0)
            };
            let errors: Vec<f64> = (0..len.len()).map(|i| if passed[i] { 0.0 } else { error(i) }).collect();
            if spent() {
                return Err(FunctionAttrsError::TooComplex);
            }
            let failing: Vec<bool> = errors.iter().map(|&err| err > tol).collect();
            let split = failing.iter().filter(|&&f| f).count();
            let projected: f64 = errors
                .iter()
                .map(|&err| if err > tol { (err / tol).powf(0.25).ceil().max(2.0) } else { 1.0 })
                .sum();
            if projected > TABLE_PROJECTION_SAFETY * MAX_TABLE_INTERVALS as f64 {
                return Err(FunctionAttrsError::TooComplex);
            }
            if split == 0 {
                let mut s = Vec::with_capacity(x.len());
                s.push(0.0);
                for l in &len {
                    s.push(s.last().expect("starts with 0") + l);
                }
                return Ok(Self { x, s, slope });
            }
            if len.len() + split > MAX_TABLE_INTERVALS {
                return Err(FunctionAttrsError::TooComplex);
            }
            let cap = len.len() + split;
            let (mut nx, mut nlen, mut nslope, mut npassed) =
                (Vec::with_capacity(cap + 1), Vec::with_capacity(cap), Vec::with_capacity(cap + 1), Vec::with_capacity(cap));
            for i in 0..len.len() {
                nx.push(x[i]);
                nslope.push(slope[i]);
                if failing[i] {
                    let m = 0.5 * (x[i] + x[i + 1]);
                    nx.push(m);
                    nslope.push(1.0 / speed(m));
                    nlen.push(integrate(&speed, x[i], m, qtol));
                    nlen.push(integrate(&speed, m, x[i + 1], qtol));
                    npassed.extend([false, false]);
                } else {
                    nlen.push(len[i]);
                    npassed.push(true);
                }
            }
            if spent() {
                return Err(FunctionAttrsError::TooComplex);
            }
            nx.push(*x.last().expect("at least two nodes"));
            nslope.push(*slope.last().expect("at least two nodes"));
            (x, len, slope, passed) = (nx, nlen, nslope, npassed);
        }
    }

    fn length(&self) -> f64 {
        *self.s.last().expect("at least two nodes")
    }

    /// Interval index containing arc length `s` (clamped).
    fn interval_of(&self, s: f64) -> usize {
        let i = self.s.partition_point(|&si| si <= s);
        i.clamp(1, self.s.len() - 1) - 1
    }

    /// `x(s)` and `dx/ds` on interval `i` (Hermite, clamped to the interval).
    fn inverse_in(&self, i: usize, s: f64) -> (f64, f64) {
        let (s0, s1) = (self.s[i], self.s[i + 1]);
        let (x0, x1) = (self.x[i], self.x[i + 1]);
        let h = s1 - s0;
        if h <= 0.0 {
            return (x0, 0.0);
        }
        let u = ((s - s0) / h).clamp(0.0, 1.0);
        let (m0, m1) = (self.slope[i] * h, self.slope[i + 1] * h);
        let (u2, u3) = (u * u, u * u * u);
        let x = (2.0 * u3 - 3.0 * u2 + 1.0) * x0
            + (u3 - 2.0 * u2 + u) * m0
            + (-2.0 * u3 + 3.0 * u2) * x1
            + (u3 - u2) * m1;
        let dx = ((6.0 * u2 - 6.0 * u) * x0 + (3.0 * u2 - 4.0 * u + 1.0) * m0 + (-6.0 * u2 + 6.0 * u) * x1
            + (3.0 * u2 - 2.0 * u) * m1)
            / h;
        (x.clamp(x0, x1), dx)
    }

    fn inverse(&self, s: f64) -> (f64, f64) {
        let s = s.clamp(0.0, self.length());
        self.inverse_in(self.interval_of(s), s)
    }
}

/// `w` rotated by heading `θ`.
fn rotate(th: HeadingAngle, w: Vec2) -> Vec2 {
    let (s, c) = th.radians().sin_cos();
    Vec2::new(c * w.x - s * w.y, s * w.x + c * w.y)
}

impl Function {
    /// Curve point at normalized `t` (exact on the graph).
    fn at(t: f64, (o, th): (Point2, HeadingAngle), a: &FunctionAttrs) -> Point2 {
        o + rotate(th, a.local_at(t).1)
    }

    /// Chart derivative `dγ/dt` at normalized `t`.
    fn tangent(t: f64, (_, th): (Point2, HeadingAngle), a: &FunctionAttrs) -> Vec2 {
        rotate(th, a.local_at(t).2)
    }

    /// Rotated local point of `f` (for points and their θ column).
    fn rotated_point(f: PointFormula, th: HeadingAngle, a: &FunctionAttrs) -> Vec2 {
        let local = match f {
            PointFormula::OnCurve {
                param: CurveParam::Abscissa(x),
            } => a.local_at_abscissa(x).0,
            PointFormula::OnCurve { param } => a.local_at(param.chart_t()).1,
            PointFormula::Start => a.local_point(a.range.0),
            PointFormula::End => a.local_point(a.range.1),
            PointFormula::Origin => Vec2::zero(),
            PointFormula::Center => unreachable!("function curve has no circular carrier"),
        };
        rotate(th, local)
    }
}

impl Geometry for Function {
    const KIND: GeometryKind = GeometryKind::Function;
    const NAME_PREFIX: &'static str = "fn";
    type Attrs = FunctionAttrs;

    fn attrs_of(a: &KindAttrs) -> &FunctionAttrs {
        match a {
            KindAttrs::Function(a) => a,
            other => unreachable!("function curve with {other:?}"),
        }
    }

    fn default_attrs() -> KindAttrs {
        KindAttrs::Function(FunctionAttrs::default())
    }

    fn shape(c: Self::Init, a: &Self::Attrs, tolerance: f64) -> Vec<ShapePiece> {
        let point = |t: f64| Self::at(t, c, a);
        let tangent = |t: f64| Self::tangent(t, c, a);
        // `capped` (the piece cap stopped the refinement) has no channel to the
        // export yet (#104); the error is spread evenly over the curve (#100).
        hermite_shape(point, tangent, &[0.0, 1.0], tolerance, sampled_deviation(point)).pieces
    }

    fn curvature(t: f64, _c: Self::Init, a: &Self::Attrs) -> f64 {
        a.curvature_at(a.local_at(t).0)
    }

    fn piece_bound(t0: f64, t1: f64, _c: Self::Init, a: &Self::Attrs) -> PieceBound {
        let l = a.length();
        let (xa, _) = a.table.inverse(t0 * l);
        let (xb, _) = a.table.inverse(t1 * l);
        // Widen to the enclosing table nodes: the inverse's error stays inside.
        let ia = a.table.interval_of((t0 * l).clamp(0.0, l));
        let ib = a.table.interval_of((t1 * l).clamp(0.0, l)) + 1;
        let (wa, wb) = (a.table.x[ia].min(xa), a.table.x[ib].max(xb));
        PieceBound {
            length: a.arc_length(xa, xb) * (1.0 + PIECE_LENGTH_MARGIN),
            curvature: a.curvature_bound(wa, wb),
        }
    }

    fn point(f: PointFormula, (o, th): Self::Init, a: &Self::Attrs) -> Point2 {
        o + Self::rotated_point(f, th, a)
    }

    fn point_jacobian(f: PointFormula, (_, th): Self::Init, a: &Self::Attrs, j: &mut DenseMatrix<'_>, scale: f64) {
        match f {
            PointFormula::Origin => j.set_scaled(scale, [[1.0, 0.0], [0.0, 1.0]]),
            _ => {
                // ∂(O + R(θ)p)/∂(Ox, Oy, θ) = [I | R′(θ)p], R′(θ)p = (−q_y, q_x).
                let q = Self::rotated_point(f, th, a);
                j.set_scaled(scale, [[1.0, 0.0, -q.y], [0.0, 1.0, q.x]]);
            }
        }
    }

    fn point_params(f: PointFormula) -> Range<usize> {
        match f {
            PointFormula::Origin => Self::origin.range(),
            PointFormula::Center => unreachable!("function curve has no circular carrier"),
            _ => Self::origin.span(Self::heading),
        }
    }

    fn vec(f: VecFormula, (o, th): Self::Init, a: &Self::Attrs) -> Vec2 {
        match f {
            VecFormula::TangentAt {
                param: CurveParam::Abscissa(x),
            } => rotate(th, a.local_at_abscissa(x).1),
            VecFormula::NormalAt {
                param: CurveParam::Abscissa(x),
            } => rotate(th, a.local_at_abscissa(x).1).left_normal(),
            VecFormula::TangentAt { param } => Self::tangent(param.chart_t(), (o, th), a),
            VecFormula::NormalAt { param } => Self::tangent(param.chart_t(), (o, th), a).left_normal(),
            VecFormula::Direction => rotate(th, Vec2::new(1.0, 0.0)),
        }
    }

    fn vec_jacobian(f: VecFormula, c: Self::Init, a: &Self::Attrs, j: &mut DenseMatrix<'_>, scale: f64) {
        // Every vector is R(θ)w with fixed w: ∂/∂θ = (−v_y, v_x).
        let v = Self::vec(f, c, a);
        j.set_scaled(scale, [[-v.y], [v.x]]);
    }

    fn vec_params(_f: VecFormula) -> Range<usize> {
        Self::heading.range()
    }

    fn length(f: LengthFormula, _c: Self::Init, a: &Self::Attrs) -> Length {
        match f {
            LengthFormula::Length => Length::new(a.length()),
            LengthFormula::Radius => unreachable!("function curve has no radius"),
        }
    }

    fn length_jacobian(_f: LengthFormula, _c: Self::Init, _a: &Self::Attrs, _j: &mut DenseMatrix<'_>, _scale: f64) {
        // The length depends on attributes only: no State columns.
    }

    fn length_params(_f: LengthFormula) -> Range<usize> {
        let start = Self::origin.range().start;
        start..start
    }
}

#[cfg(test)]
#[path = "tests_function.rs"]
mod tests_function;
