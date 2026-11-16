//! The UI model's state and its intents (ui-model: front ends drive the UI
//! model through intents; pick, box, delete the selection; tools with slots,
//! pre-fill, picks, stay active, cancel, prompt).

use std::fmt;

use constraint_runtime::{Document, GeometryId, MeshFilter, ObjectId, Point2, PointExpr, StyleRef};
use sketch_session::{Command, CommandError, GroupError, Outcome, Session};

use crate::pick::{box_select, nearest_curve, nearest_line, nearest_point, pick, Scene};
use crate::selection::{Entry, PickMode, Selection};
use crate::tool::{Filled, SlotKind, ToolKind};

/// How a box changes the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxMode {
    /// The geometries inside become the selection.
    Replace,
    /// The geometries inside are added.
    Add,
}

/// A device-neutral request from a front end. Positions and tolerances are
/// world units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Intent {
    Pick { at: Point2, tolerance: f64, mode: PickMode },
    Box { a: Point2, b: Point2, mode: BoxMode },
    Clear,
    /// Delete the selected geometries and constraints as one history step.
    Delete,
    /// Make this the active tool; its slots fill from the selection.
    ChooseTool(ToolKind),
    /// End the active tool, or clear the selection while selecting.
    Cancel,
}

/// What an intent did.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// The selection may have changed; the drawing did not.
    Selection,
    /// One history step deleted these objects (cascade included).
    Deleted(Vec<String>),
    /// The active tool issued its command (and stays active).
    Issued(Outcome),
    /// A slot of the active tool was filled; more are needed.
    Slot,
    /// The pick did not fit the next slot (why, for the user).
    Refused(String),
    /// Nothing changed.
    Nothing,
}

/// Why an intent changed nothing in the drawing although it tried.
#[derive(Clone, Debug, PartialEq)]
pub enum UiError {
    /// A delete group failed.
    Group(GroupError),
    /// The session rejected a tool's command.
    Command(CommandError),
}

