//! The session: document, history and drags.
//!
//! Atomicity and undo/redo both use Document snapshots: before a step the
//! session keeps a clone of the Document (structure, values, names); a failed
//! command restores it, a successful one pushes it onto the undo stack.
//!
//! Contradictions are held per component of the constraint graph. A
//! component whose constraints are not met at its values is frozen: no
//! constraint may touch it and no drag may start in it, and only a delete
//! inside it gets it solved again. Every other command solves and checks
//! only the components it touches.

use std::collections::HashSet;

use constraint_runtime::{
    is_valid_name, Component, ConstraintInst, CurveParam, Document, DragSession, GeometryId,
    InteractiveSolve, NameError, ObjectId, Point2, SolveError, Vector,
};

use crate::catalog::{ConstraintSpec, GeometrySpec};
use crate::command::{
    Adjusted, Command, CommandError, Conflict, DeclCommand, ExportError, HistoryError, LoadReport,
    Outcome,
};

/// Curve positions `k / DISPLACEMENT_SAMPLES` at which the displacement of an
/// adjusted component is measured (both ends included).
const DISPLACEMENT_SAMPLES: u32 = 32;
use crate::kinds::Resolver;

/// How a command runs: interactively (frozen check and eager solve), or while
/// loading a file, where the session start judges the final values instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Interactive,
    Load,
}

/// A drawing being edited: the only mutation path for front-ends.
pub struct Session {
    doc: Document,
    interact: InteractiveSolve,
    drag: DragSession,
    /// Document before the drag in progress (`Some` while dragging).
    drag_start: Option<Document>,
    /// Documents before each step, most recent last.
    undo: Vec<Document>,
    /// Undone results, most recent last.
    redo: Vec<Document>,
}

impl Session {
    /// Session on `doc` (the baseline; not an undo step). Each component
    /// whose constraints are not met gets one solve; the ones that still
    /// fail stay frozen with their values.
    pub fn new(doc: Document) -> Self {
        Self::start(doc).0
    }

