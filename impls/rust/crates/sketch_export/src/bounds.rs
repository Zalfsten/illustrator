//! Exact bounds of a scene: the smallest axis-aligned box around every drawn
//! shape (drawing-export: Export bounds are tight). Dashed strokes count as
//! undashed.

use std::f64::consts::FRAC_PI_2;

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
                for t in extrema_cubic(from, c1, c2, p) {
                    acc.add(cubic(from, c1, c2, p, t));
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
            PathSeg::Quad(c, p) => widened_box(acc, &[from, c, p], h),
            PathSeg::Cubic(c1, c2, p) => widened_box(acc, &[from, c1, c2, p], h),
            PathSeg::Move(_) | PathSeg::Close => {}
        }
    }
}

/// Control-polygon box widened by `h` (conservative for curved strokes).
fn widened_box(acc: &mut Acc, points: &[Point2], h: f64) {
    for p in points {
        for d in [Vec2::new(h, h), Vec2::new(-h, -h)] {
            acc.add(*p + d);
        }
    }
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

fn cubic(p0: Point2, c1: Point2, c2: Point2, p1: Point2, t: f64) -> Point2 {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    Point2::new(
        a * p0.x + b * c1.x + c * c2.x + d * p1.x,
        a * p0.y + b * c1.y + c * c2.y + d * p1.y,
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

/// Parameters in (0, 1) where a coordinate of a cubic is extreme.
fn extrema_cubic(p0: Point2, c1: Point2, c2: Point2, p1: Point2) -> Vec<f64> {
    let mut out = Vec::new();
    for (a, b, c, d) in [(p0.x, c1.x, c2.x, p1.x), (p0.y, c1.y, c2.y, p1.y)] {
        // Derivative / 3: A t² + B t + C.
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
                let s = disc.sqrt();
                out.push((-qb + s) / (2.0 * qa));
                out.push((-qb - s) / (2.0 * qa));
            }
        }
    }
    out.retain(|&t| inner(t));
    out
}

#[cfg(test)]
#[path = "tests_bounds.rs"]
mod tests_bounds;
