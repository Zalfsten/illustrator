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
use eframe::egui::{self, epaint, Color32, Event, PointerButton, Pos2, Rect, Sense, Stroke};
use label_layout::TypesetterState;
use sketch_export::Format;
use sketch_session::{
    Command, Session,
};

use ui_model::{mesh_polylines, BoxMode, Effect as UiEffect, Entry, Filled, Intent, PickMode, Scene, ToolKind, UiModel};

use crate::export::{ExportJob, JobStep, WAITING};
use crate::files::{Answer, Effect, FileCommand, FileState, Step};
use crate::labels::{label_paint, FallbackReason, LabelCache, PaintItem};
use crate::platform::Platform;
use crate::style_ui::{CurrentStyle, StylesWindow};
use crate::navigation::{wheel_intent, zoom_label, NavIntent, WheelMode};
use crate::viewport::{print_scale, world_box, zoom_bounds, ViewState, Viewport};

/// Ribbon tab (UI plan #90); only SKETCH has tools so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RibbonTab {
    Sketch,
    Labels,
    Functions,
    Design,
}

/// Tool shortcuts (without modifiers; F is Fit, Esc is Cancel).
const TOOL_KEYS: [(egui::Key, ToolKind); 6] = [
    (egui::Key::L, ToolKind::Segment),
    (egui::Key::J, ToolKind::Join),
    (egui::Key::P, ToolKind::Parallel),
    (egui::Key::O, ToolKind::Orthogonal),
    (egui::Key::X, ToolKind::Fix),
    (egui::Key::B, ToolKind::Brush),
];

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

/// Chord tolerance of the display Mesh in screen pixels: curves stay within
/// a quarter pixel of their polyline at every view scale.
const MESH_TOLERANCE_PX: f64 = 0.25;

/// Pick radius of the Select tool and of point drags, in screen pixels.
const PICK_RADIUS_PX: f64 = 12.0;
/// Selection highlight (halo under selected curves, ring around points).
const SELECTION: Color32 = Color32::from_rgb(255, 150, 30);

/// An export in progress: waiting for labels, or its bytes waiting for the host.
enum Exporting {
    Job(Box<ExportJob>),
    Bytes(Vec<u8>, Format),
}

/// A choice in the File menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuItem {
    File(FileCommand),
    Export(Format),
}

