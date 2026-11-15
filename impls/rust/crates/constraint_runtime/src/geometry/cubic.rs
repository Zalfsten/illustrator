//! Cubic Bézier helpers shared by shapes and exports: evaluation,
//! derivatives, axis extrema, de Casteljau splitting, arc length and its
//! inverse.
//!
//! A cubic is given by its four points `[start, c1, c2, end]` (locations);
//! derivatives are free vectors. `u ∈ [0, 1]` is the Bézier parameter, not an
//! arc-length fraction.

use super::shape::ShapePiece;
use crate::geom::{Point2, Vec2};

/// The four points `[start, c1, c2, end]` of a cubic Bézier.
pub type CubicPoints = [Point2; 4];

/// Relative accuracy of [`arc_length`] and [`param_at_length`].
const LENGTH_REL_TOL: f64 = 1e-13;
/// Deepest halving of the adaptive quadrature.
const MAX_QUADRATURE_DEPTH: u32 = 20;

/// 8-point Gauss–Legendre nodes on [−1, 1] (positive half) and weights.
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

fn lerp(a: Point2, b: Point2, u: f64) -> Point2 {
    a + (b - a) * u
}

/// Point at parameter `u` (de Casteljau, so the result is an affine
/// combination of the control points); exactly `start` / `end` at 0 / 1.
pub fn point(b: &CubicPoints, u: f64) -> Point2 {
    let [p0, p1, p2, p3] = *b;
    // `a + (b − a)·1` need not round to `b`; ends must be exact (shape
    // pieces join exactly, geometry-ir).
    if u <= 0.0 {
        return p0;
    }
    if u >= 1.0 {
        return p3;
    }
    let (a, m, c) = (lerp(p0, p1, u), lerp(p1, p2, u), lerp(p2, p3, u));
    lerp(lerp(a, m, u), lerp(m, c, u), u)
}

/// First derivative `B′(u)`.
pub fn derivative(b: &CubicPoints, u: f64) -> Vec2 {
    let [p0, p1, p2, p3] = *b;
    let v = 1.0 - u;
    ((p1 - p0) * (v * v) + (p2 - p1) * (2.0 * v * u) + (p3 - p2) * (u * u)) * 3.0
}

/// Second derivative `B″(u)`.
pub fn second_derivative(b: &CubicPoints, u: f64) -> Vec2 {
    let [p0, p1, p2, p3] = *b;
    let d0 = (p2 - p1) - (p1 - p0);
    let d1 = (p3 - p2) - (p2 - p1);
    (d0 * (1.0 - u) + d1 * u) * 6.0
}

/// Signed curvature at `u` (positive turning left); 0 where `B′` vanishes.
pub fn curvature(b: &CubicPoints, u: f64) -> f64 {
    let d1 = derivative(b, u);
    let speed = d1.length();
    if speed == 0.0 {
        return 0.0;
    }
    d1.cross(second_derivative(b, u)) / (speed * speed * speed)
}

/// Parameters in (0, 1) where the x or the y coordinate is extreme.
pub fn axis_extrema(b: &CubicPoints) -> Vec<f64> {
    let [p0, p1, p2, p3] = *b;
    let mut out = Vec::new();
    for (a, b, c, d) in [(p0.x, p1.x, p2.x, p3.x), (p0.y, p1.y, p2.y, p3.y)] {
        // Derivative / 3: A u² + B u + C.
        let qa = -a + 3.0 * b - 3.0 * c + d;
        let qb = 2.0 * (a - 2.0 * b + c);
        let qc = b - a;
        if qa.abs() < 1e-12 * (qb.abs() + qc.abs()).max(1e-300) {
            if qb != 0.0 {
                out.push(-qc / qb);
            }
        } else {
            let disc = qb * qb - 4.0 * qa * qc;
            if disc >= 0.0 {
                // Stable form: no cancellation between −qb and ±√disc.
                let q = -0.5 * (qb + qb.signum() * disc.sqrt());
                out.push(q / qa);
                if q != 0.0 {
                    out.push(qc / q);
                }
            }
        }
    }
    out.retain(|&u| u > 0.0 && u < 1.0);
    out
}

/// The two halves of `b` at parameter `u` (de Casteljau).
pub fn split(b: &CubicPoints, u: f64) -> (CubicPoints, CubicPoints) {
    let [p0, p1, p2, p3] = *b;
    let (a, m, c) = (lerp(p0, p1, u), lerp(p1, p2, u), lerp(p2, p3, u));
    let (d, e) = (lerp(a, m, u), lerp(m, c, u));
    let f = lerp(d, e, u);
    ([p0, a, d, f], [f, e, c, p3])
}

/// The part of `b` between parameters `u0 ≤ u1`, as a cubic of its own.
pub fn sub(b: &CubicPoints, u0: f64, u1: f64) -> CubicPoints {
    if u1 <= u0 {
        return [point(b, u0); 4];
    }
    let right = if u0 <= 0.0 { *b } else { split(b, u0).1 };
    if u1 >= 1.0 {
        return right;
    }
    let u0 = u0.max(0.0);
    split(&right, (u1 - u0) / (1.0 - u0)).0
}

