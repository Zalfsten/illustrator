//! Typed id → value storage for Document registration (geometry, constraint, …).

use std::marker::PhantomData;

/// Dense slot storage keyed by a typed integer id.
///
/// Ids are slot indices: [`Self::next_id`] is one past the highest slot ever
/// used, so removed ids are never reused. Lookup is an array index (no
/// hashing) and iteration follows id order, which keeps assembly row / column
/// order deterministic across runs.
#[derive(Clone, Debug)]
pub struct IdMap<Id, T> {
    slots: Vec<Option<T>>,
    len: usize,
    _id: PhantomData<fn() -> Id>,
}

impl<Id, T> Default for IdMap<Id, T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            len: 0,
            _id: PhantomData,
        }
    }
}

impl<Id: From<usize> + Into<usize> + Copy, T> IdMap<Id, T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fresh id for the next [`Self::insert`] (does not reserve it).
    pub fn next_id(&self) -> Id {
        Id::from(self.slots.len())
    }

    /// Store `value` under `id` (explicit ids from import/restore grow the slot range).
    pub fn insert(&mut self, id: Id, value: T) {
        let raw = id.into();
        if raw >= self.slots.len() {
            self.slots.resize_with(raw + 1, || None);
        }
        let slot = &mut self.slots[raw];
        debug_assert!(slot.is_none(), "duplicate id {raw}");
        if slot.is_none() {
            self.len += 1;
        }
        *slot = Some(value);
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        self.slots.get(id.into())?.as_ref()
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut T> {
        self.slots.get_mut(id.into())?.as_mut()
    }

    pub fn remove(&mut self, id: Id) -> Option<T> {
        let removed = self.slots.get_mut(id.into())?.take();
        if removed.is_some() {
            self.len -= 1;
        }
        removed
    }

    pub fn contains(&self, id: Id) -> bool {
        self.get(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Values in id order.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.slots.iter().flatten()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.slots.iter_mut().flatten()
    }

    /// `(id, value)` pairs in id order.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (Id::from(i), v)))
    }

    /// Keep only values for which `keep` returns true (slots stay allocated).
    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        for slot in &mut self.slots {
            if slot.as_ref().is_some_and(|v| !keep(v)) {
                *slot = None;
                self.len -= 1;
            }
        }
    }
}
