//! Labels on the Canvas: typeset lines (cached per line) placed in world
//! space, the last complete rendering while lines are pending, or the "not
//! typeset" fallback (label-typesetting).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use constraint_runtime::{Document, GeometryId, GeometryRole, LabelAttrs, MeshFilter, Point2};
use eframe::egui::Pos2;
use label_layout::{
    frame_map, label_requests, label_size, line_origins, split_lines, tessellate, Affine2, LineMetrics,
    LineRequest, Triangles, TypesetError, TypesetLine, Typesetter, TypesetterState,
};

use crate::viewport::Viewport;

/// Fallback metrics of the UI's monospace font, in em.
const FALLBACK_ADVANCE_EM: f64 = 0.6;
const FALLBACK_ASCENT_EM: f64 = 0.75;
const FALLBACK_DESCENT_EM: f64 = 0.25;

/// A typeset line ready to paint.
type Typeset = (LineMetrics, Triangles);

enum Entry {
    /// Requested, no result yet.
    Pending,
    Done(Result<Line, TypesetError>),
}

/// A delivered line: ready to paint, and its outlines for exports.
struct Line {
    paint: Typeset,
    outlines: Rc<TypesetLine>,
}

/// Typeset lines of the current frame's labels, and each label's last
/// complete rendering; what no label used in a frame is dropped at its end.
#[derive(Default)]
pub struct LabelCache {
    lines: HashMap<LineRequest, Entry>,
    used: HashSet<LineRequest>,
    last_complete: HashMap<GeometryId, Rc<Vec<Typeset>>>,
    any_pending: bool,
}

impl LabelCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a label waited for typesetting in the last [`label_paint`].
    pub fn any_pending(&self) -> bool {
        self.any_pending
    }

    fn take_completed(&mut self, ts: &dyn Typesetter) {
        for (req, result) in ts.take_completed() {
            let done = result.map(|l| Line {
                paint: (LineMetrics::from(&l), tessellate(&l.shapes)),
                outlines: Rc::new(l),
            });
            self.lines.insert(req, Entry::Done(done));
        }
    }

    /// Request the lines not asked for yet (each once).
    fn request(&mut self, ts: &dyn Typesetter, reqs: &[LineRequest]) {
        for req in reqs {
            self.used.insert(req.clone());
            if !self.lines.contains_key(req) {
                self.lines.insert(req.clone(), Entry::Pending);
                ts.request(req.clone());
            }
        }
    }

    /// Outlines of `reqs` for an export: requested if missing and kept for
    /// this frame like the Canvas's lines. All lines, the first failure, or
    /// `None` while any is pending.
    pub fn ensure(
        &mut self,
        ts: &dyn Typesetter,
        reqs: &[LineRequest],
    ) -> Option<Result<Vec<Rc<TypesetLine>>, TypesetError>> {
        self.take_completed(ts);
        self.request(ts, reqs);
        // An immediate typesetter has answered inside `request`.
        self.take_completed(ts);
        self.collect(reqs, |l| l.outlines.clone())
    }

    /// All lines typeset, the first failure, or `None` while any is pending.
    fn status(&self, reqs: &[LineRequest]) -> Option<Result<Vec<Typeset>, TypesetError>> {
        self.collect(reqs, |l| l.paint.clone())
    }

    fn collect<T>(&self, reqs: &[LineRequest], get: impl Fn(&Line) -> T) -> Option<Result<Vec<T>, TypesetError>> {
        let mut lines = Vec::with_capacity(reqs.len());
        let mut pending = false;
        for req in reqs {
            match self.lines.get(req) {
                Some(Entry::Done(Ok(l))) => lines.push(get(l)),
                Some(Entry::Done(Err(e))) => return Some(Err(e.clone())),
                Some(Entry::Pending) | None => pending = true,
            }
        }
        (!pending).then_some(Ok(lines))
    }

    fn sweep(&mut self, labels: &HashSet<GeometryId>) {
        let used = std::mem::take(&mut self.used);
        self.lines.retain(|k, _| used.contains(k));
        self.last_complete.retain(|id, _| labels.contains(id));
    }
}

