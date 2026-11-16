use super::*;

#[test]
fn point_minus_point_is_vec() {
    let a = Point2::new(1.0, 2.0);
    let b = Point2::new(4.0, 6.0);
    let v: Vec2 = b - a;
    assert_eq!(v, Vec2::new(3.0, 4.0));
    assert_eq!(a + v, b);
    assert_eq!(v.cross(Vec2::new(0.0, 1.0)), 3.0);
    assert_eq!(v.dot(Vec2::new(0.0, 1.0)), 4.0);
}

#[test]
fn heading_minus_heading_is_turn() {
    let h0 = HeadingAngle::new(0.5);
    let h1 = HeadingAngle::new(2.0);
    let t: TurnAngle = h1 - h0;
    assert_eq!(t.radians(), 1.5);
    assert_eq!(h0 + t, h1);
    assert_eq!(h1 - t, h0);
    assert_eq!((t + TurnAngle::new(0.25)).radians(), 1.75);
    assert_eq!((-t).radians(), -1.5);
}

#[test]
fn chart_t_serves_normalized_and_uniform() {
    assert_eq!(CurveParam::Normalized(0.25).chart_t(), 0.25);
    assert_eq!(CurveParam::Uniform(0.75).chart_t(), 0.75);
}

#[test]
#[should_panic(expected = "abscissa")]
fn chart_t_rejects_abscissa() {
    let _ = CurveParam::Abscissa(1.0).chart_t();
}
