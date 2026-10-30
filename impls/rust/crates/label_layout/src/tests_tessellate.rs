use constraint_runtime::Point2;

use super::*;

fn area(t: &Triangles) -> f64 {
    t.indices
        .chunks(3)
        .map(|tri| {
            let [a, b, c] = [0, 1, 2].map(|i| t.vertices[tri[i] as usize]);
            let cross = (b[0] - a[0]) as f64 * (c[1] - a[1]) as f64
                - (b[1] - a[1]) as f64 * (c[0] - a[0]) as f64;
            0.5 * cross.abs()
        })
        .sum()
}

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

#[test]
fn filled_unit_square() {
    let square = Shape {
        contours: vec![Contour(vec![
            Segment::MoveTo(p(0.0, 0.0)),
            Segment::LineTo(p(1.0, 0.0)),
            Segment::LineTo(p(1.0, 1.0)),
            Segment::LineTo(p(0.0, 1.0)),
            Segment::Close,
        ])],
        paint: Paint::Fill,
    };
    let t = tessellate(&[square]);
    assert_eq!(t.indices.len(), 6);
    assert!((area(&t) - 1.0).abs() < 1e-6);
}

#[test]
fn stroked_line_has_length_times_width() {
    let bar = Shape {
        contours: vec![Contour(vec![
            Segment::MoveTo(p(0.0, 0.0)),
            Segment::LineTo(p(4.0, 0.0)),
        ])],
        paint: Paint::Stroke { width: 0.5 },
    };
    let t = tessellate(&[bar]);
    assert!((area(&t) - 2.0).abs() < 1e-6, "{}", area(&t));
}

#[test]
fn no_shapes_no_triangles() {
    assert_eq!(tessellate(&[]), Triangles::default());
}
