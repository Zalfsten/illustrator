use super::*;
use crate::document::Document;
use crate::geom::Point2;

#[test]
fn one_segment_becomes_one_edge() {
    let mut doc = Document::new();
    let id = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(3.0, 4.0));
    let mesh = Mesh::from_structure(&doc.structure, doc.state());
    assert_eq!(mesh.edges.len(), 1);
    let e = &mesh.edges[0];
    assert_eq!(e.geometry, id);
    assert_eq!(e.a, Point2::new(0.0, 0.0));
    assert_eq!(e.b, Point2::new(3.0, 4.0));
}

#[test]
fn hit_endpoint_near_start_end_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(10.0, 0.0));
    let mesh = Mesh::from_structure(&doc.structure, doc.state());

    let hit_s = mesh.hit_endpoint(Point2::new(0.1, 0.0), 1.0);
    assert_eq!(hit_s, Some((id, EndpointKind::Start)));

    let hit_e = mesh.hit_endpoint(Point2::new(9.9, 0.0), 1.0);
    assert_eq!(hit_e, Some((id, EndpointKind::End)));

    assert_eq!(mesh.hit_endpoint(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn hit_geometry_near_midpoint_and_miss() {
    let mut doc = Document::new();
    let id = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(10.0, 0.0));
    let mesh = Mesh::from_structure(&doc.structure, doc.state());

    assert_eq!(
        mesh.hit_geometry(Point2::new(5.0, 0.5), 1.0),
        Some(id)
    );
    assert_eq!(mesh.hit_geometry(Point2::new(5.0, 5.0), 1.0), None);
}

#[test]
fn mesh_tracks_state_change() {
    let mut doc = Document::new();
    let id = doc.add_segment(Point2::new(0.0, 0.0), Point2::new(1.0, 0.0));
    doc.state_mut()[2] = 5.0;
    doc.state_mut()[3] = 0.0;
    let mesh = Mesh::from_structure(&doc.structure, doc.state());
    assert_eq!(mesh.edges[0].geometry, id);
    assert_eq!(mesh.edges[0].b, Point2::new(5.0, 0.0));
}
