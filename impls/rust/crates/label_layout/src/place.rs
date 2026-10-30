//! Stacking lines and placing a label in world coordinates (labels spec:
//! frame, reference point, alignment; label-typesetting: line spacing).

use constraint_runtime::{DocumentSettings, HAlign, LabelAttrs, Point2, VAlign};

use crate::typeset::TypesetLine;

/// One pt (1/72.27 inch) in millimetres.
const MM_PER_PT: f64 = 25.4 / 72.27;

/// Baselines are at least this factor times the size apart.
const BASELINE_FACTOR: f64 = 1.2;

/// Minimum gap between the visible extents of consecutive lines, in pt.
const LINE_SKIP_PT: f64 = 1.0;

/// What placement needs from a typeset line (pt).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LineMetrics {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
}

impl From<&TypesetLine> for LineMetrics {
    fn from(l: &TypesetLine) -> Self {
        Self {
            width: l.width,
            height: l.height,
            depth: l.depth,
        }
    }
}

/// Affine map `p ↦ (a·x + b·y + tx, c·x + d·y + ty)` from pt in a line's own
/// frame to world coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2 {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Affine2 {
    pub fn apply(&self, p: Point2) -> Point2 {
        Point2::new(
            self.a * p.x + self.b * p.y + self.tx,
            self.c * p.x + self.d * p.y + self.ty,
        )
    }
}

/// The label's size in pt: its own, or the document font size.
pub fn label_size(attrs: &LabelAttrs, settings: &DocumentSettings) -> f64 {
    attrs.size.unwrap_or(settings.font_size)
}

/// World units per pt at the document scale (mm per world unit).
pub fn world_per_pt(settings: &DocumentSettings) -> f64 {
    MM_PER_PT / settings.scale
}

/// Baselines of `lines` in pt (y up), the first at 0.
pub fn baselines(lines: &[LineMetrics], size_pt: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(lines.len());
    let mut y = 0.0;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            let above = &lines[i - 1];
            y -= (BASELINE_FACTOR * size_pt).max(above.depth + line.height + LINE_SKIP_PT);
        }
        out.push(y);
    }
    out
}

/// Position of each line's origin (left end of its baseline) in the label
/// frame, in pt, after alignment and offset.
pub fn line_origins(lines: &[LineMetrics], attrs: &LabelAttrs, size_pt: f64) -> Vec<Point2> {
    let base = baselines(lines, size_pt);
    let shift_y = match attrs.valign {
        VAlign::First => 0.0,
        VAlign::Last => -base.last().copied().unwrap_or(0.0),
    };
    lines
        .iter()
        .zip(base)
        .map(|(line, baseline)| {
            let shift_x = match attrs.halign {
                HAlign::Left => 0.0,
                HAlign::Center => -0.5 * line.width,
                HAlign::Right => -line.width,
            };
            Point2::new(
                attrs.offset.x + shift_x,
                attrs.offset.y + baseline + shift_y,
            )
        })
        .collect()
}

/// Map from pt in the label frame to world coordinates for a label at
/// `origin` with `heading` (radians).
pub fn frame_map(settings: &DocumentSettings, origin: Point2, heading: f64) -> Affine2 {
    let k = world_per_pt(settings);
    let (s, c) = heading.sin_cos();
    Affine2 {
        a: k * c,
        b: -k * s,
        c: k * s,
        d: k * c,
        tx: origin.x,
        ty: origin.y,
    }
}

/// One map per line from pt in that line's frame to world coordinates.
pub fn place(
    lines: &[LineMetrics],
    attrs: &LabelAttrs,
    settings: &DocumentSettings,
    origin: Point2,
    heading: f64,
) -> Vec<Affine2> {
    let frame = frame_map(settings, origin, heading);
    line_origins(lines, attrs, label_size(attrs, settings))
        .into_iter()
        .map(|o| frame.then_translate(o))
        .collect()
}

impl Affine2 {
    /// `self` after a translation by `t` (pt) in the source frame.
    pub fn then_translate(&self, t: Point2) -> Affine2 {
        Affine2 {
            tx: self.a * t.x + self.b * t.y + self.tx,
            ty: self.c * t.x + self.d * t.y + self.ty,
            ..*self
        }
    }
}

#[cfg(test)]
#[path = "tests_place.rs"]
mod tests_place;
