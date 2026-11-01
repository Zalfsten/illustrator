//! Application shell: command session, tools, and Canvas painting.
//!
//! The App edits the drawing only through [`Session`] commands and drags.

use constraint_runtime::{
    arrow_head, arrow_tips, pt_to_world, stroke_pieces, AnchorExprs, Arc, Congruence, CurveExprs,
    CurveParam, Document, EndpointExprs, EqualLength, FixPoint, GeometryId, GeometryRole, HAlign,
    HeadingAngle, InteractiveSolve, KindAttrs, Label, LabelAttrs, Length, LineExprs, LogicalPoint,
    Mesh, MeshFilter, Point2, PointExpr, Rgb, Segment, SolveError, TurnAngle, Vec2,
    VectorsOrthogonal, VectorsParallel,
};
use eframe::egui::{self, epaint, Color32, Pos2, Rect, Sense, Stroke};
use label_layout::TypesetterState;
use sketch_session::{
    Command, ConstraintSpec, FixParams, GeometrySpec, PointPair, SegmentParams, Session, VecPair,
};

use crate::export::{ExportJob, JobStep, WAITING};
use crate::files::{Answer, Effect, FileCommand, FileState, Step};
use crate::labels::{label_paint, FallbackReason, LabelCache, PaintItem};
use crate::platform::Platform;
use crate::style_ui::{CurrentStyle, StylesWindow};
use crate::viewport::{world_box, Viewport};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Select,
    Segment,
    Join,
    Parallel,
    Orthogonal,
    FixPoint,
    /// Give a curve the current style and overrides.
    Brush,
}

/// UI session: result-only vs include construction (Mesh + handles share one filter).
#[derive(Clone, Copy, PartialEq, Eq)]
enum PresentationMode {
    ResultOnly,
    IncludeConstruction,
}

impl PresentationMode {
    fn mesh_filter(self) -> MeshFilter {
        match self {
            PresentationMode::ResultOnly => MeshFilter::RESULT_ONLY,
            PresentationMode::IncludeConstruction => MeshFilter::INCLUDE_CONSTRUCTION,
        }
    }
}

/// Colour of labels shown untypeset (and of their status message).
const NOT_TYPESET: Color32 = Color32::from_rgb(220, 120, 20);
/// Colour of labels whose typesetting is pending (and of "loading typesetter…").
const PENDING: Color32 = Color32::from_rgb(140, 140, 150);

/// World AABB of the sample sketch (with margin room) and of an empty drawing.
const WORLD_MIN: Point2 = Point2 { x: -0.5, y: -0.5 };
const WORLD_MAX: Point2 = Point2 { x: 2.5, y: 2.5 };
const VIEW_MARGIN: f32 = 40.0;

/// An export in progress: waiting for labels, or its bytes waiting for the host.
enum Exporting {
    Job(Box<ExportJob>),
    Bytes(Vec<u8>),
}

/// A choice in the File menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuItem {
    File(FileCommand),
    ExportSvg,
}

/// Owns the command session; central panel is the Canvas.
pub struct App {
    session: Session,
    tool: Tool,
    presentation: PresentationMode,
    pick_points: Vec<PointExpr>,
    pick_geoms: Vec<GeometryId>,
    segment_clicks: Vec<Point2>,
    status: String,
    last_error: Option<String>,
    /// Grabbable points (congruence classes); rebuilt after structural changes.
    logical_points: Vec<LogicalPoint>,
    /// Host services (the typesetter, if this host has one).
    platform: Box<dyn Platform>,
    /// Typeset label lines and each label's last complete rendering.
    labels: LabelCache,
    /// Typesetter state or first label failure of the last paint, with its colour.
    label_status: Option<(String, Color32)>,
    /// Last fitted viewport (updated each paint from canvas rect).
    viewport: Viewport,
    /// World AABB the Viewport is fitted to; set on start, New and Open.
    world_box: (Point2, Point2),
    /// The drawing's file and unsaved state, and the file command in progress.
    files: FileState,
    /// Closing the window was confirmed (no unsaved changes, or answered).
    close_confirmed: bool,
    /// Style and overrides for new geometry and the brush (not saved).
    current: CurrentStyle,
    styles_window: StylesWindow,
    export: Option<Exporting>,
}

impl App {
    /// The sample scene on a host without a typesetter (labels show untypeset)
    /// whose files can be overwritten.
    #[cfg(test)]
    pub fn with_sample() -> Self {
        Self::with_files(crate::fake_files::FakeFiles::new(true))
    }

