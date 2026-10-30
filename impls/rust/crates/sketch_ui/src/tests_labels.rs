use std::cell::Cell;
use std::rc::Rc;

use constraint_runtime::{Document, Markup, Mesh, MeshFilter};
use eframe::egui::Pos2;
use label_layout::{Contour, Paint, Segment, Shape, TypesetError, TypesetLine, Typesetter};
use sketch_session::{Command, Session};

use super::*;

/// Each line a filled 5·n × 7 pt rectangle from its origin (depth 2); fails
/// on `\foo`; counts its calls.
struct Fake(Rc<Cell<usize>>);

impl Typesetter for Fake {
    fn typeset_line(
        &self,
        line: &str,
        _: Markup,
        _: f64,
        _: &str,
    ) -> Result<TypesetLine, TypesetError> {
        self.0.set(self.0.get() + 1);
        if line.contains(r"\foo") {
            return Err(TypesetError::Unsupported {
                construct: r"\foo".into(),
            });
        }
        let w = 5.0 * line.chars().count() as f64;
        let p = Point2::new;
        Ok(TypesetLine {
            shapes: vec![Shape {
                contours: vec![Contour(vec![
                    Segment::MoveTo(p(0.0, 0.0)),
                    Segment::LineTo(p(w, 0.0)),
                    Segment::LineTo(p(w, 7.0)),
                    Segment::LineTo(p(0.0, 7.0)),
                    Segment::Close,
                ])],
                paint: Paint::Fill,
            }],
            width: w,
            height: 7.0,
            depth: 2.0,
        })
    }
}

fn session(commands: &[&str]) -> Session {
    let mut s = Session::new(Document::new());
    for c in commands {
        s.apply(Command::from_json(c).unwrap()).unwrap();
    }
    s
}

/// `seg1` (0,0)→(1,0) and label `t` joined to `seg1.end`, parallel to it.
fn joined(source: &str, role: &str) -> Session {
    session(&[
        r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0]}"#,
        &format!(
            r#"{{"op": "create", "kind": "label", "name": "t", "role": "{role}", "origin": [1, 0], "heading": 0, "source": {}}}"#,
            serde_json::to_string(source).unwrap()
        ),
        r#"{"op": "constrain", "kind": "join", "a": "t.origin", "b": "seg1.end"}"#,
        r#"{"op": "constrain", "kind": "parallel", "a": "t.direction", "b": "seg1.direction"}"#,
    ])
}

fn viewport(scale: f64) -> Viewport {
    Viewport::new(scale, Pos2::new(100.0, 100.0))
}

fn fake() -> (LabelCache, Rc<Cell<usize>>) {
    let calls = Rc::new(Cell::new(0));
    (LabelCache::new(Some(Box::new(Fake(calls.clone())))), calls)
}

fn typeset_vertices(items: &[PaintItem]) -> &[Pos2] {
    match items {
        [PaintItem::Typeset { vertices, .. }] => vertices,
        other => panic!("{other:?}"),
    }
}

fn near(a: Pos2, b: Pos2) -> bool {
    (a - b).length() < 1e-3
}

