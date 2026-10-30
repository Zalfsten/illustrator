//! `create` kind `label`.

use constraint_runtime::{HeadingAngle, KindAttrs, Label, LabelAttrs, Point2, Vec2};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{finite, finite_point, positive, AttributeCommand, GeometryCommand};
use crate::command::CommandError;
use crate::wire::{self, WireHAlign, WireMarkup, WireVAlign};

/// Baseline frame plus text attributes. Only `origin`, `heading` and `source`
/// are required; export writes every attribute, except an inherited `size`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelParams {
    #[serde(with = "wire::point")]
    #[schemars(schema_with = "wire::point::schema")]
    pub origin: Point2,
    #[serde(with = "wire::finite")]
    /// Heading of the baseline, radians from +x.
    #[schemars(with = "f64")]
    pub heading: f64,
    /// Text, read according to `markup`.
    pub source: String,
    #[serde(default)]
    pub markup: WireMarkup,
    #[serde(default)]
    pub halign: WireHAlign,
    #[serde(default)]
    pub valign: WireVAlign,
    #[serde(default = "zero_offset", with = "wire::vector")]
    /// Offset of the reference point from the origin, in pt, in the label frame.
    #[schemars(schema_with = "wire::vector::schema")]
    pub offset: Vec2,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_option"
    )]
    /// Size in pt, greater than zero; absent to inherit the document font size.
    #[schemars(schema_with = "wire::positive_schema")]
    pub size: Option<f64>,
}

fn zero_offset() -> Vec2 {
    Vec2::zero()
}

fn positive_size(size: f64) -> Result<f64, CommandError> {
    positive("size", size)
}

impl GeometryCommand for Label {
    const TAG: &'static str = "label";
    type Params = LabelParams;

    fn init(p: &LabelParams) -> Result<Self::Init, CommandError> {
        Ok((
            finite_point("origin", p.origin)?,
            HeadingAngle::new(finite("heading", p.heading)?),
        ))
    }

    fn attrs(p: &LabelParams) -> Result<KindAttrs, CommandError> {
        finite("offset", p.offset.x)?;
        finite("offset", p.offset.y)?;
        Ok(KindAttrs::Label(LabelAttrs {
            source: p.source.clone(),
            markup: p.markup.into(),
            halign: p.halign.into(),
            valign: p.valign.into(),
            offset: p.offset,
            size: p.size.map(positive_size).transpose()?,
        }))
    }

    fn params((origin, heading): Self::Init, attrs: &KindAttrs) -> LabelParams {
        let KindAttrs::Label(a) = attrs else {
            unreachable!("a label carries label attributes");
        };
        LabelParams {
            origin,
            heading: heading.radians(),
            source: a.source.clone(),
            markup: a.markup.into(),
            halign: a.halign.into(),
            valign: a.valign.into(),
            offset: a.offset,
            size: a.size,
        }
    }
}

/// Fields of `set` for a label: each attribute optional, at least one present.
/// `size: null` restores the inherited size.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LabelSet {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::present"
    )]
    #[schemars(with = "String")]
    pub source: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::present"
    )]
    #[schemars(with = "WireMarkup")]
    pub markup: Option<WireMarkup>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::present"
    )]
    #[schemars(with = "WireHAlign")]
    pub halign: Option<WireHAlign>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::present"
    )]
    #[schemars(with = "WireVAlign")]
    pub valign: Option<WireVAlign>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::vector_option"
    )]
    #[schemars(schema_with = "wire::vector::schema")]
    pub offset: Option<Vec2>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "wire::finite_or_null"
    )]
    #[schemars(schema_with = "wire::finite_or_null::schema")]
    pub size: Option<Option<f64>>,
}

impl AttributeCommand for Label {
    type SetParams = LabelSet;

    fn is_empty(p: &LabelSet) -> bool {
        *p == LabelSet::default()
    }

    fn apply(p: &LabelSet, attrs: &mut KindAttrs) -> Result<(), CommandError> {
        let KindAttrs::Label(a) = attrs else {
            unreachable!("a label carries label attributes");
        };
        if let Some(source) = &p.source {
            a.source.clone_from(source);
        }
        if let Some(markup) = p.markup {
            a.markup = markup.into();
        }
        if let Some(halign) = p.halign {
            a.halign = halign.into();
        }
        if let Some(valign) = p.valign {
            a.valign = valign.into();
        }
        if let Some(offset) = p.offset {
            finite("offset", offset.x)?;
            finite("offset", offset.y)?;
            a.offset = offset;
        }
        if let Some(size) = p.size {
            a.size = size.map(positive_size).transpose()?;
        }
        Ok(())
    }
}