    /// The sample scene on a host without a typesetter, with file services `fake`.
    #[cfg(test)]
    pub fn with_files(fake: crate::fake_files::FakeFiles) -> Self {
        Self::new(Box::new(crate::platform::Host::new(None, Box::new(fake))))
    }

    /// The sample scene on a host with `typesetter` and file services `fake`.
    #[cfg(test)]
    pub fn with_host(typesetter: Box<dyn label_layout::Typesetter>, fake: crate::fake_files::FakeFiles) -> Self {
        Self::new(Box::new(crate::platform::Host::new(Some(typesetter), Box::new(fake))))
    }

    /// The sample scene on the host `platform`.
    pub fn new(platform: Box<dyn Platform>) -> Self {
        let mut doc = Document::new();
        // World units, y-up: horizontal + vertical legs with a quarter-circle fillet.
        const R: f64 = 0.5;
        let s0 = doc.add_geometry::<Segment>(
            GeometryRole::Result,
            (Point2::new(0.0, 0.0), Point2::new(2.0 - R, 0.0)),
        );
        let fillet = doc.add_geometry::<Arc>(
            GeometryRole::Result,
            (
                Point2::new(2.0 - R, R),
                Length::new(R),
                HeadingAngle::new(-std::f64::consts::FRAC_PI_2),
                TurnAngle::new(std::f64::consts::FRAC_PI_2),
            ),
        );
        let s1 = doc.add_geometry::<Segment>(
            GeometryRole::Result,
            (Point2::new(2.0, R), Point2::new(2.0, 2.0)),
        );
        // Demo: helper diagonal marked construction (hidden in result-only).
        let helper = doc.add_geometry::<Segment>(
            GeometryRole::Result,
            (Point2::new(0.0, 0.0), Point2::new(2.0, 2.0)),
        );
        assert!(doc.set_geometry_role(helper, GeometryRole::Construction));
        // Pin arc length (construction gauge). Prevents Δθ→0 collapse of start/end
        // even when radius can still vary under drag.
        let arc_len = R * std::f64::consts::FRAC_PI_2;
        let length_gauge = add_fixed_length_gauge(&mut doc, Point2::new(-1.0, -1.0), arc_len);
        doc.add_constraint(Congruence {
            a: Segment::end_expr(s0),
            b: Arc::start_expr(fillet),
        });
        doc.add_constraint(Congruence {
            a: Arc::end_expr(fillet),
            b: Segment::start_expr(s1),
        });
        doc.add_constraint(VectorsParallel {
            a: Segment::tangent_expr(s0, CurveParam::Normalized(0.0)),
            b: Arc::tangent_expr(fillet, CurveParam::Normalized(0.0)),
        });
        doc.add_constraint(VectorsParallel {
            a: Segment::tangent_expr(s1, CurveParam::Normalized(0.0)),
            b: Arc::tangent_expr(fillet, CurveParam::Normalized(1.0)),
        });
        doc.add_constraint(VectorsOrthogonal {
            a: Segment::direction_expr(s0),
            b: Segment::direction_expr(s1),
        });
        doc.add_constraint(FixPoint {
            point: Segment::start_expr(s0),
            target: Point2::new(0.0, 0.0),
        });
        doc.add_constraint(EqualLength {
            a: Arc::length_expr(fillet),
            b: Segment::length_expr(length_gauge),
        });
        // A label centred above the horizontal leg, following it.
        let label = doc.add_geometry_with::<Label>(
            GeometryRole::Result,
            (Point2::new((2.0 - R) / 2.0, 0.0), HeadingAngle::new(0.0)),
            KindAttrs::Label(LabelAttrs {
                source: r"leg $\ell = \frac{3}{2}$".into(),
                halign: HAlign::Center,
                offset: Vec2::new(0.0, 4.0),
                ..LabelAttrs::default()
            }),
        );
        doc.add_constraint(Congruence {
            a: Label::origin_expr(label),
            b: Segment::on_curve_expr(s0, CurveParam::Normalized(0.5)),
        });
        doc.add_constraint(VectorsParallel {
            a: Label::direction_expr(label),
            b: Segment::direction_expr(s0),
        });

        // Initial content (session baseline, not an edit).
        let _ = InteractiveSolve::new().structural_only(&mut doc);

        let presentation = PresentationMode::ResultOnly;
        let logical_points = doc.logical_points(presentation.mesh_filter());
        let session = Session::new(doc);
        let mut files = FileState::new(&session);
        files.refresh(&session, platform.files());

        Self {
            session,
            tool: Tool::Select,
            presentation,
            pick_points: Vec::new(),
            pick_geoms: Vec::new(),
            segment_clicks: Vec::new(),
            status: "Sample filleted right angle (world y-up) — drag a free end. Toggle construction to see helper."
                .into(),
            last_error: None,
            logical_points,
            platform,
            labels: LabelCache::new(),
            label_status: None,
            viewport: Viewport::new(1.0, Pos2::ZERO),
            world_box: (WORLD_MIN, WORLD_MAX),
            files,
            close_confirmed: false,
            current: CurrentStyle::default(),
            styles_window: StylesWindow::default(),
            export: None,
        }
    }

