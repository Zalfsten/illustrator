use constraint_runtime::Markup;

use super::*;

/// Width = number of characters; counts calls.
struct Count(std::cell::Cell<usize>);

impl LineTypesetter for Count {
    fn typeset_line(&self, line: &str, _: Markup, _: f64, _: &str) -> LineResult {
        self.0.set(self.0.get() + 1);
        Ok(TypesetLine {
            width: line.chars().count() as f64,
            ..TypesetLine::default()
        })
    }
}

#[test]
fn immediate_is_ready_and_delivers_in_the_same_frame() {
    let t = Immediate::new(Count(Default::default()));
    assert_eq!(t.state(), TypesetterState::Ready);
    let req = LineRequest::new("abc", Markup::Text, 10.0, "");
    t.request(req.clone());
    let done = t.take_completed();
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].0, req);
    assert_eq!(done[0].1.as_ref().unwrap().width, 3.0);
    assert!(t.take_completed().is_empty(), "delivered once");
}

#[test]
fn request_keys_compare_by_size_bits() {
    let a = LineRequest::new("x", Markup::Latex, 10.0, "");
    assert_eq!(a.size_pt(), 10.0);
    assert_ne!(a, LineRequest::new("x", Markup::Latex, 10.5, ""));
    assert_ne!(a, LineRequest::new("x", Markup::Text, 10.0, ""));
    assert_ne!(a, LineRequest::new("x", Markup::Latex, 10.0, r"\def\a{b}"));
}

#[test]
fn label_requests_split_size_and_preamble() {
    use constraint_runtime::{DocumentSettings, LabelAttrs};
    let settings = DocumentSettings {
        font_size: 12.0,
        preamble: r"\newcommand{\R}{\mathbb{R}}".into(),
        ..DocumentSettings::default()
    };
    let latex = LabelAttrs {
        source: r"leg \\ $\R$".into(),
        size: Some(8.0),
        ..LabelAttrs::default()
    };
    assert_eq!(
        label_requests(&latex, &settings),
        crate::split_lines(&latex.source, Markup::Latex)
            .into_iter()
            .map(|l| LineRequest::new(l, Markup::Latex, 8.0, &settings.preamble))
            .collect::<Vec<_>>()
    );
    assert_eq!(label_requests(&latex, &settings).len(), 2);
    let text = LabelAttrs {
        source: "a\nb\nc".into(),
        markup: Markup::Text,
        ..LabelAttrs::default()
    };
    let reqs = label_requests(&text, &settings);
    assert_eq!(
        reqs,
        ["a", "b", "c"].map(|l| LineRequest::new(l, Markup::Text, 12.0, &settings.preamble))
    );
    let empty = LabelAttrs::default();
    assert!(label_requests(&empty, &settings).is_empty());
}
