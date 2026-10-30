use super::*;

#[test]
fn removed_symbols_are_renamed_in_math_only() {
    assert_eq!(rename_in_math("$diff x$"), "$partial x$");
    assert_eq!(rename_in_math("$plus.circle.big$"), "$plus.o.big$");
    assert_eq!(rename_in_math("$sector$"), "$sector$");
    assert_eq!(rename_in_math("$#textmath[diff]$"), "$#textmath[diff]$");
    assert_eq!(rename_in_math("diff and sect"), "diff and sect");
    assert_eq!(rename_in_math("a \\$ diff"), "a \\$ diff");
}

#[test]
fn text_markup_is_one_escaped_string() {
    assert_eq!(text(r#"a\"b"#), r#"#"a\\\"b""#);
    assert_eq!(text("$x$"), "#\"$x$\"");
}

#[test]
fn unknown_commands_and_environments_are_unsupported() {
    assert_eq!(
        latex("", r"$\foo{x}$"),
        Err(TypesetError::Unsupported {
            construct: r"\foo".into()
        })
    );
    assert!(matches!(
        latex("", r"\begin{nosuchenv}x\end{nosuchenv}"),
        Err(TypesetError::Unsupported { construct }) if construct == r"\nosuchenv"
    ));
}