    /// Start a file command (no-op while dragging or while one is in progress).
    fn file_command(&mut self, cmd: FileCommand) {
        let step = self
            .files
            .command(cmd, &mut self.session, self.platform.files());
        self.file_step(step);
    }

    /// Start exporting the drawing as it is now (no-op while dragging, while a
    /// file request is open, or while an export runs).
    fn start_export(&mut self) {
        if !self.files_enabled() || self.export.is_some() {
            return;
        }
        self.export = Some(Exporting::Job(Box::new(ExportJob::new(self.session.document()))));
        self.status = "Exporting…".into();
        self.last_error = None;
    }

    /// Advance the export; returns whether it waits for labels.
    fn export_step(&mut self) -> bool {
        let next = match self.export.take() {
            None => return false,
            Some(Exporting::Job(job)) => match job.step(&mut self.labels, self.platform.typesetter()) {
                JobStep::Waiting => {
                    self.status = WAITING.into();
                    self.export = Some(Exporting::Job(job));
                    return true;
                }
                JobStep::Failed(e) => {
                    self.status = "Not exported".into();
                    self.last_error = Some(e);
                    return false;
                }
                JobStep::Ready(bytes) => bytes,
            },
            Some(Exporting::Bytes(bytes)) => bytes,
        };
        // Another file request first; the bytes wait for it.
        if self.files.busy() {
            self.export = Some(Exporting::Bytes(next));
        } else {
            self.files.export(next, self.platform.files());
        }
        false
    }

    fn file_answer(&mut self, answer: Answer) {
        let step = self
            .files
            .answer(answer, &mut self.session, self.platform.files());
        self.file_step(step);
    }

    /// Handle a completed host file request, if any.
    fn poll_files(&mut self) {
        let step = self.files.poll(&mut self.session, self.platform.files());
        self.file_step(step);
    }

    fn file_step(&mut self, step: Step) {
        match step.message {
            Some(Ok(s)) => {
                self.status = s;
                self.last_error = None;
            }
            Some(Err(e)) => self.last_error = Some(e),
            None => {}
        }
        match step.effect {
            Some(Effect::Replaced) => self.replaced(),
            Some(Effect::Close) => self.close_confirmed = true,
            None => {}
        }
    }

    /// After New or Open: drop what belonged to the old drawing, fit the view.
    fn replaced(&mut self) {
        self.pick_points.clear();
        self.pick_geoms.clear();
        self.segment_clicks.clear();
        self.labels = LabelCache::new();
        self.rebuild_logical_points();
        self.world_box = self.drawing_box().unwrap_or((WORLD_MIN, WORLD_MAX));
    }

    /// Box around the visible geometry and the origins of visible labels.
    fn drawing_box(&self) -> Option<(Point2, Point2)> {
        let doc = self.doc();
        let filter = self.presentation.mesh_filter();
        let edges = self.mesh().edges.into_iter().flat_map(|e| [e.a, e.b]);
        let origins = doc
            .geometries()
            .values()
            .filter(|g| doc.label_attrs(g.id()).is_some())
            .filter(|g| filter.include_construction || g.role() != GeometryRole::Construction)
            .filter_map(|g| doc.origin_expr(g.id()))
            .map(|e| e.evaluate(doc, doc.vector()));
        world_box(edges.chain(origins))
    }

    /// After a history step: handles and the unsaved state.
    fn history_changed(&mut self) {
        self.rebuild_logical_points();
        self.files.refresh(&self.session, self.platform.files());
    }

