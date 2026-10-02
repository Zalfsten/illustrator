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