/// Owns the command session; central panel is the Canvas.
pub struct App {
    session: Session,
    /// Selection shared by every view (ui-model).
    ui: UiModel,
    /// Screen position where a selection box drag began (Select tool).
    rubber: Option<Pos2>,
    ribbon_tab: RibbonTab,
    /// Browser (left) and inspector (right) panels shown (View → Panels).
    show_browser: bool,
    show_inspector: bool,
    presentation: PresentationMode,
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
    /// World ↔ screen mapping of the last paint (from `view` and the canvas).
    viewport: Viewport,
    /// The view (center, scale) kept across frames (system-architecture: The
    /// view keeps its state across frames).
    view: ViewState,
    /// A fit to `world_box` is due at the next paint (start, New, Open, Fit).
    pending_fit: bool,
    /// Canvas rect of the last paint (anchor fallback for menu zooms).
    canvas: Rect,
    /// World AABB the next fit uses; set on start, New, Open and Fit.
    world_box: (Point2, Point2),
    /// What the wheel / two-finger scroll does (session setting).
    wheel_mode: WheelMode,
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
            ui: UiModel::default(),
            rubber: None,
            ribbon_tab: RibbonTab::Sketch,
            show_browser: true,
            show_inspector: true,
            presentation,
            status: "Sample filleted right angle (world y-up) — drag a free end. Toggle construction to see helper."
                .into(),
            last_error: None,
            logical_points,
            platform,
            labels: LabelCache::new(),
            label_status: None,
            viewport: Viewport::new(1.0, Pos2::ZERO),
            view: ViewState {
                center: Point2::new(0.0, 0.0),
                scale: 1.0,
            },
            pending_fit: true,
            canvas: Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 600.0)),
            world_box: (WORLD_MIN, WORLD_MAX),
            wheel_mode: WheelMode::default(),
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
    fn start_export(&mut self, format: Format) {
        if !self.files_enabled() || self.export.is_some() {
            return;
        }
        self.export = Some(Exporting::Job(Box::new(ExportJob::new(self.session.document(), format))));
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
                JobStep::Ready(bytes) => (bytes, job.format()),
            },
            Some(Exporting::Bytes(bytes, format)) => (bytes, format),
        };
        let (bytes, format) = next;
        // Another file request first; the bytes wait for it.
        if self.files.busy() {
            self.export = Some(Exporting::Bytes(bytes, format));
        } else {
            self.files.export(bytes, format, self.platform.files());
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
        self.ui.clear();
        self.rubber = None;
        self.labels = LabelCache::new();
        self.rebuild_logical_points();
        self.fit_drawing();
    }

    /// Fit the view to the drawing at the next paint (system-architecture:
    /// The view can be fitted on demand); the default box when it is empty.
    fn fit_drawing(&mut self) {
        self.world_box = self.drawing_box().unwrap_or((WORLD_MIN, WORLD_MAX));
        self.pending_fit = true;
    }

    /// Bring the view up to date for `canvas`: a pending fit, the zoom bounds
    /// of the current document scale, and the Viewport for this paint.
    fn update_view(&mut self, canvas: Rect) {
        self.canvas = canvas;
        let bounds = zoom_bounds(self.session.settings());
        if self.pending_fit {
            let (min, max) = self.world_box;
            self.view = ViewState::fit(canvas, min, max, VIEW_MARGIN, bounds);
            self.pending_fit = false;
        }
        self.view.clamp(bounds);
        self.viewport = self.view.viewport(canvas);
    }

    /// Apply one navigation intent around `anchor` and refresh the Viewport.
    fn apply_intent(&mut self, intent: NavIntent, anchor: Pos2) {
        match intent {
            NavIntent::Zoom { factor } => {
                let bounds = zoom_bounds(self.session.settings());
                self.view.zoom(self.canvas, anchor, factor, bounds);
            }
            NavIntent::Pan(delta) => self.view.pan(delta),
        }
        self.viewport = self.view.viewport(self.canvas);
    }

    /// The Edit menu: undo and redo.
    fn edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Edit", |ui| {
            if ui.add_enabled(self.session.can_undo(), egui::Button::new("Undo")).clicked() {
                if self.session.undo().is_ok() {
                    self.status = "Undone".into();
                    self.last_error = None;
                    self.history_changed();
                }
                ui.close_menu();
            }
            if ui.add_enabled(self.session.can_redo(), egui::Button::new("Redo")).clicked() {
                if self.session.redo().is_ok() {
                    self.status = "Redone".into();
                    self.last_error = None;
                    self.history_changed();
                }
                ui.close_menu();
            }
        });
    }

    /// Tab row and the active tab's groups. Tool buttons send `ChooseTool`.
    fn ribbon(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (tab, text) in [
                (RibbonTab::Sketch, "SKETCH"),
                (RibbonTab::Labels, "LABELS"),
                (RibbonTab::Functions, "FUNCTIONS"),
                (RibbonTab::Design, "DESIGN"),
            ] {
                ui.selectable_value(&mut self.ribbon_tab, tab, egui::RichText::new(text).strong());
            }
        });
        // A narrow window scrolls the group row instead of cutting it off.
        egui::ScrollArea::horizontal().id_salt("ribbon_groups").show(ui, |ui| {
            ui.horizontal(|ui| match self.ribbon_tab {
                RibbonTab::Sketch => {
                    let mut chosen = None;
                    let active = self.ui.tool();
                    let mut group = |ui: &mut egui::Ui, title: &str, tools: &[(ToolKind, &str)]| {
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.small(title);
                                ui.horizontal(|ui| {
                                    for (tool, text) in tools {
                                        if ui.selectable_label(active == *tool, *text).clicked() {
                                            chosen = Some(*tool);
                                        }
                                    }
                                });
                            });
                        });
                    };
                    group(ui, "SELECT", &[(ToolKind::Select, "Select")]);
                    group(ui, "CREATE", &[(ToolKind::Segment, "Segment (L)")]);
                    group(
                        ui,
                        "CONSTRAIN",
                        &[
                            (ToolKind::Join, "Join (J)"),
                            (ToolKind::Parallel, "Parallel (P)"),
                            (ToolKind::Orthogonal, "Orthogonal (O)"),
                            (ToolKind::Fix, "Fix (X)"),
                        ],
                    );
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.small("STYLE");
                            ui.horizontal(|ui| {
                                if ui.selectable_label(active == ToolKind::Brush, "Brush (B)").clicked() {
                                    chosen = Some(ToolKind::Brush);
                                }
                                if ui.button("Styles…").clicked() {
                                    self.styles_window.open = !self.styles_window.open;
                                }
                            });
                        });
                    });
                    if let Some(tool) = chosen {
                        self.intent(Intent::ChooseTool(tool));
                    }
                }
                RibbonTab::Labels => {
                    ui.weak("Label tools — coming with #63.");
                }
                RibbonTab::Functions => {
                    ui.weak("Function curves — coming with #84.");
                }
                RibbonTab::Design => {
                    ui.weak("Design — later.");
                }
            });
        });
    }

    /// Inspector: the style for new geometry and Brush (the current look);
    /// properties of the selection follow with #93.
    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Inspector");
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::CollapsingHeader::new("Style for new geometry")
                .default_open(true)
                .show(ui, |ui| {
                    self.current.panel(ui, self.session.styles());
                    if ui.button("Styles…").clicked() {
                        self.styles_window.open = !self.styles_window.open;
                    }
                });
            ui.separator();
            ui.weak("Properties of the selection — coming with #93.");
        });
    }

    /// Zoom · tool prompt or selection · status; label status and errors below.
    fn status_line(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(zoom_label(self.view.percent(print_scale(self.session.settings()))));
            ui.separator();
            let doc = self.session.document();
            if let Some(prompt) = self.ui.prompt(doc) {
                ui.strong(prompt);
                ui.separator();
            } else if !self.ui.selection().is_empty() {
                ui.label(format!("Selected: {}", self.ui.selection().describe(doc)));
                ui.separator();
            }
            ui.label(&self.status);
        });
        if let Some((s, color)) = &self.label_status {
            ui.colored_label(*color, s);
        }
        if let Some(err) = &self.last_error {
            ui.colored_label(Color32::RED, err);
        }
    }

    /// The View menu: fit, print size, and the wheel / scroll setting.
    fn view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            if ui.button("Fit drawing (F)").clicked() {
                self.fit_drawing();
                ui.close_menu();
            }
            if ui.button("Print size (100 %)").clicked() {
                self.zoom_to_print_size();
                ui.close_menu();
            }
            ui.separator();
            ui.label("Show:");
            let mut mode = self.presentation;
            ui.radio_value(&mut mode, PresentationMode::ResultOnly, "Result only");
            ui.radio_value(&mut mode, PresentationMode::IncludeConstruction, "Include construction");
            if mode != self.presentation {
                self.set_presentation(mode);
            }
            ui.separator();
            ui.label("Panels:");
            ui.checkbox(&mut self.show_browser, "Browser");
            ui.checkbox(&mut self.show_inspector, "Inspector");
            ui.separator();
            ui.label("Wheel / scroll:");
            for (mode, text) in [
                (WheelMode::Automatic, "Automatic (wheel zooms, touchpad pans)"),
                (WheelMode::Zoom, "Zoom"),
                (WheelMode::Pan, "Pan"),
            ] {
                ui.radio_value(&mut self.wheel_mode, mode, text);
            }
        });
    }

    /// Zoom around the Canvas center to physical print size.
    fn zoom_to_print_size(&mut self) {
        let target = print_scale(self.session.settings());
        let factor = target / self.view.scale;
        self.apply_intent(NavIntent::Zoom { factor }, self.canvas.center());
    }

    /// Wheel, pinch and middle-button drag over the Canvas; `true` if the
    /// view changed this frame.
    fn navigate(&mut self, ctx: &egui::Context, response: &egui::Response) -> bool {
        let before = self.view;
        let anchor = response.hover_pos().unwrap_or(self.canvas.center());
        if response.hovered() || response.dragged() {
            let events = ctx.input(|i| i.events.clone());
            for event in events {
                let intent = match event {
                    Event::MouseWheel { unit, delta, modifiers } => {
                        wheel_intent(unit, delta, modifiers, self.wheel_mode)
                    }
                    Event::Zoom(factor) => Some(NavIntent::Zoom {
                        factor: f64::from(factor),
                    }),
                    _ => None,
                };
                if let Some(intent) = intent {
                    self.apply_intent(intent, anchor);
                }
            }
        }
        if response.dragged_by(PointerButton::Middle) {
            self.apply_intent(NavIntent::Pan(response.drag_delta()), anchor);
        }
        // `F` fits, unless a text field has the keyboard (labels, styles).
        if canvas_has_keyboard(ctx, response) && ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.fit_drawing();
        }
        self.view != before
    }

    /// Box around the visible geometry and the origins of visible labels.
    ///
    /// Uses the curves' shapes, not the Mesh: the Mesh's chord tolerance
    /// follows the current view, which this box is about to replace. The
    /// shapes take the export's view-independent tolerance for the same reason.
    fn drawing_box(&self) -> Option<(Point2, Point2)> {
        let doc = self.doc();
        let p = doc.vector();
        let tolerance = sketch_export::export_tolerance(doc.settings());
        let filter = self.presentation.mesh_filter();
        let visible = || {
            doc.geometries()
                .values()
                .filter(|g| filter.include_construction || g.role() != GeometryRole::Construction)
        };
        let curves = visible()
            .filter_map(|g| doc.curve_shape(g.id(), p, tolerance))
            .flatten()
            .flat_map(|piece| piece.bounding_points());
        let origins = visible()
            .filter(|g| doc.label_attrs(g.id()).is_some())
            .filter_map(|g| doc.origin_expr(g.id()))
            .map(|e| e.evaluate(doc, p));
        world_box(curves.chain(origins))
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
            self.viewport.px_to_world(MESH_TOLERANCE_PX),
        )
    }

    fn eval_point(&self, expr: &PointExpr) -> Point2 {
        expr.evaluate(self.doc(), self.doc().vector())
    }

    /// After any change of the drawing or the presentation mode: rebuild the
    /// grabbable points and drop selected objects that no longer exist.
    fn rebuild_logical_points(&mut self) {
        self.logical_points = self.doc().logical_points(self.presentation.mesh_filter());
        self.ui.reconcile(self.session.document());
    }

    /// Switch the presentation mode; selected objects it hides leave the
    /// selection (system-architecture: Hiding objects deselects them).
    fn set_presentation(&mut self, mode: PresentationMode) {
        self.presentation = mode;
        self.rebuild_logical_points();
        self.ui.hide_invisible(self.session.document(), mode.mesh_filter());
    }

    /// Apply a UI-model intent against what the Canvas shows.
    fn intent(&mut self, intent: Intent) {
        self.ui.set_look(self.current.look());
        let mesh = self.mesh();
        let scene = Scene {
            mesh: &mesh,
            points: &self.logical_points,
        };
        match self.ui.apply(intent, &mut self.session, &scene) {
            Ok(UiEffect::Deleted(names)) => {
                self.last_error = None;
                self.status = format!("Deleted {}", names.join(", "));
                self.files.refresh(&self.session, self.platform.files());
                self.rebuild_logical_points();
            }
            Ok(UiEffect::Nothing) if intent == Intent::Delete => {
                self.status = "Nothing selected to delete".into();
            }
            Ok(UiEffect::Issued(outcome)) => {
                self.last_error = None;
                let detail = self.ui.last_detail();
                self.status = if self.ui.tool() == ToolKind::Brush {
                    format!("Brushed {detail}")
                } else {
                    format!("Added {}{detail}", outcome.created.unwrap_or_default())
                };
                self.files.refresh(&self.session, self.platform.files());
                self.rebuild_logical_points();
            }
            Ok(UiEffect::Refused(why)) => self.status = why,
            Ok(UiEffect::Selection | UiEffect::Slot | UiEffect::Nothing) => {}
            Err(e) => {
                self.last_error = Some(e.to_string());
                self.status = format!("Rejected — drawing unchanged. {e}");
            }
        }
    }

    /// Select tool click at `screen`; `toggle` with Shift / Ctrl / Cmd.
    fn select_click(&mut self, screen: Pos2, toggle: bool) {
        let mode = if toggle { PickMode::Toggle } else { PickMode::Replace };
        self.intent(Intent::Pick {
            at: self.viewport.to_world(screen),
            tolerance: self.viewport.px_to_world(PICK_RADIUS_PX),
            mode,
        });
    }

    /// Click with a tool other than Select: offer the position to its next slot.
    fn tool_click(&mut self, screen: Pos2) {
        self.intent(Intent::Pick {
            at: self.viewport.to_world(screen),
            tolerance: self.viewport.px_to_world(PICK_RADIUS_PX),
            mode: PickMode::Replace,
        });
    }

    /// Select tool primary drag began at `press`: a point drags, empty space
    /// starts the selection box, a curve does nothing.
    fn select_drag_start(&mut self, press: Pos2) {
        if let Some(expr) = self.hit_point(press, PICK_RADIUS_PX) {
            let point = self.address(&expr);
            if let Err(e) = self.session.drag_begin(&point) {
                self.last_error = Some(e.to_string());
            }
        } else if self.hit_edge(press, PICK_RADIUS_PX).is_none() {
            self.rubber = Some(press);
        }
    }

    /// Select tool primary drag ended at `pos`: the box selects (adding with
    /// `add`), a point drag commits.
    fn select_drag_stop(&mut self, pos: Option<Pos2>, add: bool) {
        if let Some(start) = self.rubber.take() {
            if let Some(end) = pos {
                let mode = if add { BoxMode::Add } else { BoxMode::Replace };
                self.intent(Intent::Box {
                    a: self.viewport.to_world(start),
                    b: self.viewport.to_world(end),
                    mode,
                });
            }
            return;
        }
        if !self.session.is_dragging() {
            return;
        }
        if let Some(pos) = pos {
            let world = self.viewport.to_world(pos);
            let r = self.session.drag_end(world);
            self.report_drag(r);
            self.files.refresh(&self.session, self.platform.files());
        } else {
            self.session.drag_cancel();
        }
        self.ui.reconcile(self.session.document());
    }

    /// Esc cancels the tool or clears the selection (and drops a box in
    /// progress), Delete / Backspace deletes the selection, L / J / P / O / X
    /// / B choose tools; not while a text field has the keyboard.
    fn selection_keys(&mut self, ctx: &egui::Context, canvas: &egui::Response) {
        if !canvas_has_keyboard(ctx, canvas) {
            return;
        }
        let (esc, del) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
            )
        });
        if esc {
            self.rubber = None;
            self.intent(Intent::Cancel);
        }
        let tool = ctx.input(|i| {
            TOOL_KEYS
                .iter()
                .find(|(key, _)| i.modifiers.is_none() && i.key_pressed(*key))
                .map(|(_, tool)| *tool)
        });
        if let Some(tool) = tool {
            self.intent(Intent::ChooseTool(tool));
        }
        if del {
            self.intent(Intent::Delete);
        }
    }

    /// Halo under selected curves and rings around selected points; filled
    /// tool slots: halo for geometries, dots for points and positions.
    fn paint_selection(&self, painter: &egui::Painter, mesh: &Mesh) {
        let doc = self.doc();
        let filled = self.ui.filled();
        let selected = |id: GeometryId| {
            self.ui.selection().entries().contains(&Entry::Geometry(id)) || filled.contains(&Filled::Geometry(id))
        };
        for (id, polyline) in mesh_polylines(mesh) {
            if selected(id) {
                let pts: Vec<Pos2> = polyline.iter().map(|p| self.viewport.to_screen(*p)).collect();
                painter.add(egui::Shape::line(pts, Stroke::new(6.0_f32, SELECTION.gamma_multiply(0.6))));
            }
        }
        for e in self.ui.selection().entries() {
            if let Entry::Point(p) = e {
                let at = self.viewport.to_screen(p.evaluate(doc, doc.vector()));
                painter.circle_stroke(at, 9.0, Stroke::new(2.5_f32, SELECTION));
            }
        }
        for f in filled {
            let at = match f {
                Filled::Position(p) => *p,
                Filled::Point(p) => p.evaluate(doc, doc.vector()),
                Filled::Geometry(_) => continue,
            };
            painter.circle_filled(self.viewport.to_screen(at), 4.5, SELECTION);
        }
    }

    /// Apply `cmd`; status `Added <name> <detail>` or the command error.
    #[cfg(test)]
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

    #[cfg(test)]
    fn geometry_name(&self, id: GeometryId) -> &str {
        self.doc().geometry_name(id).unwrap_or("?")
    }

    /// `f` on what the Canvas shows, at the world image of `screen` with a
    /// world tolerance of `radius_px` screen pixels.
    fn hit<T>(&self, screen: Pos2, radius_px: f64, f: impl FnOnce(&Document, &Scene, Point2, f64) -> T) -> T {
        let mesh = self.mesh();
        let scene = Scene {
            mesh: &mesh,
            points: &self.logical_points,
        };
        let at = self.viewport.to_world(screen);
        f(self.doc(), &scene, at, self.viewport.px_to_world(radius_px))
    }

    fn hit_point(&self, screen: Pos2, radius_px: f64) -> Option<PointExpr> {
        self.hit(screen, radius_px, ui_model::nearest_point)
    }

    fn hit_edge(&self, screen: Pos2, radius_px: f64) -> Option<GeometryId> {
        self.hit(screen, radius_px, |_, scene, at, tol| ui_model::nearest_curve(scene, at, tol))
    }
}