    /// Typeset label as a triangle mesh; untypeset label as text in a dashed
    /// frame (grey while pending). Returns `name: error` for a failed label.
    fn paint_label(&self, painter: &egui::Painter, item: PaintItem) -> Option<String> {
        match item {
            PaintItem::Typeset {
                geometry,
                role,
                vertices,
                indices,
            } => {
                let color = match role {
                    GeometryRole::Construction => CONSTRUCTION,
                    GeometryRole::Result => self
                        .doc()
                        .resolved_style(geometry)
                        .map_or(CONSTRUCTION, |s| color32(s.color)),
                };
                let mut mesh = epaint::Mesh::default();
                for v in vertices {
                    mesh.colored_vertex(v, color);
                }
                mesh.indices = indices;
                painter.add(egui::Shape::mesh(mesh));
                None
            }
            PaintItem::Fallback {
                geometry,
                lines,
                frame,
                reason,
                ..
            } => {
                let color = if reason == FallbackReason::Pending {
                    PENDING
                } else {
                    NOT_TYPESET
                };
                for line in lines {
                    let font = egui::FontId::monospace(line.size_px.max(1.0));
                    let galley = painter.layout_no_wrap(line.text, font, color);
                    // Text is placed by its top left; lift it by the ascent.
                    let ascent = galley.rows.first().map_or(0.0, |r| r.rect.height() * 0.8);
                    let (s, c) = line.angle.sin_cos();
                    let top_left = line.baseline_left + egui::vec2(ascent * s, -ascent * c);
                    painter.add(
                        epaint::TextShape::new(top_left, galley, color).with_angle(line.angle),
                    );
                }
                let stroke = Stroke::new(1.0_f32, color);
                let mut ring = frame.to_vec();
                ring.push(frame[0]);
                painter.extend(egui::Shape::dashed_line(&ring, stroke, 4.0, 3.0));
                match reason {
                    FallbackReason::Failed(e) => {
                        let name = self.doc().geometry_name(geometry).unwrap_or("?");
                        Some(format!("{name}: {e}"))
                    }
                    _ => None,
                }
            }
        }
    }

    fn doc(&self) -> &Document {
        self.session.document()
    }

    fn mesh(&self) -> Mesh {
        Mesh::from_document(
            self.doc(),
            self.doc().vector(),
            self.presentation.mesh_filter(),
        )
    }

    fn eval_point(&self, expr: &PointExpr) -> Point2 {
        expr.evaluate(self.doc(), self.doc().vector())
    }

    fn rebuild_logical_points(&mut self) {
        self.logical_points = self.doc().logical_points(self.presentation.mesh_filter());
    }

    /// Apply `cmd`; status `Added <name> <detail>` or the command error.
    fn run(&mut self, cmd: Command, detail: String) {
        self.step(cmd, |outcome| {
            let name = outcome.created.unwrap_or_default();
            format!("Added {name}{detail}")
        });
    }

    /// Apply `cmd`; status from `done` or the command error.
    fn step(&mut self, cmd: Command, done: impl FnOnce(sketch_session::Outcome) -> String) {
        match self.session.apply(cmd) {
            Ok(outcome) => {
                self.last_error = None;
                self.status = done(outcome);
                self.files.refresh(&self.session, self.platform.files());
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                self.status = format!("Rejected — drawing unchanged. {e}");
            }
        }
        self.rebuild_logical_points();
    }

    /// Status after a drag solve.
    fn report_drag(&mut self, result: Result<(), SolveError>) {
        match result {
            Ok(()) => {
                self.last_error = None;
                self.status = format!(
                    "OK — geometries={} constraints={}",
                    self.doc().geometries().len(),
                    self.doc().constraints().len()
                );
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                self.status = format!("Solve failed — kept last good state. {e}");
            }
        }
    }

    /// Address of a picked point (`seg1.end`); handles are always addressable.
    fn address(&self, p: &PointExpr) -> String {
        self.doc().point_address(p).unwrap_or_else(|| "?".into())
    }

    fn geometry_name(&self, id: GeometryId) -> &str {
        self.doc().geometry_name(id).unwrap_or("?")
    }

    /// `<name>.direction` of a picked line (checked by the caller).
    fn direction_address(&self, id: GeometryId) -> String {
        self.doc()
            .direction_expr(id)
            .and_then(|e| self.doc().vec_address(&e))
            .unwrap_or_else(|| "?".into())
    }

