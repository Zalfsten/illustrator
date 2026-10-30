use serde_json::Value;
use sketch_session::{DeclCommand, Session};

use super::*;

/// Equal up to numbers, which may differ by `tol`.
fn close(a: &Value, b: &Value, tol: f64) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() <= tol
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| close(x, y, tol))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| close(v, w, tol)))
        }
        _ => a == b,
    }
}

#[test]
fn sample_scene_exports_and_re_executes() {
    let list = App::with_sample().session.declarative_commands().unwrap();
    let text = DeclCommand::list_to_json(&list).unwrap();
    let read = DeclCommand::list_from_json(&text).unwrap();
    assert_eq!(read, list);

    let mut s = Session::new(Document::new());
    for c in read {
        s.apply(c.into()).unwrap();
    }
    let again = DeclCommand::list_to_json(&s.declarative_commands().unwrap()).unwrap();
    let (a, b): (Value, Value) = (
        serde_json::from_str(&text).unwrap(),
        serde_json::from_str(&again).unwrap(),
    );
    assert!(close(&a, &b, 1e-9), "{text}\n{again}");
}

#[test]
fn sample_scene_file_round_trip_is_byte_identical() {
    let app = App::with_sample();
    let first = app.session.save().unwrap();
    let (loaded, report) = Session::load(&first).unwrap();
    assert!(report.adjusted.is_empty() && report.conflicts.is_empty());
    assert_eq!(loaded.save().unwrap(), first);
}

#[test]
fn drawing_with_labels_loads_paints_and_drags() {
    let bytes = br#"{"format": "illustrator-sketch", "version": 2, "scale": 10, "font_size": 10, "preamble": "", "commands": [
        {"op": "create", "kind": "segment", "name": "seg1", "start": [0, 0], "end": [1, 0]},
        {"op": "create", "kind": "label", "name": "free", "origin": [3, 3], "heading": 0, "source": "free"},
        {"op": "create", "kind": "label", "name": "tip", "origin": [1, 0], "heading": 0, "source": "$v$"},
        {"op": "constrain", "kind": "join", "name": "j", "a": "tip.origin", "b": "seg1.end"},
        {"op": "constrain", "kind": "parallel", "name": "p", "a": "tip.direction", "b": "seg1.direction"}
    ]}"#;
    let (session, report) = Session::load(bytes).unwrap();
    assert!(report.conflicts.is_empty());
    let mut app = App::with_sample();
    app.session = session;
    app.rebuild_logical_points();

    let mesh = app.mesh();
    assert_eq!(mesh.edges.len(), 1, "only the segment is painted");
    // seg1.start and the class of seg1.end / tip.origin; the free label has no handle.
    assert_eq!(app.logical_points.len(), 2);

    app.session.drag_begin("seg1.end").unwrap();
    app.session.drag_to(Point2::new(1.0, 1.0)).unwrap();
    app.session.drag_end(Point2::new(1.0, 1.0)).unwrap();
    let doc = app.doc();
    let at = |a: &str| match doc.resolve_address(a).unwrap() {
        constraint_runtime::Address::Point(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{other:?}"),
    };
    let (end, origin) = (at("seg1.end"), at("tip.origin"));
    assert!((end.x - origin.x).abs() < 1e-6 && (end.y - origin.y).abs() < 1e-6);
}
