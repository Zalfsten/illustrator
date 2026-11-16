//! Drawings for the unit tests, built through session commands.

use constraint_runtime::{Document, GeometryRole, Point2};
use sketch_session::{Command, ConstraintSpec, GeometrySpec, PointPair, SegmentParams, Session, VecPair};

use crate::selection::{entry_named, Entry};

pub fn segment_as(name: &str, role: GeometryRole, a: (f64, f64), b: (f64, f64)) -> Command {
    Command::Create {
        name: Some(name.into()),
        role,
        look: Default::default(),
        geometry: GeometrySpec::Segment(SegmentParams {
            start: Point2::new(a.0, a.1),
            end: Point2::new(b.0, b.1),
        }),
    }
}

pub fn segment(name: &str, a: (f64, f64), b: (f64, f64)) -> Command {
    segment_as(name, GeometryRole::Result, a, b)
}

pub fn join(name: &str, a: &str, b: &str) -> Command {
    Command::Constrain {
        name: Some(name.into()),
        constraint: ConstraintSpec::Congruence(PointPair { a: a.into(), b: b.into() }),
    }
}

pub fn parallel(name: &str, a: &str, b: &str) -> Command {
    Command::Constrain {
        name: Some(name.into()),
        constraint: ConstraintSpec::VectorsParallel(VecPair { a: a.into(), b: b.into() }),
    }
}

/// Session after applying `cmds` one by one.
pub fn session(cmds: Vec<Command>) -> Session {
    let mut s = Session::new(Document::new());
    for c in cmds {
        s.apply(c).unwrap();
    }
    s
}

/// `seg1` (0,0)→(1,0), `seg2` (1,0)→(2,1) joined by `j1` (seg1.end = seg2.start),
/// `seg3` (5,5)→(6,5) parallel to seg1 through `par1`.
pub fn sample() -> Session {
    session(vec![
        segment("seg1", (0.0, 0.0), (1.0, 0.0)),
        segment("seg2", (1.0, 0.0), (2.0, 1.0)),
        join("j1", "seg1.end", "seg2.start"),
        segment("seg3", (5.0, 5.0), (6.0, 5.0)),
        parallel("par1", "seg1.direction", "seg3.direction"),
    ])
}

/// The entry `name` denotes (panics if none).
pub fn entry(s: &Session, name: &str) -> Entry {
    entry_named(s.document(), name).unwrap_or_else(|| panic!("no entry {name}"))
}

/// A label at `origin`, named `name`.
pub fn label(name: &str, origin: (f64, f64)) -> Command {
    Command::from_json(&format!(
        r#"{{"op": "create", "kind": "label", "name": "{name}", "origin": [{}, {}], "heading": 0, "source": "x", "offset": [0, 0]}}"#,
        origin.0, origin.1
    ))
    .unwrap()
}

/// Mesh and logical points of `doc` under `filter` (fine chord tolerance).
pub struct Shown {
    pub mesh: constraint_runtime::Mesh,
    pub points: Vec<constraint_runtime::LogicalPoint>,
}

pub fn shown(doc: &Document, filter: constraint_runtime::MeshFilter) -> Shown {
    Shown {
        mesh: constraint_runtime::Mesh::from_document(doc, doc.vector(), filter, 1e-3),
        points: doc.logical_points(filter),
    }
}

impl Shown {
    pub fn scene(&self) -> crate::pick::Scene<'_> {
        crate::pick::Scene { mesh: &self.mesh, points: &self.points }
    }
}

/// A quarter arc around `center` with radius 1, named `name`.
pub fn arc(name: &str, center: (f64, f64)) -> Command {
    Command::Create {
        name: Some(name.into()),
        role: GeometryRole::Result,
        look: Default::default(),
        geometry: GeometrySpec::Arc(sketch_session::ArcParams {
            center: Point2::new(center.0, center.1),
            radius: 1.0,
            start_heading: 0.0,
            sweep: std::f64::consts::FRAC_PI_2,
        }),
    }
}

/// Filled slot of the object or point address `name`.
pub fn filled(s: &Session, name: &str) -> crate::tool::Filled {
    crate::tool::Filled::from_entry(&entry(s, name)).unwrap()
}
