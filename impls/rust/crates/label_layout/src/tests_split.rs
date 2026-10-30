use constraint_runtime::Markup;

use super::*;

fn latex(s: &str) -> Vec<&str> {
    split_lines(s, Markup::Latex)
}

#[test]
fn break_inside_a_matrix_stays() {
    let src = r"a\\$\begin{pmatrix}1\\2\end{pmatrix}$";
    assert_eq!(latex(src), [r"a", r"$\begin{pmatrix}1\\2\end{pmatrix}$"]);
}

#[test]
fn newline_in_text_markup() {
    assert_eq!(split_lines("x\ny", Markup::Text), ["x", "y"]);
    // A backslash pair is not a break in text markup.
    assert_eq!(split_lines(r"x\\y", Markup::Text), [r"x\\y"]);
}

#[test]
fn newline_in_latex_markup_is_not_a_break() {
    assert_eq!(latex("a\nb"), ["a\nb"]);
}

#[test]
fn groups_math_and_environments_keep_their_breaks() {
    assert_eq!(latex(r"{a\\b}"), [r"{a\\b}"]);
    assert_eq!(latex(r"$a\\b$"), [r"$a\\b$"]);
    assert_eq!(
        latex(r"\begin{tabular}{c}a\\b\end{tabular}\\c"),
        [r"\begin{tabular}{c}a\\b\end{tabular}", "c"]
    );
}

#[test]
fn escaped_dollar_does_not_open_math() {
    assert_eq!(latex(r"\$\\x"), [r"\$", "x"]);
    assert_eq!(latex(r"\{\\x"), [r"\{", "x"]);
}

#[test]
fn optional_length_belongs_to_the_break() {
    assert_eq!(latex(r"a\\[2pt]b"), ["a", "b"]);
}

#[test]
fn empty_source_and_empty_lines() {
    assert!(latex("").is_empty());
    assert!(split_lines("", Markup::Text).is_empty());
    assert_eq!(latex(r"a\\\\b"), ["a", "", "b"]);
    assert_eq!(latex(r"a\\"), ["a", ""]);
}
