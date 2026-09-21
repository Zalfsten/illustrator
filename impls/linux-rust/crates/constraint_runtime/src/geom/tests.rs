use super::*;
use crate::document::DocumentStructure;

fn store_with(seg: Segment) -> DocumentStructure {
    let mut s = DocumentStructure::new();
    s.push_geometry(Geometry::Segment(seg));
    s
}

#[test]
fn editing_state_vector_changes_endpoints() {
    let seg = Segment {
        id: GeometryId(0),
        param_offset: 0,
    };
    let mut p = vec![0.0, 0.0, 1.0, 0.0];
    assert_eq!(seg.end(&p), Point2::new(1.0, 0.0));
    p[2] = 3.0;
    p[3] = 4.0;
    assert_eq!(seg.end(&p), Point2::new(3.0, 4.0));
    assert_eq!(seg.connectivity().global_indices, vec![0, 1, 2, 3]);
}

#[test]
fn point_minus_point_is_vec_direction_is_vec() {
    let a = Point2::new(1.0, 2.0);
    let b = Point2::new(4.0, 6.0);
    let v: Vec2 = b - a;
    assert_eq!(v, Vec2::new(3.0, 4.0));
    assert_eq!(a + v, b);

    let seg = Segment {
        id: GeometryId(0),
        param_offset: 0,
    };
    let p = vec![0.0, 0.0, 1.0, 0.0];
    let d: Vec2 = seg.direction(&p);
    assert_eq!(d, Vec2::new(1.0, 0.0));
    assert_eq!(d.cross(Vec2::new(0.0, 1.0)), 1.0);
    assert_eq!(d.dot(Vec2::new(0.0, 1.0)), 0.0);
}

#[test]
fn segment_curve_position_tangent_normal() {
    let seg = Segment {
        id: GeometryId(0),
        param_offset: 0,
    };
    let store = store_with(seg.clone());
    let p = vec![0.0, 0.0, 4.0, 0.0];
    assert_eq!(seg.position_at(0.0, &p), Point2::new(0.0, 0.0));
    assert_eq!(seg.position_at(0.5, &p), Point2::new(2.0, 0.0));
    assert_eq!(seg.position_at(1.0, &p), Point2::new(4.0, 0.0));
    let tan = seg.tangent_at(0.5, &p);
    assert_eq!(tan, Vec2::new(4.0, 0.0));
    assert!((tan.length() - 4.0).abs() < 1e-12);
    let n = seg.normal_at(0.5, &p);
    assert_eq!(n, Vec2::new(0.0, 4.0));
    assert!((n.dot(tan)).abs() < 1e-12);
    assert!((n.length() - tan.length()).abs() < 1e-12);
    let (start_e, end_e) = seg.endpoints();
    assert_eq!(
        start_e.evaluate(&store, &p),
        seg.position_at(0.0, &p)
    );
    assert_eq!(end_e.evaluate(&store, &p), seg.position_at(1.0, &p));
}

#[test]
fn oncurve_uniform_matches_normalized_on_segment() {
    let seg = Segment {
        id: GeometryId(0),
        param_offset: 0,
    };
    let store = store_with(seg.clone());
    let p = vec![1.0, 2.0, 5.0, 8.0];
    let u = 0.25;
    let a = PointExpr::OnCurve {
        geometry: GeometryId(0),
        param: CurveParam::Normalized(u),
    }
    .evaluate(&store, &p);
    let b = PointExpr::OnCurve {
        geometry: GeometryId(0),
        param: CurveParam::Uniform(u),
    }
    .evaluate(&store, &p);
    assert_eq!(a, b);
    assert_eq!(a, seg.position_at(u, &p));

    let mut dx_n = [0.0; 4];
    let mut dy_n = [0.0; 4];
    let mut dx_u = [0.0; 4];
    let mut dy_u = [0.0; 4];
    PointExpr::OnCurve {
        geometry: GeometryId(0),
        param: CurveParam::Normalized(u),
    }
    .eval_point_jacobian(&store, &p, &mut dx_n, &mut dy_n);
    PointExpr::OnCurve {
        geometry: GeometryId(0),
        param: CurveParam::Uniform(u),
    }
    .eval_point_jacobian(&store, &p, &mut dx_u, &mut dy_u);
    assert_eq!(dx_n, dx_u);
    assert_eq!(dy_n, dy_u);
}

#[test]
fn vec_expr_tangent_and_normal() {
    let store = store_with(Segment {
        id: GeometryId(0),
        param_offset: 0,
    });
    let p = vec![0.0, 0.0, 0.0, 3.0];
    let t = VecExpr::TangentAt {
        geometry: GeometryId(0),
        param: CurveParam::Normalized(0.0),
    }
    .evaluate(&store, &p);
    let n = VecExpr::NormalAt {
        geometry: GeometryId(0),
        param: CurveParam::Uniform(0.5),
    }
    .evaluate(&store, &p);
    assert_eq!(t, Vec2::new(0.0, 3.0));
    assert_eq!(n, Vec2::new(-3.0, 0.0));
}
