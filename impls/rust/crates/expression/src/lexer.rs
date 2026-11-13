//! Tokens with byte spans.

use std::ops::Range;

use crate::error::{ParseError, ParseErrorKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Tok {
    Num(f64),
    Ident,
    Plus,
    Minus,
    /// `*` or `.*`
    Star,
    /// `/` or `./`
    Slash,
    /// `^` or `.^`
    Caret,
    LParen,
    RParen,
    Comma,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Token {
    pub tok: Tok,
    pub span: Range<usize>,
}

fn err(kind: ParseErrorKind, span: Range<usize>) -> ParseError {
    ParseError { kind, span }
}

/// Splits `src` into tokens, ending with [`Tok::End`] (empty span at the end).
pub(crate) fn lex(src: &str) -> Result<Vec<Token>, ParseError> {
    let b = src.as_bytes();
    let at = |i: usize| b.get(i).copied();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let start = i;
        let push = |out: &mut Vec<Token>, tok, len: usize| {
            out.push(Token {
                tok,
                span: start..start + len,
            })
        };
        match c {
            b' ' | b'\t' => i += 1,
            b'0'..=b'9' => {
                i = number(src, i)?;
                let value = src[start..i].parse().map_err(|_| {
                    err(ParseErrorKind::InvalidNumber, start..i)
                })?;
                out.push(Token {
                    tok: Tok::Num(value),
                    span: start..i,
                });
            }
            b'.' => match at(i + 1) {
                Some(b'0'..=b'9') => {
                    i = number(src, i)?;
                    let value = src[start..i]
                        .parse()
                        .map_err(|_| err(ParseErrorKind::InvalidNumber, start..i))?;
                    out.push(Token {
                        tok: Tok::Num(value),
                        span: start..i,
                    });
                }
                Some(b'*') => {
                    push(&mut out, Tok::Star, 2);
                    i += 2;
                }
                Some(b'/') => {
                    push(&mut out, Tok::Slash, 2);
                    i += 2;
                }
                Some(b'^') => {
                    push(&mut out, Tok::Caret, 2);
                    i += 2;
                }
                Some(b'\'') => return Err(err(ParseErrorKind::Unsupported("transpose"), i..i + 2)),
                Some(b'\\') => {
                    return Err(err(ParseErrorKind::Unsupported("left division"), i..i + 2))
                }
                _ => return Err(err(ParseErrorKind::UnexpectedChar('.'), i..i + 1)),
            },
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while matches!(at(i), Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')) {
                    i += 1;
                }
                out.push(Token {
                    tok: Tok::Ident,
                    span: start..i,
                });
            }
            b'+' => {
                push(&mut out, Tok::Plus, 1);
                i += 1;
            }
            b'-' => {
                push(&mut out, Tok::Minus, 1);
                i += 1;
            }
            b'*' => {
                if at(i + 1) == Some(b'*') {
                    return Err(err(ParseErrorKind::Unsupported("`**` (write `^`)"), i..i + 2));
                }
                push(&mut out, Tok::Star, 1);
                i += 1;
            }
            b'/' => {
                push(&mut out, Tok::Slash, 1);
                i += 1;
            }
            b'^' => {
                push(&mut out, Tok::Caret, 1);
                i += 1;
            }
            b'(' => {
                push(&mut out, Tok::LParen, 1);
                i += 1;
            }
            b')' => {
                push(&mut out, Tok::RParen, 1);
                i += 1;
            }
            b',' => {
                push(&mut out, Tok::Comma, 1);
                i += 1;
            }
            b'\'' => return Err(err(ParseErrorKind::Unsupported("transpose"), i..i + 1)),
            b'\\' => return Err(err(ParseErrorKind::Unsupported("left division"), i..i + 1)),
            b'=' | b'<' | b'>' | b'!' => {
                let len = if at(i + 1) == Some(b'=') { 2 } else { 1 };
                let what = if c == b'=' && len == 1 {
                    "assignment"
                } else {
                    "comparison"
                };
                return Err(err(ParseErrorKind::Unsupported(what), i..i + len));
            }
            b'~' => {
                if at(i + 1) == Some(b'=') {
                    return Err(err(ParseErrorKind::Unsupported("comparison"), i..i + 2));
                }
                return Err(err(ParseErrorKind::Unsupported("logical operator"), i..i + 1));
            }
            b'&' | b'|' => {
                let len = if at(i + 1) == Some(c) { 2 } else { 1 };
                return Err(err(ParseErrorKind::Unsupported("logical operator"), i..i + len));
            }
            b'[' | b']' | b'{' | b'}' => {
                return Err(err(ParseErrorKind::Unsupported("a matrix or bracket"), i..i + 1))
            }
            _ => {
                let ch = src[i..].chars().next().expect("in bounds");
                return Err(err(ParseErrorKind::UnexpectedChar(ch), i..i + ch.len_utf8()));
            }
        }
    }
    out.push(Token {
        tok: Tok::End,
        span: b.len()..b.len(),
    });
    Ok(out)
}

/// End of the number starting at `i`: `digits [. digits?]? | . digits`, then
/// an optional exponent. A dot followed by `*`, `/`, `^`, `'` or `\` belongs
/// to the next operator.
fn number(src: &str, mut i: usize) -> Result<usize, ParseError> {
    let b = src.as_bytes();
    let at = |i: usize| b.get(i).copied();
    let start = i;
    while matches!(at(i), Some(b'0'..=b'9')) {
        i += 1;
    }
    if at(i) == Some(b'.') && !matches!(at(i + 1), Some(b'*' | b'/' | b'^' | b'\'' | b'\\')) {
        i += 1;
        while matches!(at(i), Some(b'0'..=b'9')) {
            i += 1;
        }
    }
    if matches!(at(i), Some(b'e' | b'E')) {
        let mut j = i + 1;
        if matches!(at(j), Some(b'+' | b'-')) {
            j += 1;
        }
        if !matches!(at(j), Some(b'0'..=b'9')) {
            return Err(err(ParseErrorKind::InvalidNumber, start..j));
        }
        while matches!(at(j), Some(b'0'..=b'9')) {
            j += 1;
        }
        i = j;
    }
    Ok(i)
}

#[cfg(test)]
#[path = "tests_lexer.rs"]
mod tests_lexer;
