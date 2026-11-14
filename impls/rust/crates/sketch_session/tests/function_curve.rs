//! "Done when" of #81: a sine curve is created, joined, made parallel, solved,
//! dragged, saved, loaded and exported through the public API.

use constraint_runtime::{Address, Point2, SOLVE_TOL};
use sketch_export::{bounds, export, scene, Format, LabelLines, Part, PathSeg};
use sketch_session::{Command, Session};

fn cmd(text: &str) -> Command {
    Command::from_json(text).unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn point(s: &Session, address: &str) -> Point2 {
    let doc = s.document();
    match doc.resolve_address(address).unwrap() {
        Address::Point(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{address}: {other:?}"),
    }
}

fn coincide(s: &Session, a: &str, b: &str) {
    let (p, q) = (point(s, a), point(s, b));
    assert!((p - q).length() < SOLVE_TOL, "{a} {p:?} vs {b} {q:?}");
}

fn build() -> Session {
    let mut s = Session::new(constraint_runtime::Document::new());
    for text in [
        r#"{"op": "create", "kind": "function", "origin": [0, 0], "heading": 0, "expression": "sin(x)", "range": [0, 6.283185307179586]}"#,
        r#"{"op": "create", "kind": "segment", "start": [-1, 0.5], "end": [0.1, 0.1]}"#,
        r#"{"op": "create", "kind": "segment", "start": [6.2, 0.2], "end": [7, 1]}"#,
        r#"{"op": "create", "kind": "segment", "start": [0, -1], "end": [2, -1.3]}"#,
        r#"{"op": "constrain", "kind": "join", "a": "fn1.start", "b": "seg1.end"}"#,
        r#"{"op": "constrain", "kind": "join", "a": "fn1.end", "b": "seg2.start"}"#,
        r#"{"op": "constrain", "kind": "join", "a": "fn1.origin", "b": "seg3.start"}"#,
        r#"{"op": "constrain", "kind": "parallel", "a": "seg3.direction", "b": "fn1.direction"}"#,
    ] {
        s.apply(cmd(text)).unwrap_or_else(|e| panic!("{text}: {e}"));
    }
    s
}

fn check_constraints(s: &Session) {
    coincide(s, "fn1.start", "seg1.end");
    coincide(s, "fn1.end", "seg2.start");
    coincide(s, "fn1.origin", "seg3.start");
    let doc = s.document();
    let dir = |a: &str| match doc.resolve_address(a).unwrap() {
        Address::Vector(e) => e.evaluate(doc, doc.vector()),
        other => panic!("{other:?}"),
    };
    let (u, v) = (dir("seg3.direction"), dir("fn1.direction"));
    assert!(u.cross(v).abs() < SOLVE_TOL * u.length(), "parallel: {u:?} vs {v:?}");
}

#[test]
fn sine_curve_end_to_end() {
    let mut s = build();
    check_constraints(&s);

    // Drag a joined segment end: the curve follows as a rigid frame.
    let target = Point2::new(8.0, 2.0);
    s.drag_begin("seg2.end").unwrap();
    s.drag_to(Point2::new(7.5, 1.5)).unwrap();
    s.drag_end(target).unwrap();
    check_constraints(&s);
    assert!((point(&s, "seg2.end") - target).length() < 0.5, "the drag moved seg2.end toward the target");

    // Save and load.
    let bytes = s.save().unwrap();
    let (t, report) = Session::load(&bytes).unwrap();
    assert!(report.conflicts.is_empty());
    check_constraints(&t);
    for a in ["fn1.start", "fn1.end", "fn1.origin", "seg2.end"] {
        assert!((point(&s, a) - point(&t, a)).length() < SOLVE_TOL, "{a} after load");
    }

    // Export: the curve is written as cubic pieces inside the tight bounds.
    let doc = t.document();
    let labels = LabelLines::new();
    let sc = scene(doc, &labels).unwrap();
    let b = bounds(&sc).unwrap();
    let curve = sc.objects.iter().find(|o| o.name == "fn1").expect("fn1 is exported");
    let Part::Stroke { path, width, .. } = &curve.parts[0] else {
        panic!("a stroke expected: {:?}", curve.parts[0]);
    };
    let cubics = path.iter().filter(|p| matches!(p, PathSeg::Cubic(..))).count();
    assert!(cubics >= 4 && cubics == path.len() - 1, "{cubics} cubic pieces of {}", path.len());
    let inside = |q: Point2| {
        let m = 0.5 * width + 1e-9;
        q.x >= b.min.x - m && q.x <= b.max.x + m && q.y >= b.min.y - m && q.y <= b.max.y + m
    };
    for seg in path {
        if let PathSeg::Move(q) | PathSeg::Cubic(_, _, q) = *seg {
            assert!(inside(q), "{q:?} outside {b:?}");
        }
    }
    let svg = String::from_utf8(export(doc, &labels, Format::Svg).unwrap()).unwrap();
    assert!(svg.contains("<g id=\"fn1\">") && svg.contains(" C "), "SVG has the curve as cubics");
    let pdf = export(doc, &labels, Format::Pdf).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
}
