use super::*;
use crate::document::Document;
use crate::geom::{CurveParam, HeadingAngle, Point2};
use crate::geometry::{AnchorExprs, Chart, GeometryId, GeometryRole, LineExprs};
use crate::matrix::DenseMatrix;
use crate::vector::Vector;

const H: f64 = 1e-6;

fn doc_with_label(origin: Point2, heading: f64) -> (Document, GeometryId) {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Label>(GeometryRole::Result, (origin, HeadingAngle::new(heading)));
    (doc, id)
}

#[test]
fn label_layout_has_three_parameters() {
    assert_eq!(Label::PARAMETER_COUNT, 3);
    assert_eq!(Label::origin.range(), 0..2);
    assert_eq!(Label::heading.range(), 2..3);
}

#[test]
fn origin_and_direction_values() {
    let (doc, id) = doc_with_label(Point2::new(1.0, 2.0), std::f64::consts::FRAC_PI_2);
    let p = doc.vector();
    assert_eq!(
        Label::origin_expr(id).evaluate(&doc, p),
        Point2::new(1.0, 2.0)
    );
    let d = Label::direction_expr(id).evaluate(&doc, p);
    assert!(d.x.abs() < 1e-15 && (d.y - 1.0).abs() < 1e-15, "{d:?}");
}

#[test]
fn origin_jacobian_is_identity_on_origin_slot() {
    let (doc, id) = doc_with_label(Point2::new(1.0, 2.0), 0.3);
    let expr = Label::origin_expr(id);
    assert_eq!(expr.param_indices(&doc), vec![0, 1]);
    let mut buf = vec![0.0; 4];
    expr.accumulate_jacobian(
        &doc,
        doc.vector(),
        &mut DenseMatrix::packed(&mut buf, 2, 2),
        1.0,
    );
    assert_eq!(buf, vec![1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn direction_jacobian_matches_fd_and_uses_heading_only() {
    let (doc, id) = doc_with_label(Point2::new(1.0, 2.0), 0.7);
    let expr = Label::direction_expr(id);
    assert_eq!(expr.param_indices(&doc), vec![2]);
    let mut buf = vec![0.0; 2];
    expr.accumulate_jacobian(
        &doc,
        doc.vector(),
        &mut DenseMatrix::packed(&mut buf, 2, 1),
        1.0,
    );
    let mut pp: Vector = doc.vector().clone();
    let mut pm: Vector = doc.vector().clone();
    pp[2] += H;
    pm[2] -= H;
    let dp = expr.evaluate(&doc, &pp);
    let dm = expr.evaluate(&doc, &pm);
    let fd = [(dp.x - dm.x) / (2.0 * H), (dp.y - dm.y) / (2.0 * H)];
    for (a, b) in buf.iter().zip(fd) {
        assert!((a - b).abs() < 1e-6, "jac {a} vs fd {b}");
    }
}

#[test]
fn label_has_no_curve_expressions_by_id() {
    let (doc, id) = doc_with_label(Point2::new(0.0, 0.0), 0.0);
    assert!(doc.on_curve_expr(id, CurveParam::Normalized(0.5)).is_none());
    assert!(doc.tangent_expr(id, CurveParam::Normalized(0.5)).is_none());
    assert!(doc.normal_expr(id, CurveParam::Normalized(0.5)).is_none());
    assert!(doc.length_expr(id).is_none());
    assert!(doc.start_expr(id).is_none());
    assert!(doc.center_expr(id).is_none());
    assert_eq!(doc.origin_expr(id), Some(Label::origin_expr(id)));
    assert_eq!(doc.direction_expr(id), Some(Label::direction_expr(id)));
}
