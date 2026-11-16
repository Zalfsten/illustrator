//! Tools as slot forms (ui-model: tools declare slots by capability). Each
//! tool kind has a fixed list of slots and builds its command from the
//! filled slots.

use constraint_runtime::{Document, GeometryId, Point2, PointExpr, StylePatch, StyleRef};
use sketch_session::{
    Command, ConstraintSpec, FixParams, GeometrySpec, PointPair, SegmentParams, VecPair,
};

use crate::selection::Entry;

/// The active tool of a front end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolKind {
    /// Selecting: no slots (the default tool).
    #[default]
    Select,
    Segment,
    Join,
    Parallel,
    Orthogonal,
    Fix,
    /// Give a curve the current look.
    Brush,
}

/// What a slot asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    /// A free world position.
    Position,
    /// A logical point.
    Point,
    /// A geometry with a direction (a line).
    Direction,
    /// A geometry with a curve.
    Curve,
}

/// A filled slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Filled {
    Position(Point2),
    Point(PointExpr),
    Geometry(GeometryId),
}

impl SlotKind {
    /// "a direction" etc., for prompts and refusals.
    pub fn noun(self) -> &'static str {
        match self {
            SlotKind::Position => "a position",
            SlotKind::Point => "a point",
            SlotKind::Direction => "a direction",
            SlotKind::Curve => "a curve",
        }
    }

    /// Whether `entry` has this slot's capability.
    pub fn fits(self, entry: &Entry, doc: &Document) -> bool {
        match (self, entry) {
            (SlotKind::Point, Entry::Point(_)) => true,
            (SlotKind::Direction, Entry::Geometry(id)) => doc.caps(*id).is_some_and(|c| c.line),
            (SlotKind::Curve, Entry::Geometry(id)) => doc.caps(*id).is_some_and(|c| c.curve),
            _ => false,
        }
    }
}

impl Filled {
    /// The filled slot for a fitting selection entry.
    pub fn from_entry(entry: &Entry) -> Option<Filled> {
        match entry {
            Entry::Point(p) => Some(Filled::Point(*p)),
            Entry::Geometry(id) => Some(Filled::Geometry(*id)),
            Entry::Constraint(_) => None,
        }
    }

    /// Name, point address or `(x, y)`.
    pub fn label(&self, doc: &Document) -> String {
        match self {
            Filled::Position(p) => format!("({:.3}, {:.3})", p.x, p.y),
            Filled::Point(p) => Entry::Point(*p).label(doc),
            Filled::Geometry(id) => Entry::Geometry(*id).label(doc),
        }
    }

    /// Same object (points by congruence class); positions never repeat.
    pub fn same(&self, other: &Filled, doc: &Document) -> bool {
        match (self, other) {
            (Filled::Point(a), Filled::Point(b)) => Entry::Point(*a).same(&Entry::Point(*b), doc),
            (Filled::Geometry(a), Filled::Geometry(b)) => a == b,
            _ => false,
        }
    }
}

impl ToolKind {
    pub const ALL: [ToolKind; 7] = [
        ToolKind::Select,
        ToolKind::Segment,
        ToolKind::Join,
        ToolKind::Parallel,
        ToolKind::Orthogonal,
        ToolKind::Fix,
        ToolKind::Brush,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ToolKind::Select => "Select",
            ToolKind::Segment => "Segment",
            ToolKind::Join => "Join",
            ToolKind::Parallel => "Parallel",
            ToolKind::Orthogonal => "Orthogonal",
            ToolKind::Fix => "Fix",
            ToolKind::Brush => "Brush",
        }
    }

    pub fn slots(self) -> &'static [SlotKind] {
        use SlotKind::*;
        match self {
            ToolKind::Select => &[],
            ToolKind::Segment => &[Position, Position],
            ToolKind::Join => &[Point, Point],
            ToolKind::Parallel | ToolKind::Orthogonal => &[Direction, Direction],
            ToolKind::Fix => &[Point],
            ToolKind::Brush => &[Curve],
        }
    }

    /// The command of a complete set of `filled` slots (`look` for new
    /// geometry and Brush), with the detail shown after `Added <name>` (for
    /// Brush: the brushed geometry's name).
    /// `None` if a slot no longer resolves (its object is gone).
    pub fn command(self, doc: &Document, filled: &[Filled], look: &StyleRef) -> Option<(Command, String)> {
        let point = |i: usize| match filled.get(i)? {
            Filled::Point(p) => doc.point_address(p),
            _ => None,
        };
        let direction = |i: usize| match filled.get(i)? {
            Filled::Geometry(id) => doc.direction_expr(*id).and_then(|e| doc.vec_address(&e)),
            _ => None,
        };
        let name = |i: usize| match filled.get(i)? {
            Filled::Geometry(id) => doc.geometry_name(*id).map(str::to_owned),
            _ => None,
        };
        let constrain = |constraint| Command::Constrain { name: None, constraint };
        Some(match self {
            ToolKind::Select => return None,
            ToolKind::Segment => {
                let (Filled::Position(start), Filled::Position(end)) = (filled.first()?, filled.get(1)?) else {
                    return None;
                };
                let cmd = Command::Create {
                    name: None,
                    role: constraint_runtime::GeometryRole::Result,
                    look: look.clone(),
                    geometry: GeometrySpec::Segment(SegmentParams { start: *start, end: *end }),
                };
                (cmd, String::new())
            }
            ToolKind::Join => {
                let (a, b) = (point(0)?, point(1)?);
                let detail = format!(" ({a} = {b})");
                (constrain(ConstraintSpec::Congruence(PointPair { a: a.into(), b: b.into() })), detail)
            }
            ToolKind::Parallel | ToolKind::Orthogonal => {
                let detail = format!(" ({}, {})", name(0)?, name(1)?);
                let pair = VecPair { a: direction(0)?.into(), b: direction(1)?.into() };
                let spec = if self == ToolKind::Parallel {
                    ConstraintSpec::VectorsParallel(pair)
                } else {
                    ConstraintSpec::VectorsOrthogonal(pair)
                };
                (constrain(spec), detail)
            }
            ToolKind::Fix => {
                let Filled::Point(p) = filled.first()? else { return None };
                let address = point(0)?;
                let target = p.evaluate(doc, doc.vector());
                let detail = format!(" ({address})");
                (constrain(ConstraintSpec::FixPoint(FixParams { point: address.into(), target })), detail)
            }
            ToolKind::Brush => {
                let target = name(0)?;
                (brush_command(&target, look), target)
            }
        })
    }
}

/// `set_style` that makes geometry `name` look like a new geometry drawn
/// with `look`: its style, the overrides it sets, the others removed.
pub fn brush_command(name: &str, look: &StyleRef) -> Command {
    let o = &look.overrides;
    Command::SetStyle {
        name: name.to_owned(),
        style: Some(look.style.clone()),
        patch: StylePatch {
            width: Some(o.width),
            color: Some(o.color),
            dash: Some(o.dash),
            arrow: Some(o.arrow),
        },
    }
}

#[cfg(test)]
#[path = "tests_tool.rs"]
mod tests_tool;
