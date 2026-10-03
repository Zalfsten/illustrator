use super::*;
use crate::geom::{Angle, Length, Point2, Vec2};

#[test]
fn state_atom_point_and_vec_round_trip_xy_order() {
    let p = Point2::new(1.5, -2.25);
    let mut buf = [0.0; Point2::DOF_COUNT];
    p.write_params(&mut buf);
    assert_eq!(buf, [1.5, -2.25]);
    assert_eq!(Point2::read_params(&buf), p);

    let v = Vec2::new(3.0, 4.0);
    v.write_params(&mut buf);
    assert_eq!(buf, [3.0, 4.0]);
    assert_eq!(Vec2::read_params(&buf), v);
}

#[test]
fn state_atom_angle_and_length_round_trip() {
    let a = Angle::new(1.25);
    let mut buf = [0.0; Angle::DOF_COUNT];
    a.write_params(&mut buf);
    assert_eq!(buf, [1.25]);
    assert_eq!(Angle::read_params(&buf), a);

    let len = Length::new(-3.5);
    let mut buf1 = [0.0; Length::DOF_COUNT];
    len.write_params(&mut buf1);
    assert_eq!(buf1, [-3.5]);
    assert_eq!(Length::read_params(&buf1), len);
}
