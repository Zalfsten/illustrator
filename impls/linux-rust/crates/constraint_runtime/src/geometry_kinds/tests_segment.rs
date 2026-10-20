use super::*;
use crate::document::Document;
use crate::geom::{CurveParam, Point2, Vec2};
use crate::geometry::StateAtom;
use crate::geometry::{
    Chart, CurveExprs, EndpointExprs, GeometryId, GeometryRole, GeometryShell,
};
use crate::geometry_kinds::GeometryInst;
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

fn store_with(g: GeometryInst) -> Document {
    let mut doc = Document::new();
    doc.push_geometry(g);
    doc
}

fn segment_geom(id: usize, param_offset: usize) -> GeometryInst {
    GeometryInst::of::<Segment>(GeometryShell::new(GeometryId(id), GeometryRole::Result, param_offset))
}

#[test]
fn segment_layout_and_param_block() {
    let offset = 0;
    assert_eq!(Segment::PARAMETER_COUNT, 2 * Point2::DOF_COUNT);
    assert_eq!(Segment::start.range().collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(Segment::end.range().collect::<Vec<_>>(), vec![2, 3]);
    let init = (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    assert_eq!(
        <Segment as Chart>::pack(init).as_ref(),
        &[0.0, 0.0, 1.0, 0.0]
    );
    assert_eq!(Segment::unpack(&[0.0, 0.0, 1.0, 0.0]), init);
    assert_eq!(
        segment_geom(0, offset).param_block(),
        0..4
    );
    assert_eq!(Segment::param_block_at(2), 2..6);
}

#[test]
fn reindex_after_removal_shifts_later_blocks() {
    let mut g = segment_geom(1, 4);
    g.reindex_after_removal(0, 4);
    assert_eq!(g.param_block().start, 0);
    g.reindex_after_removal(0, 4);
    assert_eq!(g.param_block().start, 0);
}

#[test]
fn editing_state_vector_changes_endpoints() {
    let store = store_with(segment_geom(0, 0));
    let mut p = Vector::from(vec![0.0, 0.0, 1.0, 0.0]);
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
fn length_expr_is_endpoint_euclidean_distance() {
    let store = store_with(segment_geom(0, 0));
    let p = Vector::from(vec![0.0, 0.0, 3.0, 4.0]);
    let len = Segment::length_expr(GeometryId(0))
        .evaluate(&store, &p)
        .value();
    assert!((len - 5.0).abs() < 1e-12);
}

#[test]
fn segment_endpoint_override_matches_on_curve_but_sparser_connectivity() {
    let store = store_with(segment_geom(0, 0));
    let p = Vector::from(vec![1.0, 2.0, 5.0, 6.0]);
    let id = GeometryId(0);
    let start = Segment::start_expr(id);
    let at0 = Segment::on_curve_expr(id, CurveParam::Normalized(0.0));
    assert_eq!(start.evaluate(&store, &p), at0.evaluate(&store, &p));
    assert_eq!(start.param_indices(&store), vec![0, 1]);
    assert_eq!(at0.param_indices(&store), vec![0, 1, 2, 3]);

    let end = Segment::end_expr(id);
    let at1 = Segment::on_curve_expr(id, CurveParam::Normalized(1.0));
    assert_eq!(end.evaluate(&store, &p), at1.evaluate(&store, &p));
    assert_eq!(end.param_indices(&store), vec![2, 3]);
    assert_eq!(at1.param_indices(&store), vec![0, 1, 2, 3]);
}

#[test]
fn tangent_expr_is_end_minus_start() {
    let store = store_with(segment_geom(0, 0));
    let p = Vector::from(vec![0.0, 0.0, 1.0, 0.0]);
    let d = Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)).evaluate(&store, &p);
    assert_eq!(d, Vec2::new(1.0, 0.0));
    assert_eq!(d.cross(Vec2::new(0.0, 1.0)), 1.0);
    assert_eq!(d.dot(Vec2::new(0.0, 1.0)), 0.0);
}

#[test]
fn segment_curve_position_tangent_normal_via_expr() {
    let store = store_with(segment_geom(0, 0));
    let p = Vector::from(vec![0.0, 0.0, 2.0, 0.0]);
    let mid = Segment::on_curve_expr(GeometryId(0), CurveParam::Normalized(0.5)).evaluate(&store, &p);
    assert_eq!(mid, Point2::new(1.0, 0.0));
    let t = Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)).evaluate(&store, &p);
    assert_eq!(t, Vec2::new(2.0, 0.0));
    let n = Segment::normal_expr(GeometryId(0), CurveParam::Normalized(0.0)).evaluate(&store, &p);
    assert_eq!(n, Vec2::new(0.0, 2.0));
}

#[test]
fn oncurve_uniform_matches_normalized_on_segment() {
    let store = store_with(segment_geom(0, 0));
    let p = Vector::from(vec![0.0, 0.0, 4.0, 2.0]);
    let u = 0.25;
    let a = Segment::on_curve_expr(GeometryId(0), CurveParam::Normalized(u)).evaluate(&store, &p);
    let b = Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(u)).evaluate(&store, &p);
    assert_eq!(a, b);
    let n = Segment::PARAMETER_COUNT;
    let mut buf_n = vec![0.0; 2 * n];
    let mut buf_u = vec![0.0; 2 * n];
    {
        let mut jn = DenseMatrix::packed(&mut buf_n, 2, n);
        Segment::on_curve_expr(GeometryId(0), CurveParam::Normalized(u))
            .accumulate_jacobian(&store, &p, &mut jn, 1.0);
    }
    {
        let mut ju = DenseMatrix::packed(&mut buf_u, 2, n);
        Segment::on_curve_expr(GeometryId(0), CurveParam::Uniform(u))
            .accumulate_jacobian(&store, &p, &mut ju, 1.0);
    }
    assert_eq!(buf_n, buf_u);
}

#[test]
fn vec_expr_tangent_and_normal() {
    let store = store_with(segment_geom(0, 0));
    let p = Vector::from(vec![1.0, 1.0, 4.0, 5.0]);
    let t = Segment::tangent_expr(GeometryId(0), CurveParam::Normalized(0.0)).evaluate(&store, &p);
    let n = Segment::normal_expr(GeometryId(0), CurveParam::Uniform(0.5)).evaluate(&store, &p);
    assert_eq!(t, Vec2::new(3.0, 4.0));
    assert_eq!(n, t.left_normal());
}

#[test]
fn segment_canonical_is_identity() {
    let init = (Point2::new(1.0, -2.0), Point2::new(-3.0, 0.5));
    assert_eq!(<Segment as Geometry>::canonical(init), init);
}
