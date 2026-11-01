use label_layout::Immediate;
use sketch_session::{Command, Session};

use super::*;
use crate::fake_typesetter::{Boxes, Manual};

fn doc(commands: &[&str]) -> Document {
    let mut s = Session::new(Document::new());
    for c in commands {
        s.apply(Command::from_json(c).unwrap()).unwrap();
    }
    s.document().clone()
}

const SEG: &str = r#"{"op": "create", "kind": "segment", "start": [0, 0], "end": [1, 0]}"#;

fn label(name: &str, role: &str, source: &str) -> String {
    format!(
        r#"{{"op": "create", "kind": "label", "name": "{name}", "role": "{role}", "origin": [0, 1], "heading": 0, "source": {}}}"#,
        serde_json::to_string(source).unwrap()
    )
}

fn svg(step: JobStep) -> String {
    match step {
        JobStep::Ready(bytes) => String::from_utf8(bytes).unwrap(),
        other => panic!("ready expected, got {other:?}"),
    }
}

#[test]
fn drawing_without_labels_needs_no_typesetter() {
    let job = ExportJob::new(&doc(&[SEG]), Format::Svg);
    let text = svg(job.step(&mut LabelCache::new(), None));
    assert!(text.contains(r#"<g id="seg1">"#));
}

#[test]
fn labels_are_exported_as_outlines() {
    let d = doc(&[SEG, &label("t", "result", "ab")]);
    let ts = Immediate::new(Boxes);
    let text = svg(ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), Some(&ts)));
    assert!(text.contains(r#"<g id="t">"#), "{text}");
    assert!(text.contains(r##"fill="#000000" fill-rule="nonzero""##));
    assert!(!text.contains("<text"));
}

#[test]
fn export_waits_for_labels_being_typeset() {
    let d = doc(&[SEG, &label("t", "result", "ab")]);
    let ts = Manual::loading();
    let mut cache = LabelCache::new();
    let job = ExportJob::new(&d, Format::Svg);
    assert_eq!(job.step(&mut cache, Some(&ts)), JobStep::Waiting);
    assert_eq!(job.step(&mut cache, Some(&ts)), JobStep::Waiting, "requested once, still waiting");
    ts.complete();
    let text = svg(job.step(&mut cache, Some(&ts)));
    assert!(text.contains(r#"<g id="t">"#));
}

#[test]
fn failing_labels_are_named() {
    let d = doc(&[
        SEG,
        &label("good", "result", "ok"),
        &label("lbl3", "result", r"\foo{x}"),
    ]);
    let ts = Immediate::new(Boxes);
    let JobStep::Failed(e) = ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), Some(&ts)) else {
        panic!("failure expected")
    };
    assert!(e.starts_with("Cannot export: lbl3: "), "{e}");
    assert!(e.contains(r"\foo"), "{e}");
    assert!(!e.contains("good"), "{e}");
}

#[test]
fn construction_label_does_not_block() {
    let d = doc(&[SEG, &label("help", "construction", r"\foo")]);
    let ts = Immediate::new(Boxes);
    let text = svg(ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), Some(&ts)));
    assert!(!text.contains("help"));
}

#[test]
fn no_typesetter_or_failed_load_names_the_labels() {
    let d = doc(&[SEG, &label("t", "result", "ab"), &label("u", "result", "cd")]);
    assert_eq!(
        ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), None),
        JobStep::Failed("Cannot export: t: no typesetter available; u: no typesetter available".into())
    );
    struct Broken;
    impl Typesetter for Broken {
        fn request(&self, _: LineRequest) {}
        fn take_completed(&self) -> Vec<(LineRequest, label_layout::LineResult)> {
            Vec::new()
        }
        fn state(&self) -> TypesetterState {
            TypesetterState::Failed("no wasm".into())
        }
    }
    let JobStep::Failed(e) = ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), Some(&Broken)) else {
        panic!("failure expected")
    };
    assert!(e.contains("t: typesetter could not be loaded: no wasm"), "{e}");
}

#[test]
fn empty_label_needs_no_typesetting() {
    let d = doc(&[SEG, &label("t", "result", "")]);
    let text = svg(ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), None));
    assert!(!text.contains(r#"id="t""#));
}

#[test]
fn nothing_visible_is_not_exported() {
    let d = doc(&[r#"{"op": "create", "kind": "segment", "role": "construction", "start": [0, 0], "end": [1, 0]}"#]);
    assert_eq!(
        ExportJob::new(&d, Format::Svg).step(&mut LabelCache::new(), None),
        JobStep::Failed("Nothing to export".into())
    );
}

#[test]
fn pdf_export_waits_and_names_failures_like_svg() {
    let d = doc(&[SEG, &label("t", "result", "ab")]);
    let ts = Manual::loading();
    let mut cache = LabelCache::new();
    let job = ExportJob::new(&d, Format::Pdf);
    assert_eq!(job.format(), Format::Pdf);
    assert_eq!(job.step(&mut cache, Some(&ts)), JobStep::Waiting);
    ts.complete();
    let JobStep::Ready(bytes) = job.step(&mut cache, Some(&ts)) else {
        panic!("ready expected")
    };
    assert!(bytes.starts_with(b"%PDF-"));
    let bad = doc(&[SEG, &label("lbl3", "result", r"\foo")]);
    let JobStep::Failed(e) = ExportJob::new(&bad, Format::Pdf).step(&mut LabelCache::new(), Some(&Immediate::new(Boxes)))
    else {
        panic!("failure expected")
    };
    assert!(e.starts_with("Cannot export: lbl3: "), "{e}");
}
