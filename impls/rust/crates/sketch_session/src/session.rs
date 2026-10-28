//! The session: document, history and drags.
//!
//! Atomicity and undo/redo both use Document snapshots: before a step the
//! session keeps a clone of the Document (structure, values, names); a failed
//! command restores it, a successful one pushes it onto the undo stack.

use constraint_runtime::{
    is_valid_name, Address, Arc, Congruence, ConstraintInst, Document, DragSession, EqualLength,
    FixPoint, Geometry, GeometryRole, HeadingAngle, InteractiveSolve, Length, LengthExpr,
    NameError, ObjectId, Point2, PointExpr, Segment, SolveError, TurnAngle, VecExpr,
    VectorsOrthogonal, VectorsParallel,
};

use crate::command::{Command, CommandError, Family, HistoryError, Outcome};

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
    /// Session on `doc` (the baseline; not an undo step).
    pub fn new(doc: Document) -> Self {
        Self {
            doc,
            interact: InteractiveSolve::new(),
            drag: DragSession::new(),
            drag_start: None,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn document(&self) -> &Document {
        &self.doc
    }

    /// Apply `cmd` atomically: on error the drawing is exactly as before.
    pub fn apply(&mut self, cmd: Command) -> Result<Outcome, CommandError> {
        if self.is_dragging() {
            return Err(CommandError::Busy);
        }
        let snapshot = self.doc.clone();
        match self.execute(cmd) {
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
        let expr = self.point(point)?;
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

    fn execute(&mut self, cmd: Command) -> Result<Outcome, CommandError> {
        match cmd {
            Command::CreateSegment {
                start,
                end,
                role,
                name,
            } => self.create::<Segment>((start, end), role, name),
            Command::CreateArc {
                center,
                radius,
                start_heading,
                sweep,
                role,
                name,
            } => {
                let init = (
                    center,
                    Length::new(radius),
                    HeadingAngle::new(start_heading),
                    TurnAngle::new(sweep),
                );
                self.create::<Arc>(init, role, name)
            }
            Command::Join { a, b, name } => {
                let c = Congruence {
                    a: self.point(&a)?,
                    b: self.point(&b)?,
                };
                self.constrain(c, name)
            }
            Command::Parallel { a, b, name } => {
                let c = VectorsParallel {
                    a: self.vector(&a)?,
                    b: self.vector(&b)?,
                };
                self.constrain(c, name)
            }
            Command::Perpendicular { a, b, name } => {
                let c = VectorsOrthogonal {
                    a: self.vector(&a)?,
                    b: self.vector(&b)?,
                };
                self.constrain(c, name)
            }
            Command::Fix {
                point,
                target,
                name,
            } => {
                let c = FixPoint {
                    point: self.point(&point)?,
                    target,
                };
                self.constrain(c, name)
            }
            Command::EqualLength { a, b, name } => {
                let c = EqualLength {
                    a: self.length(&a)?,
                    b: self.length(&b)?,
                };
                self.constrain(c, name)
            }
            Command::Delete { name } => {
                let removal = match self.object(&name)? {
                    ObjectId::Geometry(id) => self.doc.remove_geometry(id),
                    ObjectId::Constraint(id) => self.doc.remove_constraint(id),
                }
                .expect("looked-up objects exist");
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

    /// New geometry; no solve (an unconstrained geometry violates nothing).
    fn create<K: Geometry>(
        &mut self,
        init: K::Init,
        role: GeometryRole,
        name: Option<String>,
    ) -> Result<Outcome, CommandError> {
        self.check_new_name(name.as_deref())?;
        let id = self.doc.add_geometry::<K>(role, init);
        self.created(ObjectId::Geometry(id), name)
    }

    /// New constraint, then the eager structural solve.
    fn constrain(
        &mut self,
        c: impl Into<ConstraintInst>,
        name: Option<String>,
    ) -> Result<Outcome, CommandError> {
        self.check_new_name(name.as_deref())?;
        let id = self.doc.add_constraint(c);
        let outcome = self.created(ObjectId::Constraint(id), name)?;
        self.interact
            .structural_only(&mut self.doc)
            .map_err(|e| CommandError::Unsatisfiable {
                residual: e.residual_norm,
            })?;
        Ok(outcome)
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

    fn point(&self, address: &str) -> Result<PointExpr, CommandError> {
        match self.doc.resolve_address(address)? {
            Address::Point(e) => Ok(e),
            _ => Err(wrong_family(address, Family::Point)),
        }
    }

    fn vector(&self, address: &str) -> Result<VecExpr, CommandError> {
        match self.doc.resolve_address(address)? {
            Address::Vector(e) => Ok(e),
            _ => Err(wrong_family(address, Family::Vector)),
        }
    }

    fn length(&self, address: &str) -> Result<LengthExpr, CommandError> {
        match self.doc.resolve_address(address)? {
            Address::Length(e) => Ok(e),
            _ => Err(wrong_family(address, Family::Length)),
        }
    }
}

fn wrong_family(address: &str, expected: Family) -> CommandError {
    CommandError::WrongFamily {
        address: address.to_owned(),
        expected,
    }
}

#[cfg(test)]
#[path = "tests_session.rs"]
mod tests_session;
