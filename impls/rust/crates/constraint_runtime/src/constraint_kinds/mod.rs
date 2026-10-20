//! Concrete structural constraint kinds and the closed document catalog.
//!
//! Add a kind: new `mod` + file, `pub use`, then one name in [`catalog`].

pub mod catalog;
pub mod congruence;
pub mod equal_length;
pub mod fix_point;
pub mod vectors_orthogonal;
pub mod vectors_parallel;

pub use catalog::ConstraintInst;
pub use congruence::Congruence;
pub use equal_length::EqualLength;
pub use fix_point::FixPoint;
pub use vectors_orthogonal::VectorsOrthogonal;
pub use vectors_parallel::VectorsParallel;
