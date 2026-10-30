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
