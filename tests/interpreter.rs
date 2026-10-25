mod common;
use common::*;

#[test]
fn hello() {
    assert_eq!(run("spell main() { cast \"The gates are open.\" }"), "The gates are open.\n");
}

#[test]
fn arithmetic_and_precedence() {
    assert_eq!(main_body("cast 2 + 3 * 4\ncast (2 + 3) * 4\ncast -2 ** 2\ncast 2 ** 3 ** 2\ncast 10 - 4 - 3"), "14\n20\n-4\n512\n3\n");
}

#[test]
fn division_semantics() {
    assert_eq!(main_body("cast 7 / 2\ncast 7 // 2\ncast -7 // 2\ncast -7 % 3\ncast 6 / 3"), "3.5\n3\n-4\n2\n2.0\n");
}

#[test]
fn bigint_no_overflow() {
    assert_eq!(main_body("cast 2 ** 100"), "1267650600228229401496703205376\n");
    assert_eq!(main_body("cast 9223372036854775807 + 1\ncast -9223372036854775807 - 2"), "9223372036854775808\n-9223372036854775809\n");
    assert_eq!(main_body("bind f = 1\nfor i in range(1, 26) { f = f * i }\ncast f"), "15511210043330985984000000\n");
}

#[test]
fn booleans_print_as_on_off() {
    assert_eq!(main_body("cast On\ncast Off\ncast 1 < 2\ncast [On, Off, null]"), "On\nOff\nOn\n[On, Off, null]\n");
}

#[test]
fn logic_short_circuits() {
    assert_eq!(main_body("cast Off and (1 / 0 > 1)\ncast On or (1 / 0 > 1)\ncast not Off"), "Off\nOn\nOn\n");
}

#[test]
fn strict_conditions() {
    let e = body_error("if 1 { cast 1 }");
    assert_eq!(e.code, "E3001");
    assert!(e.help.unwrap().contains("x != 0"));
    assert_eq!(body_error("while [] { }").code, "E3001");
    assert_eq!(body_error("cast On and 1").code, "E3001");
    assert_eq!(body_error("cast not 0").code, "E3001");
}

#[test]
fn equality_rules() {
    assert_eq!(
        main_body("cast 1 == 1.0\ncast On == 1\ncast \"1\" == 1\ncast null == null\ncast [1, [2]] == [1, [2]]\ncast {\"a\": 1} == {\"a\": 1}"),
        "On\nOff\nOff\nOn\nOn\nOn\n"
    );
}

#[test]
fn text_plus_number_is_an_error_with_help() {
    let e = body_error("cast \"a\" + 1");
    assert_eq!(e.code, "E3003");
    assert!(e.help.unwrap().contains("text("));
    assert_eq!(main_body("cast \"a\" + text(1)"), "a1\n");
}

#[test]
fn compare_mixed_types() {
    assert_eq!(body_error("cast 1 < \"a\"").code, "E3003");
}

#[test]
fn bind_assign_and_scope() {
    assert_eq!(main_body("bind x = 1\nx = x + 1\ncast x"), "2\n");
    assert_eq!(main_body("bind x = 1\nif On {\n bind x = 5\n cast x\n}\ncast x"), "5\n1\n");
}

#[test]
fn assign_to_undeclared_and_redeclare() {
    let e = body_error("y = 1");
    assert_eq!(e.code, "E1001");
    assert!(e.help.unwrap().contains("bind y"));
    assert_eq!(body_error("bind a = 1\nbind a = 2").code, "E3002");
}

#[test]
fn assign_outer_variable_from_inner_block() {
    assert_eq!(main_body("bind n = 0\nfor i in range(3) { n = n + i }\ncast n"), "3\n");
}

#[test]
fn unknown_name_suggestions() {
    let e = body_error("bind mana = 1\ncast Mana");
    assert_eq!(e.code, "E1001");
    assert!(e.help.unwrap().contains("mana"));
    assert!(body_error("cast true").help.unwrap().contains("On and Off"));
    assert!(body_error("cast on").help.unwrap().starts_with("Arcana's boolean"));
}

