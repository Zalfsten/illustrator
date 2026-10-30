//! The Typst typesetter against the label-typesetting spec scenarios.

use constraint_runtime::Markup;
use label_layout::{TypesetError, TypesetLine, Typesetter};
use label_typst::TypstTypesetter;

fn latex(line: &str, preamble: &str) -> Result<TypesetLine, TypesetError> {
    TypstTypesetter::new().typeset_line(line, Markup::Latex, 10.0, preamble)
}

fn ok(line: &str) -> TypesetLine {
    latex(line, "").unwrap_or_else(|e| panic!("{line}: {e}"))
}

#[test]
fn text_with_math_and_fractions_typeset() {
    for line in [
        r"velocity $\vec v$",
        r"$\frac{a}{b}$",
        r"plain words",
        r"$\int_0^1 f(x)\,\mathrm dx$",
    ] {
        let l = ok(line);
        assert!(l.width > 0.0 && l.height > 0.0, "{line}: {l:?}");
        assert!(!l.shapes.is_empty(), "{line}");
    }
}

#[test]
fn macro_from_the_preamble() {
    let with_macro = latex(r"$x \in \R$", r"\newcommand{\R}{\mathbb{R}}").unwrap();
    let direct = ok(r"$x \in \mathbb{R}$");
    assert_eq!(with_macro, direct);
}

fn text(line: &str) -> TypesetLine {
    TypstTypesetter::new()
        .typeset_line(line, Markup::Text, 10.0, "")
        .unwrap_or_else(|e| panic!("{line}: {e}"))
}

/// Advance width of `c` at 10 pt in New Computer Modern 10 Regular.
fn advance(c: char) -> f64 {
    let face =
        ttf_parser::Face::parse(include_bytes!("../assets/fonts/NewCM10-Regular.otf"), 0).unwrap();
    let g = face.glyph_index(c).unwrap();
    f64::from(face.glyph_hor_advance(g).unwrap()) * 10.0 / f64::from(face.units_per_em())
}

#[test]
fn single_line_metrics() {
    let l = text("Ag");
    assert!(l.width > 0.0);
    assert!(l.height > 0.0 && l.depth > 0.0, "{l:?}");
    // Cap height of New Computer Modern is about 0.68 em, descender of g about 0.2 em.
    assert!((6.0..7.5).contains(&l.height), "{l:?}");
    assert!((1.5..2.5).contains(&l.depth), "{l:?}");
}

#[test]
fn width_is_the_advance_as_set() {
    let x = advance('x');
    assert!(
        (text("x").width - x).abs() < 1e-6,
        "{} vs {x}",
        text("x").width
    );
    // The same in latex markup: the line separator after the preamble adds no space.
    assert!((ok("x").width - x).abs() < 1e-6, "{} vs {x}", ok("x").width);
}

#[test]
fn text_markup_is_literal() {
    let literal = text(r"$\alpha$");
    let expected: f64 = r"$\alpha$".chars().map(advance).sum();
    assert!(
        (literal.width - expected).abs() < 0.5,
        "{} vs {expected}",
        literal.width
    );
    let math = ok(r"$\alpha$");
    assert!(
        math.width < literal.width / 2.0,
        "math {} literal {}",
        math.width,
        literal.width
    );
    // Characters that are special in Typst markup stay themselves.
    let special = text(r#"50% #a "q" *b* _c_"#);
    assert!(special.width > 0.0);
}

#[test]
fn empty_line_has_no_extent() {
    let l = ok("");
    assert_eq!((l.width, l.height, l.depth), (0.0, 0.0, 0.0));
    assert!(l.shapes.iter().all(|s| s.contours.is_empty()));
}

#[test]
fn unknown_command_is_named() {
    assert_eq!(
        latex(r"$\foo{x}$", ""),
        Err(TypesetError::Unsupported {
            construct: r"\foo".into()
        })
    );
    assert_eq!(
        latex(r"\begin{nosuchenv}x\end{nosuchenv}", ""),
        Err(TypesetError::Unsupported {
            construct: r"\nosuchenv".into()
        })
    );
}

#[test]
fn unclosed_math_is_invalid() {
    assert!(
        matches!(latex("$x", ""), Err(TypesetError::Invalid { .. })),
        "{:?}",
        latex("$x", "")
    );
}

#[test]
fn a_failure_does_not_affect_the_next_line() {
    let before = ok(r"$\vec v$");
    let _ = latex(r"$\foo$", "");
    let _ = latex("$x", "");
    assert_eq!(ok(r"$\vec v$"), before);
}

/// label-typesetting "Tall line pushes apart", through splitting and stacking.
#[test]
fn tall_line_pushes_apart_end_to_end() {
    let source = r"$\displaystyle\sum_{i=1}^{n} i$\\x";
    let lines: Vec<label_layout::LineMetrics> = label_layout::split_lines(source, Markup::Latex)
        .into_iter()
        .map(|l| (&ok(l)).into())
        .collect();
    assert_eq!(lines.len(), 2);
    let b = label_layout::place::baselines(&lines, 10.0);
    assert!(b[0] - b[1] > 12.0, "{b:?}");
    let gap = (b[0] - lines[0].depth) - (b[1] + lines[1].height);
    assert!((gap - 1.0).abs() < 0.01, "gap {gap}");
}

/// label-typesetting "Plain lines" with real metrics.
#[test]
fn plain_lines_end_to_end() {
    let lines: Vec<label_layout::LineMetrics> =
        ["a", "b"].iter().map(|l| (&text(l)).into()).collect();
    assert_eq!(label_layout::place::baselines(&lines, 10.0), [0.0, -12.0]);
}

/// label-typesetting "Break inside a matrix stays": the second line holds both rows.
#[test]
fn matrix_rows_stay_on_their_line() {
    let source = r"a\\$\begin{pmatrix}1\\2\end{pmatrix}$";
    let parts = label_layout::split_lines(source, Markup::Latex);
    assert_eq!(parts.len(), 2);
    let matrix = ok(parts[1]);
    let one_row = ok(r"$\begin{pmatrix}1\end{pmatrix}$");
    assert!(
        matrix.height + matrix.depth > 1.5 * (one_row.height + one_row.depth),
        "{matrix:?}"
    );
}
