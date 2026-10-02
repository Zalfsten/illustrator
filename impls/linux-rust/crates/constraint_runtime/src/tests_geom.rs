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
fn point_jacobian_endpoint_identity_is_i2() {
    let j = PointJacobian::endpoint_identity();
    assert_eq!(j.n(), 2);
    assert_eq!(j.dx(), &[1.0, 0.0]);
    assert_eq!(j.dy(), &[0.0, 1.0]);
}

#[test]
fn point_jacobian_zeros_respects_runtime_width() {
    let j = PointJacobian::zeros(4);
    assert_eq!(j.n(), 4);
    assert_eq!(j.dx(), &[0.0; 4]);
    assert_eq!(j.dy(), &[0.0; 4]);
}
