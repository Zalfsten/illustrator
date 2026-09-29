use super::*;
use crate::document::Document;
use crate::geom::{GeometryRole, Point2};
use crate::geometries::{GeometryBody, Segment};

#[test]
fn one_segment_becomes_one_edge() {
    let mut doc = Document::new();
    let id = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(3.0, 4.0)));
    let mesh = Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::RESULT_ONLY);
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
    let id = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let mesh = Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::RESULT_ONLY);

    let hit_s = mesh.hit_endpoint(Point2::new(0.1, 0.0), 1.0);
    assert_eq!(hit_s, Some(Segment::start_expr(id)));

    let hit_e = mesh.hit_endpoint(Point2::new(9.9, 0.0), 1.0);
    assert_eq!(hit_e, Some(Segment::end_expr(id)));

    assert_eq!(mesh.hit_endpoint(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn hit_geometry_near_midpoint_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let mesh = Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::RESULT_ONLY);

    assert_eq!(
        mesh.hit_geometry(Point2::new(5.0, 0.5), 1.0),
        Some(id)
    );
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn mesh_tracks_state_change() {
    let mut doc = Document::new();
    let id = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    doc.state_mut()[2] = 5.0;
    doc.state_mut()[3] = 0.0;
    let mesh = Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::RESULT_ONLY);
    assert_eq!(mesh.edges[0].geometry, id);
    assert_eq!(mesh.edges[0].b, Point2::new(5.0, 0.0));
}

#[test]
fn construction_omitted_from_result_only_mesh() {
    let mut doc = Document::new();
    let result = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)));
    let construction = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 1.0), Point2::new(1.0, 1.0)));
    assert!(doc.set_geometry_role(construction, GeometryRole::Construction));

    let result_only =
        Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::RESULT_ONLY);
    assert_eq!(result_only.edges.len(), 1);
    assert_eq!(result_only.edges[0].geometry, result);
    assert!(result_only
        .edges
        .iter()
        .all(|e| e.geometry != construction));

    let included =
        Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::INCLUDE_CONSTRUCTION);
    assert_eq!(included.edges.len(), 2);
    assert!(included.edges.iter().any(|e| e.geometry == construction));
    assert!(included
        .edges
        .iter()
        .any(|e| e.geometry == construction && e.role == GeometryRole::Construction));
}

#[test]
fn result_only_hits_ignore_construction() {
    let mut doc = Document::new();
    let _result = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 0.0), Point2::new(10.0, 0.0)));
    let construction = doc.add_geometry(GeometryRole::Result, GeometryBody::Segment, &Segment::params(Point2::new(0.0, 5.0), Point2::new(10.0, 5.0)));
    assert!(doc.set_geometry_role(construction, GeometryRole::Construction));

    let mesh = Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::RESULT_ONLY);
    assert_eq!(mesh.hit_endpoint(Point2::new(0.0, 5.0), 1.0), None);
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);

    let full =
        Mesh::from_structure(&doc.structure, doc.state(), MeshFilter::INCLUDE_CONSTRUCTION);
    assert_eq!(
        full.hit_endpoint(Point2::new(0.0, 5.0), 1.0),
        Some(Segment::start_expr(construction))
    );
    assert_eq!(
        full.hit_geometry(Point2::new(5.0, 5.0), 1.0),
        Some(construction)
    );
}
