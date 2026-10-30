//! Label: a baseline frame (origin + heading) that carries text attributes.
//!
//! Not a curve: no on-curve positions, tangents, length or Mesh edges. The
//! text's extent comes from typesetting and is not a parameter.

use std::ops::Range;

use crate::geom::{HeadingAngle, Point2, Vec2};
use crate::geometry::capabilities::geometry_caps;
use crate::geometry::chart::chart_atoms;
use crate::geometry::{Geometry, PointFormula, VecFormula};
use crate::geometry_kinds::{GeometryKind, KindAttrs};
use crate::matrix::DenseMatrix;

/// Label kind (type-level marker; instances live in
/// [`crate::geometry_kinds::GeometryInst`]).
///
/// Layout via `chart_atoms!`: [`Self::origin`] (Point2), [`Self::heading`]
/// (heading of the baseline). The direction is the unit vector of the heading,
/// so its Jacobian touches only the heading column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Label;

chart_atoms! {
    Label {
        origin: Point2,
        heading: HeadingAngle,
    }
}

// Capability factory impls + runtime Caps.
geometry_caps!(Label: anchor, line);

impl Geometry for Label {
    const KIND: GeometryKind = GeometryKind::Label;
    const NAME_PREFIX: &'static str = "lbl";

    fn default_attrs() -> KindAttrs {
        KindAttrs::Label(LabelAttrs::default())
    }

    fn point(f: PointFormula, (origin, _heading): Self::Init) -> Point2 {
        match f {
            PointFormula::Origin => origin,
            _ => unreachable!("label has only an origin point"),
        }
    }

    fn point_jacobian(f: PointFormula, _c: Self::Init, j: &mut DenseMatrix<'_>, scale: f64) {
        match f {
            PointFormula::Origin => j.set_scaled(scale, [[1.0, 0.0], [0.0, 1.0]]),
            _ => unreachable!("label has only an origin point"),
        }
    }

    fn point_params(f: PointFormula) -> Range<usize> {
        match f {
            PointFormula::Origin => Self::origin.range(),
            _ => unreachable!("label has only an origin point"),
        }
    }

    fn vec(f: VecFormula, (_origin, heading): Self::Init) -> Vec2 {
        match f {
            VecFormula::Direction => {
                let th = heading.radians();
                Vec2::new(th.cos(), th.sin())
            }
            _ => unreachable!("label has only a direction vector"),
        }
    }

    fn vec_jacobian(
        f: VecFormula,
        (_origin, heading): Self::Init,
        j: &mut DenseMatrix<'_>,
        scale: f64,
    ) {
        match f {
            // ∂(cos θ, sin θ)/∂θ = (-sin θ, cos θ)
            VecFormula::Direction => {
                let th = heading.radians();
                j.set_scaled(scale, [[-th.sin()], [th.cos()]]);
            }
            _ => unreachable!("label has only a direction vector"),
        }
    }

    fn vec_params(f: VecFormula) -> Range<usize> {
        match f {
            VecFormula::Direction => Self::heading.range(),
            _ => unreachable!("label has only a direction vector"),
        }
    }
}

/// How a label's source is read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Markup {
    /// LaTeX in text mode: `$...$` math, `\\` breaks a line, a newline is a space.
    #[default]
    Latex,
    /// Every character stands for itself; a newline breaks a line.
    Text,
}

/// Horizontal placement of the text block (and of its lines) at the reference point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Whose baseline passes through the reference point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VAlign {
    /// First line; further lines extend below.
    #[default]
    First,
    /// Last line; earlier lines extend above.
    Last,
}

/// Text attributes of a label (not parameters; never solved).
#[derive(Clone, Debug, PartialEq)]
pub struct LabelAttrs {
    pub source: String,
    pub markup: Markup,
    pub halign: HAlign,
    pub valign: VAlign,
    /// Free vector in the label frame (x along the direction, y a quarter
    /// turn counterclockwise), in pt — not in world units.
    pub offset: Vec2,
    /// Size in pt; `None` inherits the document font size.
    pub size: Option<f64>,
}

/// Empty source, markup `latex`, `left` / `first`, no offset, inherited size.
impl Default for LabelAttrs {
    fn default() -> Self {
        Self {
            source: String::new(),
            markup: Markup::default(),
            halign: HAlign::default(),
            valign: VAlign::default(),
            offset: Vec2::zero(),
            size: None,
        }
    }
}

#[cfg(test)]
#[path = "tests_label.rs"]
mod tests_label;
