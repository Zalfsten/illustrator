//! The selection shared by every view (ui-model: one selection, a logical
//! point is one entry, selection follows the drawing).
//!
//! Entries hold object identity. Ids survive renames and undo / redo (whole
//! Document snapshots), so names shown are always current.

use constraint_runtime::{ConstraintId, Document, GeometryId, ObjectId, PointExpr};

/// One selected object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Entry {
    Geometry(GeometryId),
    Constraint(ConstraintId),
    /// A logical point: the congruence class of this expression.
    Point(PointExpr),
}

impl Entry {
    /// Same object; points compare by congruence class.
    pub fn same(&self, other: &Entry, doc: &Document) -> bool {
        match (self, other) {
            (Entry::Point(a), Entry::Point(b)) => a == b || doc.point_class(a).contains(b),
            _ => self == other,
        }
    }

    /// Whether the object still exists in `doc`.
    pub fn exists(&self, doc: &Document) -> bool {
        match self {
            Entry::Geometry(id) => doc.geometries().contains(*id),
            Entry::Constraint(id) => doc.constraints().contains(*id),
            // The address needs the point's geometry.
            Entry::Point(p) => doc.point_address(p).is_some(),
        }
    }

    /// Name, or for a point an address of its class (`seg1.end`).
    pub fn label(&self, doc: &Document) -> String {
        let name = match self {
            Entry::Geometry(id) => doc.geometry_name(*id).map(str::to_owned),
            Entry::Constraint(id) => doc.constraint_name(*id).map(str::to_owned),
            Entry::Point(p) => doc.point_address(p),
        };
        name.unwrap_or_else(|| "?".into())
    }
}

/// How a pick or name selection changes the selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickMode {
    /// The object becomes the only entry.
    Replace,
    /// The object is added, or removed if already selected.
    Toggle,
}

/// Selected objects in insertion order, without duplicates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    entries: Vec<Entry>,
}

impl Selection {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn contains(&self, e: &Entry, doc: &Document) -> bool {
        self.entries.iter().any(|x| x.same(e, doc))
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Add `e` unless already selected.
    pub fn add(&mut self, e: Entry, doc: &Document) {
        if !self.contains(&e, doc) {
            self.entries.push(e);
        }
    }

    /// Add `e`, or remove it if selected.
    pub fn toggle(&mut self, e: Entry, doc: &Document) {
        let before = self.entries.len();
        self.entries.retain(|x| !x.same(&e, doc));
        if self.entries.len() == before {
            self.entries.push(e);
        }
    }

    /// Apply `mode` with `e`.
    pub fn pick(&mut self, e: Entry, mode: PickMode, doc: &Document) {
        match mode {
            PickMode::Replace => {
                self.entries.clear();
                self.entries.push(e);
            }
            PickMode::Toggle => self.toggle(e, doc),
        }
    }

    /// Keep only entries whose object still exists (after any command, undo,
    /// redo or drag).
    pub fn reconcile(&mut self, doc: &Document) {
        self.entries.retain(|e| e.exists(doc));
    }

    /// Keep only entries for which `keep` holds.
    pub fn retain(&mut self, keep: impl FnMut(&Entry) -> bool) {
        self.entries.retain(keep);
    }

    /// Select by name or point address, for views other than the Canvas.
    /// Ignores visibility. `false` if nothing has that name or address.
    pub fn select_name(&mut self, doc: &Document, name: &str, mode: PickMode) -> bool {
        let Some(e) = entry_named(doc, name) else {
            return false;
        };
        self.pick(e, mode, doc);
        true
    }

    /// `seg1, seg2.end` (empty for no selection).
    pub fn describe(&self, doc: &Document) -> String {
        self.entries.iter().map(|e| e.label(doc)).collect::<Vec<_>>().join(", ")
    }
}

/// The entry `name` denotes: an object name or a point address.
pub fn entry_named(doc: &Document, name: &str) -> Option<Entry> {
    match doc.lookup(name) {
        Some(ObjectId::Geometry(id)) => Some(Entry::Geometry(id)),
        Some(ObjectId::Constraint(id)) => Some(Entry::Constraint(id)),
        None => match doc.resolve_address(name) {
            Ok(constraint_runtime::Address::Point(p)) => Some(Entry::Point(p)),
            _ => None,
        },
    }
}

#[cfg(test)]
#[path = "tests_selection.rs"]
mod tests_selection;
