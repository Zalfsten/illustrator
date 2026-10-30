use label_layout::wire::{WireRequest, WorkerMessage};
use label_layout::{LineRequest, TypesetError, TypesetLine, TypesetterState};

use super::*;

fn req(line: &str) -> LineRequest {
    LineRequest::new(line, constraint_runtime::Markup::Latex, 10.0, "")
}

fn answer(id: u32, width: f64) -> String {
    let line = TypesetLine {
        width,
        ..TypesetLine::default()
    };
    WorkerMessage::line(id, &Ok(line)).to_json().unwrap()
}

#[test]
fn requests_wait_for_ready_then_flush_in_order() {
    let mut c = Core::new();
    assert_eq!(c.state(), TypesetterState::Loading);
    assert!(c.request(req("a")).is_none());
    assert!(c.request(req("b")).is_none());
    let flush = c.on_message(&WorkerMessage::Ready {}.to_json().unwrap());
    assert_eq!(c.state(), TypesetterState::Ready);
    let lines: Vec<String> = flush.iter().map(|w: &WireRequest| w.line.clone()).collect();
    assert_eq!(lines, ["a", "b"]);
    // After ready, requests go out at once.
    assert_eq!(c.request(req("c")).map(|w| w.line), Some("c".into()));
}

#[test]
fn answers_are_matched_by_id() {
    let mut c = Core::new();
    c.on_message(&WorkerMessage::Ready {}.to_json().unwrap());
    let a = c.request(req("a")).unwrap();
    let b = c.request(req("bb")).unwrap();
    c.on_message(&answer(b.id, 2.0));
    c.on_message(&answer(a.id, 1.0));
    c.on_message(&answer(999, 9.0)); // unknown id: ignored
    let done = c.take_completed();
    assert_eq!(done.len(), 2);
    assert_eq!(done[0].0, req("bb"));
    assert_eq!(done[0].1.as_ref().unwrap().width, 2.0);
    assert_eq!(done[1].0, req("a"));
    assert!(c.take_completed().is_empty());
}

#[test]
fn errors_are_delivered_as_results() {
    let mut c = Core::new();
    c.on_message(&WorkerMessage::Ready {}.to_json().unwrap());
    let w = c.request(req(r"$\foo$")).unwrap();
    let err = TypesetError::Unsupported {
        construct: r"\foo".into(),
    };
    c.on_message(
        &WorkerMessage::line(w.id, &Err(err.clone()))
            .to_json()
            .unwrap(),
    );
    assert_eq!(c.take_completed()[0].1, Err(err));
}

#[test]
fn a_request_in_flight_is_not_sent_twice() {
    let mut c = Core::new();
    c.on_message(&WorkerMessage::Ready {}.to_json().unwrap());
    let first = c.request(req("a")).unwrap();
    assert!(c.request(req("a")).is_none());
    c.on_message(&answer(first.id, 1.0));
    // Answered: a new request for the same line goes out again.
    assert!(c.request(req("a")).is_some());
}

#[test]
fn failure_stops_everything() {
    let mut c = Core::new();
    c.request(req("a"));
    c.on_error("404");
    assert_eq!(c.state(), TypesetterState::Failed("404".into()));
    assert!(c.request(req("b")).is_none());
    assert!(c
        .on_message(&WorkerMessage::Ready {}.to_json().unwrap())
        .is_empty());
    assert_eq!(c.state(), TypesetterState::Failed("404".into()));
    assert!(c.take_completed().is_empty());
}

#[test]
fn unreadable_messages_are_ignored() {
    let mut c = Core::new();
    assert!(c.on_message("not json").is_empty());
    assert_eq!(c.state(), TypesetterState::Loading);
}