/// What the Canvas paints for one label, in screen coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum PaintItem {
    /// Typeset lines as triangles (possibly the last complete rendering
    /// while new lines are pending).
    Typeset {
        geometry: GeometryId,
        role: GeometryRole,
        vertices: Vec<Pos2>,
        indices: Vec<u32>,
    },
    /// The source as plain text in a dashed frame, marked as not typeset.
    Fallback {
        geometry: GeometryId,
        role: GeometryRole,
        lines: Vec<FallbackLine>,
        /// Corners of the block's frame, counterclockwise from bottom left.
        frame: [Pos2; 4],
        reason: FallbackReason,
    },
}

/// Why a label shows its fallback.
#[derive(Clone, Debug, PartialEq)]
pub enum FallbackReason {
    /// Lines are still being typeset (or the typesetter is loading).
    Pending,
    /// A line failed to typeset.
    Failed(TypesetError),
    /// This host has no typesetter.
    NoTypesetter,
    /// The typesetter could not be loaded.
    LoadFailed(String),
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

/// Paint items for the labels visible under `filter`. Completed results are
/// taken from `typesetter` and missing lines requested from it.
pub fn label_paint(
    doc: &Document,
    viewport: &Viewport,
    filter: MeshFilter,
    cache: &mut LabelCache,
    typesetter: Option<&dyn Typesetter>,
) -> Vec<PaintItem> {
    let settings = doc.settings().clone();
    let state = typesetter.map(|ts| ts.state());
    if let Some(ts) = typesetter {
        cache.take_completed(ts);
    }
    cache.any_pending = false;
    let mut labels = HashSet::new();
    let mut out = Vec::new();
    for g in doc.geometries().values() {
        let Some(attrs) = doc.label_attrs(g.id()) else {
            continue;
        };
        labels.insert(g.id());
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
        let place = |reason| {
            fallback(
                g.id(),
                g.role(),
                attrs,
                &sources,
                size,
                &frame,
                heading,
                viewport,
                reason,
            )
        };

        let ts = match (typesetter, &state) {
            (None, _) | (_, None) => {
                out.push(place(FallbackReason::NoTypesetter));
                continue;
            }
            (Some(_), Some(TypesetterState::Failed(msg))) => {
                out.push(place(FallbackReason::LoadFailed(msg.clone())));
                continue;
            }
            (Some(ts), Some(_)) => ts,
        };
        let reqs = label_requests(attrs, &settings);
        cache.request(ts, &reqs);
        // An immediate typesetter has answered inside `request`.
        cache.take_completed(ts);

        match cache.status(&reqs) {
            Some(Ok(lines)) => {
                let lines = Rc::new(lines);
                out.push(typeset(
                    g.id(),
                    g.role(),
                    attrs,
                    size,
                    &frame,
                    viewport,
                    &lines,
                ));
                cache.last_complete.insert(g.id(), lines);
            }
            Some(Err(e)) => out.push(place(FallbackReason::Failed(e))),
            None => {
                cache.any_pending = true;
                match cache.last_complete.get(&g.id()) {
                    Some(lines) => out.push(typeset(
                        g.id(),
                        g.role(),
                        attrs,
                        size,
                        &frame,
                        viewport,
                        lines,
                    )),
                    None => out.push(place(FallbackReason::Pending)),
                }
            }
        }
    }
    cache.sweep(&labels);
    out
}

/// Typeset lines placed by the label's current frame, in screen space.
fn typeset(
    geometry: GeometryId,
    role: GeometryRole,
    attrs: &LabelAttrs,
    size: f64,
    frame: &Affine2,
    viewport: &Viewport,
    lines: &[Typeset],
) -> PaintItem {
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
    PaintItem::Typeset {
        geometry,
        role,
        vertices,
        indices,
    }
}

#[allow(clippy::too_many_arguments)]
fn fallback(
    geometry: GeometryId,
    role: GeometryRole,
    attrs: &LabelAttrs,
    sources: &[&str],
    size: f64,
    frame: &Affine2,
    heading: f64,
    viewport: &Viewport,
    reason: FallbackReason,
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
        reason,
    }
}

#[cfg(test)]
#[path = "tests_labels.rs"]
mod tests_labels;
