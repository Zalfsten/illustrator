//! Application shell: Document, Interact, tools, and Canvas painting.

use constraint_runtime::{
    Arc, Congruence, ConstraintId, CurveExprs, CurveParam, Document, DragSession, EndpointExprs,
    EqualLength, FixPoint, GeometryId, GeometryRole, HeadingAngle, InteractiveSolve, Length,
    LineExprs, Mesh, MeshFilter, Point2, PointExpr, Segment, SolveError, TurnAngle,
    VectorsOrthogonal, VectorsParallel,
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
            status: "Sample filleted right angle (world y-up) — drag a free end. Toggle construction to see helper."
                .into(),
            last_error: None,
            logical_points,
            viewport: Viewport::new(1.0, Pos2::ZERO),
        }
    }

    fn mesh(&self) -> Mesh {
        Mesh::from_document(
            &self.doc,
            self.doc.vector(),
            self.presentation.mesh_filter(),
        )
    }

    fn eval_point(&self, expr: &PointExpr) -> Point2 {
        expr.evaluate(&self.doc, self.doc.vector())
    }

    fn rebuild_logical_points(&mut self) {
        self.logical_points = logical_points_for(&self.doc, self.presentation.mesh_filter());
    }

    /// Status after a solve: `ok` on success, the solve error otherwise.
    fn report(&mut self, result: Result<(), SolveError>, ok: String) {
        match result {
            Ok(()) => {
                self.last_error = None;
                self.status = ok;
            }
            Err(e) => {
                self.last_error = Some(e.to_string());
                self.status = format!("Solve failed — kept last good state. {e}");
            }
        }
        self.rebuild_logical_points();
    }

    /// Generic success status (counts of objects).
    fn counts(&self) -> String {
        format!(
            "OK — geometries={} constraints={}",
            self.doc.geometries().len(),
            self.doc.constraints().len()
        )
    }

    /// Address of a picked point for status text (`seg1.end`).
    fn address(&self, p: &PointExpr) -> String {
        self.doc.point_address(p).unwrap_or_else(|| "?".into())
    }

    fn geometry_name(&self, id: GeometryId) -> &str {
        self.doc.geometry_name(id).unwrap_or("?")
    }

    fn constraint_name(&self, id: ConstraintId) -> &str {
        self.doc.constraint_name(id).unwrap_or("?")
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
                best = Some(*expr);
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
                    let id = self
                        .doc
                        .add_geometry::<Segment>(GeometryRole::Result, (a, b));
                    self.segment_clicks.clear();
                    self.rebuild_logical_points();
                    self.status = format!("Added {}", self.geometry_name(id));
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
                        let c = self.doc.add_constraint(Congruence { a, b });
                        self.pick_points.clear();
                        let ok = format!(
                            "Added {} ({} = {})",
                            self.constraint_name(c),
                            self.address(&a),
                            self.address(&b)
                        );
                        let r = self.interact.structural_only(&mut self.doc);
                        self.report(r, ok);
                    } else {
                        self.status = format!("Join: {} → pick second point", self.address(&expr));
                    }
                }
            }
            Tool::Parallel | Tool::Orthogonal => {
                if let Some(id) = self.hit_edge(screen, 24.0) {
                    // Lines only: the direction is a curve invariant; arcs have none.
                    if !self.doc.caps(id).is_some_and(|c| c.line) {
                        self.status = "Parallel/Orthogonal needs two line segments".into();
                        return;
                    }
                    self.pick_geoms.push(id);
                    if self.pick_geoms.len() >= 2 {
                        let direction = |id| {
                            self.doc
                                .direction_expr(id)
                                .expect("line capability checked on pick")
                        };
                        let a = direction(self.pick_geoms[0]);
                        let b = direction(self.pick_geoms[1]);
                        let c = if self.tool == Tool::Parallel {
                            self.doc.add_constraint(VectorsParallel { a, b })
                        } else {
                            self.doc.add_constraint(VectorsOrthogonal { a, b })
                        };
                        let ok = format!(
                            "Added {} ({}, {})",
                            self.constraint_name(c),
                            self.geometry_name(self.pick_geoms[0]),
                            self.geometry_name(self.pick_geoms[1])
                        );
                        self.pick_geoms.clear();
                        let r = self.interact.structural_only(&mut self.doc);
                        self.report(r, ok);
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
                    let p = self.eval_point(&expr);
                    let c = self.doc.add_constraint(FixPoint {
                        point: expr,
                        target: p,
                    });
                    let ok = format!(
                        "Added {} ({})",
                        self.constraint_name(c),
                        self.address(&expr)
                    );
                    let r = self.interact.structural_only(&mut self.doc);
                    self.report(r, ok);
                }
            }
            Tool::Select => {}
        }
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

fn logical_points_for(doc: &Document, filter: MeshFilter) -> Vec<PointExpr> {
    // Handles from Mesh endpoints (congruence-collapsed); not Mesh hit for drag.
    let mesh = Mesh::from_document(doc, doc.vector(), filter);
    let mut keep = Vec::new();
    for expr in mesh.endpoint_exprs() {
        let p = expr.evaluate(doc, doc.vector());
        let duplicate = keep.iter().any(|k: &PointExpr| {
            let q = k.evaluate(doc, doc.vector());
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
    doc.points_identified_with(a).contains(b)
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
                let p = expr.evaluate(&self.doc, self.doc.vector());
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
                        let ok = self.counts();
                        self.report(r, ok);
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
