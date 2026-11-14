//! Exact bounds of a scene: the smallest axis-aligned box around every drawn
//! shape (drawing-export: Export bounds are tight), curved strokes measured
//! from the curve, not from its control points. Dashed strokes count as
//! undashed.

use std::f64::consts::FRAC_PI_2;

use constraint_runtime::geometry::cubic::{self, CubicPoints};
use constraint_runtime::{Point2, Vec2};

use crate::scene::{on_circle, Part, PathSeg, Scene};

/// Axis-aligned box in world coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Point2,
    pub max: Point2,
}

impl Bounds {
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
}

#[derive(Default)]
struct Acc(Option<Bounds>);

impl Acc {
    fn add(&mut self, p: Point2) {
        self.0 = Some(match self.0 {
            None => Bounds { min: p, max: p },
            Some(b) => Bounds {
                min: Point2::new(b.min.x.min(p.x), b.min.y.min(p.y)),
                max: Point2::new(b.max.x.max(p.x), b.max.y.max(p.y)),
            },
        });
    }
}

/// Bounds of everything `scene` draws; `None` if it draws nothing.
pub fn bounds(scene: &Scene) -> Option<Bounds> {
    let mut acc = Acc::default();
    for part in scene.objects.iter().flat_map(|o| &o.parts) {
        match part {
            Part::Stroke { path, width, .. } => stroked(&mut acc, path, *width),
            Part::Fill { contours, .. } => contours.iter().for_each(|c| filled(&mut acc, c)),
            Part::Outline { contours, width, .. } => contours.iter().for_each(|c| stroked(&mut acc, c, *width)),
        }
    }
    acc.0.filter(|b| b.width() > 0.0 || b.height() > 0.0)
}

/// Each command with the point it starts from.
fn with_starts(path: &[PathSeg]) -> impl Iterator<Item = (Point2, PathSeg)> + '_ {
    let mut current = None;
    let mut first = None;
    path.iter().filter_map(move |&seg| {
        let from = current;
        match seg {
            PathSeg::Move(p) => {
                first = Some(p);
                current = Some(p);
                None
            }
            PathSeg::Close => {
                current = first;
                Some((from?, PathSeg::Line(first?)))
            }
            _ => {
                current = seg.end();
                Some((from?, seg))
            }
        }
    })
}

/// Region a fill covers: its outline, curve extrema included.
fn filled(acc: &mut Acc, path: &[PathSeg]) {
    for (from, seg) in with_starts(path) {
        acc.add(from);
        match seg {
            PathSeg::Quad(c, p) => {
                for t in extrema_quad(from, c, p) {
                    acc.add(quad(from, c, p, t));
                }
            }
            PathSeg::Cubic(c1, c2, p) => {
                let b = [from, c1, c2, p];
                for t in cubic::axis_extrema(&b) {
                    acc.add(cubic::point(&b, t));
                }
            }
            PathSeg::Arc {
                center,
                radius,
                start,
                sweep,
            } => {
                for a in axis_headings(start, sweep) {
                    acc.add(on_circle(center, radius, a));
                }
            }
            _ => {}
        }
        if let Some(p) = seg.end() {
            acc.add(p);
        }
    }
}

/// Region a stroke of `width` with flat ends covers.
fn stroked(acc: &mut Acc, path: &[PathSeg], width: f64) {
    let h = 0.5 * width;
    for (from, seg) in with_starts(path) {
        match seg {
            PathSeg::Line(to) => {
                let d = to - from;
                let len = d.length();
                if len == 0.0 {
                    continue;
                }
                let n = d.left_normal() * (h / len);
                for q in [from + n, from - n, to + n, to - n] {
                    acc.add(q);
                }
            }
            PathSeg::Arc {
                center,
                radius,
                start,
                sweep,
            } => {
                // Annular sector between the signed radii r − h and r + h.
                let ends = [start, start + sweep];
                for r in [radius - h, radius + h] {
                    for a in ends.into_iter().chain(axis_headings(start, sweep)) {
                        acc.add(on_circle(center, r, a));
                    }
                }
            }
            PathSeg::Quad(c, p) => {
                // Degree elevation: the same curve as a cubic.
                let (c1, c2) = (from + (c - from) * (2.0 / 3.0), p + (c - p) * (2.0 / 3.0));
                stroked_cubic(acc, &[from, c1, c2, p], h);
            }
            PathSeg::Cubic(c1, c2, p) => stroked_cubic(acc, &[from, c1, c2, p], h),
            PathSeg::Move(_) | PathSeg::Close => {}
        }
    }
}

/// Curvature samples deciding whether a cubic stroke may have offset cusps.
const CURVATURE_SAMPLES: usize = 32;
/// Samples bracketing the numeric maximum on such strokes.
const OFFSET_SAMPLES: usize = 64;

