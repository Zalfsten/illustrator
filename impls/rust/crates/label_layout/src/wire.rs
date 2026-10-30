//! JSON messages between the app and a typesetting worker, through mirror
//! types: `constraint_runtime` stays serde-free, and the format is decided
//! here rather than derived from the core types.
//!
//! App -> worker: [`WireRequest`]. Worker -> app: [`WorkerMessage`] (`ready`
//! once, then one `line` per request). Points are `[x, y]`; numbers must be
//! finite (writing NaN or infinity is an error, never `null`).

use constraint_runtime::{Markup, Point2};
use serde::{Deserialize, Serialize};

use crate::port::{LineRequest, LineResult};
use crate::typeset::{Contour, Paint, Segment, Shape, TypesetError, TypesetLine};

/// One line to typeset, with the id its answer carries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRequest {
    pub id: u32,
    pub line: String,
    pub markup: WireMarkup,
    #[serde(with = "finite")]
    pub size: f64,
    pub preamble: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireMarkup {
    Latex,
    Text,
}

/// Worker -> app.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerMessage {
    /// The typesetter is initialized; requests are answered from now on.
    Ready {},
    /// The answer to request `id`.
    Line { id: u32, result: WireResult },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireResult {
    Ok(WireLine),
    Err(WireError),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireError {
    Unsupported(String),
    Invalid(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireLine {
    #[serde(with = "finite")]
    pub width: f64,
    #[serde(with = "finite")]
    pub height: f64,
    #[serde(with = "finite")]
    pub depth: f64,
    pub shapes: Vec<WireShape>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireShape {
    pub paint: WirePaint,
    pub contours: Vec<Vec<WireSegment>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WirePaint {
    Fill {},
    Stroke {
        #[serde(with = "finite")]
        width: f64,
    },
}

/// A path command; locations are `[x, y]` in pt.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WireSegment {
    Move(#[serde(with = "point")] Point2),
    Line(#[serde(with = "point")] Point2),
    Quad(
        #[serde(with = "point")] Point2,
        #[serde(with = "point")] Point2,
    ),
    Cubic(
        #[serde(with = "point")] Point2,
        #[serde(with = "point")] Point2,
        #[serde(with = "point")] Point2,
    ),
    Close,
}

impl WireRequest {
    pub fn new(id: u32, req: &LineRequest) -> Self {
        Self {
            id,
            line: req.line.clone(),
            markup: match req.markup {
                Markup::Latex => WireMarkup::Latex,
                Markup::Text => WireMarkup::Text,
            },
            size: req.size_pt(),
            preamble: req.preamble.clone(),
        }
    }

    pub fn request(&self) -> LineRequest {
        let markup = match self.markup {
            WireMarkup::Latex => Markup::Latex,
            WireMarkup::Text => Markup::Text,
        };
        LineRequest::new(&self.line, markup, self.size, &self.preamble)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_json(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text)
    }
}

impl WorkerMessage {
    /// The answer to request `id`.
    pub fn line(id: u32, result: &LineResult) -> Self {
        WorkerMessage::Line {
            id,
            result: match result {
                Ok(line) => WireResult::Ok(WireLine::from(line)),
                Err(TypesetError::Unsupported { construct }) => {
                    WireResult::Err(WireError::Unsupported(construct.clone()))
                }
                Err(TypesetError::Invalid { message }) => {
                    WireResult::Err(WireError::Invalid(message.clone()))
                }
            },
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn from_json(text: &str) -> serde_json::Result<Self> {
        serde_json::from_str(text)
    }
}

impl From<WireResult> for LineResult {
    fn from(r: WireResult) -> Self {
        match r {
            WireResult::Ok(line) => Ok(line.into()),
            WireResult::Err(WireError::Unsupported(construct)) => {
                Err(TypesetError::Unsupported { construct })
            }
            WireResult::Err(WireError::Invalid(message)) => Err(TypesetError::Invalid { message }),
        }
    }
}

impl From<&TypesetLine> for WireLine {
    fn from(l: &TypesetLine) -> Self {
        WireLine {
            width: l.width,
            height: l.height,
            depth: l.depth,
            shapes: l
                .shapes
                .iter()
                .map(|s| WireShape {
                    paint: match s.paint {
                        Paint::Fill => WirePaint::Fill {},
                        Paint::Stroke { width } => WirePaint::Stroke { width },
                    },
                    contours: s
                        .contours
                        .iter()
                        .map(|Contour(segs)| segs.iter().map(|&g| g.into()).collect())
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<WireLine> for TypesetLine {
    fn from(l: WireLine) -> Self {
        TypesetLine {
            width: l.width,
            height: l.height,
            depth: l.depth,
            shapes: l
                .shapes
                .into_iter()
                .map(|s| Shape {
                    paint: match s.paint {
                        WirePaint::Fill {} => Paint::Fill,
                        WirePaint::Stroke { width } => Paint::Stroke { width },
                    },
                    contours: s
                        .contours
                        .into_iter()
                        .map(|segs| Contour(segs.into_iter().map(Into::into).collect()))
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<Segment> for WireSegment {
    fn from(s: Segment) -> Self {
        match s {
            Segment::MoveTo(p) => WireSegment::Move(p),
            Segment::LineTo(p) => WireSegment::Line(p),
            Segment::QuadTo(c, p) => WireSegment::Quad(c, p),
            Segment::CubicTo(c1, c2, p) => WireSegment::Cubic(c1, c2, p),
            Segment::Close => WireSegment::Close,
        }
    }
}

impl From<WireSegment> for Segment {
    fn from(s: WireSegment) -> Self {
        match s {
            WireSegment::Move(p) => Segment::MoveTo(p),
            WireSegment::Line(p) => Segment::LineTo(p),
            WireSegment::Quad(c, p) => Segment::QuadTo(c, p),
            WireSegment::Cubic(c1, c2, p) => Segment::CubicTo(c1, c2, p),
            WireSegment::Close => Segment::Close,
        }
    }
}

fn check<E: serde::de::Error>(v: f64) -> Result<f64, E> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(E::custom("non-finite number"))
    }
}

/// `f64` that must be finite (`#[serde(with = "finite")]`).
mod finite {
    use serde::{ser, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
        if !v.is_finite() {
            return Err(ser::Error::custom("non-finite number"));
        }
        v.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        super::check(f64::deserialize(d)?)
    }
}

/// [`Point2`] as `[x, y]`, both finite (`#[serde(with = "point")]`).
mod point {
    use constraint_runtime::Point2;
    use serde::{ser, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(p: &Point2, s: S) -> Result<S::Ok, S::Error> {
        if !(p.x.is_finite() && p.y.is_finite()) {
            return Err(ser::Error::custom("non-finite number"));
        }
        [p.x, p.y].serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Point2, D::Error> {
        let [x, y] = <[f64; 2]>::deserialize(d)?;
        Ok(Point2::new(super::check(x)?, super::check(y)?))
    }
}

#[cfg(test)]
#[path = "tests_wire.rs"]
mod tests_wire;
