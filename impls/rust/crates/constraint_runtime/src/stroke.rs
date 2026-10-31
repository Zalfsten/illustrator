//! Stroke geometry of a styled curve (styling spec): physical sizes in
//! world units, arrow heads on the exact curve, trimming and dashing by arc
//! length. Pure functions, shared by the Canvas and future exports.

use crate::document::{Document, DocumentSettings};
use crate::expr::{PointExpr, VecExpr};
use crate::geom::{CurveParam, Point2, Vec2};
use crate::geometry::{GeometryId, PointFormula, VecFormula};
use crate::style::{Dash, Style};
use crate::vector::Vector;

/// Millimetres per TeX point.
pub const MM_PER_PT: f64 = 25.4 / 72.27;

/// `pt` TeX points in world units at the document scale.
pub fn pt_to_world(pt: f64, settings: &DocumentSettings) -> f64 {
    pt * MM_PER_PT / settings.scale
}

impl Dash {
    /// Alternating dash and gap lengths in pt for stroke width `width_pt`,
    /// starting with a dash; empty for `solid`.
    pub fn pattern(self, width_pt: f64) -> Vec<f64> {
        match self {
            Dash::Solid => Vec::new(),
            Dash::Dashed => vec![3.0, 3.0],
            Dash::Dotted => vec![width_pt, 2.0],
            Dash::DashDot => vec![3.0, 2.0, width_pt, 2.0],
        }
    }
}

/// Length of an arrow head in pt for stroke width `width_pt`: 3 pt + 4.5 w.
pub fn arrow_length_pt(width_pt: f64) -> f64 {
    3.0 + 4.5 * width_pt
}

/// Arrow head triangle `[tip, left base corner, right base corner]`.
/// `out` is the unit tangent pointing away from the curve; the head is
/// [`arrow_length_pt`] long and 0.75 of that wide.
pub fn arrow_head(tip: Point2, out: Vec2, width_pt: f64, settings: &DocumentSettings) -> [Point2; 3] {
    let len = pt_to_world(arrow_length_pt(width_pt), settings);
    let base = tip - out * len;
    let half = out.left_normal() * (0.375 * len);
    [tip, base + half, base - half]
}

/// A curve end that can carry an arrow head: the exact endpoint and the
/// unit tangent pointing away from the curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrowTip {
    pub point: Point2,
    pub out: Vec2,
}

/// Start and end tips of geometry `id` from its endpoint and tangent
/// expressions; `None` for a kind without the curve and endpoints
/// capabilities, or an end with a zero tangent.
pub fn arrow_tips(doc: &Document, p: &Vector, id: GeometryId) -> (Option<ArrowTip>, Option<ArrowTip>) {
    let Some(caps) = doc.caps(id) else {
        return (None, None);
    };
    if !(caps.curve && caps.endpoints) {
        return (None, None);
    }
    // Same expressions as `EndpointExprs` / `CurveExprs`; the caps say the kind has them.
    let tip = |point: PointFormula, t: f64, sign: f64| {
        let tangent = VecExpr::new(id, VecFormula::TangentAt { param: CurveParam::Normalized(t) }).evaluate(doc, p);
        let len = tangent.length();
        (len > 0.0 && len.is_finite()).then(|| ArrowTip {
            point: PointExpr::new(id, point).evaluate(doc, p),
            out: tangent * (sign / len),
        })
    };
    (tip(PointFormula::Start, 0.0, -1.0), tip(PointFormula::End, 1.0, 1.0))
}

fn polyline_length(polyline: &[Point2]) -> f64 {
    polyline.windows(2).map(|w| (w[1] - w[0]).length()).sum()
}

/// Point at arc length `s` along the edge `a`–`b` of length `len`.
fn along(a: Point2, b: Point2, s: f64, len: f64) -> Point2 {
    if len > 0.0 {
        a + (b - a) * (s / len)
    } else {
        a
    }
}

/// Part of `polyline` between arc length `from` and `to` (clamped); empty
/// if that part is empty.
fn sub_polyline(polyline: &[Point2], from: f64, to: f64) -> Vec<Point2> {
    let mut out = Vec::new();
    if to <= from {
        return out;
    }
    let mut s0 = 0.0;
    for w in polyline.windows(2) {
        let len = (w[1] - w[0]).length();
        let s1 = s0 + len;
        if s1 > from && s0 <= to {
            if out.is_empty() {
                out.push(along(w[0], w[1], (from - s0).max(0.0), len));
            }
            if s1 <= to {
                out.push(w[1]);
            } else {
                out.push(along(w[0], w[1], to - s0, len));
                break;
            }
        }
        s0 = s1;
    }
    if out.len() < 2 {
        out.clear();
    }
    out
}

/// `polyline` without the first `start_len` and the last `end_len` of its
/// arc length; empty if nothing is left.
pub fn trim(polyline: &[Point2], start_len: f64, end_len: f64) -> Vec<Point2> {
    let total = polyline_length(polyline);
    sub_polyline(polyline, start_len, total - end_len)
}

/// Dashes of `polyline` for `pattern` (alternating dash and gap lengths,
/// starting with a dash). `phase` is the arc length of the curve before the
/// polyline's start, so a trimmed polyline keeps the curve's pattern. An
/// empty pattern gives the whole polyline.
pub fn dash(polyline: &[Point2], pattern: &[f64], phase: f64) -> Vec<Vec<Point2>> {
    let total = polyline_length(polyline);
    if polyline.len() < 2 {
        return Vec::new();
    }
    let period: f64 = pattern.iter().sum();
    if pattern.is_empty() || !(period > 0.0 && period.is_finite()) {
        return vec![polyline.to_vec()];
    }
    // Rounding of the pattern lengths must not leave a sliver at the end.
    let end = total - 1e-9 * period;
    let mut pieces = Vec::new();
    // Curve arc length where the period containing `phase` starts.
    let mut s = -(phase.rem_euclid(period));
    'periods: loop {
        for (i, &len) in pattern.iter().enumerate() {
            if s >= end {
                break 'periods;
            }
            if i % 2 == 0 {
                let piece = sub_polyline(polyline, s.max(0.0), (s + len).min(total));
                if !piece.is_empty() {
                    pieces.push(piece);
                }
            }
            s += len;
        }
    }
    pieces
}

/// Visible stroke pieces of a curve's `polyline` in `style`: trimmed where
/// arrow heads sit, then dashed. Empty when the heads cover the curve.
pub fn stroke_pieces(polyline: &[Point2], style: &Style, settings: &DocumentSettings) -> Vec<Vec<Point2>> {
    let head = pt_to_world(arrow_length_pt(style.width), settings);
    let start = if style.arrow.at_start() { head } else { 0.0 };
    let end = if style.arrow.at_end() { head } else { 0.0 };
    let trimmed = trim(polyline, start, end);
    let pattern: Vec<f64> = style
        .dash
        .pattern(style.width)
        .into_iter()
        .map(|pt| pt_to_world(pt, settings))
        .collect();
    dash(&trimmed, &pattern, start)
}

#[cfg(test)]
#[path = "tests_stroke.rs"]
mod tests_stroke;
