//! Appearance of geometries (styling spec): named styles, per-geometry
//! overrides and their resolution. Never on the solve path.

use std::collections::BTreeMap;

use crate::geometry::GeometryId;
use crate::names::is_valid_name;

/// Name of the style that always exists.
pub const DEFAULT_STYLE: &str = "default";

/// An sRGB colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u8, pub u8, pub u8);

/// Dash pattern of a stroke.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Dash {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
}

/// Which ends of a curve carry an arrow head.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Arrow {
    #[default]
    None,
    Start,
    End,
    Both,
}

impl Arrow {
    pub fn at_start(self) -> bool {
        matches!(self, Arrow::Start | Arrow::Both)
    }

    pub fn at_end(self) -> bool {
        matches!(self, Arrow::End | Arrow::Both)
    }
}

/// A value for any subset of the style properties (a named style, or a
/// geometry's overrides). `width` is in pt.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StyleProps {
    pub width: Option<f64>,
    pub color: Option<Rgb>,
    pub dash: Option<Dash>,
    pub arrow: Option<Arrow>,
}

impl StyleProps {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    /// `self`, with the properties it does not set taken from `fallback`.
    pub fn or(&self, fallback: &StyleProps) -> StyleProps {
        StyleProps {
            width: self.width.or(fallback.width),
            color: self.color.or(fallback.color),
            dash: self.dash.or(fallback.dash),
            arrow: self.arrow.or(fallback.arrow),
        }
    }

    /// Every set width is finite and greater than zero.
    pub fn is_valid(&self) -> bool {
        self.width.is_none_or(valid_width)
    }
}

/// Change of a [`StyleProps`]: per property absent (leave), `Some(Some)`
/// (set) or `Some(None)` (remove).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StylePatch {
    pub width: Option<Option<f64>>,
    pub color: Option<Option<Rgb>>,
    pub dash: Option<Option<Dash>>,
    pub arrow: Option<Option<Arrow>>,
}

impl StylePatch {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    /// `props` with this patch applied.
    pub fn applied_to(&self, props: &StyleProps) -> StyleProps {
        StyleProps {
            width: self.width.unwrap_or(props.width),
            color: self.color.unwrap_or(props.color),
            dash: self.dash.unwrap_or(props.dash),
            arrow: self.arrow.unwrap_or(props.arrow),
        }
    }

    /// Every width it sets is finite and greater than zero.
    pub fn is_valid(&self) -> bool {
        self.width.flatten().is_none_or(valid_width)
    }
}

fn valid_width(w: f64) -> bool {
    w.is_finite() && w > 0.0
}

/// Every property resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub width: f64,
    pub color: Rgb,
    pub dash: Dash,
    pub arrow: Arrow,
}

impl Style {
    /// The look of a property set nowhere: 0.4 pt, black, solid, no arrow.
    pub const BUILT_IN: Style = Style {
        width: 0.4,
        color: Rgb(0, 0, 0),
        dash: Dash::Solid,
        arrow: Arrow::None,
    };

    fn from_props(p: &StyleProps) -> Style {
        let b = Self::BUILT_IN;
        Style {
            width: p.width.unwrap_or(b.width),
            color: p.color.unwrap_or(b.color),
            dash: p.dash.unwrap_or(b.dash),
            arrow: p.arrow.unwrap_or(b.arrow),
        }
    }
}

/// A geometry's style: the named style it refers to and its overrides.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleRef {
    pub style: String,
    pub overrides: StyleProps,
}

impl Default for StyleRef {
    fn default() -> Self {
        Self {
            style: DEFAULT_STYLE.to_owned(),
            overrides: StyleProps::default(),
        }
    }
}

/// Why a style edit was rejected; nothing changed.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleError {
    /// Not a valid style name (object-name syntax).
    InvalidName,
    /// No style of that name.
    Unknown(String),
    /// `default`, or a style these geometries refer to.
    InUse(Vec<GeometryId>),
    /// A width that is not finite or not greater than zero.
    InvalidWidth,
    /// The geometry is not in the document.
    UnknownGeometry,
}

/// The style table of a drawing; `default` is always present.
#[derive(Clone, Debug, PartialEq)]
pub struct Styles(BTreeMap<String, StyleProps>);

impl Default for Styles {
    fn default() -> Self {
        Self(BTreeMap::from([(DEFAULT_STYLE.to_owned(), StyleProps::default())]))
    }
}

impl Styles {
    pub fn get(&self, name: &str) -> Option<&StyleProps> {
        self.0.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    /// All styles in name order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &StyleProps)> {
        self.0.iter().map(|(n, p)| (n.as_str(), p))
    }

    /// Resolve `r`: override, then its named style, then `default`, then
    /// [`Style::BUILT_IN`]. An unknown named style counts as empty.
    pub fn resolve(&self, r: &StyleRef) -> Style {
        let empty = StyleProps::default();
        let named = self.get(&r.style).unwrap_or(&empty);
        let default = self.get(DEFAULT_STYLE).unwrap_or(&empty);
        Style::from_props(&r.overrides.or(named).or(default))
    }

    /// Create style `name` from `patch`, or apply `patch` to it.
    pub fn define(&mut self, name: &str, patch: &StylePatch) -> Result<(), StyleError> {
        if !is_valid_name(name) {
            return Err(StyleError::InvalidName);
        }
        if !patch.is_valid() {
            return Err(StyleError::InvalidWidth);
        }
        let props = self.0.entry(name.to_owned()).or_default();
        *props = patch.applied_to(props);
        Ok(())
    }

    /// Insert or replace style `name` as a whole.
    pub fn insert(&mut self, name: &str, props: StyleProps) -> Result<(), StyleError> {
        if !is_valid_name(name) {
            return Err(StyleError::InvalidName);
        }
        if !props.is_valid() {
            return Err(StyleError::InvalidWidth);
        }
        self.0.insert(name.to_owned(), props);
        Ok(())
    }

    pub(crate) fn remove(&mut self, name: &str) -> Option<StyleProps> {
        self.0.remove(name)
    }
}

#[cfg(test)]
#[path = "tests_style.rs"]
mod tests_style;
