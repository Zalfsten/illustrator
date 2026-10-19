use super::*;
use crate::document::Document;
use crate::geometry_kinds::GeometryInst;
use crate::geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry::{Chart, CurveExprs, EndpointExprs, GeometryId, GeometryRole, GeometryShell};
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

type ArcInit = (Point2, Length, HeadingAngle, TurnAngle);

fn store_with(g: GeometryInst) -> Document {
    let mut doc = Document::new();
    doc.push_geometry(g);
    doc
}

fn arc_geom(id: usize, param_offset: usize) -> GeometryInst {
    GeometryInst::of::<Arc>(GeometryShell::new(GeometryId(id), GeometryRole::Result, param_offset))
}

fn unit_quarter() -> (ArcInit, Vector) {
    let init = (
        Point2::new(0.0, 0.0),
        Length::new(1.0),
        HeadingAngle::new(0.0),
        TurnAngle::new(std::f64::consts::FRAC_PI_2),
    );
    let packed = <Arc as Chart>::pack(init);
    (init, Vector::from(packed.as_ref().to_vec()))
}

#[test]
fn arc_layout_and_param_block() {
    assert_eq!(Arc::PARAMETER_COUNT, 5);
    assert_eq!(Arc::center.range().collect::<Vec<_>>(), vec![0, 1]);
    assert_eq!(Arc::radius.range().collect::<Vec<_>>(), vec![2]);
    assert_eq!(Arc::start.range().collect::<Vec<_>>(), vec![3]);
    assert_eq!(Arc::sweep.range().collect::<Vec<_>>(), vec![4]);
    let init = (
        Point2::new(1.0, 2.0),
        Length::new(3.0),
        HeadingAngle::new(0.5),
        TurnAngle::new(-0.25),
    );
    let packed = <Arc as Chart>::pack(init);
    assert_eq!(packed.as_ref(), &[1.0, 2.0, 3.0, 0.5, -0.25]);
    assert_eq!(Arc::unpack(packed.as_ref()), init);
    assert_eq!(arc_geom(0, 0).param_block(), 0..5);
}

#[test]
fn on_curve_endpoints_and_length() {
    let store = store_with(arc_geom(0, 0));
    let (_init, p) = unit_quarter();
    let id = GeometryId(0);

    let start = Arc::start_expr(id).evaluate(&store, &p);
    let end = Arc::end_expr(id).evaluate(&store, &p);
    assert!((start.x - 1.0).abs() < 1e-12 && start.y.abs() < 1e-12);
    assert!(end.x.abs() < 1e-12 && (end.y - 1.0).abs() < 1e-12);

    let mid = Arc::on_curve_expr(id, CurveParam::Normalized(0.5)).evaluate(&store, &p);
    let expected = std::f64::consts::FRAC_PI_4;
    assert!((mid.x - expected.cos()).abs() < 1e-12);
    assert!((mid.y - expected.sin()).abs() < 1e-12);

    let len = Arc::length_expr(id).evaluate(&store, &p).value();
    assert!((len - std::f64::consts::FRAC_PI_2).abs() < 1e-12);

    let u = Arc::on_curve_expr(id, CurveParam::Uniform(0.25)).evaluate(&store, &p);
    let n = Arc::on_curve_expr(id, CurveParam::Normalized(0.25)).evaluate(&store, &p);
    assert_eq!(u, n);
}

#[test]
fn heading_and_sweep_exprs_match_chart() {
    let store = store_with(arc_geom(0, 0));
    let (init, p) = unit_quarter();
    let id = GeometryId(0);
    assert_eq!(Arc::start_heading_expr(id).evaluate(&store, &p), init.2);
    assert_eq!(Arc::sweep_expr(id).evaluate(&store, &p), init.3);
    assert_eq!(Arc::start_heading_expr(id).param_indices(&store), vec![3]);
    assert_eq!(Arc::sweep_expr(id).param_indices(&store), vec![4]);
}