/// Gauss–Legendre estimate of the arc length on `[u0, u1]`.
fn gauss_length(b: &CubicPoints, u0: f64, u1: f64) -> f64 {
    let (mid, half) = (0.5 * (u0 + u1), 0.5 * (u1 - u0));
    let mut sum = 0.0;
    for (x, w) in GL_NODES.iter().zip(GL_WEIGHTS) {
        sum += w * (derivative(b, mid - half * x).length() + derivative(b, mid + half * x).length());
    }
    sum * half
}

fn adaptive_length(b: &CubicPoints, u0: f64, u1: f64, whole: f64, abs_tol: f64, depth: u32) -> f64 {
    let m = 0.5 * (u0 + u1);
    let (l, r) = (gauss_length(b, u0, m), gauss_length(b, m, u1));
    if depth >= MAX_QUADRATURE_DEPTH || (l + r - whole).abs() <= abs_tol {
        return l + r;
    }
    adaptive_length(b, u0, m, l, 0.5 * abs_tol, depth + 1) + adaptive_length(b, m, u1, r, 0.5 * abs_tol, depth + 1)
}

/// Arc length of `b` between parameters `u0 ≤ u1` (adaptive Gauss–Legendre,
/// relative accuracy about 1e-13).
pub fn arc_length(b: &CubicPoints, u0: f64, u1: f64) -> f64 {
    // The tolerance refers to the whole curve's scale, so tiny sub-intervals
    // do not demand more relative accuracy than the total needs.
    arc_length_at_scale(b, u0, u1, gauss_length(b, 0.0, 1.0))
}

/// [`arc_length`] with the whole curve's length estimate `scale` given.
fn arc_length_at_scale(b: &CubicPoints, u0: f64, u1: f64, scale: f64) -> f64 {
    if u1 <= u0 {
        return 0.0;
    }
    let whole = gauss_length(b, u0, u1);
    adaptive_length(b, u0, u1, whole, LENGTH_REL_TOL * scale.max(whole), 0)
}

/// Parameter `u` at which the arc length from the start is `length`
/// (clamped to `[0, 1]`); safeguarded Newton iteration.
pub fn param_at_length(b: &CubicPoints, length: f64) -> f64 {
    let total = arc_length(b, 0.0, 1.0);
    if length <= 0.0 || total == 0.0 {
        return 0.0;
    }
    if length >= total {
        return 1.0;
    }
    let tol = LENGTH_REL_TOL * total;
    let scale = gauss_length(b, 0.0, 1.0);
    let (mut lo, mut hi) = (0.0, 1.0);
    let mut u = length / total;
    for _ in 0..100 {
        let f = arc_length_at_scale(b, 0.0, u, scale) - length;
        if f.abs() <= tol {
            break;
        }
        if f > 0.0 {
            hi = u;
        } else {
            lo = u;
        }
        if hi - lo <= f64::EPSILON {
            break;
        }
        let speed = derivative(b, u).length();
        let newton = u - f / speed;
        u = if speed > 0.0 && newton > lo && newton < hi { newton } else { 0.5 * (lo + hi) };
    }
    u
}

/// Most pieces [`hermite_shape`] produces; beyond it, pieces are accepted
/// without further halving (like the display Mesh's edge cap).
pub const MAX_SHAPE_PIECES: usize = 4096;
/// Equal pieces each knot interval starts with, so a sampled deviation
/// estimate never judges one huge piece.
const INITIAL_PIECES_PER_INTERVAL: usize = 4;
/// Matched samples per piece in [`sampled_deviation`]: `k/8` for `k = 1…7`.
const DEVIATION_SAMPLES: usize = 8;
/// Factor on the largest sampled distance in [`sampled_deviation`].
const DEVIATION_SAFETY: f64 = 2.0;

/// Cubic Hermite piece of the curve `point` on `[t0, t1]` in Bézier form:
/// exact end points, end tangents `tangent(t) = dγ/dt`.
pub fn hermite_piece(
    point: impl Fn(f64) -> Point2,
    tangent: impl Fn(f64) -> Vec2,
    t0: f64,
    t1: f64,
) -> ShapePiece {
    let h = t1 - t0;
    let (start, end) = (point(t0), point(t1));
    ShapePiece::Cubic {
        start,
        c1: start + tangent(t0) * (h / 3.0),
        c2: end - tangent(t1) * (h / 3.0),
        end,
    }
}

/// Result of [`hermite_shape`].
#[derive(Clone, Debug, PartialEq)]
pub struct HermiteShape {
    /// The pieces, in curve order.
    pub pieces: Vec<ShapePiece>,
    /// Whether [`MAX_SHAPE_PIECES`] stopped the refinement while some piece
    /// still exceeded the tolerance; the remaining error is then spread over
    /// the curve (largest first), not left in one part.
    pub capped: bool,
}

/// A piece waiting for refinement, ordered by its deviation (ties: earlier
/// pieces first, so the result is deterministic).
struct Pending {
    deviation: f64,
    slot: usize,
}

