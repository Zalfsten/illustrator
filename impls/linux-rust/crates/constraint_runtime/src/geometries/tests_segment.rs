use super::*;
use crate::document::DocumentStructure;
use crate::geom::{CurveParam, Geometry, GeometryId, GeometryRole, Point2, Vec2};

fn store_with(g: Geometry) -> DocumentStructure {
    let mut s = DocumentStructure::new();
    s.push_geometry(g);
    s
}

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
fn editing_state_vector_changes_endpoints() {
    let store = store_with(segment_geom(0, 0));
    let mut p = vec![0.0, 0.0, 1.0, 0.0];
    assert_eq!(
        Segment::end_expr(GeometryId(0)).evaluate(&store, &p),
        Point2::new(1.0, 0.0)
    );
    p[2] = 3.0;
    p[3] = 4.0;
    assert_eq!(
        Segment::end_expr(GeometryId(0)).evaluate(&store, &p),
        Point2::new(3.0, 4.0)
    );
}

#[test]
fn direction_expr_is_end_minus_start() {
    let store = store_with(segment_geom(0, 0));
    let p = vec![0.0, 0.0, 1.0, 0.0];
    let d = Segment::direction_expr(GeometryId(0)).evaluate(&store, &p);
    assert_eq!(d, Vec2::new(1.0, 0.0));
    assert_eq!(d.cross(Vec2::new(0.0, 1.0)), 1.0);
    assert_eq!(d.dot(Vec2::new(0.0, 1.0)), 0.0);
}

#[test]
fn segment_curve_position_tangent_normal_via_expr() {
    let g = segment_geom(0, 0);
    let store = store_with(g.clone());
    let p = vec![0.0, 0.0, 4.0, 0.0];
    let id = g.id;
    assert_eq!(
        Segment::on_curve_expr(id, CurveParam::Normalized(0.0)).evaluate(&store, &p),
        Point2::new(0.0, 0.0)
    );
    assert_eq!(
        Segment::on_curve_expr(id, CurveParam::Normalized(0.5)).evaluate(&store, &p),
        Point2::new(2.0, 0.0)
    );
    assert_eq!(
        Segment::on_curve_expr(id, CurveParam::Normalized(1.0)).evaluate(&store, &p),
        Point2::new(4.0, 0.0)
    );
    let tan = Segment::tangent_expr(id, CurveParam::Normalized(0.5)).evaluate(&store, &p);
    assert_eq!(tan, Vec2::new(4.0, 0.0));
    assert!((tan.length() - 4.0).abs() < 1e-12);
    let n = Segment::normal_expr(id, CurveParam::Normalized(0.5)).evaluate(&store, &p);
    assert_eq!(n, Vec2::new(0.0, 4.0));
    assert!((n.dot(tan)).abs() < 1e-12);
    assert!((n.length() - tan.length()).abs() < 1e-12);
    assert_eq!(
        Segment::start_expr(id).evaluate(&store, &p),
        Segment::on_curve_expr(id, CurveParam::Normalized(0.0)).evaluate(&store, &p)
    );
    assert_eq!(
        Segment::end_expr(id).evaluate(&store, &p),
        Segment::on_curve_expr(id, CurveParam::Normalized(1.0)).evaluate(&store, &p)
    );
}

#[test]
fn oncurve_uniform_matches_normalized_on_segment() {
    let g = segment_geom(0, 0);
    let store = store_with(g);
    let p = vec![1.0, 2.0, 5.0, 8.0];
    let u = 0.25;
    let a = Segment::on_curve_expr(GeometryId(0), CurveParam::Normalized(u)).evaluate(&store, &p);
    let b = Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(u)).evaluate(&store, &p);
    assert_eq!(a, b);
    assert_eq!(a, Point2::new(2.0, 3.5));

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