#[test]
fn cannot_overwrite_builtin() {
    assert_eq!(body_error("length = 3").code, "E3002");
    assert_eq!(main_body("bind length = 3\ncast length"), "3\n");
}

#[test]
fn loops() {
    assert_eq!(main_body("for i in range(3) { cast i }"), "0\n1\n2\n");
    assert_eq!(main_body("for i in range(3, 0, -1) { cast i }"), "3\n2\n1\n");
    assert_eq!(main_body("for i in range(0) { cast i }"), "");
    assert_eq!(main_body("for c in \"ab\" { cast c }"), "a\nb\n");
    assert_eq!(main_body("for k in {\"x\": 1, \"y\": 2} { cast k }"), "x\ny\n");
    assert_eq!(body_error("for i in range(1, 5, 0) { }").code, "E2006");
    assert!(body_error("for i in 5 { }").help.unwrap().contains("range(5)"));
}

#[test]
fn break_continue() {
    assert_eq!(main_body("for i in range(10) {\n if i == 2 { continue }\n if i == 4 { break }\n cast i\n}"), "0\n1\n3\n");
}

#[test]
fn break_outside_loop_is_error() {
    assert_eq!(error_of("spell main() { break }").code, "E2003");
    assert_eq!(error_of("spell main() { for i in range(2) { spell f() { break } } }").code, "E2003");
}

#[test]
fn for_iterates_over_snapshot() {
    assert_eq!(main_body("bind xs = [1, 2]\nfor x in xs { append(xs, x) }\ncast xs"), "[1, 2, 1, 2]\n");
}

#[test]
fn functions_recursion_closures() {
    let src = "
spell fact(n) {
    if n <= 1 { return 1 }
    return n * fact(n - 1)
}
spell make_counter() {
    bind count_ = 0
    spell next() {
        count_ = count_ + 1
        return count_
    }
    return next
}
spell main() {
    cast fact(10)
    bind c = make_counter()
    c()
    c()
    cast c()
    bind d = make_counter()
    cast d()
}
";
    assert_eq!(run(src), "3628800\n3\n1\n");
}

#[test]
fn default_args_evaluated_each_call() {
    let src = "
spell f(x = []) {
    append(x, 1)
    return x
}
spell main() {
    cast f()
    cast f()
}
";
    assert_eq!(run(src), "[1]\n[1]\n");
}

#[test]
fn arity_errors() {
    let e = error_of("spell add(a, b) { return a + b }\nspell main() { add(1) }");
    assert_eq!(e.code, "E3004");
    assert!(e.help.unwrap().contains("add(a, b)"));
    assert_eq!(error_of("spell main() { length() }").code, "E3004");
    assert_eq!(error_of("spell main() { 5() }").code, "E3005");
}

#[test]
fn mutual_recursion_and_forward_reference() {
    let src = "
spell is_even(n) { if n == 0 { return On }
 return is_odd(n - 1) }
spell is_odd(n) { if n == 0 { return Off }
 return is_even(n - 1) }
spell main() { cast is_even(10) }
";
    assert_eq!(run(src), "On\n");
}

#[test]
fn recursion_limit() {
    let e = run_with("spell f(n) { return f(n + 1) }\nspell main() { f(0) }", "", 50, None).unwrap_err();
    assert_eq!(e.code, "E2007");
}

#[test]
fn default_depth_limit_does_not_crash() {
    let e = run_with("spell f(n) { return f(n + 1) }\nspell main() { f(0) }", "", 1000, None).unwrap_err();
    assert_eq!(e.code, "E2007");
    let out = run_with("spell f(n) { if n == 0 { return 0 }\n return 1 + f(n - 1) }\nspell main() { cast f(900) }", "", 1000, None).unwrap();
    assert_eq!(out, "900\n");
}

