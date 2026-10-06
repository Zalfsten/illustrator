//! Typed document id allocation (geometry, constraint, …).

use std::marker::PhantomData;

/// Allocates non-colliding typed ids for Document registration.
///
/// Owned by Document — separate instances for each id namespace
/// (e.g. [`crate::GeometryId`], [`crate::ConstraintId`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdGenerator<Id> {
    next: usize,
    _marker: PhantomData<Id>,
}

impl<Id> Default for IdGenerator<Id> {
    fn default() -> Self {
        Self {
            next: 0,
            _marker: PhantomData,
        }
    }
}

impl<Id> IdGenerator<Id> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Id: From<usize> + Into<usize> + Copy> IdGenerator<Id> {
    /// Next fresh id; advances the counter.
    pub fn allocate(&mut self) -> Id {
        let id = Id::from(self.next);
        self.next += 1;
        id
    }

    /// Record an explicit id (import/restore) so later [`Self::allocate`] values do not collide.
    pub fn observe(&mut self, id: Id) {
        let raw = id.into();
        if raw >= self.next {
            self.next = raw + 1;
        }
    }
}
