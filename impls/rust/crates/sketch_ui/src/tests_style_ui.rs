use super::*;

#[test]
fn default_current_style_is_plain() {
    let c = CurrentStyle::default();
    assert_eq!(c.look(), StyleRef::default());
}

#[test]
fn switched_on_overrides_only() {
    let c = CurrentStyle {
        style: "axis".into(),
        color: Switch {
            on: true,
            value: Rgb(0xc0, 0, 0),
        },
        width: Switch { on: false, value: 2.0 },
        ..CurrentStyle::default()
    };
    let look = c.look();
    assert_eq!(look.style, "axis");
    assert_eq!(look.overrides.color, Some(Rgb(0xc0, 0, 0)));
    assert_eq!(look.overrides.width, None, "width is switched off");
}

#[test]
fn brush_sets_every_property() {
    let c = CurrentStyle {
        dash: Switch {
            on: true,
            value: Dash::Dotted,
        },
        ..CurrentStyle::default()
    };
    let Command::SetStyle { name, style, patch } = c.brush("seg1") else {
        panic!("not a set_style");
    };
    assert_eq!(
        (name.as_str(), style.as_deref()),
        ("seg1", Some(DEFAULT_STYLE))
    );
    assert_eq!(
        patch,
        StylePatch {
            width: Some(None),
            color: Some(None),
            dash: Some(Some(Dash::Dotted)),
            arrow: Some(None),
        }
    );
}

#[test]
fn deleted_current_style_falls_back_to_default() {
    let mut c = CurrentStyle {
        style: "vector".into(),
        ..CurrentStyle::default()
    };
    c.keep_valid(&Styles::default());
    assert_eq!(c.style, DEFAULT_STYLE);
}