    fn hit_point(&self, screen: Pos2, radius_px: f64) -> Option<PointExpr> {
        let query = self.viewport.to_world(screen);
        let radius = self.viewport.px_to_world(radius_px);
        let mut best = None;
        let mut best_d = radius;
        for lp in &self.logical_points {
            let p = self.eval_point(&lp.representative);
            let d = (p.x - query.x).hypot(p.y - query.y);
            if d < best_d {
                best_d = d;
                best = Some(lp.representative);
            }
        }
        best
    }

    fn hit_end(&self, screen: Pos2, radius_px: f64) -> Option<PointExpr> {
        let query = self.viewport.to_world(screen);
        let radius = self.viewport.px_to_world(radius_px);
        self.mesh().hit_endpoint(query, radius)
    }

    fn hit_edge(&self, screen: Pos2, radius_px: f64) -> Option<GeometryId> {
        let query = self.viewport.to_world(screen);
        let radius = self.viewport.px_to_world(radius_px);
        self.mesh().hit_geometry(query, radius)
    }

    fn handle_click(&mut self, screen: Pos2) {
        let world = self.viewport.to_world(screen);
        match self.tool {
            Tool::Segment => {
                self.segment_clicks.push(world);
                if self.segment_clicks.len() >= 2 {
                    let a = self.segment_clicks[0];
                    let b = self.segment_clicks[1];
                    self.segment_clicks.clear();
                    let cmd = Command::Create {
                        name: None,
                        role: GeometryRole::Result,
                        look: self.current.look(),
                        geometry: GeometrySpec::Segment(SegmentParams { start: a, end: b }),
                    };
                    self.run(cmd, String::new());
                } else {
                    self.status = "Segment: click second endpoint".into();
                }
            }
            Tool::Join => {
                if let Some(expr) = self.hit_end(screen, 12.0) {
                    self.pick_points.push(expr);
                    if self.pick_points.len() >= 2 {
                        let a = self.address(&self.pick_points[0]);
                        let b = self.address(&self.pick_points[1]);
                        self.pick_points.clear();
                        let detail = format!(" ({a} = {b})");
                        let join = ConstraintSpec::Congruence(PointPair {
                            a: a.into(),
                            b: b.into(),
                        });
                        self.run(constrain(join), detail);
                    } else {
                        self.status = format!("Join: {} → pick second point", self.address(&expr));
                    }
                }
            }
            Tool::Parallel | Tool::Orthogonal => {
                if let Some(id) = self.hit_edge(screen, 24.0) {
                    // Lines only: the direction is a curve invariant; arcs have none.
                    if !self.doc().caps(id).is_some_and(|c| c.line) {
                        self.status = "Parallel/Orthogonal needs two line segments".into();
                        return;
                    }
                    self.pick_geoms.push(id);
                    if self.pick_geoms.len() >= 2 {
                        let detail = format!(
                            " ({}, {})",
                            self.geometry_name(self.pick_geoms[0]),
                            self.geometry_name(self.pick_geoms[1])
                        );
                        let a = self.direction_address(self.pick_geoms[0]);
                        let b = self.direction_address(self.pick_geoms[1]);
                        self.pick_geoms.clear();
                        let pair = VecPair {
                            a: a.into(),
                            b: b.into(),
                        };
                        let spec = if self.tool == Tool::Parallel {
                            ConstraintSpec::VectorsParallel(pair)
                        } else {
                            ConstraintSpec::VectorsOrthogonal(pair)
                        };
                        self.run(constrain(spec), detail);
                    } else {
                        let tool = if self.tool == Tool::Parallel {
                            "Parallel"
                        } else {
                            "Orthogonal"
                        };
                        self.status =
                            format!("{tool}: {} → pick second line", self.geometry_name(id));
                    }
                }
            }
            Tool::FixPoint => {
                if let Some(expr) = self.hit_point(screen, 12.0) {
                    let target = self.eval_point(&expr);
                    let point = self.address(&expr);
                    let detail = format!(" ({point})");
                    let fix = ConstraintSpec::FixPoint(FixParams {
                        point: point.into(),
                        target,
                    });
                    self.run(constrain(fix), detail);
                }
            }
            Tool::Brush => {
                if let Some(id) = self.hit_edge(screen, 12.0) {
                    let name = self.geometry_name(id).to_owned();
                    let cmd = self.current.brush(&name);
                    self.step(cmd, |_| format!("Brushed {name}"));
                }
            }
            Tool::Select => {}
        }
    }
}

/// Constraint command with an automatic name.
fn constrain(constraint: ConstraintSpec) -> Command {
    Command::Constrain {
        name: None,
        constraint,
    }
}

