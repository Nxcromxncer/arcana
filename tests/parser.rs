use arcana::ast::{BinOp, Expr, Stmt, UnOp};
use arcana::errors::ArcanaError;
use arcana::parser::parse_source;

fn parse_main(body: &str) -> Vec<Stmt> {
    let p = parse_source(&format!("spell main() {{\n{}\n}}", body), "<input>").unwrap();
    match p.stmts.into_iter().next().unwrap() {
        Stmt::SpellDecl { body, .. } => std::rc::Rc::try_unwrap(body).ok().unwrap(),
        _ => panic!("not a spell"),
    }
}

fn err(src: &str) -> Box<ArcanaError> {
    parse_source(src, "<input>").unwrap_err()
}

#[test]
fn precedence_tree() {
    let stmts = parse_main("cast 1 + 2 * 3");
    match &stmts[0] {
        Stmt::Cast { value: Expr::Binary { op: BinOp::Add, right, .. }, .. } => {
            assert!(matches!(**right, Expr::Binary { op: BinOp::Mul, .. }));
        }
        _ => panic!("unexpected tree"),
    }
}

#[test]
fn power_binds_tighter_than_unary_minus() {
    let stmts = parse_main("cast -2 ** 2");
    match &stmts[0] {
        Stmt::Cast { value: Expr::Unary { op: UnOp::Neg, operand, .. }, .. } => {
            assert!(matches!(**operand, Expr::Binary { op: BinOp::Pow, .. }));
        }
        _ => panic!("unexpected tree"),
    }
}

#[test]
fn chained_comparison_rejected() {
    assert_eq!(err("spell main() { cast 1 < 2 < 3 }").code, "E4003");
}

#[test]
fn reserved_words_cannot_be_names() {
    assert_eq!(err("spell main() { bind On = 1 }").code, "E4006");
    assert_eq!(err("spell main() { bind count = 1 }").code, "E4006");
}

#[test]
fn stray_else() {
    let e = err("spell main() {\n if On { }\n else { }\n}");
    assert!(e.help.unwrap().contains("same line"));
}

#[test]
fn expression_statement_must_be_call() {
    assert_eq!(err("spell main() { 1 + 2 }").code, "E4004");
    assert_eq!(err("spell main() { f() = 3 }").code, "E4004");
}

#[test]
fn return_outside_spell() {
    let c = err("spell main() { }\nreturn 1").code;
    assert!(c == "E4002" || c == "E4005");
}

#[test]
fn missing_braces_and_unclosed_block() {
    assert_eq!(err("spell main() { cast 1").code, "E4001");
    assert_eq!(err("spell main() cast 1").code, "E4001");
}

#[test]
fn two_statements_on_one_line_rejected() {
    assert_eq!(err("spell main() { cast 1 cast 2 }").code, "E4001");
}

#[test]
fn try_requires_catch_or_finally() {
    assert_eq!(err("spell main() { try { } }").code, "E4001");
}

#[test]
fn param_rules() {
    assert_eq!(err("spell f(a = 1, b) { }").code, "E4001");
    assert_eq!(err("spell f(a, a) { }").code, "E3002");
}

#[test]
fn semicolons_rejected_by_lexer() {
    assert_eq!(err("spell main() { cast 1; }").code, "E1002");
}