impl fmt::Display for UiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiError::Group(e) => e.fmt(f),
            UiError::Command(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for UiError {}

impl From<GroupError> for UiError {
    fn from(e: GroupError) -> Self {
        UiError::Group(e)
    }
}

/// Interaction state shared by every view of a front end.
#[derive(Clone, Debug, Default)]
pub struct UiModel {
    selection: Selection,
    tool: ToolKind,
    /// Filled slots of `tool`, in slot order.
    filled: Vec<Filled>,
    /// Style and overrides for new geometry and Brush.
    look: StyleRef,
    /// Detail of the last issued command (` (seg1.end = seg2.start)`).
    last_detail: String,
}

impl UiModel {
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// For views that select by name ([`Selection::select_name`]).
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    pub fn tool(&self) -> ToolKind {
        self.tool
    }

    pub fn filled(&self) -> &[Filled] {
        &self.filled
    }

    /// The front end's current look (style and overrides).
    pub fn set_look(&mut self, look: StyleRef) {
        self.look = look;
    }

    /// Detail of the last issued command, for `Added <name><detail>`.
    pub fn last_detail(&self) -> &str {
        &self.last_detail
    }

    /// `Parallel: seg1 → pick a direction`; `None` while selecting.
    pub fn prompt(&self, doc: &Document) -> Option<String> {
        let next = self.tool.slots().get(self.filled.len())?;
        let done: Vec<String> = self.filled.iter().map(|f| f.label(doc)).collect();
        let done = if done.is_empty() { String::new() } else { format!("{} ", done.join(", ")) };
        Some(format!("{}: {done}→ pick {}", self.tool.name(), next.noun()))
    }

    /// Apply `intent`. `scene` is what the Canvas shows of `session`'s drawing.
    pub fn apply(&mut self, intent: Intent, session: &mut Session, scene: &Scene) -> Result<Effect, UiError> {
        let doc = session.document();
        match intent {
            Intent::ChooseTool(tool) => self.choose_tool(tool, session),
            Intent::Cancel => {
                if self.tool == ToolKind::Select {
                    self.selection.clear();
                } else {
                    self.tool = ToolKind::Select;
                    self.filled.clear();
                }
                Ok(Effect::Selection)
            }
            Intent::Pick { at, tolerance, .. } if self.tool != ToolKind::Select => {
                self.pick_slot(at, tolerance, session, scene)
            }
            Intent::Box { .. } if self.tool != ToolKind::Select => Ok(Effect::Nothing),
            Intent::Pick { at, tolerance, mode } => {
                match (pick(doc, scene, at, tolerance), mode) {
                    (Some(e), _) => self.selection.pick(e, mode, doc),
                    (None, PickMode::Replace) => self.selection.clear(),
                    (None, PickMode::Toggle) => return Ok(Effect::Nothing),
                }
                Ok(Effect::Selection)
            }
            Intent::Box { a, b, mode } => {
                if mode == BoxMode::Replace {
                    self.selection.clear();
                }
                for id in box_select(scene, a, b) {
                    self.selection.add(Entry::Geometry(id), doc);
                }
                Ok(Effect::Selection)
            }
            Intent::Clear => {
                self.selection.clear();
                Ok(Effect::Selection)
            }
            Intent::Delete => Ok(self.delete(session)?),
        }
    }

    /// Set the tool, move fitting selection entries into its slots, clear the
    /// selection, and issue at once if complete.
    fn choose_tool(&mut self, tool: ToolKind, session: &mut Session) -> Result<Effect, UiError> {
        self.tool = tool;
        self.filled.clear();
        if tool == ToolKind::Select {
            return Ok(Effect::Selection);
        }
        let doc = session.document();
        let slots = tool.slots();
        for e in self.selection.entries() {
            let Some(next) = slots.get(self.filled.len()) else { break };
            if next.fits(e, doc) {
                self.filled.extend(Filled::from_entry(e));
            }
        }
        self.selection.clear();
        if self.filled.len() == slots.len() {
            self.issue(session)
        } else {
            Ok(Effect::Selection)
        }
    }

    /// Offer the object (or position) at `at` to the next empty slot.
    fn pick_slot(&mut self, at: Point2, tolerance: f64, session: &mut Session, scene: &Scene) -> Result<Effect, UiError> {
        let Some(&next) = self.tool.slots().get(self.filled.len()) else {
            return Ok(Effect::Nothing);
        };
        let doc = session.document();
        let candidate = match next {
            SlotKind::Position => Some(Filled::Position(at)),
            SlotKind::Point => nearest_point(doc, scene, at, tolerance).map(Filled::Point),
            SlotKind::Direction => nearest_line(doc, scene, at, tolerance).map(Filled::Geometry),
            SlotKind::Curve => nearest_curve(scene, at, tolerance).map(Filled::Geometry),
        };
        let Some(candidate) = candidate else {
            return Ok(Effect::Refused(format!("{}: needs {}", self.tool.name(), next.noun())));
        };
        if self.filled.iter().any(|f| f.same(&candidate, doc)) {
            return Ok(Effect::Refused(format!("{}: {} is already picked", self.tool.name(), candidate.label(doc))));
        }
        self.filled.push(candidate);
        if self.filled.len() == self.tool.slots().len() {
            self.issue(session)
        } else {
            Ok(Effect::Slot)
        }
    }

    /// Issue the complete tool's command; the tool stays with empty slots.
    fn issue(&mut self, session: &mut Session) -> Result<Effect, UiError> {
        let filled = std::mem::take(&mut self.filled);
        let Some((cmd, detail)) = self.tool.command(session.document(), &filled, &self.look) else {
            return Ok(Effect::Refused(format!("{}: an input no longer exists", self.tool.name())));
        };
        self.last_detail = detail;
        let outcome = session.apply(cmd).map_err(UiError::Command)?;
        self.selection.reconcile(session.document());
        Ok(Effect::Issued(outcome))
    }

    /// Constraints first, then geometries: every name still exists at its
    /// turn (a geometry's cascade only removes constraints). A point adds
    /// every curve with an endpoint in its congruence class.
    fn delete(&mut self, session: &mut Session) -> Result<Effect, GroupError> {
        let doc = session.document();
        let entries = self.selection.entries();
        let mut names: Vec<String> = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Constraint(id) => doc.constraint_name(*id).map(str::to_owned),
                _ => None,
            })
            .collect();
        let mut geometries: Vec<GeometryId> = Vec::new();
        for e in entries {
            match e {
                Entry::Geometry(id) => geometries.push(*id),
                Entry::Point(p) => geometries.extend(doc.point_class(p).iter().filter_map(|q| geometry_of(doc, q))),
                Entry::Constraint(_) => {}
            }
        }
        let mut seen: Vec<GeometryId> = Vec::new();
        for id in geometries {
            if !seen.contains(&id) {
                seen.push(id);
                names.extend(doc.geometry_name(id).map(str::to_owned));
            }
        }
        if names.is_empty() {
            return Ok(Effect::Nothing);
        }
        let cmds = names.into_iter().map(|name| Command::Delete { name }).collect();
        let outcomes = session.apply_group(cmds)?;
        self.selection.reconcile(session.document());
        Ok(Effect::Deleted(outcomes.into_iter().flat_map(|o| o.removed).collect()))
    }

    /// Drop entries whose object no longer exists (after any command, undo,
    /// redo or drag).
    pub fn reconcile(&mut self, doc: &Document) {
        self.selection.reconcile(doc);
        // A filled slot whose object is gone empties the slots after it too.
        if let Some(i) = self.filled.iter().position(|f| match f {
            Filled::Position(_) => false,
            Filled::Point(p) => !Entry::Point(*p).exists(doc),
            Filled::Geometry(id) => !doc.geometries().contains(*id),
        }) {
            self.filled.truncate(i);
        }
    }

    /// Empty the selection and end the tool (New, Open).
    pub fn clear(&mut self) {
        self.selection.clear();
        self.tool = ToolKind::Select;
        self.filled.clear();
    }

    /// After a presentation-mode change: drop geometries `filter` hides and
    /// points with no visible endpoint left in their class. Constraints stay.
    pub fn hide_invisible(&mut self, doc: &Document, filter: MeshFilter) {
        let visible_points = doc.logical_points(filter);
        self.selection.retain(|e| match e {
            Entry::Geometry(id) => doc
                .geometries()
                .get(*id)
                .is_some_and(|g| filter.include_construction || g.role() != constraint_runtime::GeometryRole::Construction),
            Entry::Constraint(_) => true,
            Entry::Point(p) => visible_points.iter().any(|lp| lp.members.contains(p)),
        });
    }
}

/// The geometry a point expression belongs to (the name part of its address).
fn geometry_of(doc: &Document, p: &PointExpr) -> Option<GeometryId> {
    let address = doc.point_address(p)?;
    let name = address.split('.').next()?;
    match doc.lookup(name)? {
        ObjectId::Geometry(id) => Some(id),
        ObjectId::Constraint(_) => None,
    }
}

#[cfg(test)]
#[path = "tests_model.rs"]
mod tests_model;
