//! Model E constraint runtime: global state vector, connectivity assembly,
//! analytical Jacobians, and a swappable solver port (owned sparse
//! Levenberg–Marquardt backend, pure Rust).

pub mod address;
pub mod assemble;
pub mod components;
pub mod constraint;
pub mod constraint_kinds;
pub mod document;
pub mod expr;
pub mod geom;
pub mod geometry;
pub mod geometry_kinds;
pub mod id;
pub mod interact;
pub mod matrix;
pub mod mesh;
pub mod names;
pub mod point_classes;
pub mod solver;
pub mod stroke;
pub mod style;
pub mod vector;

#[cfg(test)]
mod test_alloc;

pub use address::{Address, AddressError};
pub use assemble::{
    structural_residual_norm, AssembledSystem, SoftContribution, SoftLengthFloor, SoftPointFollow,
};
pub use components::Component;
pub use constraint::{ConstraintEntry, ConstraintId, ConstraintKind};
pub use constraint_kinds::{
    Congruence, ConstraintInst, EqualLength, FixPoint, VectorsOrthogonal, VectorsParallel,
};
pub use document::{Document, DocumentSettings, Removal};
pub use expr::{
    Expr, HeadingAngleExpr, LengthExpr, PointExpr, TurnAngleExpr, VecExpr, VecJet,
    MAX_FORMULA_LOCAL_DOF,
};
pub use geom::{CurveParam, HeadingAngle, Length, Point2, TurnAngle, Vec2};
pub use geometry::{
    AnchorExprs, Caps, Chart, ChartSlot, CircularExprs, CurveExprs, EndpointExprs, Geometry, GeometryCaps,
    GeometryId, GeometryRole, GeometryShell, LineExprs, ShapePiece, StateAtom,
};
pub use geometry_kinds::{
    Arc, GeometryInst, GeometryKind, HAlign, KindAttrs, Label, LabelAttrs, Markup, Segment, VAlign,
};
pub use id::IdMap;
pub use interact::{DragSession, InteractiveSolve, SoftTerms, SolveError};
pub use matrix::{Block, BlockSparseMatrix, DenseMatrix};
pub use mesh::{Mesh, MeshEdge, MeshFilter, MAX_CHORD_TURN};
pub use names::{is_valid_name, NameError, ObjectId};
pub use point_classes::LogicalPoint;
pub use solver::{LmSolver, SolveOutcome, SolverOptions, SolverPort, StubSolver, SystemEval};
pub use stroke::{
    arrow_head, arrow_length_pt, arrow_tips, dash, pt_to_world, stroke_pieces, trim, ArrowTip, MM_PER_PT,
};
pub use style::{
    Arrow, Dash, Rgb, Style, StyleError, StylePatch, StyleProps, StyleRef, Styles, DEFAULT_STYLE,
};
pub use vector::Vector;

pub const SOLVE_TOL: f64 = 1e-6;

/// Structural residual norm up to which a solve result may be committed and
/// a component counts as consistent (one threshold for both).
pub const COMMIT_TOL: f64 = 10.0 * SOLVE_TOL;