/// The fixed look of construction geometry: colour and width in points.
const CONSTRUCTION: Color32 = Color32::from_rgb(160, 160, 180);
const CONSTRUCTION_WIDTH: f32 = 1.5;

fn color32(Rgb(r, g, b): Rgb) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// One curve as painted: stroke pieces (screen polylines) of `width`, and
/// filled arrow heads, all in `color`.
#[derive(Clone, Debug, PartialEq)]
struct CurvePaint {
    pieces: Vec<Vec<Pos2>>,
    heads: Vec<[Pos2; 3]>,
    width: f32,
    color: Color32,
}

/// Paint of geometry `id` with display polyline `polyline` (world): a result
/// curve in its resolved style, never thinner than `min_width` screen units;
/// construction in the fixed construction look.
fn curve_paint(
    doc: &Document,
    id: GeometryId,
    polyline: &[Point2],
    viewport: &Viewport,
    min_width: f32,
) -> CurvePaint {
    let screen = |line: &[Point2]| line.iter().map(|&p| viewport.to_screen(p)).collect();
    let style = match doc.geometry(id).map(|g| g.role()) {
        Some(GeometryRole::Result) => doc.resolved_style(id),
        _ => None,
    };
    let Some(style) = style else {
        return CurvePaint {
            pieces: vec![screen(polyline)],
            heads: Vec::new(),
            width: CONSTRUCTION_WIDTH,
            color: CONSTRUCTION,
        };
    };
    let settings = doc.settings();
    let (start, end) = arrow_tips(doc, doc.vector(), id);
    let heads = [(style.arrow.at_start(), start), (style.arrow.at_end(), end)]
        .into_iter()
        .filter_map(|(on, tip)| tip.filter(|_| on))
        .map(|tip| {
            arrow_head(tip.point, tip.out, style.width, settings).map(|p| viewport.to_screen(p))
        })
        .collect();
    let width = (pt_to_world(style.width, settings) * viewport.scale) as f32;
    CurvePaint {
        pieces: stroke_pieces(polyline, &style, settings)
            .iter()
            .map(|piece| screen(piece))
            .collect(),
        heads,
        width: width.max(min_width),
        color: color32(style.color),
    }
}

/// The Mesh as one polyline per geometry (its edges are consecutive and in
/// curve order).
fn polylines(mesh: &Mesh) -> Vec<(GeometryId, Vec<Point2>)> {
    let mut out: Vec<(GeometryId, Vec<Point2>)> = Vec::new();
    for e in &mesh.edges {
        match out.last_mut() {
            Some((id, line)) if *id == e.geometry => line.push(e.b),
            _ => out.push((e.geometry, vec![e.a, e.b])),
        }
    }
    out
}

/// Construction segment of fixed length `len` starting at `origin` (axis +x).
fn add_fixed_length_gauge(doc: &mut Document, origin: Point2, len: f64) -> GeometryId {
    let end = Point2::new(origin.x + len, origin.y);
    let id = doc.add_geometry::<Segment>(GeometryRole::Result, (origin, end));
    assert!(doc.set_geometry_role(id, GeometryRole::Construction));
    doc.add_constraint(FixPoint {
        point: Segment::start_expr(id),
        target: origin,
    });
    doc.add_constraint(FixPoint {
        point: Segment::end_expr(id),
        target: end,
    });
    id
}

/// Keyboard shortcuts of the file commands (Save As before Save: `consume_shortcut`
/// ignores an extra Shift).
const FILE_SHORTCUTS: [(egui::Modifiers, egui::Key, FileCommand); 4] = [
    (
        egui::Modifiers::COMMAND.plus(egui::Modifiers::SHIFT),
        egui::Key::S,
        FileCommand::SaveAs,
    ),
    (egui::Modifiers::COMMAND, egui::Key::S, FileCommand::Save),
    (egui::Modifiers::COMMAND, egui::Key::N, FileCommand::New),
    (egui::Modifiers::COMMAND, egui::Key::O, FileCommand::Open),
];

impl App {
    /// Whether file commands can start now.
    fn files_enabled(&self) -> bool {
        !self.session.is_dragging() && !self.files.busy()
    }