#[test]
fn tangent_normal_left_frame() {
    let store = store_with(arc_geom(0, 0));
    let (_init, p) = unit_quarter();
    let id = GeometryId(0);
    let t = Arc::tangent_expr(id, CurveParam::Normalized(0.0)).evaluate(&store, &p);
    // at θ=0, γ' = R Δθ (0, 1) for R=1, Δθ=π/2
    assert!((t.x).abs() < 1e-12);
    assert!((t.y - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    let n = Arc::normal_expr(id, CurveParam::Normalized(0.0)).evaluate(&store, &p);
    assert_eq!(n, t.left_normal());
}

fn fd_jac_rows(
    p: &Vector,
    rows: usize,
    cols: usize,
    mut eval_row: impl FnMut(&Vector, usize) -> f64,
) -> Vec<f64> {
    const EPS: f64 = 1e-6;
    let mut out = vec![0.0; rows * cols];
    for k in 0..cols {
        let mut pp = p.clone();
        let mut pm = p.clone();
        pp[k] += EPS;
        pm[k] -= EPS;
        for i in 0..rows {
            out[i * cols + k] = (eval_row(&pp, i) - eval_row(&pm, i)) / (2.0 * EPS);
        }
    }
    out
}

#[test]
fn on_curve_jacobian_matches_fd() {
    let store = store_with(arc_geom(0, 0));
    let p = Vector::from(vec![0.5, -0.25, 2.0, 0.3, 1.1]);
    let id = GeometryId(0);
    let expr = Arc::on_curve_expr(id, CurveParam::Normalized(0.4));
    let n = Arc::PARAMETER_COUNT;
    let mut buf = vec![0.0; 2 * n];
    {
        let mut j = DenseMatrix::packed(&mut buf, 2, n);
        expr.accumulate_jacobian(&store, &p, &mut j, 1.0);
    }
    let fd = fd_jac_rows(&p, 2, n, |q, row| {
        let pt = expr.evaluate(&store, q);
        if row == 0 {
            pt.x
        } else {
            pt.y
        }
    });
    for (a, b) in buf.iter().zip(fd.iter()) {
        assert!((a - b).abs() < 1e-5, "jac {a} vs fd {b}");
    }
}

#[test]
fn length_jacobian_matches_fd() {
    let store = store_with(arc_geom(0, 0));
    let p = Vector::from(vec![0.0, 0.0, 1.5, 0.0, 0.8]);
    let id = GeometryId(0);
    let expr = Arc::length_expr(id);
    let n = Arc::PARAMETER_COUNT;
    let mut buf = vec![0.0; n];
    {
        let mut j = DenseMatrix::packed(&mut buf, 1, n);
        expr.accumulate_jacobian(&store, &p, &mut j, 1.0);
    }
    let fd = fd_jac_rows(&p, 1, n, |q, _| expr.evaluate(&store, q).value());
    for (a, b) in buf.iter().zip(fd.iter()) {
        assert!((a - b).abs() < 1e-5, "jac {a} vs fd {b}");
    }
}

#[test]
fn tangent_jacobian_matches_fd() {
    let store = store_with(arc_geom(0, 0));
    let p = Vector::from(vec![0.0, 0.0, 1.2, 0.4, 0.9]);
    let id = GeometryId(0);
    let expr = Arc::tangent_expr(id, CurveParam::Normalized(0.3));
    let n = Arc::PARAMETER_COUNT;
    let mut buf = vec![0.0; 2 * n];
    {
        let mut j = DenseMatrix::packed(&mut buf, 2, n);
        expr.accumulate_jacobian(&store, &p, &mut j, 1.0);
    }
    let fd = fd_jac_rows(&p, 2, n, |q, row| {
        let v = expr.evaluate(&store, q);
        if row == 0 {
            v.x
        } else {
            v.y
        }
    });
    for (a, b) in buf.iter().zip(fd.iter()) {
        assert!((a - b).abs() < 1e-5, "jac {a} vs fd {b}");
    }
}

#[test]
fn endpoint_start_sparser_than_full_chart() {
    let store = store_with(arc_geom(0, 0));
    let (_init, p) = unit_quarter();
    let id = GeometryId(0);
    let start = Arc::start_expr(id);
    let at0 = Arc::on_curve_expr(id, CurveParam::Normalized(0.0));
    assert_eq!(start.evaluate(&store, &p), at0.evaluate(&store, &p));
    assert_eq!(start.param_indices(&store), vec![0, 1, 2, 3]);
    assert_eq!(at0.param_indices(&store), vec![0, 1, 2, 3, 4]);
    assert_eq!(start.param_count(&store), 4);
}
