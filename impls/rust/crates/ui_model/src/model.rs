//! The UI model's state and its intents (ui-model: front ends drive the UI
//! model through intents; pick, box, delete the selection).

use std::collections::HashSet;

use constraint_runtime::{Document, GeometryId, MeshFilter, Point2, PointExpr};
use sketch_session::{Command, GroupError, Session};

use crate::pick::{box_select, pick, Scene};
use crate::selection::{Entry, PickMode, Selection};

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
    /// Delete the selected geometries and constraints as one history step; a
    /// selected point deletes the curves shown on the Canvas with an endpoint
    /// in its class.
    Delete,
}

/// What an intent did.
#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    /// The selection may have changed; the drawing did not.
    Selection,
    /// One history step deleted these objects (cascade included).
    Deleted(Vec<String>),
    /// Nothing changed.
    Nothing,
}

/// Interaction state shared by every view of a front end.
#[derive(Clone, Debug, Default)]
pub struct UiModel {
    selection: Selection,
}

impl UiModel {
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// For views that select by name ([`Selection::select_name`]).
    pub fn selection_mut(&mut self) -> &mut Selection {
        &mut self.selection
    }

    /// Apply `intent`. `scene` is what the Canvas shows of `session`'s drawing.
    pub fn apply(&mut self, intent: Intent, session: &mut Session, scene: &Scene) -> Result<Effect, GroupError> {
        let doc = session.document();
        match intent {
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
            Intent::Delete => self.delete(session, scene),
        }
    }

    /// Constraints first, then geometries: every name still exists at its
    /// turn (a geometry's cascade only removes constraints). A point adds
    /// every curve with an endpoint in its congruence class that the Canvas
    /// shows: not curves through it (on-curve points), arc centers or label
    /// origins, and not curves hidden by the presentation mode (ui-model:
    /// Delete the selection; system-architecture: Hiding objects deselects
    /// them).
    fn delete(&mut self, session: &mut Session, scene: &Scene) -> Result<Effect, GroupError> {
        let doc = session.document();
        let shown: HashSet<GeometryId> = scene.mesh.edges.iter().map(|e| e.geometry).collect();
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
                Entry::Point(p) => geometries.extend(
                    doc.point_class(p)
                        .iter()
                        .filter(|q| is_endpoint(doc, q))
                        .map(PointExpr::referenced_geometry)
                        .filter(|g| shown.contains(g)),
                ),
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
    }

    /// Empty the selection (New, Open).
    pub fn clear(&mut self) {
        self.selection.clear();
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

/// Whether `p` is the start or end point of its geometry.
fn is_endpoint(doc: &Document, p: &PointExpr) -> bool {
    let g = p.referenced_geometry();
    doc.start_expr(g).as_ref() == Some(p) || doc.end_expr(g).as_ref() == Some(p)
}

#[cfg(test)]
#[path = "tests_model.rs"]
mod tests_model;