#[test]
fn step_limit() {
    let e = run_with("spell main() { while On { } }", "", 100, Some(1000)).unwrap_err();
    assert_eq!(e.code, "E2009");
}

#[test]
fn no_main() {
    assert_eq!(error_of("bind x = 1").code, "E2004");
}

#[test]
fn top_level_statements_rejected() {
    assert_eq!(error_of("cast \"hi\"").code, "E4002");
}

#[test]
fn collections() {
    assert_eq!(
        main_body("bind xs = [1, 2, 3]\nxs[0] = 10\nappend(xs, 4)\ncast xs\ncast length(xs)\ncast pop(xs)\ncast 2 in xs"),
        "[10, 2, 3, 4]\n4\n4\nOn\n"
    );
    assert_eq!(
        main_body("bind m = {\"a\": 1}\nm[\"b\"] = 2\ncast m\ncast keys(m)\ncast values(m)\ncast \"z\" in m"),
        "{\"a\": 1, \"b\": 2}\n[\"a\", \"b\"]\n[1, 2]\nOff\n"
    );
}

#[test]
fn collections_are_shared_references() {
    assert_eq!(main_body("bind a = [1]\nbind b = a\nappend(b, 2)\ncast a\nbind c = copy(a)\nappend(c, 3)\ncast a"), "[1, 2]\n[1, 2]\n");
}

#[test]
fn index_errors() {
    assert_eq!(body_error("bind xs = [1]\ncast xs[1]").code, "E2002");
    assert_eq!(body_error("bind xs = [1]\ncast xs[-1]").code, "E2002");
    assert_eq!(body_error("bind m = {\"a\": 1}\ncast m[\"b\"]").code, "E2005");
    assert_eq!(body_error("cast 5[0]").code, "E3003");
    assert_eq!(body_error("bind s = \"ab\"\ns[0] = \"x\"").code, "E3003");
    assert_eq!(body_error("bind m = {}\nm[[1]] = 1").code, "E3003");
    assert_eq!(main_body("cast \"abc\"[1]"), "b\n");
}

#[test]
fn multiline_literals_and_continuation() {
    let src = "spell main() {\n bind xs = [\n  1,\n  2,\n]\n bind total = 1 +\n  2\n cast xs\n cast total\n cast {\n \"a\": 1,\n }\n}";
    assert_eq!(run(src), "[1, 2]\n3\n{\"a\": 1}\n");
}

