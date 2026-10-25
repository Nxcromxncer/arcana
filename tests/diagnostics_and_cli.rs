mod common;
use arcana::checker::check_source;
use common::*;
use std::path::PathBuf;
use std::process::{Command, Output};

fn arcana(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arcana")).args(args).output().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn tmpdir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("arcana-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn diagnostic_format_matches_spec() {
    let e = error_of("spell main() {\n    bind mana = 1\n    cast mnaa\n}\n");
    let text = e.render();
    assert_eq!(text.lines().next().unwrap(), "error E1001: Unknown name 'mnaa'");
    assert!(text.contains("  --> test.arc:3:10"));
    assert!(text.contains("3 |     cast mnaa"));
    assert!(text.contains("^^^^"));
    assert!(text.contains("help: did you mean 'mana'?"));
    assert!(text.contains("note: names are case-sensitive"));
    assert_eq!(e.category(), "name");
}

#[test]
fn categories() {
    assert_eq!(error_of("cast \"x\"").category(), "syntax");
    assert_eq!(body_error("cast 1 / 0").category(), "runtime");
    assert_eq!(body_error("cast \"a\" + 1").category(), "type");
}

#[test]
fn builtin_error_points_at_call_site() {
    let e = body_error("cast integer(\"x\")");
    assert_eq!(e.span.unwrap().line, 2);
    assert_eq!(e.filename.as_deref(), Some("test.arc"));
}

#[test]
fn check_finds_unknown_names_and_redeclarations() {
    let errs = check_source("spell main() {\n bind a = 1\n bind a = 2\n cast b\n}\n", "<input>");
    let codes: Vec<&str> = errs.iter().map(|e| e.code.as_str()).collect();
    assert_eq!(codes, vec!["E3002", "E1001"]);
}

#[test]
fn check_allows_forward_references_and_builtins() {
    assert!(check_source("spell main() { cast later(1) + length([1]) }\nspell later(x) { return x }", "<input>").is_empty());
}

#[test]
fn check_reports_syntax_error() {
    assert!(check_source("spell main( {", "<input>")[0].code.starts_with("E4"));
}

#[test]
fn check_accepts_valid_examples() {
    for name in [
        "hello", "mana", "spells", "trials", "inputs", "lib", "imports", "use_lib", "fizzbuzz", "inventory", "closures", "guess",
        "safe_math", "wordcount", "factorial", "sum_input",
    ] {
        let path = format!("examples/{}.arc", name);
        let src = std::fs::read_to_string(&path).unwrap();
        assert!(check_source(&src, &path).is_empty(), "{}", name);
    }
}

#[test]
fn cli_exit_codes() {
    let d = tmpdir("exit");
    let good = d.join("good.arc");
    std::fs::write(&good, "spell main() { cast \"hi\" }\n").unwrap();
    let bad = d.join("bad.arc");
    std::fs::write(&bad, "spell main() { cast nope }\n").unwrap();
    let missing = d.join("missing.arc");

    let o = arcana(&["run", good.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(out(&o), "hi\n");

    let o = arcana(&["run", bad.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("E1001"));

    assert_eq!(arcana(&["run", missing.to_str().unwrap()]).status.code(), Some(2));
    assert_eq!(arcana(&["check", good.to_str().unwrap()]).status.code(), Some(0));
    assert_eq!(arcana(&["check", bad.to_str().unwrap()]).status.code(), Some(1));
    let o = arcana(&["version"]);
    assert_eq!(o.status.code(), Some(0));
    assert!(out(&o).contains("Arcana 0.1.0"));
    assert_eq!(arcana(&[]).status.code(), Some(2));
    assert_eq!(arcana(&["bogus"]).status.code(), Some(2));
}

#[test]
fn cli_run_shorthand_and_limits() {
    let d = tmpdir("short");
    let f = d.join("loop.arc");
    std::fs::write(&f, "spell main() { while On { } }\n").unwrap();
    let o = arcana(&["run", f.to_str().unwrap(), "--max-steps", "500"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("E2009"));
    let g = d.join("g.arc");
    std::fs::write(&g, "spell main() { cast 1 }\n").unwrap();
    assert_eq!(out(&arcana(&[g.to_str().unwrap()])), "1\n");
}

#[test]
fn cli_tokens() {
    let d = tmpdir("tok");
    let f = d.join("t.arc");
    std::fs::write(&f, "bind x = On ? hi\n").unwrap();
    let o = arcana(&["tokens", f.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0));
    let s = out(&o);
    assert!(s.contains("KW       bind") && s.contains("KW       On") && !s.contains("hi"));
}

#[test]
fn imports() {
    let d = tmpdir("imp");
    std::fs::write(d.join("util.arc"), "bind pi = 3\nspell twice(x) { return x * 2 }\n").unwrap();
    std::fs::create_dir_all(d.join("pkg")).unwrap();
    std::fs::write(d.join("pkg").join("deep.arc"), "spell hi() { return \"deep\" }\n").unwrap();
    let main = d.join("main.arc");
    std::fs::write(
        &main,
        "import util\nimport pkg.deep as d\nfrom util import twice\nspell main() {\n    cast util.pi\n    cast twice(4)\n    cast d.hi()\n}\n",
    )
    .unwrap();
    let o = arcana(&["run", main.to_str().unwrap()]);
    assert_eq!(out(&o), "3\n8\ndeep\n");
    assert_eq!(o.status.code(), Some(0));
}

#[test]
fn import_errors() {
    let d = tmpdir("imperr");
    let w = |n: &str, s: &str| std::fs::write(d.join(n), s).unwrap();
    w("a.arc", "import b\n");
    w("b.arc", "import a\n");
    w("m.arc", "import a\nspell main() { }\n");
    w("n.arc", "import nothere\nspell main() { }\n");
    w("ok.arc", "spell f() { }\n");
    w("p.arc", "from ok import zzz\nspell main() { }\n");
    let code = |n: &str| {
        let o = arcana(&["run", d.join(n).to_str().unwrap()]);
        assert_eq!(o.status.code(), Some(1));
        err(&o)
    };
    assert!(code("m.arc").contains("E5002"));
    assert!(code("n.arc").contains("E5001"));
    assert!(code("p.arc").contains("E5003"));
}

#[test]
fn error_inside_imported_module_names_that_file() {
    let d = tmpdir("modfile");
    std::fs::write(d.join("lib.arc"), "spell boom() {\n    return 1 / 0\n}\n").unwrap();
    let m = d.join("main.arc");
    std::fs::write(&m, "import lib\nspell main() { lib.boom() }\n").unwrap();
    let o = arcana(&["run", m.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1));
    let e = err(&o);
    assert!(e.contains("lib.arc:2") && e.contains("E2001"), "{}", e);
}

/// The programs shown in docs/06-cookbook.md must keep producing the documented output.
#[test]
fn documented_examples_produce_documented_output() {
    let run = |name: &str, stdin: &str| -> String {
        use std::io::Write;
        let mut child = Command::new(env!("CARGO_BIN_EXE_arcana"))
            .args(["run", &format!("examples/{}.arc", name)])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
        let o = child.wait_with_output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{}: {}", name, String::from_utf8_lossy(&o.stderr));
        String::from_utf8_lossy(&o.stdout).into_owned()
    };
    assert!(run("fizzbuzz", "").starts_with("1\n2\nFizz\n4\nBuzz\n"));
    assert!(run("fizzbuzz", "").ends_with("14\nFizzBuzz\n"));
    assert_eq!(
        run("inventory", ""),
        "POTION x3\nSCROLL x12\nWAND x1\nTotal items: 16\npotion, scroll, wand\n{\"potion\": 3, \"scroll\": 12, \"wand\": 1}\nOn\n[\"potion\", \"scroll\", \"wand\"]\n"
    );
    assert_eq!(run("wordcount", ""), "the: 3\nwand: 1\nand: 2\nstaff: 1\nscroll: 1\n");
    assert_eq!(run("closures", ""), "3\n110\n120\n");
    assert_eq!(run("factorial", ""), "15511210043330985984000000\n1267650600228229401496703205376\nanacra\n");
    assert_eq!(
        run("safe_math", ""),
        "division attempted\n3.0\nCaught MathError (E2001): Division by zero\ndivision attempted\nnull\nUserError: insufficient mana\n"
    );
    assert!(run("guess", "abc\n3\n9\n7\n").ends_with("Correct! It took 3 tries.\n"));
    assert!(run("sum_input", "10\n20\nabc\n5\n\n").ends_with("sum of 3 numbers = 35\n"));
    assert_eq!(run("imports", ""), "1.0\n42\n8\n");
}