/// Unit tangent of `b` at `u`; where `B′` vanishes (a control point on its
/// end), the limit direction from the higher derivatives or the chord.
fn unit_tangent(b: &CubicPoints, u: f64) -> Vec2 {
    let scale = (b[3] - b[0]).length() + (b[1] - b[0]).length() + (b[2] - b[1]).length();
    let tiny = 1e-14 * scale.max(f64::MIN_POSITIVE);
    let d1 = cubic::derivative(b, u);
    let dir = if d1.length() > tiny {
        d1
    } else {
        let d2 = cubic::second_derivative(b, u);
        // At u = 1 the curve arrives from behind: B′(1 − ε) ≈ −ε·B″(1).
        let d2 = if u > 0.5 { d2 * -1.0 } else { d2 };
        if d2.length() > tiny { d2 } else { b[3] - b[0] }
    };
    let len = dir.length();
    if len == 0.0 {
        Vec2::new(1.0, 0.0)
    } else {
        dir * (1.0 / len)
    }
}

/// Point on the stroke's side `side` (±1) at `u`: `B(u) + side·h·n(u)`.
fn offset(b: &CubicPoints, u: f64, h: f64, side: f64) -> Point2 {
    cubic::point(b, u) + unit_tangent(b, u).left_normal() * (side * h)
}

/// Region a butt-ended stroke of half-width `h` along cubic `b` covers.
///
/// The region is the union of the normal segments `B(u) ± h·n(u)`, so its
/// extremes lie on the two offset curves. Along side `s`, a coordinate's
/// derivative is `B′ₓ·(1 − s·h·κ)`: extremes are at the ends, where `B′ₓ = 0`,
/// or where `h·|κ| = 1` (offset cusps). Pieces whose sampled curvature comes
/// near `1/h` additionally get a numeric maximization per direction.
fn stroked_cubic(acc: &mut Acc, b: &CubicPoints, h: f64) {
    let extrema = cubic::axis_extrema(b);
    for side in [1.0, -1.0] {
        for u in [0.0, 1.0].into_iter().chain(extrema.iter().copied()) {
            acc.add(offset(b, u, h, side));
        }
    }
    let max_curvature = (0..=CURVATURE_SAMPLES)
        .map(|k| cubic::curvature(b, k as f64 / CURVATURE_SAMPLES as f64).abs())
        .fold(0.0, f64::max);
    if h * max_curvature < 0.5 {
        return;
    }
    let dirs = [Vec2::new(1.0, 0.0), Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(0.0, -1.0)];
    for side in [1.0, -1.0] {
        for dir in dirs {
            let f = |u: f64| (offset(b, u, h, side) - b[0]).dot(dir);
            let n = OFFSET_SAMPLES;
            let best = (0..=n).max_by(|&i, &j| f(i as f64 / n as f64).total_cmp(&f(j as f64 / n as f64)));
            let i = best.expect("samples exist");
            let lo = (i.saturating_sub(1)) as f64 / n as f64;
            let hi = ((i + 1).min(n)) as f64 / n as f64;
            acc.add(offset(b, golden_max(f, lo, hi), h, side));
        }
    }
}

/// Maximizer of `f` on `[lo, hi]` by golden-section search (unimodal there).
fn golden_max(f: impl Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    const R: f64 = 0.618_033_988_749_894_8;
    let (mut a, mut b) = (hi - R * (hi - lo), lo + R * (hi - lo));
    let (mut fa, mut fb) = (f(a), f(b));
    while hi - lo > 1e-13 {
        if fa < fb {
            lo = a;
            (a, fa) = (b, fb);
            b = lo + R * (hi - lo);
            fb = f(b);
        } else {
            hi = b;
            (b, fb) = (a, fa);
            a = hi - R * (hi - lo);
            fa = f(a);
        }
    }
    let mid = 0.5 * (lo + hi);
    // The bracket's ends may be the maximum (a sample at u = 0 or 1).
    [lo, mid, hi].into_iter().max_by(|&x, &y| f(x).total_cmp(&f(y))).expect("three candidates")
}

/// Multiples of π/2 within the closed heading range of an arc.
fn axis_headings(start: f64, sweep: f64) -> impl Iterator<Item = f64> {
    let (lo, hi) = if sweep < 0.0 { (start + sweep, start) } else { (start, start + sweep) };
    let k0 = (lo / FRAC_PI_2).ceil() as i64;
    let k1 = (hi / FRAC_PI_2).floor() as i64;
    (k0..=k1).map(|k| k as f64 * FRAC_PI_2)
}

fn quad(p0: Point2, c: Point2, p1: Point2, t: f64) -> Point2 {
    let u = 1.0 - t;
    Point2::new(
        u * u * p0.x + 2.0 * u * t * c.x + t * t * p1.x,
        u * u * p0.y + 2.0 * u * t * c.y + t * t * p1.y,
    )
}

fn inner(t: f64) -> bool {
    t > 0.0 && t < 1.0
}

/// Parameters in (0, 1) where a coordinate of a quadratic is extreme.
fn extrema_quad(p0: Point2, c: Point2, p1: Point2) -> Vec<f64> {
    let root = |a: f64, b: f64, d: f64| {
        let den = a - 2.0 * b + d;
        (den != 0.0).then(|| (a - b) / den).filter(|&t| inner(t))
    };
    [root(p0.x, c.x, p1.x), root(p0.y, c.y, p1.y)].into_iter().flatten().collect()
}

#[cfg(test)]
#[path = "tests_bounds.rs"]
mod tests_bounds;
