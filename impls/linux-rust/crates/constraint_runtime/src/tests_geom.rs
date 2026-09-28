use super::*;
use crate::document::DocumentStructure;

fn store_with(g: Geometry) -> DocumentStructure {
    let mut s = DocumentStructure::new();
    s.push_geometry(g);
    s
}

fn segment_geom(id: usize, param_offset: usize) -> Geometry {
    Segment::into_geometry(GeometryId(id), GeometryRole::Result, param_offset)
}

#[test]
fn editing_state_vector_changes_endpoints() {
    let offset = 0;
    let mut p = vec![0.0, 0.0, 1.0, 0.0];
    assert_eq!(Segment::end(offset, &p), Point2::new(1.0, 0.0));
    p[2] = 3.0;
    p[3] = 4.0;
    assert_eq!(Segment::end(offset, &p), Point2::new(3.0, 4.0));
    assert_eq!(
        segment_geom(0, offset).connectivity().global_indices,
        vec![0, 1, 2, 3]
    );
}

#[test]
fn point_minus_point_is_vec_direction_is_vec() {
    let a = Point2::new(1.0, 2.0);
    let b = Point2::new(4.0, 6.0);
    let v: Vec2 = b - a;
    assert_eq!(v, Vec2::new(3.0, 4.0));
    assert_eq!(a + v, b);

    let p = vec![0.0, 0.0, 1.0, 0.0];
    let d: Vec2 = Segment::direction(0, &p);
    assert_eq!(d, Vec2::new(1.0, 0.0));
    assert_eq!(d.cross(Vec2::new(0.0, 1.0)), 1.0);
    assert_eq!(d.dot(Vec2::new(0.0, 1.0)), 0.0);
}

#[test]
fn segment_curve_position_tangent_normal() {
    let g = segment_geom(0, 0);
    let store = store_with(g.clone());
    let p = vec![0.0, 0.0, 4.0, 0.0];
    assert_eq!(g.position_at(0.0, &p), Point2::new(0.0, 0.0));
    assert_eq!(g.position_at(0.5, &p), Point2::new(2.0, 0.0));
    assert_eq!(g.position_at(1.0, &p), Point2::new(4.0, 0.0));
    let tan = g.tangent_at(0.5, &p);
    assert_eq!(tan, Vec2::new(4.0, 0.0));
    assert!((tan.length() - 4.0).abs() < 1e-12);
    let n = g.normal_at(0.5, &p);
    assert_eq!(n, Vec2::new(0.0, 4.0));
    assert!((n.dot(tan)).abs() < 1e-12);
    assert!((n.length() - tan.length()).abs() < 1e-12);
    let start_e = Segment::start_expr(g.id);
    let end_e = Segment::end_expr(g.id);
    assert_eq!(start_e.evaluate(&store, &p), g.position_at(0.0, &p));
    assert_eq!(end_e.evaluate(&store, &p), g.position_at(1.0, &p));
}

#[test]
fn oncurve_uniform_matches_normalized_on_segment() {
    let g = segment_geom(0, 0);
    let store = store_with(g.clone());
    let p = vec![1.0, 2.0, 5.0, 8.0];
    let u = 0.25;
    let a = Segment::on_curve_expr(GeometryId(0), CurveParam::Normalized(u)).evaluate(&store, &p);
    let b = Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(u)).evaluate(&store, &p);
    assert_eq!(a, b);
    assert_eq!(a, g.position_at(u, &p));

    let mut dx_n = [0.0; 4];
    let mut dy_n = [0.0; 4];
    let mut dx_u = [0.0; 4];
    let mut dy_u = [0.0; 4];
    Segment::on_curve_expr(GeometryId(0), CurveParam::Normalized(u))
        .eval_point_jacobian(&store, &p, &mut dx_n, &mut dy_n);
    Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(u))
        .eval_point_jacobian(&store, &p, &mut dx_u, &mut dy_u);
    assert_eq!(dx_n, dx_u);
    assert_eq!(dy_n, dy_u);
}

#[test]
fn vec_expr_tangent_and_normal() {
    let store = store_with(segment_geom(0, 0));
    let p = vec![0.0, 0.0, 0.0, 3.0];
    let t = Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)).evaluate(&store, &p);
    let n = Segment::normal_expr(GeometryId(0), CurveParam::Uniform(0.5)).evaluate(&store, &p);
    assert_eq!(t, Vec2::new(0.0, 3.0));
    assert_eq!(n, Vec2::new(-3.0, 0.0));
}
