use super::*;
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle, Vec2};

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
fn state_atom_heading_turn_and_length_round_trip() {
    let h = HeadingAngle::new(1.25);
    let mut buf = [0.0; HeadingAngle::DOF_COUNT];
    h.write_params(&mut buf);
    assert_eq!(buf, [1.25]);
    assert_eq!(HeadingAngle::read_params(&buf), h);

    let t = TurnAngle::new(-0.75);
    let mut buf_t = [0.0; TurnAngle::DOF_COUNT];
    t.write_params(&mut buf_t);
    assert_eq!(buf_t, [-0.75]);
    assert_eq!(TurnAngle::read_params(&buf_t), t);

    let len = Length::new(-3.5);
    let mut buf1 = [0.0; Length::DOF_COUNT];
    len.write_params(&mut buf1);
    assert_eq!(buf1, [-3.5]);
    assert_eq!(Length::read_params(&buf1), len);
}
