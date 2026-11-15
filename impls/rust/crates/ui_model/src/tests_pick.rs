use super::*;
use crate::test_util::*;
use constraint_runtime::{GeometryRole, MeshFilter};

const ALL: MeshFilter = MeshFilter { include_construction: true };

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

#[test]
fn point_wins_over_curve() {
    // seg2 ends at (2, 1); seg4 passes 0.05 below that end.
    let s = session(vec![
        segment("seg2", (1.0, 0.0), (2.0, 1.0)),
        segment("seg4", (0.0, 0.95), (4.0, 0.95)),
    ]);
    let doc = s.document();
    let sh = shown(doc, ALL);
    let hit = pick(doc, &sh.scene(), p(2.0, 0.96), 0.2);
    assert_eq!(hit.map(|e| e.label(doc)).as_deref(), Some("seg2.end"));
}

#[test]
fn curve_when_no_point_is_near() {
    let s = sample();
    let doc = s.document();
    let sh = shown(doc, ALL);
    let hit = pick(doc, &sh.scene(), p(0.5, 0.05), 0.1);
    assert_eq!(hit.map(|e| e.label(doc)).as_deref(), Some("seg1"));
    assert_eq!(pick(doc, &sh.scene(), p(3.0, 3.0), 0.1), None);
}

#[test]
fn tolerance_comes_from_the_front_end() {
    let s = sample();
    let doc = s.document();
    let sh = shown(doc, ALL);
    // 0.3 above the middle of seg1.
    assert_eq!(pick(doc, &sh.scene(), p(0.5, 0.3), 0.2), None);
    assert_eq!(pick(doc, &sh.scene(), p(0.5, 0.3), 0.4).map(|e| e.label(doc)).as_deref(), Some("seg1"));
}

#[test]
fn only_geometries_completely_inside() {
    // seg1 (0,0)–(1,0) inside, seg2 (1,0)–(2,1) half inside, seg3 far away.
    let s = sample();
    let doc = s.document();
    let sh = shown(doc, ALL);
    let names = |ids: Vec<GeometryId>| ids.iter().map(|id| doc.geometry_name(*id).unwrap().to_owned()).collect::<Vec<_>>();
    assert_eq!(names(box_select(&sh.scene(), p(-0.5, -0.5), p(1.5, 0.5))), ["seg1"]);
    // Corners in either order.
    assert_eq!(names(box_select(&sh.scene(), p(1.5, 0.5), p(-0.5, -0.5))), ["seg1"]);
    assert_eq!(names(box_select(&sh.scene(), p(-1.0, -1.0), p(3.0, 2.0))), ["seg1", "seg2"]);
}

#[test]
fn labels_are_never_box_selected() {
    let s = session(vec![label("lbl1", (1.0, 1.0))]);
    let doc = s.document();
    let sh = shown(doc, ALL);
    assert!(box_select(&sh.scene(), p(-10.0, -10.0), p(10.0, 10.0)).is_empty());
}

#[test]
fn hidden_construction_is_not_picked() {
    let s = session(vec![segment_as("aux", GeometryRole::Construction, (0.0, 0.0), (1.0, 0.0))]);
    let doc = s.document();
    let result_only = shown(doc, MeshFilter::RESULT_ONLY);
    assert_eq!(pick(doc, &result_only.scene(), p(0.5, 0.0), 0.1), None);
    assert_eq!(pick(doc, &result_only.scene(), p(1.0, 0.0), 0.1), None, "nor its end point");
    let all = shown(doc, ALL);
    assert_eq!(pick(doc, &all.scene(), p(0.5, 0.0), 0.1).map(|e| e.label(doc)).as_deref(), Some("aux"));
}
