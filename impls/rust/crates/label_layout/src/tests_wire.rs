use constraint_runtime::{Markup, Point2};

use super::*;

fn p(x: f64, y: f64) -> Point2 {
    Point2::new(x, y)
}

/// Bit-exact comparison (PartialEq equates 0.0 and -0.0).
fn same<T: std::fmt::Debug + PartialEq>(a: &T, b: &T) -> bool {
    a == b && format!("{a:?}") == format!("{b:?}")
}

fn sample_line() -> TypesetLine {
    TypesetLine {
        width: 1.0 / 3.0,
        height: 7.25,
        depth: -0.0,
        shapes: vec![
            Shape {
                paint: Paint::Fill,
                contours: vec![Contour(vec![
                    Segment::MoveTo(p(0.1, -0.0)),
                    Segment::LineTo(p(1e-300, 2.0)),
                    Segment::QuadTo(p(0.5, 0.25), p(1.0 / 3.0, 0.0)),
                    Segment::CubicTo(p(1.0, 2.0), p(3.0, 4.0), p(5.0, 6.0)),
                    Segment::Close,
                ])],
            },
            Shape {
                paint: Paint::Stroke { width: 0.4 },
                contours: vec![Contour(vec![
                    Segment::MoveTo(p(0.0, 3.0)),
                    Segment::LineTo(p(4.0, 3.0)),
                ])],
            },
        ],
    }
}

#[test]
fn request_round_trip_with_preamble_and_unicode() {
    let req = LineRequest::new(
        "Geschwindigkeit $\\vec v$ – ä",
        Markup::Latex,
        1.0 / 3.0,
        "\\newcommand{\\R}{\\mathbb{R}}\n\\def\\a{b}",
    );
    let wire = WireRequest::new(7, &req);
    let back = WireRequest::from_json(&wire.to_json().unwrap()).unwrap();
    assert_eq!(back.id, 7);
    assert_eq!(back.request(), req);
}

#[test]
fn line_round_trip_is_bit_exact() {
    let line = sample_line();
    let msg = WorkerMessage::line(3, &Ok(line.clone()));
    let text = msg.to_json().unwrap();
    let WorkerMessage::Line { id, result } = WorkerMessage::from_json(&text).unwrap() else {
        panic!("{text}");
    };
    assert_eq!(id, 3);
    let back: LineResult = result.into();
    assert!(same(&back.unwrap(), &line), "{text}");
}

#[test]
fn errors_and_ready_round_trip() {
    for err in [
        TypesetError::Unsupported {
            construct: r"\foo".into(),
        },
        TypesetError::Invalid {
            message: "unclosed $".into(),
        },
    ] {
        let text = WorkerMessage::line(1, &Err(err.clone())).to_json().unwrap();
        let WorkerMessage::Line { result, .. } = WorkerMessage::from_json(&text).unwrap() else {
            panic!("{text}");
        };
        assert_eq!(LineResult::from(result), Err(err));
    }
    let ready = WorkerMessage::Ready {}.to_json().unwrap();
    assert_eq!(ready, r#"{"ready":{}}"#);
    assert_eq!(
        WorkerMessage::from_json(&ready).unwrap(),
        WorkerMessage::Ready {}
    );
}

#[test]
fn malformed_messages_are_errors() {
    for text in [
        "",
        r#"{"line":{"id":1,"result":{"ok":{"width":1,"height":1,"depth":0,"shapes":[{"paint":{"fill":{}},"contours":[[{"move":[0,0"#,
        r#"{"bogus":{}}"#,
        r#"{"line":{"id":1,"result":{"ok":{"width":1,"height":1,"depth":0,"shapes":[{"paint":{"fill":{}},"contours":[[{"move":[0,0,0]}]]}]}}}}"#,
        r#"{"line":{"id":1,"result":{"maybe":1}}}"#,
    ] {
        assert!(WorkerMessage::from_json(text).is_err(), "{text}");
    }
    assert!(WireRequest::from_json(
        r#"{"id":1,"line":"x","markup":"latex","size":10,"preamble":"","extra":1}"#
    )
    .is_err());
    assert!(WireRequest::from_json(
        r#"{"id":1,"line":"x","markup":"LaTeX","size":10,"preamble":""}"#
    )
    .is_err());
}

#[test]
fn non_finite_numbers_cannot_be_written() {
    let mut line = sample_line();
    line.width = f64::NAN;
    assert!(WorkerMessage::line(1, &Ok(line)).to_json().is_err());
    let mut line = sample_line();
    line.shapes[0].contours[0].0[0] = Segment::MoveTo(p(f64::INFINITY, 0.0));
    assert!(WorkerMessage::line(1, &Ok(line)).to_json().is_err());
}
