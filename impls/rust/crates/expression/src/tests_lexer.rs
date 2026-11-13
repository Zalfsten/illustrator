use super::*;

fn toks(src: &str) -> Vec<Tok> {
    lex(src).unwrap().into_iter().map(|t| t.tok).collect()
}

fn error(src: &str) -> ParseError {
    lex(src).unwrap_err()
}

#[test]
fn number_forms() {
    for (src, value) in [
        ("2", 2.0),
        ("2.5", 2.5),
        (".5", 0.5),
        ("5.", 5.0),
        ("1e-3", 1e-3),
        ("1.5E+3", 1500.0),
        ("5.e2", 500.0),
    ] {
        assert_eq!(toks(src), vec![Tok::Num(value), Tok::End], "{src}");
    }
}

#[test]
fn dot_before_operator_belongs_to_the_operator() {
    let tokens = lex("5.*x").unwrap();
    assert_eq!(tokens[0].tok, Tok::Num(5.0));
    assert_eq!(tokens[0].span, 0..1);
    assert_eq!(tokens[1].tok, Tok::Star);
    assert_eq!(tokens[1].span, 1..3);
    assert_eq!(tokens[2].tok, Tok::Ident);
    assert_eq!(toks("2./x")[1], Tok::Slash);
    assert_eq!(toks("2.^x")[1], Tok::Caret);
}

#[test]
fn operators_identifiers_and_spans() {
    let tokens = lex(" sin( x ) + 2").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| t.tok).collect();
    assert_eq!(
        kinds,
        vec![Tok::Ident, Tok::LParen, Tok::Ident, Tok::RParen, Tok::Plus, Tok::Num(2.0), Tok::End]
    );
    assert_eq!(tokens[0].span, 1..4);
    assert_eq!(tokens[2].span, 6..7);
    assert_eq!(tokens.last().unwrap().span, 13..13);
    assert_eq!(toks("a,b")[1], Tok::Comma);
    assert_eq!(toks("x-y")[1], Tok::Minus);
}

#[test]
fn rejected_characters_have_dedicated_errors() {
    for (src, what, span) in [
        ("x'", "transpose", 1..2),
        ("x.'", "transpose", 1..3),
        ("x\\2", "left division", 1..2),
        ("x.\\2", "left division", 1..3),
        ("x**2", "`**` (write `^`)", 1..3),
        ("x==1", "comparison", 1..3),
        ("x~=1", "comparison", 1..3),
        ("x<1", "comparison", 1..2),
        ("x>=1", "comparison", 1..3),
        ("x!=1", "comparison", 1..3),
        ("x=1", "assignment", 1..2),
        ("x&1", "logical operator", 1..2),
        ("x||1", "logical operator", 1..3),
        ("~x", "logical operator", 0..1),
        ("[x]", "a matrix or bracket", 0..1),
    ] {
        let e = error(src);
        assert_eq!(e.kind, ParseErrorKind::Unsupported(what), "{src}");
        assert_eq!(e.span, span, "{src}");
    }
}

#[test]
fn unexpected_characters_and_bad_numbers() {
    assert_eq!(error("x;").kind, ParseErrorKind::UnexpectedChar(';'));
    assert_eq!(error("x # 2").span, 2..3);
    assert_eq!(error("é").kind, ParseErrorKind::UnexpectedChar('é'));
    assert_eq!(error("é").span, 0..2);
    assert_eq!(error("x . 2").kind, ParseErrorKind::UnexpectedChar('.'));
    let e = error("2e");
    assert_eq!(e.kind, ParseErrorKind::InvalidNumber);
    assert_eq!(e.span, 0..2);
    assert_eq!(error("1e+x").kind, ParseErrorKind::InvalidNumber);
}
