use constraint_runtime::Point2;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct P(#[serde(with = "point")] Point2);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct F(#[serde(with = "finite")] f64);

fn bits(p: Point2) -> (u64, u64) {
    (p.x.to_bits(), p.y.to_bits())
}

#[test]
fn point_round_trips_bit_for_bit() {
    for v in [0.1, 1.0 / 3.0, 1e-300, -0.0, f64::MAX, f64::MIN_POSITIVE] {
        let p = Point2::new(v, -v);
        let text = serde_json::to_string(&P(p)).unwrap();
        let back: P = serde_json::from_str(&text).unwrap();
        assert_eq!(bits(back.0), bits(p), "{text}");
    }
}

#[test]
fn point_is_a_pair() {
    let P(p) = serde_json::from_str("[3, -1.5]").unwrap();
    assert_eq!(p, Point2::new(3.0, -1.5));
    assert_eq!(serde_json::to_string(&P(p)).unwrap(), "[3.0,-1.5]");
}

#[test]
fn point_rejects_other_shapes() {
    for text in [
        "[0,0,0]",
        "[0]",
        "[]",
        "\"x\"",
        "{\"x\":0,\"y\":0}",
        "[0,null]",
    ] {
        assert!(serde_json::from_str::<P>(text).is_err(), "{text}");
    }
}

#[test]
fn non_finite_numbers_are_not_written() {
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(serde_json::to_string(&P(Point2::new(v, 0.0))).is_err());
        assert!(serde_json::to_string(&P(Point2::new(0.0, v))).is_err());
        assert!(serde_json::to_string(&F(v)).is_err());
    }
}

#[test]
fn out_of_range_numbers_are_not_read() {
    assert!(serde_json::from_str::<P>("[1e999, 0]").is_err());
    assert!(serde_json::from_str::<F>("-1e999").is_err());
    assert!(serde_json::from_str::<F>("NaN").is_err());
}

#[test]
fn addresses_are_plain_strings() {
    let a: PointAddr = serde_json::from_str("\"seg1.end\"").unwrap();
    assert_eq!(a.as_str(), "seg1.end");
    assert_eq!(
        serde_json::to_string(&VecAddr::from("seg1.direction")).unwrap(),
        "\"seg1.direction\""
    );
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Size {
    #[serde(default, skip_serializing_if = "Option::is_none", with = "finite_or_null")]
    size: Option<Option<f64>>,
}

#[test]
fn size_absent_null_or_number() {
    let read = |t: &str| serde_json::from_str::<Size>(t).unwrap().size;
    assert_eq!(read("{}"), None);
    assert_eq!(read(r#"{"size": null}"#), Some(None));
    assert_eq!(read(r#"{"size": 8}"#), Some(Some(8.0)));
    for size in [None, Some(None), Some(Some(8.5))] {
        let text = serde_json::to_string(&Size { size }).unwrap();
        assert_eq!(read(&text), size, "{text}");
    }
    assert!(serde_json::to_string(&Size { size: Some(Some(f64::NAN)) }).is_err());
}

#[test]
fn label_enums_use_lowercase_tags() {
    assert_eq!(serde_json::to_string(&WireMarkup::Latex).unwrap(), "\"latex\"");
    assert_eq!(serde_json::to_string(&WireHAlign::Center).unwrap(), "\"center\"");
    assert_eq!(serde_json::to_string(&WireVAlign::Last).unwrap(), "\"last\"");
    assert_eq!(serde_json::from_str::<WireMarkup>("\"text\"").unwrap(), WireMarkup::Text);
    for bad in ["\"Latex\"", "\"LEFT\"", "\"middle\""] {
        assert!(serde_json::from_str::<WireMarkup>(bad).is_err(), "{bad}");
        assert!(serde_json::from_str::<WireHAlign>(bad).is_err(), "{bad}");
        assert!(serde_json::from_str::<WireVAlign>(bad).is_err(), "{bad}");
    }
}
