use std::cell::{Cell, RefCell};
use std::rc::Rc;

use constraint_runtime::{Document, Markup, Mesh, MeshFilter};
use eframe::egui::Pos2;
use label_layout::{
    Contour, Immediate, LineRequest, LineResult, LineTypesetter, Paint, Segment, Shape,
    TypesetError, TypesetLine, Typesetter, TypesetterState,
};
use sketch_session::{Command, Session};

use super::*;

/// Each line a filled 5·n × 7 pt rectangle from its origin (depth 2); fails
/// on `\foo`; counts its calls.
struct Fake(Rc<Cell<usize>>);

impl LineTypesetter for Fake {
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

/// A cache with the typesetter it is painted with.
struct Env {
    cache: LabelCache,
    ts: Option<Box<dyn Typesetter>>,
}

impl Env {
    fn none() -> Self {
        Env {
            cache: LabelCache::new(),
            ts: None,
        }
    }

    fn paint(&mut self, doc: &Document, vp: &Viewport, filter: MeshFilter) -> Vec<PaintItem> {
        label_paint(doc, vp, filter, &mut self.cache, self.ts.as_deref())
    }
}

/// The counting fake behind an immediate port.
fn fake() -> (Env, Rc<Cell<usize>>) {
    let calls = Rc::new(Cell::new(0));
    let env = Env {
        cache: LabelCache::new(),
        ts: Some(Box::new(Immediate::new(Fake(calls.clone())))),
    };
    (env, calls)
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
    let items = cache.paint(doc, &vp, MeshFilter::RESULT_ONLY);
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
    let extent = |scale: f64, cache: &mut Env| {
        let vp = viewport(scale);
        let items = cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
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
    assert!(cache
        .paint(s.document(), &vp, MeshFilter::RESULT_ONLY)
        .is_empty());
    assert_eq!(
        cache
            .paint(s.document(), &vp, MeshFilter::INCLUDE_CONSTRUCTION)
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
    let items = cache.paint(doc, &viewport(100.0), MeshFilter::RESULT_ONLY);
    assert_eq!(items.len(), 1);
}

#[test]
fn unchanged_lines_are_typeset_once() {
    let s = joined(r"a\\b", "result");
    let (mut cache, calls) = fake();
    let vp = viewport(100.0);
    cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert_eq!(calls.get(), 2, "two lines, typeset once each");
}

#[test]
fn unused_lines_are_dropped() {
    let mut s = joined("a", "result");
    let (mut cache, calls) = fake();
    let vp = viewport(100.0);
    cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    s.apply(
        Command::from_json(r#"{"op": "set", "kind": "label", "name": "t", "source": "b"}"#)
            .unwrap(),
    )
    .unwrap();
    cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert_eq!(cache.cache.lines.len(), 1);
    s.undo().unwrap();
    cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert_eq!(calls.get(), 3, "`a` was dropped and typeset again");
}

#[test]
fn failure_is_marked_in_place() {
    let s = joined(r"$\foo$", "result");
    let (mut cache, _) = fake();
    let vp = viewport(100.0);
    let items = cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    match &items[..] {
        [PaintItem::Fallback { lines, reason, .. }] => {
            assert_eq!(
                reason,
                &FallbackReason::Failed(TypesetError::Unsupported {
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
    let mut cache = Env::none();
    let vp = viewport(100.0);
    let items = cache.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    match &items[..] {
        [PaintItem::Fallback {
            lines,
            reason: FallbackReason::NoTypesetter,
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

// --- Asynchronous typesetting: pending state ---

/// An asynchronous fake: requests wait until `complete` answers them with
/// [`Fake`]; counts requests.
struct Manual {
    state: RefCell<TypesetterState>,
    waiting: RefCell<Vec<LineRequest>>,
    done: RefCell<Vec<(LineRequest, LineResult)>>,
    requests: Cell<usize>,
}

impl Manual {
    fn new(state: TypesetterState) -> Rc<Self> {
        Rc::new(Manual {
            state: RefCell::new(state),
            waiting: RefCell::new(Vec::new()),
            done: RefCell::new(Vec::new()),
            requests: Cell::new(0),
        })
    }

    /// Answer every waiting request.
    fn complete(&self) {
        let engine = Fake(Rc::new(Cell::new(0)));
        for req in self.waiting.borrow_mut().drain(..) {
            let r = engine.typeset_line(&req.line, req.markup, req.size_pt(), &req.preamble);
            self.done.borrow_mut().push((req, r));
        }
        *self.state.borrow_mut() = TypesetterState::Ready;
    }
}

struct Shared(Rc<Manual>);

impl Typesetter for Shared {
    fn request(&self, req: LineRequest) {
        self.0.requests.set(self.0.requests.get() + 1);
        self.0.waiting.borrow_mut().push(req);
    }
    fn take_completed(&self) -> Vec<(LineRequest, LineResult)> {
        std::mem::take(&mut self.0.done.borrow_mut())
    }
    fn state(&self) -> TypesetterState {
        self.0.state.borrow().clone()
    }
}

fn manual(state: TypesetterState) -> (Env, Rc<Manual>) {
    let m = Manual::new(state);
    let env = Env {
        cache: LabelCache::new(),
        ts: Some(Box::new(Shared(m.clone()))),
    };
    (env, m)
}

fn set_source(s: &mut Session, source: &str) {
    let cmd = format!(
        r#"{{"op": "set", "kind": "label", "name": "t", "source": {}}}"#,
        serde_json::to_string(source).unwrap()
    );
    s.apply(Command::from_json(&cmd).unwrap()).unwrap();
}

#[test]
fn typesetter_still_loading() {
    let s = joined("ab", "result");
    let (mut env, _) = manual(TypesetterState::Loading);
    let items = env.paint(s.document(), &viewport(100.0), MeshFilter::RESULT_ONLY);
    assert!(
        matches!(
            &items[..],
            [PaintItem::Fallback {
                reason: FallbackReason::Pending,
                ..
            }]
        ),
        "{items:?}"
    );
    assert!(env.cache.any_pending());
}

#[test]
fn changed_line_keeps_the_last_rendering() {
    let mut s = joined("ab", "result");
    let (mut env, m) = manual(TypesetterState::Ready);
    let vp = viewport(100.0);
    env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    m.complete();
    let first = env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert!(!env.cache.any_pending());
    set_source(&mut s, "abcd");
    let while_pending = env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert_eq!(
        while_pending, first,
        "previous rendering while the new line is pending"
    );
    assert!(env.cache.any_pending());
    m.complete();
    let after = env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert_ne!(typeset_vertices(&after), typeset_vertices(&first));
}

#[test]
fn label_moves_while_pending() {
    let mut s = joined("ab", "result");
    let (mut env, m) = manual(TypesetterState::Ready);
    let vp = viewport(100.0);
    env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    m.complete();
    env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    set_source(&mut s, "abcd");
    s.drag_begin("seg1.end").unwrap();
    s.drag_to(Point2::new(1.0, 1.0)).unwrap();
    s.drag_end(Point2::new(1.0, 1.0)).unwrap();
    let doc = s.document();
    let items = env.paint(doc, &vp, MeshFilter::RESULT_ONLY);
    let end = match doc.resolve_address("seg1.end").unwrap() {
        constraint_runtime::Address::Point(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{other:?}"),
    };
    let corner = vp.to_screen(end);
    assert!(env.cache.any_pending());
    assert!(
        typeset_vertices(&items).iter().any(|v| near(*v, corner)),
        "{items:?}"
    );
}

#[test]
fn immediate_typesetter_shows_no_pending_form() {
    let s = joined(r"a\\b", "result");
    let (mut env, _) = fake();
    let items = env.paint(s.document(), &viewport(100.0), MeshFilter::RESULT_ONLY);
    assert!(
        matches!(&items[..], [PaintItem::Typeset { .. }]),
        "{items:?}"
    );
    assert!(!env.cache.any_pending());
}

#[test]
fn typesetter_fails_to_load() {
    let mut s = joined("ab", "result");
    s.apply(
        Command::from_json(
            r#"{"op": "create", "kind": "label", "origin": [0, 1], "heading": 0, "source": "c"}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let (mut env, m) = manual(TypesetterState::Failed("offline".into()));
    let items = env.paint(s.document(), &viewport(100.0), MeshFilter::RESULT_ONLY);
    assert_eq!(items.len(), 2);
    for item in &items {
        assert!(
            matches!(item, PaintItem::Fallback { reason: FallbackReason::LoadFailed(msg), .. } if msg == "offline"),
            "{item:?}"
        );
    }
    assert_eq!(m.requests.get(), 0, "no requests to a failed typesetter");
}

#[test]
fn a_pending_line_is_requested_once() {
    let s = joined("ab", "result");
    let (mut env, m) = manual(TypesetterState::Loading);
    let vp = viewport(100.0);
    env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert_eq!(m.requests.get(), 1);
}

// --- Lines for an export (svg-export D8) ---

fn req(line: &str) -> LineRequest {
    LineRequest::new(line, Markup::Latex, 10.0, "")
}

#[test]
fn ensured_lines_survive_the_sweep() {
    let s = joined("a", "result");
    let (mut env, calls) = fake();
    let vp = viewport(100.0);
    let ts = env.ts.take().unwrap();
    // A line no label of the drawing uses (the drawing changed since the export started).
    let gone = [req("old")];
    let first = env.cache.ensure(ts.as_ref(), &gone).unwrap().unwrap();
    assert_eq!(first[0].width, 15.0);
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut env.cache, Some(ts.as_ref()));
    let again = env.cache.ensure(ts.as_ref(), &gone).unwrap().unwrap();
    assert!(Rc::ptr_eq(&first[0], &again[0]), "kept, not typeset again");
    assert_eq!(calls.get(), 2, "`old` and the label's `a`, once each");
    // Without `ensure` in a frame, the line is dropped at the next sweep.
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut env.cache, Some(ts.as_ref()));
    label_paint(s.document(), &vp, MeshFilter::RESULT_ONLY, &mut env.cache, Some(ts.as_ref()));
    assert!(!env.cache.lines.contains_key(&gone[0]));
}

#[test]
fn canvas_and_export_share_results() {
    let s = joined("ab", "result");
    let (mut env, m) = manual(TypesetterState::Loading);
    let vp = viewport(100.0);
    env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    let attrs = s.document().label_attrs(s.document().geometries().values().nth(1).unwrap().id()).unwrap();
    let reqs = label_requests(attrs, s.document().settings());
    let ts = env.ts.take().unwrap();
    assert_eq!(env.cache.ensure(ts.as_ref(), &reqs), None, "pending");
    assert_eq!(m.requests.get(), 1, "the export reuses the Canvas's request");
    m.complete();
    let lines = env.cache.ensure(ts.as_ref(), &reqs).unwrap().unwrap();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].width, 10.0);
    env.ts = Some(ts);
    let items = env.paint(s.document(), &vp, MeshFilter::RESULT_ONLY);
    assert!(matches!(&items[..], [PaintItem::Typeset { .. }]), "{items:?}");
}

#[test]
fn ensure_reports_a_failure() {
    let (mut env, _) = fake();
    let ts = env.ts.take().unwrap();
    let err = env.cache.ensure(ts.as_ref(), &[req("x"), req(r"\foo")]).unwrap().unwrap_err();
    assert!(matches!(err, TypesetError::Unsupported { .. }), "{err:?}");
}
