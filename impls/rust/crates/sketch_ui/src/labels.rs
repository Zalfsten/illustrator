//! Labels on the Canvas: typeset lines (cached per line) placed in world
//! space, or the "not typeset" fallback (label-typesetting).

use std::collections::{HashMap, HashSet};

use constraint_runtime::{
    Document, GeometryId, GeometryRole, LabelAttrs, Markup, MeshFilter, Point2,
};
use eframe::egui::Pos2;
use label_layout::{
    frame_map, label_size, line_origins, split_lines, tessellate, LineMetrics, Triangles,
    TypesetError, Typesetter,
};

use crate::viewport::Viewport;

/// Fallback metrics of the UI's monospace font, in em.
const FALLBACK_ADVANCE_EM: f64 = 0.6;
const FALLBACK_ASCENT_EM: f64 = 0.75;
const FALLBACK_DESCENT_EM: f64 = 0.25;

/// (line source, markup, size bits, preamble).
type LineKey = (String, Markup, u64, String);

/// Typeset lines of the current frame's labels; lines no label used in a
/// frame are dropped at its end.
pub struct LabelCache {
    typesetter: Option<Box<dyn Typesetter>>,
    lines: HashMap<LineKey, Result<(LineMetrics, Triangles), TypesetError>>,
    used: HashSet<LineKey>,
}

impl LabelCache {
    /// `None`: no typesetter on this host; every label shows its fallback.
    pub fn new(typesetter: Option<Box<dyn Typesetter>>) -> Self {
        Self {
            typesetter,
            lines: HashMap::new(),
            used: HashSet::new(),
        }
    }

    fn line(
        &mut self,
        line: &str,
        markup: Markup,
        size: f64,
        preamble: &str,
    ) -> Option<&Result<(LineMetrics, Triangles), TypesetError>> {
        let ts = self.typesetter.as_ref()?;
        let key: LineKey = (line.to_owned(), markup, size.to_bits(), preamble.to_owned());
        self.used.insert(key.clone());
        Some(self.lines.entry(key).or_insert_with(|| {
            ts.typeset_line(line, markup, size, preamble)
                .map(|l| (LineMetrics::from(&l), tessellate(&l.shapes)))
        }))
    }

    fn sweep(&mut self) {
        let used = std::mem::take(&mut self.used);
        self.lines.retain(|k, _| used.contains(k));
    }
}

/// What the Canvas paints for one label, in screen coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintItem {
    /// Typeset lines as triangles.
    Typeset {
        geometry: GeometryId,
        role: GeometryRole,
        vertices: Vec<Pos2>,
        indices: Vec<u32>,
    },
    /// The source as plain text, marked as not typeset; `error` is `None`
    /// when there is no typesetter.
    Fallback {
        geometry: GeometryId,
        role: GeometryRole,
        lines: Vec<FallbackLine>,
        /// Corners of the block's frame, counterclockwise from bottom left.
        frame: [Pos2; 4],
        error: Option<TypesetError>,
    },
}

/// One fallback line: its source text, from the left end of its baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct FallbackLine {
    pub text: String,
    pub baseline_left: Pos2,
    /// Clockwise screen angle of the baseline, radians.
    pub angle: f32,
    pub size_px: f32,
}

/// Paint items for the labels visible under `filter`.
pub fn label_paint(
    doc: &Document,
    viewport: &Viewport,
    filter: MeshFilter,
    cache: &mut LabelCache,
) -> Vec<PaintItem> {
    let settings = doc.settings().clone();
    let mut out = Vec::new();
    for g in doc.geometries().values() {
        let Some(attrs) = doc.label_attrs(g.id()) else {
            continue;
        };
        if !filter.include_construction && g.role() == GeometryRole::Construction {
            continue;
        }
        let p = doc.vector();
        let origin = doc
            .origin_expr(g.id())
            .expect("labels have an origin")
            .evaluate(doc, p);
        let dir = doc
            .direction_expr(g.id())
            .expect("labels have a direction")
            .evaluate(doc, p);
        let heading = dir.y.atan2(dir.x);
        let frame = frame_map(&settings, origin, heading);
        let size = label_size(attrs, &settings);
        let sources = split_lines(&attrs.source, attrs.markup);
        if sources.is_empty() {
            continue;
        }

        let typeset: Option<Result<Vec<(LineMetrics, Triangles)>, TypesetError>> = sources
            .iter()
            .map(|l| {
                cache
                    .line(l, attrs.markup, size, &settings.preamble)
                    .cloned()
            })
            .collect::<Option<Vec<_>>>()
            .map(|lines| lines.into_iter().collect());

        match typeset {
            Some(Ok(lines)) => {
                let metrics: Vec<LineMetrics> = lines.iter().map(|(m, _)| *m).collect();
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                for ((_, tris), o) in lines.iter().zip(line_origins(&metrics, attrs, size)) {
                    let map = frame.then_translate(o);
                    let base = vertices.len() as u32;
                    vertices.extend(tris.vertices.iter().map(|v| {
                        let w = map.apply(Point2::new(f64::from(v[0]), f64::from(v[1])));
                        viewport.to_screen(w)
                    }));
                    indices.extend(tris.indices.iter().map(|i| base + i));
                }
                out.push(PaintItem::Typeset {
                    geometry: g.id(),
                    role: g.role(),
                    vertices,
                    indices,
                });
            }
            failed => out.push(fallback(
                g.id(),
                g.role(),
                attrs,
                &sources,
                size,
                &frame,
                heading,
                viewport,
                failed.and_then(Result::err),
            )),
        }
    }
    cache.sweep();
    out
}

#[allow(clippy::too_many_arguments)]
fn fallback(
    geometry: GeometryId,
    role: GeometryRole,
    attrs: &LabelAttrs,
    sources: &[&str],
    size: f64,
    frame: &label_layout::Affine2,
    heading: f64,
    viewport: &Viewport,
    error: Option<TypesetError>,
) -> PaintItem {
    let metrics: Vec<LineMetrics> = sources
        .iter()
        .map(|s| LineMetrics {
            width: FALLBACK_ADVANCE_EM * size * s.chars().count() as f64,
            height: FALLBACK_ASCENT_EM * size,
            depth: FALLBACK_DESCENT_EM * size,
        })
        .collect();
    let origins = line_origins(&metrics, attrs, size);
    let screen = |p: Point2| viewport.to_screen(frame.apply(p));
    // Screen px per pt: the frame map's scale times the viewport's.
    let px_per_pt = frame.a.hypot(frame.c) * viewport.scale;
    let lines = sources
        .iter()
        .zip(&origins)
        .map(|(text, o)| FallbackLine {
            text: (*text).to_owned(),
            baseline_left: screen(*o),
            angle: -heading as f32,
            size_px: (size * px_per_pt) as f32,
        })
        .collect();
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for (m, o) in metrics.iter().zip(&origins) {
        x0 = x0.min(o.x);
        x1 = x1.max(o.x + m.width);
        y0 = y0.min(o.y - m.depth);
        y1 = y1.max(o.y + m.height);
    }
    PaintItem::Fallback {
        geometry,
        role,
        lines,
        frame: [
            screen(Point2::new(x0, y0)),
            screen(Point2::new(x1, y0)),
            screen(Point2::new(x1, y1)),
            screen(Point2::new(x0, y1)),
        ],
        error,
    }
}

#[cfg(test)]
#[path = "tests_labels.rs"]
mod tests_labels;
