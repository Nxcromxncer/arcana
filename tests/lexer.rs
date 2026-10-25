use arcana::lexer::Lexer;
use arcana::tokens::Kind;

fn toks(src: &str) -> Vec<(Kind, String)> {
    Lexer::new(src, "<input>")
        .tokenize()
        .unwrap()
        .into_iter()
        .filter(|t| !matches!(t.kind, Kind::Newline | Kind::Eof))
        .map(|t| (t.kind, t.display()))
        .collect()
}

fn s(kind: Kind, v: &str) -> (Kind, String) {
    (kind, v.to_string())
}

#[test]
fn question_mark_starts_a_comment() {
    assert_eq!(
        toks("bind x = 1 ? a comment\n? whole line\nx"),
        vec![s(Kind::Kw, "bind"), s(Kind::Ident, "x"), s(Kind::Op, "="), s(Kind::Int, "1"), s(Kind::Ident, "x")]
    );
}

#[test]
fn question_mark_inside_text_is_kept() {
    assert_eq!(toks("\"why? because\""), vec![s(Kind::Text, "why? because")]);
}

#[test]
fn on_off_are_keywords_and_true_false_are_not() {
    assert_eq!(
        toks("On Off true false"),
        vec![s(Kind::Kw, "On"), s(Kind::Kw, "Off"), s(Kind::Ident, "true"), s(Kind::Ident, "false")]
    );
}

#[test]
fn on_is_case_sensitive() {
    assert_eq!(toks("on ON")[0].0, Kind::Ident);
}

#[test]
fn numbers() {
    assert_eq!(
        toks("1 2.5 1_000 1e3 2.5e-1"),
        vec![
            s(Kind::Int, "1"),
            s(Kind::Decimal, "2.5"),
            s(Kind::Int, "1000"),
            s(Kind::Decimal, "1000.0"),
            s(Kind::Decimal, "0.25")
        ]
    );
}

#[test]
fn lexical_errors() {
    for (src, code) in [("5x", "E1005"), ("\"open", "E1003"), ("\"a\\q\"", "E1004"), ("@", "E1002"), ("#x", "E1002"), ("1__0", "E1005")] {
        let e = Lexer::new(src, "<input>").tokenize().unwrap_err();
        assert_eq!(e.code, code, "source {:?}", src);
    }
}

#[test]
fn escapes_and_positions() {
    let t = Lexer::new("\"a\\n\\u{41}\"", "<input>").tokenize().unwrap();
    assert_eq!(t[0].text(), "a\nA");
    let t2 = Lexer::new("  bind", "<input>").tokenize().unwrap();
    assert_eq!((t2[0].line, t2[0].col), (1, 3));
}
