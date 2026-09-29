use super::*;

fn segment_geom(id: usize, param_offset: usize) -> Geometry {
    Segment::into_geometry(GeometryId(id), GeometryRole::Result, param_offset)
}

#[test]
fn segment_layout_and_connectivity() {
    let offset = 0;
    assert_eq!(Segment::STRIDE, 4);
    assert_eq!(
        Segment::params(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
        [0.0, 0.0, 1.0, 0.0]
    );
    assert_eq!(
        segment_geom(0, offset).connectivity().global_indices,
        vec![0, 1, 2, 3]
    );
    assert_eq!(Segment::all_param_indices(2), [2, 3, 4, 5]);
}

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
fn reindex_after_removal_shifts_later_blocks() {
    let mut g = segment_geom(1, 4);
    g.reindex_after_removal(0, 4);
    assert_eq!(g.param_offset, 0);
    g.reindex_after_removal(0, 4);
    assert_eq!(g.param_offset, 0);
}
