//! Application shell: Document, Interact, tools, and Canvas painting.

use constraint_runtime::{
    Constraint, Document, DragSession, GeometryId, GeometryRole, InteractiveSolve,
    Mesh, MeshFilter, Point2, PointExpr, SolveError, Segment,
};
use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke};

use crate::viewport::Viewport;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Select,
    Segment,
    Join,
    Parallel,
    Orthogonal,
    FixPoint,
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

/// Fixed world AABB used to fit the Viewport (sample sketch + margin room).
const WORLD_MIN: Point2 = Point2 { x: -0.5, y: -0.5 };
const WORLD_MAX: Point2 = Point2 { x: 2.5, y: 2.5 };
const VIEW_MARGIN: f32 = 40.0;

/// Owns Document and Interact; central panel is the Canvas.
pub struct App {
    doc: Document,
    interact: InteractiveSolve,
    drag: DragSession,
    tool: Tool,
    presentation: PresentationMode,
    pick_points: Vec<PointExpr>,
    pick_geoms: Vec<GeometryId>,
    segment_clicks: Vec<Point2>,
    status: String,
    last_error: Option<String>,
    logical_points: Vec<PointExpr>,
    /// Last fitted viewport (updated each paint from canvas rect).
    viewport: Viewport,
}

impl App {
    pub fn with_sample() -> Self {
        let mut doc = Document::new();
        // World units, y-up: horizontal then vertical leg of a right angle.
        let s0 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(2.0, 0.0)));
        let s1 = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(2.0, 0.0), Point2::new(2.0, 2.0)));
        // Demo: helper diagonal marked construction (hidden in result-only).
        let helper = doc.add_geometry::<Segment>(GeometryRole::Result, (Point2::new(0.0, 0.0), Point2::new(2.0, 2.0)));
        assert!(doc.set_geometry_role(helper, GeometryRole::Construction));
        doc.add_constraint(Constraint::Congruence {
            a: Segment::end_expr(s0),
            b: Segment::start_expr(s1),
        });
        doc.add_constraint(Constraint::Orthogonal {
            a: Segment::direction_expr(s0),
            b: Segment::direction_expr(s1),
        });
        doc.add_constraint(Constraint::FixPoint {
            point: Segment::start_expr(s0),
            target: Point2::new(0.0, 0.0),
        });

        let mut interact = InteractiveSolve::new();
        let _ = interact.structural_only(&mut doc);

        let presentation = PresentationMode::ResultOnly;
        let logical_points = logical_points_for(&doc, presentation.mesh_filter());

        Self {
            doc,
            interact,
            drag: DragSession::new(),
            tool: Tool::Select,
            presentation,
            pick_points: Vec::new(),
            pick_geoms: Vec::new(),
            segment_clicks: Vec::new(),
            status: "Sample right angle (world y-up) — drag the free end. Toggle construction to see helper."
                .into(),
            last_error: None,
            logical_points,
            viewport: Viewport::new(1.0, Pos2::ZERO),
        }
    }

    fn mesh(&self) -> Mesh {
        Mesh::from_structure(
            &self.doc.structure,
            self.doc.state(),
            self.presentation.mesh_filter(),
        )
    }

    fn eval_point(&self, expr: &PointExpr) -> Point2 {
        expr.evaluate(&self.doc.structure, self.doc.state())
    }

    fn rebuild_logical_points(&mut self) {
        self.logical_points =
            logical_points_for(&self.doc, self.presentation.mesh_filter());
    }

    fn report(&mut self, result: Result<(), SolveError>) {
        match result {
            Ok(()) => {
                self.last_error = None;
                self.status = format!(
                    "OK — geometries={} constraints={}",
                    self.doc.geometries().len(),
                    self.doc.constraints().len()
                );
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                self.status = format!("Solve failed — kept last good state. {e}");
            }
        }
        self.rebuild_logical_points();
    }

    fn hit_point(&self, screen: Pos2, radius_px: f64) -> Option<PointExpr> {
        let query = self.viewport.to_world(screen);
        let radius = self.viewport.px_to_world(radius_px);
        let mut best = None;
        let mut best_d = radius;
        for expr in &self.logical_points {
            let p = self.eval_point(expr);
            let d = (p.x - query.x).hypot(p.y - query.y);
            if d < best_d {
                best_d = d;
                best = Some(expr.clone());
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
                    self.doc.add_geometry::<Segment>(GeometryRole::Result, (a, b));
                    self.segment_clicks.clear();
                    self.rebuild_logical_points();
                    self.status = format!(
                        "Added segment — geometries={}",
                        self.doc.geometries().len()
                    );
                } else {
                    self.status = "Segment: click second endpoint".into();
                }
            }
            Tool::Join => {
                if let Some(expr) = self.hit_end(screen, 12.0) {
                    self.pick_points.push(expr);
                    if self.pick_points.len() >= 2 {
                        let a = self.pick_points[0];
                        let b = self.pick_points[1];
                        self.doc.add_constraint(Constraint::Congruence { a, b });
                        self.pick_points.clear();
                        let r = self.interact.structural_only(&mut self.doc);
                        self.report(r);
                        self.status = "Congruence between points".into();
                    } else {
                        self.status = "Join: pick second point".into();
                    }
                }
            }
            Tool::Parallel | Tool::Orthogonal => {
                if let Some(id) = self.hit_edge(screen, 24.0) {
                    self.pick_geoms.push(id);
                    if self.pick_geoms.len() >= 2 {
                        let a = self.pick_geoms[0];
                        let b = self.pick_geoms[1];
                        let c = if self.tool == Tool::Parallel {
                            Constraint::Parallel {
                                a: Segment::direction_expr(a),
                                b: Segment::direction_expr(b),
                            }
                        } else {
                            Constraint::Orthogonal {
                                a: Segment::direction_expr(a),
                                b: Segment::direction_expr(b),
                            }
                        };
                        self.doc.add_constraint(c);
                        self.pick_geoms.clear();
                        let r = self.interact.structural_only(&mut self.doc);
                        self.report(r);
                    } else {
                        self.status = "Pick second geometry".into();
                    }
                }
            }
            Tool::FixPoint => {
                if let Some(expr) = self.hit_point(screen, 12.0) {
                    let p = self.eval_point(&expr);
                    self.doc.add_constraint(Constraint::FixPoint {
                        point: expr,
                        target: p,
                    });
                    let r = self.interact.structural_only(&mut self.doc);
                    self.report(r);
                    self.status = "FixPoint added".into();
                }
            }
            Tool::Select => {}
        }
    }
}

