//! Shared benchmark / test scene: open staircase chain of `n` segments.
//!
//! Congruence joints, VectorsOrthogonal between neighbours, VectorsParallel every 5th pair,
//! EqualLength every 3rd pair, FixPoint on the first start; small deterministic
//! perturbation so solves start near a consistent configuration.

use constraint_runtime::{
    Congruence, CurveExprs, Document, EndpointExprs, EqualLength, FixPoint, GeometryId, GeometryRole,
    LineExprs, Point2, Segment, VectorsOrthogonal, VectorsParallel,
};

/// Deterministic pseudo-random offset in [-0.05, 0.05].
fn jitter(k: usize) -> f64 {
    let x = (k as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((x >> 33) as f64 / (1u64 << 31) as f64 - 0.5) * 0.1
}

pub fn staircase(n: usize) -> (Document, Vec<GeometryId>) {
    let mut doc = Document::new();
    let mut ids = Vec::with_capacity(n);
    let mut at = Point2::new(0.0, 0.0);
    for i in 0..n {
        let next = if i % 2 == 0 {
            Point2::new(at.x + 1.0, at.y)
        } else {
            Point2::new(at.x, at.y + 1.0)
        };
        let a = Point2::new(at.x + jitter(4 * i), at.y + jitter(4 * i + 1));
        let b = Point2::new(next.x + jitter(4 * i + 2), next.y + jitter(4 * i + 3));
        ids.push(doc.add_geometry::<Segment>(GeometryRole::Result, (a, b)));
        at = next;
    }
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(ids[0]),
        target: Point2::new(0.0, 0.0),
    });
    for i in 0..n - 1 {
        let (s, t) = (ids[i], ids[i + 1]);
        doc.add_constraint(Congruence {
            a: Segment::end_expr(s),
            b: Segment::start_expr(t),
        });
        doc.add_constraint(VectorsOrthogonal {
            a: Segment::direction_expr(s),
            b: Segment::direction_expr(t),
        });
        if i % 3 == 0 {
            doc.add_constraint(EqualLength {
                a: Segment::length_expr(s),
                b: Segment::length_expr(t),
            });
        }
        if i % 5 == 0 && i + 2 < n {
            doc.add_constraint(VectorsParallel {
                a: Segment::direction_expr(s),
                b: Segment::direction_expr(ids[i + 2]),
            });
        }
    }
    (doc, ids)
}

