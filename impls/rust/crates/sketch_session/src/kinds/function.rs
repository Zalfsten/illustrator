//! `create` / `set` kind `function` (sketch-commands: Function interchange
//! fields, Function set command).

use constraint_runtime::{Function, FunctionAttrs, FunctionAttrsError, HeadingAngle, KindAttrs, Point2};
use expression::DomainCheckError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{finite, finite_point, AttributeCommand, GeometryCommand};
use crate::command::CommandError;
use crate::wire;

/// Frame plus the graph of `y = f(x)`. `scale` defaults to `[1, 1]`; export
/// writes every attribute.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionParams {
    #[serde(with = "wire::point")]
    #[schemars(schema_with = "wire::point::schema")]
    pub origin: Point2,
    #[serde(with = "wire::finite")]
    /// Heading of the frame's x axis, radians from +x.
    #[schemars(with = "f64")]
    pub heading: f64,
    /// `f(x)` in the expression language (variable `x`).
    pub expression: String,
    /// `[x0, x1]` with `x0 < x1`.
    pub range: [f64; 2],
    /// `[su, sv]`, both greater than zero; absent for `[1, 1]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f64; 2]>,
}

/// Fields of `set` for a function curve: each optional, at least one present.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FunctionSet {
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "String")]
    pub expression: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "[f64; 2]")]
    pub range: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none", with = "wire::present")]
    #[schemars(with = "[f64; 2]")]
    pub scale: Option<[f64; 2]>,
}

/// Checked attributes, with errors in command terms.
fn build(source: &str, [x0, x1]: [f64; 2], [su, sv]: [f64; 2]) -> Result<FunctionAttrs, CommandError> {
    FunctionAttrs::new(source, (x0, x1), (su, sv)).map_err(|e| match e {
        FunctionAttrsError::Range => CommandError::InvalidValue {
            field: "range",
            reason: "not finite with x0 < x1",
        },
        FunctionAttrsError::Scale => CommandError::InvalidValue {
            field: "scale",
            reason: "not greater than zero",
        },
        FunctionAttrsError::Parse(p) => CommandError::InvalidExpression {
            message: format!("{p} at bytes {}..{} of `{source}`", p.span.start, p.span.end),
        },
        FunctionAttrsError::Domain(DomainCheckError::TooComplex { range: (a, b) }) => {
            CommandError::InvalidExpression {
                message: format!("the domain check exceeded its budget in [{a}, {b}]; try to simplify it"),
            }
        }
        FunctionAttrsError::Domain(d) => CommandError::InvalidExpression { message: d.to_string() },
        FunctionAttrsError::TooComplex => CommandError::InvalidExpression {
            message: "the curve exceeded its budget; try to simplify it".to_owned(),
        },
    })
}

fn function_attrs(attrs: &KindAttrs) -> &FunctionAttrs {
    let KindAttrs::Function(a) = attrs else {
        unreachable!("a function curve carries function attributes");
    };
    a
}

impl GeometryCommand for Function {
    const TAG: &'static str = "function";
    type Params = FunctionParams;

    fn init(p: &FunctionParams) -> Result<Self::Init, CommandError> {
        Ok((
            finite_point("origin", p.origin)?,
            HeadingAngle::new(finite("heading", p.heading)?),
        ))
    }

    fn attrs(p: &FunctionParams) -> Result<KindAttrs, CommandError> {
        let scale = p.scale.unwrap_or([1.0, 1.0]);
        Ok(KindAttrs::Function(build(&p.expression, p.range, scale)?))
    }

    fn params((origin, heading): Self::Init, attrs: &KindAttrs) -> FunctionParams {
        let a = function_attrs(attrs);
        let ((x0, x1), (su, sv)) = (a.range(), a.scale());
        FunctionParams {
            origin,
            heading: heading.radians(),
            expression: a.source().to_owned(),
            range: [x0, x1],
            scale: Some([su, sv]),
        }
    }
}

impl AttributeCommand for Function {
    type SetParams = FunctionSet;
    const CHANGES_GEOMETRY: bool = true;

    fn is_empty(p: &FunctionSet) -> bool {
        *p == FunctionSet::default()
    }

    fn apply(p: &FunctionSet, attrs: &mut KindAttrs) -> Result<(), CommandError> {
        let a = function_attrs(attrs);
        let ((x0, x1), (su, sv)) = (a.range(), a.scale());
        let source = p.expression.clone().unwrap_or_else(|| a.source().to_owned());
        let next = build(&source, p.range.unwrap_or([x0, x1]), p.scale.unwrap_or([su, sv]))?;
        *attrs = KindAttrs::Function(next);
        Ok(())
    }
}