    /// [`Session::new`], also reporting what the start-up solves changed.
    pub(crate) fn start(doc: Document) -> (Self, LoadReport) {
        let mut s = Self {
            doc,
            interact: InteractiveSolve::new(),
            drag: DragSession::new(),
            drag_start: None,
            undo: Vec::new(),
            redo: Vec::new(),
        };
        let mut report = LoadReport::default();
        for c in s.doc.components() {
            if s.doc.is_consistent(&c) {
                continue;
            }
            let before = s.doc.clone_vector();
            if s.interact.structural_scoped(&mut s.doc, &c).is_ok() {
                report.adjusted.push(Adjusted {
                    geometries: s.names(&c.geometries),
                    displacement: displacement(&s.doc, &c, &before),
                });
            }
        }
        report.conflicts = s.conflicts();
        (s, report)
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    pub(crate) fn into_document(self) -> Document {
        self.doc
    }

    /// The conflicted (frozen) components, in order of their first geometry.
    pub fn conflicts(&self) -> Vec<Conflict> {
        let doc = &self.doc;
        let name = |id| doc.name(id).expect("objects are named").to_owned();
        doc.components()
            .into_iter()
            .filter(|c| !doc.is_consistent(c))
            .map(|c| Conflict {
                geometries: self.names(&c.geometries),
                constraints: c
                    .constraints
                    .iter()
                    .map(|&k| name(ObjectId::Constraint(k)))
                    .collect(),
                residual: doc.component_residual(&c, doc.vector()),
            })
            .collect()
    }

    /// The drawing as declarative commands: geometries, then constraints, each
    /// in creation order, with current names, roles, values and addresses.
    pub fn declarative_commands(&self) -> Result<Vec<DeclCommand>, ExportError> {
        let doc = &self.doc;
        let name = |id| doc.name(id).expect("objects are named").to_owned();
        let mut out = Vec::new();
        for g in doc.geometries().values() {
            out.push(DeclCommand::Create {
                name: name(ObjectId::Geometry(g.id())),
                role: g.role(),
                geometry: GeometrySpec::of(doc, g),
            });
        }
        for e in doc.constraints().values() {
            let name = name(ObjectId::Constraint(e.id));
            let Some(constraint) = ConstraintSpec::of(doc, e.body()) else {
                return Err(ExportError::NotAddressable { constraint: name });
            };
            out.push(DeclCommand::Constrain { name, constraint });
        }
        Ok(out)
    }

    /// Apply `cmd` atomically: on error the drawing is exactly as before.
    pub fn apply(&mut self, cmd: Command) -> Result<Outcome, CommandError> {
        if self.is_dragging() {
            return Err(CommandError::Busy);
        }
        let snapshot = self.doc.clone();
        match self.execute(cmd, Mode::Interactive) {
            Ok(outcome) => {
                self.undo.push(snapshot);
                self.redo.clear();
                Ok(outcome)
            }
            Err(e) => {
                self.doc = snapshot;
                Err(e)
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Restore the drawing from before the most recent step.
    pub fn undo(&mut self) -> Result<(), HistoryError> {
        if self.is_dragging() {
            return Err(HistoryError::Busy);
        }
        let prev = self.undo.pop().ok_or(HistoryError::NothingToUndo)?;
        self.redo.push(std::mem::replace(&mut self.doc, prev));
        Ok(())
    }

    /// Restore the most recently undone result.
    pub fn redo(&mut self) -> Result<(), HistoryError> {
        if self.is_dragging() {
            return Err(HistoryError::Busy);
        }
        let next = self.redo.pop().ok_or(HistoryError::NothingToRedo)?;
        self.undo.push(std::mem::replace(&mut self.doc, next));
        Ok(())
    }

    pub fn is_dragging(&self) -> bool {
        self.drag_start.is_some()
    }

    /// Start dragging the point at `point` (an address).
    pub fn drag_begin(&mut self, point: &str) -> Result<(), CommandError> {
        if self.is_dragging() {
            return Err(CommandError::Busy);
        }
        let expr = Resolver::new(&self.doc).point(&point.into())?;
        self.check_not_frozen(expr.referenced_geometry())?;
        self.drag_start = Some(self.doc.clone());
        self.drag.pointer_down(&self.doc, expr);
        Ok(())
    }

    /// Solve towards `target`; on failure the last consistent state stays.
    pub fn drag_to(&mut self, target: Point2) -> Result<(), SolveError> {
        self.drag
            .pointer_move(&mut self.interact, &mut self.doc, (target.x, target.y))
    }

    /// Finish the drag at `target`. Success is one undo step; on failure the
    /// drawing returns to the state before the drag.
    pub fn drag_end(&mut self, target: Point2) -> Result<(), SolveError> {
        let Some(start) = self.drag_start.take() else {
            return Ok(());
        };
        let result = self
            .drag
            .pointer_up(&mut self.interact, &mut self.doc, (target.x, target.y));
        match result {
            Ok(()) => {
                self.undo.push(start);
                self.redo.clear();
            }
            Err(_) => self.doc = start,
        }
        result
    }

    /// Abort the drag; the drawing returns to the state before it.
    pub fn drag_cancel(&mut self) {
        if let Some(start) = self.drag_start.take() {
            self.drag.cancel(&mut self.doc);
            self.doc = start;
        }
    }

    pub(crate) fn execute(&mut self, cmd: Command, mode: Mode) -> Result<Outcome, CommandError> {
        match cmd {
            Command::Create {
                name,
                role,
                geometry,
            } => {
                // No solve: an unconstrained geometry violates nothing.
                self.check_new_name(name.as_deref())?;
                let id = geometry.add_to(&mut self.doc, role)?;
                self.created(ObjectId::Geometry(id), name)
            }
            Command::Constrain { name, constraint } => {
                let c = constraint.build(&Resolver::new(&self.doc))?;
                self.constrain(c, name, mode)
            }
            Command::Delete { name } => {
                let id = self.object(&name)?;
                let before = self.doc.component_of(self.anchor(id));
                let was_frozen = !self.doc.is_consistent(&before);
                let removal = match id {
                    ObjectId::Geometry(id) => self.doc.remove_geometry(id),
                    ObjectId::Constraint(id) => self.doc.remove_constraint(id),
                }
                .expect("looked-up objects exist");
                // Removing constraints never breaks a consistent component;
                // the pieces of a frozen one get one chance to solve.
                if was_frozen {
                    self.resolve(&before.geometries);
                }
                Ok(Outcome {
                    created: None,
                    removed: removal.names().map(str::to_owned).collect(),
                })
            }
            Command::Rename { name, new_name } => {
                let id = self.object(&name)?;
                self.doc.rename(id, &new_name)?;
                Ok(Outcome::default())
            }
            Command::SetRole { name, role } => match self.object(&name)? {
                ObjectId::Geometry(id) => {
                    self.doc.set_geometry_role(id, role);
                    Ok(Outcome::default())
                }
                ObjectId::Constraint(_) => Err(CommandError::WrongObjectType {
                    name,
                    expected: "geometry",
                }),
            },
        }
    }

    /// New constraint, then the eager structural solve of the component it
    /// ends up in (its operands' components merged). Loading skips both the
    /// frozen check and the solve.
    fn constrain(
        &mut self,
        c: ConstraintInst,
        name: Option<String>,
        mode: Mode,
    ) -> Result<Outcome, CommandError> {
        self.check_new_name(name.as_deref())?;
        let geometries = c.referenced_geometries();
        if mode == Mode::Load {
            let id = self.doc.add_constraint(c);
            return self.created(ObjectId::Constraint(id), name);
        }
        for &g in &geometries {
            self.check_not_frozen(g)?;
        }
        let id = self.doc.add_constraint(c);
        let outcome = self.created(ObjectId::Constraint(id), name)?;
        let scope = self.doc.component_of(geometries[0]);
        self.interact
            .structural_scoped(&mut self.doc, &scope)
            .map_err(|e| CommandError::Unsatisfiable {
                residual: e.residual_norm,
            })?;
        Ok(outcome)
    }

    fn check_not_frozen(&self, g: GeometryId) -> Result<(), CommandError> {
        if self.doc.is_consistent(&self.doc.component_of(g)) {
            return Ok(());
        }
        let name = self.doc.geometry_name(g).expect("geometries are named");
        Err(CommandError::Frozen {
            name: name.to_owned(),
        })
    }

    /// A geometry in the component of `id`.
    fn anchor(&self, id: ObjectId) -> GeometryId {
        match id {
            ObjectId::Geometry(g) => g,
            ObjectId::Constraint(k) => {
                let c = self.doc.constraint(k).expect("looked-up objects exist");
                c.referenced_geometries()[0]
            }
        }
    }

    /// One structural solve for each inconsistent component that holds one of
    /// `geometries` (the survivors of a frozen component after a delete).
    fn resolve(&mut self, geometries: &[GeometryId]) {
        let mut done = HashSet::new();
        for &g in geometries {
            if self.doc.geometry(g).is_none() || done.contains(&g) {
                continue;
            }
            let c = self.doc.component_of(g);
            done.extend(c.geometries.iter().copied());
            if !self.doc.is_consistent(&c) {
                let _ = self.interact.structural_scoped(&mut self.doc, &c);
            }
        }
    }

    fn names(&self, geometries: &[GeometryId]) -> Vec<String> {
        geometries
            .iter()
            .map(|&g| {
                let name = self.doc.geometry_name(g).expect("geometries are named");
                name.to_owned()
            })
            .collect()
    }

    fn check_new_name(&self, name: Option<&str>) -> Result<(), CommandError> {
        match name {
            Some(n) if !is_valid_name(n) => Err(NameError::Invalid.into()),
            Some(n) if self.doc.lookup(n).is_some() => Err(NameError::Taken.into()),
            _ => Ok(()),
        }
    }

    /// Give `id` its requested name and report it.
    fn created(&mut self, id: ObjectId, name: Option<String>) -> Result<Outcome, CommandError> {
        if let Some(n) = name {
            self.doc.rename(id, &n)?;
        }
        let name = self.doc.name(id).expect("new objects are named");
        Ok(Outcome {
            created: Some(name.to_owned()),
            removed: Vec::new(),
        })
    }

    fn object(&self, name: &str) -> Result<ObjectId, CommandError> {
        self.doc
            .lookup(name)
            .ok_or_else(|| CommandError::UnknownName(name.to_owned()))
    }
}

/// Largest distance a curve point of `c` moved from `before` to the committed
/// values, compared at the same curve position (sampled).
fn displacement(doc: &Document, c: &Component, before: &Vector) -> f64 {
    let mut max: f64 = 0.0;
    for &g in &c.geometries {
        for k in 0..=DISPLACEMENT_SAMPLES {
            let t = f64::from(k) / f64::from(DISPLACEMENT_SAMPLES);
            let Some(p) = doc.on_curve_expr(g, CurveParam::Normalized(t)) else {
                break;
            };
            let moved = p.evaluate(doc, doc.vector()) - p.evaluate(doc, before);
            max = max.max(moved.length());
        }
    }
    max
}

#[cfg(test)]
#[path = "tests_session.rs"]
mod tests_session;
