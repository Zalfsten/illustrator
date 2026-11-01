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
    for p in [
        Point2::new(40.0, 20.0),
        Point2::new(60.0, 30.0),
        Point2::new(50.0, 35.0),
    ] {
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
            look: Default::default(),
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

// --- The Canvas shows result geometry in its style ---

use constraint_runtime::{Arrow, Dash, DocumentSettings, StyleProps, StyleRef, MM_PER_PT};

/// A document where 1 pt is one world unit, with segment (0, 0)–(`len`, 0)
/// in `role` overriding `look`.
fn styled_segment(len: f64, role: GeometryRole, look: StyleProps) -> (Document, GeometryId) {
    let mut doc = Document::new();
    doc.set_settings(DocumentSettings {
        scale: MM_PER_PT,
        ..DocumentSettings::default()
    });
    let id = doc.add_geometry::<Segment>(role, (Point2::new(0.0, 0.0), Point2::new(len, 0.0)));
    let look = StyleRef {
        overrides: look,
        ..StyleRef::default()
    };
    doc.set_geometry_style(id, look).unwrap();
    (doc, id)
}

fn paint_of(doc: &Document, id: GeometryId, viewport: &Viewport) -> CurvePaint {
    let mesh = Mesh::from_document(doc, doc.vector(), MeshFilter::INCLUDE_CONSTRUCTION);
    let lines = polylines(&mesh);
    let (_, line) = lines.iter().find(|(g, _)| *g == id).unwrap();
    curve_paint(doc, id, line, viewport, 1.0)
}

fn near(a: Pos2, b: Pos2) -> bool {
    (a - b).length() < 1e-3
}

#[test]
fn dashed_segment_yields_its_dashes() {
    let look = StyleProps {
        dash: Some(Dash::Dashed),
        ..StyleProps::default()
    };
    let (doc, id) = styled_segment(12.0, GeometryRole::Result, look);
    let vp = Viewport::new(10.0, Pos2::new(5.0, 100.0));
    let paint = paint_of(&doc, id, &vp);
    let spans: Vec<(Pos2, Pos2)> = paint
        .pieces
        .iter()
        .map(|p| (p[0], *p.last().unwrap()))
        .collect();
    assert_eq!(spans.len(), 2);
    assert!(near(spans[0].0, Pos2::new(5.0, 100.0)) && near(spans[0].1, Pos2::new(35.0, 100.0)));
    assert!(near(spans[1].0, Pos2::new(65.0, 100.0)) && near(spans[1].1, Pos2::new(95.0, 100.0)));
    assert!(paint.heads.is_empty());
    assert_eq!(paint.color, Color32::BLACK);
    assert!((paint.width - 4.0).abs() < 1e-4, "0.4 pt at 10 px per pt");
}

#[test]
fn arrow_head_tip_is_the_endpoint() {
    let look = StyleProps {
        arrow: Some(Arrow::End),
        color: Some(Rgb(0xc0, 0, 0)),
        ..StyleProps::default()
    };
    let (doc, id) = styled_segment(20.0, GeometryRole::Result, look);
    let vp = Viewport::new(2.0, Pos2::new(0.0, 50.0));
    let paint = paint_of(&doc, id, &vp);
    assert_eq!(paint.heads.len(), 1);
    assert!(near(
        paint.heads[0][0],
        vp.to_screen(Point2::new(20.0, 0.0))
    ));
    assert_eq!(paint.color, Color32::from_rgb(0xc0, 0, 0));
    assert_eq!(paint.pieces.len(), 1);
    assert!(near(
        *paint.pieces[0].last().unwrap(),
        vp.to_screen(Point2::new(15.2, 0.0))
    ));
}

#[test]
fn thin_line_stays_visible() {
    let look = StyleProps {
        width: Some(0.1),
        ..StyleProps::default()
    };
    let (doc, id) = styled_segment(10.0, GeometryRole::Result, look);
    let paint = paint_of(&doc, id, &Viewport::new(2.0, Pos2::ZERO));
    assert_eq!(paint.width, 1.0, "0.1 pt is 0.2 px here");
}

#[test]
fn construction_ignores_its_style() {
    let look = StyleProps {
        arrow: Some(Arrow::End),
        color: Some(Rgb(0xc0, 0, 0)),
        dash: Some(Dash::Dashed),
        ..StyleProps::default()
    };
    let (doc, id) = styled_segment(20.0, GeometryRole::Construction, look);
    let paint = paint_of(&doc, id, &Viewport::new(2.0, Pos2::ZERO));
    assert!(paint.heads.is_empty());
    assert_eq!(paint.color, CONSTRUCTION);
    assert_eq!(paint.pieces.len(), 1);
    assert_eq!(paint.width, CONSTRUCTION_WIDTH);
}

#[test]
fn polylines_group_edges_per_geometry() {
    let app = App::with_sample();
    let lines = polylines(&app.mesh());
    let mut ids: Vec<GeometryId> = lines.iter().map(|(g, _)| *g).collect();
    let n = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), n, "each geometry once");
    assert_eq!(n, 3, "two legs and the fillet");
}

// --- Style row and brush ---

use crate::style_ui::Switch;

fn apply_json(app: &mut App, text: &str) {
    app.session
        .apply(Command::from_json(text).unwrap())
        .unwrap();
}

