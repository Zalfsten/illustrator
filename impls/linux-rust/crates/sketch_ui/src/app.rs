//! Application shell: Document, Interact, tools, and Canvas painting.

use constraint_runtime::{
    Constraint, Document, DragSession, EndpointKind, EndpointRef, GeometryId, InteractiveSolve,
    Mesh, Point2, PointExpr, SolveError,
};
use eframe::egui::{self, Color32, Pos2, Sense, Stroke};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Select,
    Segment,
    Join,
    Parallel,
    Orthogonal,
    FixPoint,
}

/// Owns Document and Interact; central panel is the Canvas.
pub struct App {
    doc: Document,
    interact: InteractiveSolve,
    drag: DragSession,
    tool: Tool,
    pick_ends: Vec<(GeometryId, EndpointKind)>,
    segment_clicks: Vec<Point2>,
    status: String,
    last_error: Option<String>,
    logical_points: Vec<PointExpr>,
}

impl App {
    pub fn with_sample() -> Self {
        let mut app = Self {
            doc: Document::new(),
            interact: InteractiveSolve::default(),
            drag: DragSession::new(),
            tool: Tool::Select,
            pick_ends: Vec::new(),
            segment_clicks: Vec::new(),
            status: "Sample right angle — drag the free end.".into(),
            last_error: None,
            logical_points: Vec::new(),
        };
        let s0 = app
            .doc
            .add_segment(Point2::new(120.0, 320.0), Point2::new(320.0, 320.0));
        let s1 = app
            .doc
            .add_segment(Point2::new(320.0, 320.0), Point2::new(320.0, 140.0));
        app.doc.join_endpoints(
            EndpointRef::new(s0, EndpointKind::End),
            EndpointRef::new(s1, EndpointKind::Start),
        );
        app.doc.add_constraint(Constraint::Orthogonal { a: s0, b: s1 });
        app.doc.add_constraint(Constraint::FixPoint {
            point: PointExpr::Endpoint {
                geometry: s0,
                end: EndpointKind::Start,
            },
            target: Point2::new(120.0, 320.0),
        });
        let _ = app.interact.structural_only(&mut app.doc);
        app.rebuild_logical_points();
        app
    }

    fn mesh(&self) -> Mesh {
        Mesh::from_structure(&self.doc.structure, self.doc.state())
    }

    fn eval_point(&self, expr: PointExpr) -> Point2 {
        expr.evaluate(&self.doc.structure, self.doc.state())
    }

