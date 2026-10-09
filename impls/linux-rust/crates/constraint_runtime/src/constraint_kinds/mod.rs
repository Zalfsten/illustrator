//! Concrete structural constraint kinds and the closed document catalog.
//!
//! Add a kind: new `mod` + file, `pub use`, then one name in [`catalog`].

pub mod catalog;
pub mod congruence;
pub mod equal_length;
pub mod fix_point;
pub mod orthogonal;
pub mod parallel;

pub use catalog::ConstraintInst;
pub use congruence::Congruence;
pub use equal_length::EqualLength;
pub use fix_point::FixPoint;
pub use orthogonal::Orthogonal;
pub use parallel::Parallel;