/// `create` of `name` in the exported drawing.
fn exported(app: &App, name: &str) -> String {
    let list = app.session.declarative_commands().unwrap();
    list.iter()
        .map(|c| c.to_json().unwrap())
        .find(|t| t.contains(&format!("\"name\":\"{name}\"")))
        .unwrap()
}

#[test]
fn segment_tool_draws_with_the_current_style() {
    let mut app = App::with_sample();
    apply_json(
        &mut app,
        r#"{"op": "define_style", "style": "axis", "width": 0.8}"#,
    );
    app.current.style = "axis".into();
    app.current.color = Switch {
        on: true,
        value: Rgb(0xc0, 0, 0),
    };
    app.tool = Tool::Segment;
    app.handle_click(app.viewport.to_screen(Point2::new(5.0, 5.0)));
    app.handle_click(app.viewport.to_screen(Point2::new(6.0, 5.0)));
    let name = app.status.strip_prefix("Added ").unwrap().to_owned();
    let text = exported(&app, &name);
    assert!(
        text.ends_with(r##""style":"axis","color":"#c00000"}"##),
        "{text}"
    );
}

#[test]
fn brush_replaces_style_and_overrides() {
    let mut app = App::with_sample();
    apply_json(
        &mut app,
        r##"{"op": "create", "kind": "segment", "name": "odd", "start": [5, 5], "end": [6, 5], "color": "#c00000", "dash": "dotted"}"##,
    );
    app.tool = Tool::Brush;
    app.handle_click(app.viewport.to_screen(Point2::new(5.5, 5.0)));
    assert_eq!(app.status, "Brushed odd");
    let text = exported(&app, "odd");
    assert!(
        !text.contains("color") && !text.contains("dash") && !text.contains("style"),
        "{text}"
    );
    assert!(app.session.undo().is_ok());
    assert!(exported(&app, "odd").contains("dotted"));
}

// --- Export SVG (drawing-export) ---

use crate::export::WAITING;
use crate::fake_typesetter::{Boxes, Manual};

fn pending_svg(fake: &FakeFiles) -> String {
    let (overwrite, suggested, bytes) = fake.save_request();
    assert_eq!(overwrite, None);
    assert_eq!(suggested.as_deref(), Some("untitled.svg"));
    String::from_utf8(bytes).unwrap()
}

#[test]
fn sample_scene_exports_through_the_host() {
    let fake = FakeFiles::new(true);
    let mut app = App::with_host(Box::new(label_layout::Immediate::new(Boxes)), fake.clone());
    app.start_export();
    assert!(!app.export_step(), "typeset at once, nothing to wait for");
    let svg = pending_svg(&fake);
    assert!(svg.starts_with("<?xml"));
    for id in ["seg1", "arc1", "seg2"] {
        assert!(svg.contains(&format!(r#"<g id="{id}">"#)), "{id}");
    }
    assert!(!svg.contains("seg3"), "construction helper left out");
    fake.save_as("figure.svg");
    app.poll_files();
    assert_eq!(app.status, "Exported figure.svg");
    assert_eq!(app.files.file_name(), None, "the drawing keeps no file");
}

#[test]
fn export_waits_while_the_typesetter_loads() {
    let fake = FakeFiles::new(true);
    let ts = Manual::loading();
    let mut app = App::with_host(Box::new(ts.clone()), fake.clone());
    app.start_export();
    assert!(app.export_step());
    assert_eq!(app.status, WAITING);
    assert_eq!(fake.pending_saves(), 0);
    ts.complete();
    assert!(!app.export_step());
    assert_eq!(fake.pending_saves(), 1);
}

#[test]
fn failing_label_is_named_and_nothing_is_saved() {
    let fake = FakeFiles::new(true);
    let mut app = App::with_files(fake.clone());
    let label = app
        .doc()
        .geometries()
        .values()
        .find(|g| app.doc().label_attrs(g.id()).is_some())
        .map(|g| app.doc().geometry_name(g.id()).unwrap().to_owned())
        .unwrap();
    app.start_export();
    app.export_step();
    let err = app.last_error.clone().unwrap();
    assert_eq!(err, format!("Cannot export: {label}: no typesetter available"));
    assert_eq!(fake.pending_saves(), 0);
    assert!(app.export.is_none(), "the export ended");
}

#[test]
fn edit_while_waiting_does_not_change_the_file() {
    let fake = FakeFiles::new(true);
    let ts = Manual::loading();
    let mut app = App::with_host(Box::new(ts.clone()), fake.clone());
    app.start_export();
    assert!(app.export_step());
    apply_json(
        &mut app,
        r#"{"op": "create", "kind": "segment", "name": "late", "start": [0, 0], "end": [1, 1]}"#,
    );
    ts.complete();
    app.export_step();
    assert!(!pending_svg(&fake).contains("late"));
}

#[test]
fn no_export_during_a_drag_or_a_running_export() {
    let fake = FakeFiles::new(true);
    let mut app = App::with_host(Box::new(Manual::loading()), fake.clone());
    app.session.drag_begin("seg1.start").unwrap();
    app.start_export();
    assert!(app.export.is_none());
    app.session.drag_cancel();
    app.start_export();
    assert!(app.export.is_some());
    app.status.clear();
    app.start_export();
    assert!(app.status.is_empty(), "a second export does not start");
}