    /// Frame start: completed host requests, the close request, shortcuts.
    fn file_frame(&mut self, ctx: &egui::Context) {
        let repaint = ctx.clone();
        self.files.set_waker(move || repaint.request_repaint());
        self.poll_files();
        if ctx.input(|i| i.viewport().close_requested())
            && !self.close_confirmed
            && self.files.unsaved()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.file_command(FileCommand::Close);
        }
        for (modifiers, key, cmd) in FILE_SHORTCUTS {
            let shortcut = egui::KeyboardShortcut::new(modifiers, key);
            if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                self.file_command(cmd);
            }
        }
        if self.close_confirmed {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// The File menu; returns the chosen item.
    fn file_menu(&self, ui: &mut egui::Ui) -> Option<MenuItem> {
        let enabled = self.files_enabled();
        let mut chosen = None;
        ui.menu_button("File", |ui| {
            let entries = [
                ("New", FileCommand::New),
                ("Open…", FileCommand::Open),
                ("Save", FileCommand::Save),
                ("Save As…", FileCommand::SaveAs),
            ];
            for (text, cmd) in entries {
                let shortcut = FILE_SHORTCUTS
                    .iter()
                    .find(|(_, _, c)| *c == cmd)
                    .map(|&(m, k, _)| ui.ctx().format_shortcut(&egui::KeyboardShortcut::new(m, k)))
                    .unwrap_or_default();
                let button = egui::Button::new(text).shortcut_text(shortcut);
                if ui.add_enabled(enabled, button).clicked() {
                    chosen = Some(MenuItem::File(cmd));
                    ui.close_menu();
                }
            }
            ui.separator();
            let export = egui::Button::new("Export SVG…");
            if ui.add_enabled(enabled && self.export.is_none(), export).clicked() {
                chosen = Some(MenuItem::ExportSvg);
                ui.close_menu();
            }
        });
        chosen
    }

    /// The unsaved-changes question while it is open.
    fn unsaved_dialog(&mut self, ctx: &egui::Context) {
        if !self.files.asking() {
            return;
        }
        let name = self.files.file_name().unwrap_or("untitled").to_owned();
        let mut answer = None;
        egui::Window::new("Unsaved changes")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.label(format!("Save the changes to {name}?"));
                ui.horizontal(|ui| {
                    for (text, a) in [
                        ("Save", Answer::Save),
                        ("Discard", Answer::Discard),
                        ("Cancel", Answer::Cancel),
                    ] {
                        if ui.button(text).clicked() {
                            answer = Some(a);
                        }
                    }
                });
            });
        if let Some(a) = answer {
            self.file_answer(a);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.file_frame(ctx);
        if self.export_step() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        self.unsaved_dialog(ctx);
        if self.styles_window.open && !self.files.asking() {
            if let Some(cmd) = self.styles_window.show(ctx, self.session.styles()) {
                self.step(cmd, |_| "Styles changed".into());
            }
        }
        // The question is modal: tools and Canvas wait for the answer.
        let idle = !self.files.asking();
        egui::TopBottomPanel::top("tools").show(ctx, |ui| {
            if !idle {
                ui.disable();
            }
            ui.horizontal(|ui| {
                match self.file_menu(ui) {
                    Some(MenuItem::File(cmd)) => self.file_command(cmd),
                    Some(MenuItem::ExportSvg) => self.start_export(),
                    None => {}
                }
                ui.separator();
                ui.label("Tool:");
                ui.selectable_value(&mut self.tool, Tool::Select, "Select/Drag");
                ui.selectable_value(&mut self.tool, Tool::Segment, "Segment");
                ui.selectable_value(&mut self.tool, Tool::Join, "Join");
                ui.selectable_value(&mut self.tool, Tool::Parallel, "Parallel");
                ui.selectable_value(&mut self.tool, Tool::Orthogonal, "Orthogonal");
                ui.selectable_value(&mut self.tool, Tool::FixPoint, "FixPoint");
                ui.selectable_value(&mut self.tool, Tool::Brush, "Brush");
                let undo = ui.add_enabled(self.session.can_undo(), egui::Button::new("Undo"));
                if undo.clicked() && self.session.undo().is_ok() {
                    self.status = "Undone".into();
                    self.last_error = None;
                    self.history_changed();
                }
                let redo = ui.add_enabled(self.session.can_redo(), egui::Button::new("Redo"));
                if redo.clicked() && self.session.redo().is_ok() {
                    self.status = "Redone".into();
                    self.last_error = None;
                    self.history_changed();
                }
                if ui.button("Clear picks").clicked() {
                    self.pick_points.clear();
                    self.pick_geoms.clear();
                    self.segment_clicks.clear();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Show:");
                let prev = self.presentation;
                ui.selectable_value(
                    &mut self.presentation,
                    PresentationMode::ResultOnly,
                    "Result only",
                );
                ui.selectable_value(
                    &mut self.presentation,
                    PresentationMode::IncludeConstruction,
                    "Include construction",
                );
                if self.presentation != prev {
                    self.rebuild_logical_points();
                }
            });
            ui.horizontal(|ui| {
                self.current.keep_valid(self.session.styles());
                self.current.row(ui, self.session.styles());
                if ui.button("Styles…").clicked() {
                    self.styles_window.open = !self.styles_window.open;
                }
            });
            ui.label(&self.status);
            if let Some((s, color)) = &self.label_status {
                ui.colored_label(*color, s);
            }
            if let Some(err) = &self.last_error {
                ui.colored_label(Color32::RED, err);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Canvas: Mesh (world) → Viewport → screen; drag → Interact in world.
            let sense = if idle {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            };
            let (response, painter) = ui.allocate_painter(ui.available_size(), sense);
            let canvas: Rect = response.rect;
            let (min, max) = self.world_box;
            self.viewport = Viewport::fit(canvas, min, max, VIEW_MARGIN);

            let one_px = 1.0 / ctx.pixels_per_point();
            for (id, polyline) in polylines(&self.mesh()) {
                let paint = curve_paint(self.doc(), id, &polyline, &self.viewport, one_px);
                let stroke = Stroke::new(paint.width, paint.color);
                for piece in paint.pieces {
                    painter.add(egui::Shape::line(piece, stroke));
                }
                for head in paint.heads {
                    painter.add(egui::Shape::convex_polygon(
                        head.to_vec(),
                        paint.color,
                        Stroke::NONE,
                    ));
                }
            }
            let items = label_paint(
                self.session.document(),
                &self.viewport,
                self.presentation.mesh_filter(),
                &mut self.labels,
                self.platform.typesetter(),
            );
            let mut failure = None;
            for item in items {
                let f = self.paint_label(&painter, item);
                failure = failure.or(f);
            }
            self.label_status = match self.platform.typesetter().map(|t| t.state()) {
                Some(TypesetterState::Failed(msg)) => Some((
                    format!("typesetter could not be loaded: {msg}"),
                    NOT_TYPESET,
                )),
                Some(TypesetterState::Loading) if self.labels.any_pending() => {
                    Some(("loading typesetter…".to_owned(), PENDING))
                }
                _ => failure.map(|f| (f, NOT_TYPESET)),
            };
            for lp in &self.logical_points {
                let p = self.eval_point(&lp.representative);
                painter.circle_filled(
                    self.viewport.to_screen(p),
                    5.0,
                    Color32::from_rgb(30, 100, 200),
                );
            }
            for p in &self.segment_clicks {
                painter.circle_filled(
                    self.viewport.to_screen(*p),
                    4.0,
                    Color32::from_rgb(200, 120, 40),
                );
            }

            if self.tool == Tool::Select {
                if response.drag_started() {
                    // egui reports drag start only after the pointer moved past the
                    // click threshold; hit-test where the button went down, not where
                    // the pointer is now (coarse browser events can be > 12 px away).
                    let press = ui.input(|i| i.pointer.press_origin());
                    if let Some(pos) = press.or_else(|| response.interact_pointer_pos()) {
                        if let Some(expr) = self.hit_point(pos, 12.0) {
                            let point = self.address(&expr);
                            if let Err(e) = self.session.drag_begin(&point) {
                                self.last_error = Some(e.to_string());
                            }
                        }
                    }
                }
                if response.dragged() && self.session.is_dragging() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let world = self.viewport.to_world(pos);
                        if let Err(e) = self.session.drag_to(world) {
                            self.last_error = Some(e.to_string());
                            self.status = "Solve failed during drag — kept last good".into();
                        } else {
                            self.last_error = None;
                        }
                    }
                }
                if response.drag_stopped() && self.session.is_dragging() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let world = self.viewport.to_world(pos);
                        let r = self.session.drag_end(world);
                        self.report_drag(r);
                        self.files.refresh(&self.session, self.platform.files());
                    } else {
                        self.session.drag_cancel();
                    }
                }
            } else if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.handle_click(pos);
                }
            }
        });
    }
}

#[cfg(test)]
#[path = "tests_app.rs"]
mod tests_app;
