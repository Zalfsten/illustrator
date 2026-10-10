use super::*;
use crate::document::Document;
use crate::geom::{HeadingAngle, Length, Point2, TurnAngle};
use crate::geometry_kinds::{Arc, Segment};
use crate::geometry::{EndpointExprs, GeometryRole};

#[test]
fn one_segment_becomes_one_edge() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(3.0, 4.0)));
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);
    assert_eq!(mesh.edges.len(), 1);
    let e = &mesh.edges[0];
    assert_eq!(e.geometry, id);
    assert_eq!(e.role, GeometryRole::Result);
    assert_eq!(e.a, Point2::new(0.0, 0.0));
    assert_eq!(e.b, Point2::new(3.0, 4.0));
}

#[test]
fn hit_endpoint_near_start_end_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);

    let hit_s = mesh.hit_endpoint(Point2::new(0.1, 0.0), 1.0);
    assert_eq!(hit_s, Some(Segment::start_expr(id)));

    let hit_e = mesh.hit_endpoint(Point2::new(9.9, 0.0), 1.0);
    assert_eq!(hit_e, Some(Segment::end_expr(id)));

    assert_eq!(mesh.hit_endpoint(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn hit_geometry_near_midpoint_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);

    assert_eq!(
        mesh.hit_geometry(Point2::new(5.0, 0.5), 1.0),
        Some(id)
    );
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn mesh_tracks_state_change() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    doc.vector_mut()[2] = 5.0;
    doc.vector_mut()[3] = 0.0;
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);
    assert_eq!(mesh.edges[0].geometry, id);
    assert_eq!(mesh.edges[0].b, Point2::new(5.0, 0.0));
}

#[test]
fn construction_omitted_from_result_only_mesh() {
    let mut doc = Document::new();
    let result = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let construction = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 1.0), Point2::new(1.0, 1.0)));
    assert!(doc.set_geometry_role(construction, GeometryRole::Construction));

    let result_only =
        Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);
    assert_eq!(result_only.edges.len(), 1);
    assert_eq!(result_only.edges[0].geometry, result);
    assert!(result_only
        .edges
        .iter()
        .all(|e| e.geometry != construction));

    let included =
        Mesh::from_document(&doc, doc.vector(), MeshFilter::INCLUDE_CONSTRUCTION);
    assert_eq!(included.edges.len(), 2);
    assert!(included.edges.iter().any(|e| e.geometry == construction));
    assert!(included
        .edges
        .iter()
        .any(|e| e.geometry == construction && e.role == GeometryRole::Construction));
}

#[test]
fn arc_tessellates_to_multiple_edges() {
    let mut doc = Document::new();
    let id = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(0.0, 0.0),
            Length::new(1.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::FRAC_PI_2),
        ),
    );
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);
    assert!(mesh.edges.len() > 1);
    assert!(mesh.edges.iter().all(|e| e.geometry == id));
    let start = Arc::start_expr(id).evaluate(&doc, doc.vector());
    let end = Arc::end_expr(id).evaluate(&doc, doc.vector());
    assert_eq!(mesh.edges[0].a, start);
    assert_eq!(mesh.edges.last().unwrap().b, end);
    assert_eq!(
        mesh.hit_endpoint(Point2::new(1.0, 0.05), 0.2),
        Some(Arc::start_expr(id))
    );
    assert_eq!(
        mesh.hit_endpoint(Point2::new(0.05, 1.0), 0.2),
        Some(Arc::end_expr(id))
    );
}

#[test]
fn segment_and_arc_mesh_side_by_side() {
    let mut doc = Document::new();
    let seg = doc.add_geometry::<Segment>(
        GeometryRole::Result,
        (Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)),
    );
    let arc = doc.add_geometry::<Arc>(
        GeometryRole::Result,
        (
            Point2::new(2.0, 0.0),
            Length::new(1.0),
            HeadingAngle::new(0.0),
            TurnAngle::new(std::f64::consts::PI),
        ),
    );
    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);
    let seg_edges: Vec<_> = mesh.edges.iter().filter(|e| e.geometry == seg).collect();
    let arc_edges: Vec<_> = mesh.edges.iter().filter(|e| e.geometry == arc).collect();
    assert_eq!(seg_edges.len(), 1);
    assert!(arc_edges.len() > 1);
}

#[test]
fn result_only_hits_ignore_construction() {
    let mut doc = Document::new();
    let _result = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let construction = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 5.0), Point2::new(10.0, 5.0)));
    assert!(doc.set_geometry_role(construction, GeometryRole::Construction));

    let mesh = Mesh::from_document(&doc, doc.vector(), MeshFilter::RESULT_ONLY);
    assert_eq!(mesh.hit_endpoint(Point2::new(0.0, 5.0), 1.0), None);
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);

    let full =
        Mesh::from_document(&doc, doc.vector(), MeshFilter::INCLUDE_CONSTRUCTION);
    assert_eq!(
        full.hit_endpoint(Point2::new(0.0, 5.0), 1.0),
        Some(Segment::start_expr(construction))
    );
    assert_eq!(
        full.hit_geometry(Point2::new(5.0, 5.0), 1.0),
        Some(construction)
    );
}