/// Canvas shortcuts apply: no widget has the keyboard focus, or the Canvas
/// has it (clicking it focuses it). `Context::wants_keyboard_input` is not
/// enough: in egui it is true for any focused widget, the Canvas included.
fn canvas_has_keyboard(ctx: &egui::Context, canvas: &egui::Response) -> bool {
    ctx.memory(|m| m.focused()).is_none_or(|id| id == canvas.id)
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
            for (text, format) in [("Export SVG…", Format::Svg), ("Export PDF…", Format::Pdf)] {
                let button = egui::Button::new(text);
                if ui.add_enabled(enabled && self.export.is_none(), button).clicked() {
                    chosen = Some(MenuItem::Export(format));
                    ui.close_menu();
                }
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
        self.current.keep_valid(self.session.styles());
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            if !idle {
                ui.disable();
            }
            egui::menu::bar(ui, |ui| {
                match self.file_menu(ui) {
                    Some(MenuItem::File(cmd)) => self.file_command(cmd),
                    Some(MenuItem::Export(format)) => self.start_export(format),
                    None => {}
                }
                self.edit_menu(ui);
                self.view_menu(ui);
            });
        });
        egui::TopBottomPanel::top("ribbon").show(ctx, |ui| {
            if !idle {
                ui.disable();
            }
            self.ribbon(ui);
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| self.status_line(ui));
        if self.show_browser {
            egui::SidePanel::left("browser").resizable(true).default_width(180.0).show(ctx, |ui| {
                ui.heading("Browser");
                ui.weak("Objects by name — coming with #92.");
            });
        }
        if self.show_inspector {
            egui::SidePanel::right("inspector").resizable(true).default_width(220.0).show(ctx, |ui| {
                if !idle {
                    ui.disable();
                }
                self.inspector(ui);
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Canvas: Mesh (world) → Viewport → screen; drag → Interact in world.
            let sense = if idle {
                Sense::click_and_drag()
            } else {
                Sense::hover()
            };
            let (response, painter) = ui.allocate_painter(ui.available_size(), sense);
            let canvas: Rect = response.rect;
            self.update_view(canvas);
            let view_changed = idle && self.navigate(ctx, &response);

            if idle {
                self.selection_keys(ctx, &response);
            }
            let one_px = 1.0 / ctx.pixels_per_point();
            let mesh = self.mesh();
            self.paint_selection(&painter, &mesh);
            for (id, polyline) in mesh_polylines(&mesh) {
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
            if self.ui.tool() == ToolKind::Select {
                let modifiers = ctx.input(|i| i.modifiers);
                let extend = modifiers.shift || modifiers.command || modifiers.ctrl;
                // Only the primary button drags points or boxes; the middle button pans.
                if response.drag_started_by(PointerButton::Primary) {
                    // egui reports drag start only after the pointer moved past the
                    // click threshold; hit-test where the button went down, not where
                    // the pointer is now (coarse browser events can be > 12 px away).
                    let press = ui.input(|i| i.pointer.press_origin());
                    if let Some(pos) = press.or_else(|| response.interact_pointer_pos()) {
                        self.select_drag_start(pos);
                    }
                }
                // Pointer motion, or a zoom / pan under a still pointer: the
                // dragged point follows the pointer's world image in the new view.
                let retarget = response.dragged_by(PointerButton::Primary) || view_changed;
                if retarget && self.session.is_dragging() {
                    if let Some(pos) = response.interact_pointer_pos().or_else(|| ctx.pointer_latest_pos()) {
                        let world = self.viewport.to_world(pos);
                        if let Err(e) = self.session.drag_to(world) {
                            self.last_error = Some(e.to_string());
                            self.status = "Solve failed during drag — kept last good".into();
                        } else {
                            self.last_error = None;
                        }
                    }
                }
                if let (Some(start), Some(now)) = (self.rubber, response.interact_pointer_pos()) {
                    let rect = Rect::from_two_pos(start, now);
                    painter.rect(
                        rect,
                        0.0,
                        SELECTION.gamma_multiply(0.08),
                        Stroke::new(1.0_f32, SELECTION),
                    );
                }
                if response.drag_stopped_by(PointerButton::Primary) {
                    self.select_drag_stop(response.interact_pointer_pos(), extend);
                } else if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.select_click(pos, extend);
                    }
                }
            } else if response.clicked_by(PointerButton::Primary) {
                if let Some(pos) = response.interact_pointer_pos() {
                    self.tool_click(pos);
                }
            }
        });
    }
}

#[cfg(test)]
#[path = "tests_app.rs"]
mod tests_app;