#[test]
fn try_catch_finally() {
    let src = "
spell main() {
    try {
        cast 1 / 0
    } catch e {
        cast e[\"code\"]
        cast e[\"kind\"]
    } finally {
        cast \"done\"
    }
    for i in range(3) {
        try {
            if i == 1 { continue }
            cast i
        } finally {
            cast \"f\"
        }
    }
}
";
    assert_eq!(run(src), "E2001\nMathError\ndone\n0\nf\nf\n2\nf\n");
}

#[test]
fn programmer_errors_are_not_catchable() {
    assert_eq!(body_error("try { cast missing } catch e { cast 1 }").code, "E1001");
    assert_eq!(body_error("try { cast \"a\" + 1 } catch e { cast 1 }").code, "E3003");
}

#[test]
fn fail_and_uncaught_recoverable_errors_propagate() {
    assert_eq!(main_body("try { fail(\"boom\") } catch e { cast e[\"message\"] }"), "boom\n");
    assert_eq!(body_error("try { cast 1 / 0 } finally { cast 1 }").code, "E2001");
}

#[test]
fn return_inside_try_runs_finally() {
    let src = "spell f() {\n try { return 1 } finally { cast \"cleanup\" }\n}\nspell main() { cast f() }";
    assert_eq!(run(src), "cleanup\n1\n");
}

#[test]
fn input_functions() {
    assert_eq!(main_body_in("bind n = ask(\"name? \")\ncast n", "Mira\n"), "name? Mira\n");
    assert_eq!(main_body_in("cast ask_integer() + 1\ncast ask_decimal() * 2", "41\n1.5\n"), "42\n3.0\n");
    let body = "try { cast ask_integer(\"n: \") } catch e { cast e[\"code\"] }";
    assert_eq!(main_body_in(body, "abc\n"), "n: E2010\n");
    assert_eq!(main_body_in(body, ""), "n: E2010\n");
}

#[test]
fn conversions() {
    assert_eq!(
        main_body(
            "cast integer(\"42\")\ncast decimal(\"2.5\")\ncast integer(3.0)\ncast round(2.5)\ncast round(-2.5)\ncast text(On)\ncast type_of(1)\ncast type_of(1.0)\ncast type_of(null)"
        ),
        "42\n2.5\n3\n3\n-3\nOn\ninteger\ndecimal\nnull\n"
    );
    assert_eq!(body_error("cast integer(\"x\")").code, "E2011");
    assert_eq!(body_error("cast integer(3.5)").code, "E2011");
    assert_eq!(body_error("cast decimal(\"nan\")").code, "E2011");
}

#[test]
fn numeric_edge_cases() {
    assert_eq!(body_error("cast 1 / 0").code, "E2001");
    assert_eq!(body_error("cast 1 // 0").code, "E2001");
    assert_eq!(body_error("cast 5 % 0").code, "E2001");
    assert_eq!(body_error("cast 10.0 ** 400").code, "E2008");
    assert_eq!(body_error("cast (0 - 8) ** 0.5").code, "E2008");
    assert_eq!(body_error("cast 2 ** 100000000").code, "E2008");
    assert_eq!(main_body("cast 0.1 + 0.2"), "0.30000000000000004\n");
}

#[test]
fn float_formatting_matches_python_repr() {
    assert_eq!(
        main_body("cast 1e16\ncast 1e15\ncast 0.0001\ncast 0.00001\ncast 1.5e-7\ncast 123456789.125\ncast 1e22\ncast 2.0 ** 70\ncast 1 / 3\ncast -0.5"),
        "1e+16\n1000000000000000.0\n0.0001\n1e-05\n1.5e-07\n123456789.125\n1e+22\n1.1805916207174113e+21\n0.3333333333333333\n-0.5\n"
    );
}

#[test]
fn mixed_number_semantics() {
    assert_eq!(main_body("cast 7.5 // 2\ncast -7.5 % 2\ncast 2 ** -1\ncast 7 / 2.0\ncast 1 < 1.5\ncast 3 == 3.0"), "3.0\n0.5\n0.5\n3.5\nOn\nOn\n");
    assert_eq!(main_body("cast 2 ** 64 / 2 ** 62\ncast 10 ** 30 / 10 ** 28"), "4.0\n100.0\n");
}

#[test]
fn text_functions() {
    assert_eq!(
        main_body("cast upper(\"ab\")\ncast split(\"a,b\", \",\")\ncast join([\"a\", \"b\"], \"-\")\ncast length(\"héllo\")\ncast \"ell\" in \"hello\""),
        "AB\n[\"a\", \"b\"]\na-b\n5\nOn\n"
    );
}

#[test]
fn value_text_forms() {
    assert_eq!(
        main_body("cast [\"a\", 1, 2.5, null]\ncast range(3)\ncast length"),
        "[\"a\", 1, 2.5, null]\nrange(0, 3, 1)\n<built-in spell length>\n"
    );
    assert_eq!(main_body("bind xs = [1]\nappend(xs, xs)\ncast xs"), "[1, [...]]\n");
}

#[test]
fn min_max() {
    assert_eq!(main_body("cast min(3, 1, 2)\ncast max([4, 9])"), "1\n9\n");
}

#[test]
fn ranges() {
    assert_eq!(main_body("cast 3 in range(5)\ncast 5 in range(5)\ncast range(10, 0, -3)[2]\ncast length(range(1, 10, 4))"), "On\nOff\n4\n3\n");
}