fn logical_points_for(doc: &Document, filter: MeshFilter) -> Vec<PointExpr> {
    // Handles from Mesh endpoints (congruence-collapsed); not Mesh hit for drag.
    let mesh = Mesh::from_structure(&doc.structure, doc.state(), filter);
    let mut exprs = Vec::new();
    for e in &mesh.edges {
        exprs.push(Segment::start_expr(e.geometry));
        exprs.push(Segment::end_expr(e.geometry));
    }
    let mut keep = Vec::new();
    for expr in exprs {
        let p = expr.evaluate(&doc.structure, doc.state());
        let duplicate = keep.iter().any(|k: &PointExpr| {
            let q = k.evaluate(&doc.structure, doc.state());
            (p.x - q.x).hypot(p.y - q.y) < 1e-3 || congruent(doc, &expr, k)
        });
        if !duplicate {
            keep.push(expr);
        }
    }
    keep
}

fn congruent(doc: &Document, a: &PointExpr, b: &PointExpr) -> bool {
    if a == b {
        return true;
    }
    doc.constraints().iter().any(|c| {
        matches!(
            c,
            Constraint::Congruence { a: ca, b: cb }
                if (ca == a && cb == b) || (ca == b && cb == a)
        )
    })
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("tools").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Tool:");
                ui.selectable_value(&mut self.tool, Tool::Select, "Select/Drag");
                ui.selectable_value(&mut self.tool, Tool::Segment, "Segment");
                ui.selectable_value(&mut self.tool, Tool::Join, "Join");
                ui.selectable_value(&mut self.tool, Tool::Parallel, "Parallel");
                ui.selectable_value(&mut self.tool, Tool::Orthogonal, "Orthogonal");
                ui.selectable_value(&mut self.tool, Tool::FixPoint, "FixPoint");
                if ui.button("Undo drag").clicked() && self.drag.undo(&mut self.doc) {
                    self.status = "Undid last drag".into();
                    self.rebuild_logical_points();
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
            ui.label(&self.status);
            if let Some(err) = &self.last_error {
                ui.colored_label(Color32::RED, err);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Canvas: Mesh (world) → Viewport → screen; drag → Interact in world.
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            let canvas: Rect = response.rect;
            self.viewport = Viewport::fit(canvas, WORLD_MIN, WORLD_MAX, VIEW_MARGIN);

            let mesh = self.mesh();
            for e in &mesh.edges {
                let stroke = if e.role == GeometryRole::Construction {
                    Stroke::new(1.5_f32, Color32::from_rgb(160, 160, 180))
                } else {
                    Stroke::new(2.0_f32, Color32::from_rgb(40, 40, 40))
                };
                let a = self.viewport.to_screen(e.a);
                let b = self.viewport.to_screen(e.b);
                painter.line_segment([a, b], stroke);
            }
            for expr in &self.logical_points {
                let p = expr.evaluate(&self.doc.structure, self.doc.state());
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
                    if let Some(pos) = response.interact_pointer_pos() {
                        if let Some(expr) = self.hit_point(pos, 12.0) {
                            self.drag.pointer_down(&self.doc, expr);
                        }
                    }
                }
                if response.dragged() && self.drag.is_dragging() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let world = self.viewport.to_world(pos);
                        if let Err(e) = self.drag.pointer_move(
                            &mut self.interact,
                            &mut self.doc,
                            (world.x, world.y),
                        ) {
                            self.last_error = Some(e.to_string());
                            self.status = "Solve failed during drag — kept last good".into();
                        } else {
                            self.last_error = None;
                        }
                        self.rebuild_logical_points();
                    }
                }
                if response.drag_stopped() && self.drag.is_dragging() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let world = self.viewport.to_world(pos);
                        let r = self.drag.pointer_up(
                            &mut self.interact,
                            &mut self.doc,
                            (world.x, world.y),
                        );
                        self.report(r);
                    } else {
                        self.drag.cancel(&mut self.doc);
                        self.rebuild_logical_points();
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