    fn rebuild_logical_points(&mut self) {
        // Handles from Mesh endpoints (congruence-collapsed); not Mesh hit for drag.
        let mut exprs = Vec::new();
        for e in &self.mesh().edges {
            exprs.push(PointExpr::Endpoint {
                geometry: e.geometry,
                end: EndpointKind::Start,
            });
            exprs.push(PointExpr::Endpoint {
                geometry: e.geometry,
                end: EndpointKind::End,
            });
        }
        let mut keep = Vec::new();
        for expr in exprs {
            let p = expr.evaluate(&self.doc.structure, self.doc.state());
            let duplicate = keep.iter().any(|k: &PointExpr| {
                let q = k.evaluate(&self.doc.structure, self.doc.state());
                (p.x - q.x).hypot(p.y - q.y) < 1e-3 || congruent(&self.doc, expr, *k)
            });
            if !duplicate {
                keep.push(expr);
            }
        }
        self.logical_points = keep;
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

    fn hit_point(&self, pos: Pos2, radius: f64) -> Option<PointExpr> {
        let mut best = None;
        let mut best_d = radius;
        for expr in &self.logical_points {
            let p = self.eval_point(*expr);
            let d = (p.x - pos.x as f64).hypot(p.y - pos.y as f64);
            if d < best_d {
                best_d = d;
                best = Some(*expr);
            }
        }
        best
    }

    fn hit_end(&self, pos: Pos2, radius: f64) -> Option<(GeometryId, EndpointKind)> {
        self.mesh()
            .hit_endpoint(Point2::new(pos.x as f64, pos.y as f64), radius)
    }

    fn hit_edge(&self, pos: Pos2, radius: f64) -> Option<GeometryId> {
        self.mesh()
            .hit_geometry(Point2::new(pos.x as f64, pos.y as f64), radius)
    }

    fn handle_click(&mut self, pos: Pos2) {
        match self.tool {
            Tool::Segment => {
                self.segment_clicks
                    .push(Point2::new(pos.x as f64, pos.y as f64));
                if self.segment_clicks.len() >= 2 {
                    let a = self.segment_clicks[0];
                    let b = self.segment_clicks[1];
                    self.doc.add_segment(a, b);
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
                if let Some(h) = self.hit_end(pos, 12.0) {
                    self.pick_ends.push(h);
                    if self.pick_ends.len() >= 2 {
                        let a = self.pick_ends[0];
                        let b = self.pick_ends[1];
                        self.doc
                            .join_endpoints(EndpointRef::new(a.0, a.1), EndpointRef::new(b.0, b.1));
                        self.pick_ends.clear();
                        let r = self.interact.structural_only(&mut self.doc);
                        self.report(r);
                        self.status = "Joined endpoints (Congruence)".into();
                    } else {
                        self.status = "Join: pick second endpoint".into();
                    }
                }
            }
            Tool::Parallel | Tool::Orthogonal => {
                if let Some(id) = self.hit_edge(pos, 24.0) {
                    self.pick_ends.push((id, EndpointKind::Start));
                    if self.pick_ends.len() >= 2 {
                        let a = self.pick_ends[0].0;
                        let b = self.pick_ends[1].0;
                        let c = if self.tool == Tool::Parallel {
                            Constraint::Parallel { a, b }
                        } else {
                            Constraint::Orthogonal { a, b }
                        };
                        self.doc.add_constraint(c);
                        self.pick_ends.clear();
                        let r = self.interact.structural_only(&mut self.doc);
                        self.report(r);
                    } else {
                        self.status = "Pick second geometry".into();
                    }
                }
            }
            Tool::FixPoint => {
                if let Some(expr) = self.hit_point(pos, 12.0) {
                    let p = self.eval_point(expr);
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

fn congruent(doc: &Document, a: PointExpr, b: PointExpr) -> bool {
    if a == b {
        return true;
    }
    doc.constraints().iter().any(|c| {
        matches!(
            c,
            Constraint::Congruence { a: ca, b: cb }
                if (*ca == a && *cb == b) || (*ca == b && *cb == a)
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
                    self.pick_ends.clear();
                    self.segment_clicks.clear();
                }
            });
            ui.label(&self.status);
            if let Some(err) = &self.last_error {
                ui.colored_label(Color32::RED, err);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Canvas: Mesh from Document+State; drag → Interact (logical points).
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());

            let mesh = self.mesh();
            for e in &mesh.edges {
                painter.line_segment(
                    [
                        Pos2::new(e.a.x as f32, e.a.y as f32),
                        Pos2::new(e.b.x as f32, e.b.y as f32),
                    ],
                    Stroke::new(2.0_f32, Color32::from_rgb(40, 40, 40)),
                );
            }
            for expr in &self.logical_points {
                let p = expr.evaluate(&self.doc.structure, self.doc.state());
                painter.circle_filled(
                    Pos2::new(p.x as f32, p.y as f32),
                    5.0,
                    Color32::from_rgb(30, 100, 200),
                );
            }
            for p in &self.segment_clicks {
                painter.circle_filled(
                    Pos2::new(p.x as f32, p.y as f32),
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
                        if let Err(e) = self.drag.pointer_move(
                            &mut self.interact,
                            &mut self.doc,
                            (pos.x as f64, pos.y as f64),
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
                        let r = self.drag.pointer_up(
                            &mut self.interact,
                            &mut self.doc,
                            (pos.x as f64, pos.y as f64),
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
