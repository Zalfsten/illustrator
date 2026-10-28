//! User-visible object names, unique at any time.
//!
//! Geometries and constraints share one namespace. Automatic names are
//! `<prefix><n>` where `n` is one more than the highest number in use with that
//! prefix. The number is derived from the current names (no stored counter), so
//! a name that is no longer in use may be given out again.

use std::collections::{BTreeSet, HashMap};

use crate::constraint::ConstraintId;
use crate::geometry::GeometryId;
use crate::id::IdMap;

/// Document object addressed by a name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectId {
    Geometry(GeometryId),
    Constraint(ConstraintId),
}

/// Why a name cannot be assigned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameError {
    /// Not `[A-Za-z_][A-Za-z0-9_]*`.
    Invalid,
    /// Used by another live object.
    Taken,
    /// The object to rename is not in the document.
    UnknownObject,
}

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NameError::Invalid => "invalid name (expected [A-Za-z_][A-Za-z0-9_]*)",
            NameError::Taken => "name already in use",
            NameError::UnknownObject => "no such object",
        })
    }
}

impl std::error::Error for NameError {}

/// `[A-Za-z_][A-Za-z0-9_]*` (ASCII, case-sensitive).
pub fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// `seg12` → `("seg", 12)`. `None` without a canonical decimal suffix: no
/// digits, a leading zero (`seg01`), or beyond `u64`. Such names are not
/// counted by auto-naming and cannot collide with an automatic name.
fn split_number(name: &str) -> Option<(&str, u64)> {
    let stem_len = name.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    let (stem, digits) = name.split_at(stem_len);
    if digits.is_empty() || (digits.len() > 1 && digits.starts_with('0')) {
        return None;
    }
    digits.parse().ok().map(|n| (stem, n))
}

/// Name table: name ↔ object in O(1), plus a stem → numbers index for
/// [`Self::next_auto`]. Owned by [`crate::document::Document`], which keeps it
/// consistent with its stores.
#[derive(Clone, Debug, Default)]
pub struct Names {
    by_name: HashMap<String, ObjectId>,
    geometry: IdMap<GeometryId, String>,
    constraint: IdMap<ConstraintId, String>,
    /// Stem → numbers in use with that stem (canonical decimal suffixes only).
    numbers: HashMap<String, BTreeSet<u64>>,
}

impl Names {
    pub fn name(&self, id: ObjectId) -> Option<&str> {
        match id {
            ObjectId::Geometry(g) => self.geometry.get(g),
            ObjectId::Constraint(c) => self.constraint.get(c),
        }
        .map(String::as_str)
    }

    pub fn lookup(&self, name: &str) -> Option<ObjectId> {
        self.by_name.get(name).copied()
    }

    /// Next automatic name for `prefix`: highest number in use + 1, or 1.
    pub fn next_auto(&self, prefix: &str) -> String {
        let used = self.numbers.get(prefix);
        let n = match used.and_then(|s| s.last()) {
            None => 1,
            Some(&max) => max.checked_add(1).unwrap_or_else(|| {
                // `u64::MAX` is taken (only by a user name): reuse the lowest gap.
                (1..).find(|n| !used.unwrap().contains(n)).unwrap()
            }),
        };
        format!("{prefix}{n}")
    }

    /// Register `name` for a new object (`name` valid and free).
    pub(crate) fn insert(&mut self, id: ObjectId, name: String) {
        debug_assert!(is_valid_name(&name), "invalid name {name:?}");
        debug_assert!(self.lookup(&name).is_none(), "name {name:?} taken");
        self.index(&name);
        self.by_name.insert(name.clone(), id);
        match id {
            ObjectId::Geometry(g) => self.geometry.insert(g, name),
            ObjectId::Constraint(c) => self.constraint.insert(c, name),
        }
    }

    /// Drop the name of `id`, returning it.
    pub(crate) fn remove(&mut self, id: ObjectId) -> Option<String> {
        let name = match id {
            ObjectId::Geometry(g) => self.geometry.remove(g),
            ObjectId::Constraint(c) => self.constraint.remove(c),
        }?;
        self.by_name.remove(&name);
        self.unindex(&name);
        Some(name)
    }

    /// Rename `id` to `new` (valid, and free unless it is `id`'s own name).
    pub fn rename(&mut self, id: ObjectId, new: &str) -> Result<(), NameError> {
        let Some(old) = self.name(id) else {
            return Err(NameError::UnknownObject);
        };
        if old == new {
            return Ok(());
        }
        if !is_valid_name(new) {
            return Err(NameError::Invalid);
        }
        if self.lookup(new).is_some() {
            return Err(NameError::Taken);
        }
        self.remove(id);
        self.insert(id, new.to_owned());
        Ok(())
    }

    fn index(&mut self, name: &str) {
        if let Some((stem, n)) = split_number(name) {
            self.numbers.entry(stem.to_owned()).or_default().insert(n);
        }
    }

    fn unindex(&mut self, name: &str) {
        if let Some((stem, n)) = split_number(name) {
            if let Some(set) = self.numbers.get_mut(stem) {
                set.remove(&n);
                if set.is_empty() {
                    self.numbers.remove(stem);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests_names.rs"]
mod tests_names;
