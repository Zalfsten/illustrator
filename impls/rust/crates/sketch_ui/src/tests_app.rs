use serde_json::Value;
use sketch_session::{DeclCommand, Session};

use super::*;

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
