use super::*;
use crate::geom::{Point2, Vec2};

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