#[test]
fn label_follows_its_geometry() {
    let mut s = joined("ab", "result");
    let (mut cache, _) = fake();
    let vp = viewport(100.0);
    s.drag_begin("seg1.end").unwrap();
    s.drag_to(Point2::new(1.0, 1.0)).unwrap();
    s.drag_end(Point2::new(1.0, 1.0)).unwrap();
    let doc = s.document();
    let items = label_paint(doc, &vp, MeshFilter::RESULT_ONLY, &mut cache);
    let end = match doc.resolve_address("seg1.end").unwrap() {
        constraint_runtime::Address::Point(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{other:?}"),
    };
    // The rectangle's corner (0, 0) pt is the label origin.
    let corner = vp.to_screen(end);
    assert!(
        typeset_vertices(&items).iter().any(|v| near(*v, corner)),
        "{items:?} {corner:?}"
    );
}

#[test]
fn zoom_keeps_proportions() {
    let s = joined("ab", "result");
    let (mut cache, _) = fake();
    let extent = |scale: f64, cache: &mut LabelCache| {
        let vp = viewport(scale);
        let items = label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, cache);
        let v = typeset_vertices(&items);
        let (min, max) = v
            .iter()
            .fold((v[0], v[0]), |(a, b), p| (a.min(*p), b.max(*p)));
        // Label width, and distance of its far corner from seg1.start.
        (
            (max.x - min.x),
            (max - vp.to_screen(Point2::new(0.0, 0.0))).length(),
        )
    };
    let (w1, d1) = extent(100.0, &mut cache);
    let (w2, d2) = extent(200.0, &mut cache);
    assert!(
        (w2 / w1 - 2.0).abs() < 1e-4 && (d2 / d1 - 2.0).abs() < 1e-4,
        "{w1} {w2} {d1} {d2}"
    );
}

#[test]
fn hidden_construction_label() {
    let s = joined("ab", "construction");
    let (mut cache, _) = fake();
    let vp = viewport(100.0);
    assert!(label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache).is_empty());
    assert_eq!(
        label_paint(
            s.document(),
            &vp,
            MeshFilter::INCLUDE_CONSTRUCTION,
            &mut cache
        )
        .len(),
        1
    );
}

#[test]
fn labels_are_painted_beside_the_mesh() {
    let s = joined("ab", "result");
    let (mut cache, _) = fake();
    let doc = s.document();
    let mesh = Mesh::from_document(doc, doc.vector(), MeshFilter::RESULT_ONLY);
    assert_eq!(mesh.edges.len(), 1);
    let items = label_paint(doc, &viewport(100.0), MeshFilter::RESULT_ONLY, &mut cache);
    assert_eq!(items.len(), 1);
}

#[test]
fn unchanged_lines_are_typeset_once() {
    let s = joined(r"a\\b", "result");
    let (mut cache, calls) = fake();
    let vp = viewport(100.0);
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    assert_eq!(calls.get(), 2, "two lines, typeset once each");
}

#[test]
fn unused_lines_are_dropped() {
    let mut s = joined("a", "result");
    let (mut cache, calls) = fake();
    let vp = viewport(100.0);
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    s.apply(
        Command::from_json(r#"{"op": "set", "kind": "label", "name": "t", "source": "b"}"#)
            .unwrap(),
    )
    .unwrap();
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    assert_eq!(cache.lines.len(), 1);
    s.undo().unwrap();
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    assert_eq!(calls.get(), 3, "`a` was dropped and typeset again");
}

#[test]
fn failure_is_marked_in_place() {
    let s = joined(r"$\foo$", "result");
    let (mut cache, _) = fake();
    let vp = viewport(100.0);
    let items = label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    match &items[..] {
        [PaintItem::Fallback { lines, error, .. }] => {
            assert_eq!(
                error,
                &Some(TypesetError::Unsupported {
                    construct: r"\foo".into()
                })
            );
            assert_eq!(lines[0].text, r"$\foo$");
            assert!(near(
                lines[0].baseline_left,
                vp.to_screen(Point2::new(1.0, 0.0))
            ));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn no_typesetter_available() {
    let s = joined(r"a\\b", "result");
    let mut cache = LabelCache::new(None);
    let vp = viewport(100.0);
    let items = label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut cache);
    match &items[..] {
        [PaintItem::Fallback {
            lines,
            error: None,
            frame,
            ..
        }] => {
            assert_eq!(
                lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(),
                ["a", "b"]
            );
            let origin = vp.to_screen(Point2::new(1.0, 0.0));
            assert!(near(lines[0].baseline_left, origin));
            // The second line is below the first (screen y grows downward).
            assert!(lines[1].baseline_left.y > lines[0].baseline_left.y);
            assert!(frame[0].y > frame[3].y, "{frame:?}");
        }
        other => panic!("{other:?}"),
    }
}