impl PartialEq for Pending {
    fn eq(&self, o: &Self) -> bool {
        self.cmp(o) == std::cmp::Ordering::Equal
    }
}
impl Eq for Pending {}
impl PartialOrd for Pending {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Pending {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.deviation.total_cmp(&o.deviation).then(o.slot.cmp(&self.slot))
    }
}

/// Cubic pieces describing the curve `point` (`γ(t)`, `t ∈ [0, 1]`, with
/// derivative `tangent`) within `tolerance` according to `deviation`
/// (geometry-ir: Shapes without closed-form pieces stay within the
/// tolerance).
///
/// `knots` are ascending from 0 to 1 and always become joints (for example
/// branch points). Each knot interval starts as 4 equal Hermite pieces; the
/// piece with the largest `deviation(t0, t1, &piece)` above `tolerance` is
/// halved next, until no piece exceeds it or [`MAX_SHAPE_PIECES`] pieces
/// exist ([`HermiteShape::capped`]). Refining by priority keeps the error
/// even when the cap is reached; depth-first halving spent the whole budget
/// on the start of the curve and left its end as coarse initial pieces.
/// Joints and ends are exact curve points, and the tangent is continuous
/// across joints.
///
/// `deviation` is pluggable: [`sampled_deviation`] is an estimate; a provable
/// bound can replace it without changing callers (#99).
pub fn hermite_shape(
    point: impl Fn(f64) -> Point2,
    tangent: impl Fn(f64) -> Vec2,
    knots: &[f64],
    tolerance: f64,
    deviation: impl Fn(f64, f64, &ShapePiece) -> f64,
) -> HermiteShape {
    debug_assert!(knots.len() >= 2 && knots[0] == 0.0 && knots[knots.len() - 1] == 1.0);
    debug_assert!(knots.windows(2).all(|w| w[0] < w[1]), "knots must ascend: {knots:?}");
    let n = INITIAL_PIECES_PER_INTERVAL;
    // (t0, t1, piece) per slot; halving replaces a slot and appends one.
    let mut slots: Vec<(f64, f64, ShapePiece)> = Vec::with_capacity((knots.len() - 1) * n);
    let mut queue = std::collections::BinaryHeap::new();
    type Slots = Vec<(f64, f64, ShapePiece)>;
    let add = |a: f64, b: f64, slots: &mut Slots, queue: &mut std::collections::BinaryHeap<Pending>, slot: Option<usize>| {
        let piece = hermite_piece(&point, &tangent, a, b);
        let d = deviation(a, b, &piece);
        let slot = match slot {
            Some(i) => {
                slots[i] = (a, b, piece);
                i
            }
            None => {
                slots.push((a, b, piece));
                slots.len() - 1
            }
        };
        let m = 0.5 * (a + b);
        if d > tolerance && m > a && m < b {
            queue.push(Pending { deviation: d, slot });
        }
    };
    for w in knots.windows(2) {
        let (k0, k1) = (w[0], w[1]);
        for i in 0..n {
            let a = if i == 0 { k0 } else { k0 + (k1 - k0) * i as f64 / n as f64 };
            let b = if i + 1 == n { k1 } else { k0 + (k1 - k0) * (i + 1) as f64 / n as f64 };
            add(a, b, &mut slots, &mut queue, None);
        }
    }
    while slots.len() < MAX_SHAPE_PIECES {
        let Some(Pending { slot, .. }) = queue.pop() else {
            break;
        };
        let (a, b, _) = slots[slot];
        let m = 0.5 * (a + b);
        add(a, m, &mut slots, &mut queue, Some(slot));
        add(m, b, &mut slots, &mut queue, None);
    }
    let capped = !queue.is_empty();
    slots.sort_by(|x, y| x.0.total_cmp(&y.0));
    HermiteShape {
        pieces: slots.into_iter().map(|(_, _, piece)| piece).collect(),
        capped,
    }
}

/// Deviation estimate for [`hermite_shape`]: the largest distance between
/// `point(t0 + (k/8)·h)` and the piece at Bézier parameter `k/8`
/// (`k = 1…7`), times a safety factor 2. Distances at matched parameters are
/// at least the geometric distances at those samples; between samples this is
/// an estimate, not a bound (#99).
pub fn sampled_deviation(point: impl Fn(f64) -> Point2) -> impl Fn(f64, f64, &ShapePiece) -> f64 {
    move |t0, t1, piece| {
        let ShapePiece::Cubic { start, c1, c2, end } = *piece else {
            return 0.0;
        };
        let b = [start, c1, c2, end];
        let worst = (1..DEVIATION_SAMPLES)
            .map(|k| {
                let u = k as f64 / DEVIATION_SAMPLES as f64;
                (self::point(&b, u) - point(t0 + u * (t1 - t0))).length()
            })
            .fold(0.0, f64::max);
        DEVIATION_SAFETY * worst
    }
}

#[cfg(test)]
#[path = "tests_cubic.rs"]
mod tests_cubic;
