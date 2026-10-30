use serde_json::Value;
use sketch_session::{DeclCommand, Session};

use super::*;
use crate::fake_files::FakeFiles;

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

/// A drawing of one segment from (40, 20) to (60, 30) and a label at (50, 35).
const FAR_AWAY: &[u8] = br#"{"format": "illustrator-sketch", "version": 2, "scale": 10, "font_size": 10, "preamble": "", "commands": [
    {"op": "create", "kind": "segment", "name": "far", "start": [40, 20], "end": [60, 30]},
    {"op": "create", "kind": "label", "name": "tag", "origin": [50, 35], "heading": 0, "source": "far"}
]}"#;

fn open_far_away() -> (App, FakeFiles) {
    let fake = FakeFiles::new(true);
    let mut app = App::with_files(fake.clone());
    app.pick_points.push(app.logical_points[0].representative);
    app.segment_clicks.push(Point2::new(1.0, 1.0));
    app.file_command(FileCommand::Open);
    fake.open_file("far.sketch.json", FAR_AWAY);
    app.poll_files();
    (app, fake)
}

#[test]
fn opened_file_replaces_the_drawing_and_resets_picks() {
    let (app, _) = open_far_away();
    assert_eq!(app.status, "Opened far.sketch.json");
    assert_eq!(app.mesh().edges.len(), 1);
    assert_eq!(app.doc().geometries().len(), 2);
    assert!(!app.session.can_undo());
    assert!(app.pick_points.is_empty() && app.segment_clicks.is_empty());
    assert_eq!(app.logical_points.len(), 2, "the segment's two ends");
}

#[test]
fn opened_drawing_lies_inside_the_canvas() {
    let (app, _) = open_far_away();
    let canvas = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 600.0));
    let (min, max) = app.world_box;
    let vp = Viewport::fit(canvas, min, max, VIEW_MARGIN);
    for p in [Point2::new(40.0, 20.0), Point2::new(60.0, 30.0), Point2::new(50.0, 35.0)] {
        assert!(canvas.contains(vp.to_screen(p)), "{p:?} off screen");
    }
}

#[test]
fn new_gives_the_default_box_and_edits_do_not_refit() {
    let (mut app, _) = open_far_away();
    let fitted = app.world_box;
    app.run(
        Command::Create {
            name: None,
            role: GeometryRole::Result,
            geometry: GeometrySpec::Segment(SegmentParams {
                start: Point2::new(0.0, 0.0),
                end: Point2::new(1.0, 0.0),
            }),
        },
        String::new(),
    );
    assert_eq!(app.world_box, fitted, "an edit keeps the view");
    app.file_command(FileCommand::New);
    assert!(app.files.asking(), "the edit is unsaved");
    app.file_answer(Answer::Discard);
    assert_eq!(app.world_box, (WORLD_MIN, WORLD_MAX));
    assert!(app.doc().geometries().is_empty());
}
