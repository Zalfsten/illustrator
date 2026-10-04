//! Concrete structural constraint kinds. One file per kind.

pub mod congruence;
pub mod equal_length;
pub mod fix_point;
pub mod orthogonal;
pub mod parallel;

pub use congruence::Congruence;
pub use equal_length::EqualLength;
pub use fix_point::FixPoint;
pub use orthogonal::Orthogonal;
pub use parallel::Parallel;
