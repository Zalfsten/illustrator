//! A label source split into lines at its explicit breaks (label-typesetting
//! "Lines are split at explicit breaks").
//!
//! `latex`: `\\` at the top level only, i.e. outside `{…}`, `$…$` and
//! `\begin…\end`; an optional `[length]` after `\\` belongs to the break.
//! `text`: every newline. A small scanner, not a LaTeX parser.

use constraint_runtime::Markup;

/// The lines of `source` (an empty source has none).
pub fn split_lines(source: &str, markup: Markup) -> Vec<&str> {
    if source.is_empty() {
        return Vec::new();
    }
    match markup {
        Markup::Text => source.split('\n').collect(),
        Markup::Latex => split_latex(source),
    }
}

fn split_latex(source: &str) -> Vec<&str> {
    let b = source.as_bytes();
    let (mut braces, mut envs, mut math) = (0usize, 0usize, false);
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => {
                let rest = &source[i + 1..];
                if rest.starts_with('\\') {
                    if braces == 0 && envs == 0 && !math {
                        lines.push(&source[start..i]);
                        i += 2;
                        i += optional_length(&source[i..]);
                        start = i;
                        continue;
                    }
                    i += 2;
                    continue;
                }
                if rest.starts_with("begin{") {
                    envs += 1;
                } else if rest.starts_with("end{") {
                    envs = envs.saturating_sub(1);
                }
                // Skip the escaped character (`\$`, `\{`, `\}`) or the
                // command's first letter.
                i += 1 + rest.chars().next().map_or(0, char::len_utf8);
                continue;
            }
            b'{' => braces += 1,
            b'}' => braces = braces.saturating_sub(1),
            b'$' => math = !math,
            _ => {}
        }
        i += 1;
    }
    lines.push(&source[start..]);
    lines
}

/// Length of a `[…]` directly after `\\` (whitespace-free), or 0.
fn optional_length(rest: &str) -> usize {
    if !rest.starts_with('[') {
        return 0;
    }
    rest.find(']').map_or(0, |end| end + 1)
}

#[cfg(test)]
#[path = "tests_split.rs"]
mod tests_split;
