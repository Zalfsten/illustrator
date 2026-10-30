//! A label line's source to Typst markup.

use label_layout::TypesetError;

/// `latex` markup: LaTeX text mode with `$...$` math, after `preamble`
/// (whose `\newcommand`s mitex expands).
pub fn latex(preamble: &str, line: &str) -> Result<String, TypesetError> {
    let source = format!("{preamble}\n{line}");
    let typst = mitex::convert_text(&source, None).map_err(classify)?;
    Ok(rename_in_math(&typst))
}

/// `text` markup: every character as itself.
pub fn text(line: &str) -> String {
    let mut s = String::with_capacity(line.len() + 8);
    s.push_str("#\"");
    for c in line.chars() {
        match c {
            '\\' => s.push_str("\\\\"),
            '"' => s.push_str("\\\""),
            _ => s.push(c),
        }
    }
    s.push('"');
    s
}

/// mitex's error for a command or environment it has no definition of.
fn classify(message: String) -> TypesetError {
    let body = message.strip_prefix("error: ").unwrap_or(&message);
    for prefix in ["unknown command: ", "unknown environment: "] {
        if let Some(name) = body.strip_prefix(prefix) {
            return TypesetError::Unsupported {
                construct: name.trim().to_owned(),
            };
        }
    }
    TypesetError::Invalid { message }
}

/// Symbol names mitex 0.2.4 still emits but Typst 0.15 removed (codex 0.3
/// changelog), with their replacements. A name matches as a whole dotted
/// identifier or as its prefix (`plus.circle.big` -> `plus.o.big`).
const RENAMED: &[(&str, &str)] = &[
    ("angle.l", "chevron.l"),
    ("angle.r", "chevron.r"),
    ("arrow.l.dash", "arrow.l.dashed"),
    ("arrow.r.dash", "arrow.r.dashed"),
    ("ast.circle", "convolve.o"),
    ("bracket.l.double", "bracket.l.stroked"),
    ("bracket.r.double", "bracket.r.stroked"),
    ("circle.nested", "compose.o"),
    ("dash.circle", "dash.o"),
    ("diff", "partial"),
    ("dot.circle", "dot.o"),
    ("minus.circle", "minus.o"),
    ("ohm.inv", "Omega.inv"),
    ("planck.reduce", "planck"),
    ("plus.circle", "plus.o"),
    ("sect", "inter"),
    ("times.circle", "times.o"),
];

/// Apply [`RENAMED`] to identifiers in math (`$...$`) outside text content
/// (`[...]`, e.g. `\text{diff}`); markup text keeps its words.
pub fn rename_in_math(typst: &str) -> String {
    let mut out = String::with_capacity(typst.len());
    let (mut math, mut text_depth) = (false, 0usize);
    let mut chars = typst.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '\\' => {
                out.push(c);
                if let Some((_, e)) = chars.next() {
                    out.push(e);
                }
            }
            '$' => {
                math = !math;
                text_depth = 0;
                out.push(c);
            }
            '[' if math => {
                text_depth += 1;
                out.push(c);
            }
            ']' if math => {
                text_depth = text_depth.saturating_sub(1);
                out.push(c);
            }
            c if c.is_ascii_alphabetic() => {
                let len = typst[i..]
                    .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.'))
                    .unwrap_or(typst.len() - i);
                let ident = &typst[i..i + len];
                for _ in 1..ident.chars().count() {
                    chars.next();
                }
                let renamed = (math && text_depth == 0)
                    .then(|| {
                        RENAMED.iter().find_map(|(old, new)| {
                            let tail = ident.strip_prefix(old)?;
                            (tail.is_empty() || tail.starts_with('.'))
                                .then(|| format!("{new}{tail}"))
                        })
                    })
                    .flatten();
                out.push_str(renamed.as_deref().unwrap_or(ident));
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
#[path = "tests_convert.rs"]
mod tests_convert;
