//! Command-line interface.
//!
//! Exit codes: 0 success, 1 Arcana diagnostic (syntax/type/runtime/import
//! error), 2 usage or file-access problem.

use crate::checker::check_source;
use crate::errors::ArcanaError;
use crate::interpreter::{Interpreter, Options};
use crate::lexer::Lexer;
use crate::tokens::Kind;
use crate::VERSION;
use std::io::Write;

const USAGE: &str = "usage: arcana <command> [options]

commands:
  run <file.arc> [--max-steps N] [--max-depth N]   run a grimoire (.arc file)
  check <file.arc>                                 check a grimoire without running it
  tokens <file.arc>                                print the token stream
  version                                          print the version

example:
  arcana run hello.arc";

fn usage_error(msg: &str) -> i32 {
    eprintln!("{}", USAGE);
    eprintln!("arcana: error: {}", msg);
    2
}

fn read_source(path: &str) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            eprintln!("error: '{}' is not valid UTF-8 text", path);
            None
        }
        Err(e) => {
            let msg = e.to_string();
            let msg = msg.split(" (os error").next().unwrap_or(&msg).to_string();
            eprintln!("error: cannot read '{}': {}", path, msg);
            None
        }
    }
}

/// Run `f` on a thread with a very large stack so deep recursion is safe.
fn run_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let mut f = Some(f);
    for size in [1usize << 30, 256 << 20, 64 << 20] {
        let job = f.take().expect("job");
        let (tx, rx) = std::sync::mpsc::channel();
        let slot = std::sync::Arc::new(std::sync::Mutex::new(Some(job)));
        let slot2 = slot.clone();
        let spawned = std::thread::Builder::new().stack_size(size).spawn(move || {
            let job = slot2.lock().unwrap().take();
            if let Some(job) = job {
                let _ = tx.send(job());
            }
        });
        match spawned {
            Ok(handle) => {
                let _ = handle.join();
                return rx.recv().ok();
            }
            Err(_) => {
                // thread could not be created with this stack size; try a smaller one
                f = slot.lock().unwrap().take();
                if f.is_none() {
                    return None;
                }
            }
        }
    }
    None
}

fn parse_int_opt(name: &str, v: &str) -> Result<i64, i32> {
    v.parse::<i64>().map_err(|_| usage_error(&format!("argument {}: invalid int value: '{}'", name, v)))
}

fn cmd_run(args: &[String]) -> i32 {
    let mut file: Option<String> = None;
    let mut opts = Options::default();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let (flag, inline): (&str, Option<&str>) = match a.split_once('=') {
            Some((f, v)) if f.starts_with("--") => (f, Some(v)),
            _ => (a.as_str(), None),
        };
        match flag {
            "--max-steps" | "--max-depth" => {
                let value = match inline {
                    Some(v) => v.to_string(),
                    None => {
                        i += 1;
                        match args.get(i) {
                            Some(v) => v.clone(),
                            None => return usage_error(&format!("argument {}: expected one argument", flag)),
                        }
                    }
                };
                let n = match parse_int_opt(flag, &value) {
                    Ok(n) => n,
                    Err(code) => return code,
                };
                if flag == "--max-steps" {
                    opts.max_steps = Some(n);
                } else {
                    opts.max_depth = n.max(0) as usize;
                }
            }
            _ if a.starts_with("--") => return usage_error(&format!("unrecognized arguments: {}", a)),
            _ => {
                if file.is_some() {
                    return usage_error(&format!("unrecognized arguments: {}", a));
                }
                file = Some(a.clone());
            }
        }
        i += 1;
    }
    let file = match file {
        Some(f) => f,
        None => return usage_error("the following arguments are required: file"),
    };
    let source = match read_source(&file) {
        Some(s) => s,
        None => return 2,
    };
    let fname = file.clone();
    let result = run_big_stack(move || -> Result<(), Box<ArcanaError>> {
        let mut interp = Interpreter::new(opts);
        let r = interp.run_source(&source, &fname, true);
        interp.flush();
        r.map(|_| ())
    });
    match result {
        Some(Ok(())) => 0,
        Some(Err(e)) => {
            let _ = std::io::stdout().flush();
            eprintln!("{}", e.render());
            1
        }
        None => {
            eprintln!("error: the interpreter stopped unexpectedly");
            1
        }
    }
}

fn cmd_check(args: &[String]) -> i32 {
    let file = match args {
        [f] => f,
        [] => return usage_error("the following arguments are required: file"),
        [_, extra, ..] => return usage_error(&format!("unrecognized arguments: {}", extra)),
    };
    let source = match read_source(file) {
        Some(s) => s,
        None => return 2,
    };
    let errors = check_source(&source, file);
    if errors.is_empty() {
        println!("No problems found in {}", file);
        return 0;
    }
    for e in &errors {
        eprintln!("{}", e.render());
        eprintln!();
    }
    let n = errors.len();
    eprintln!("{} problem{} found in {}", n, if n != 1 { "s" } else { "" }, file);
    1
}

/// Python-style `repr` of a short string.
fn py_repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    let mut out = String::new();
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

fn cmd_tokens(args: &[String]) -> i32 {
    let file = match args {
        [f] => f,
        [] => return usage_error("the following arguments are required: file"),
        [_, extra, ..] => return usage_error(&format!("unrecognized arguments: {}", extra)),
    };
    let source = match read_source(file) {
        Some(s) => s,
        None => return 2,
    };
    let tokens = match Lexer::new(&source, file).tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e.render());
            return 1;
        }
    };
    let mut out = String::new();
    for t in &tokens {
        let shown = match t.kind {
            Kind::Text | Kind::Newline => py_repr(t.text()),
            _ => t.display(),
        };
        out.push_str(&format!("{}:{:<4} {:<8} {}\n", t.line, t.col, t.kind.name(), shown));
    }
    print!("{}", out);
    0
}

pub fn main_with_args(args: &[String]) -> i32 {
    let (cmd, rest) = match args.split_first() {
        Some(x) => x,
        None => return usage_error("the following arguments are required: command"),
    };
    match cmd.as_str() {
        "-h" | "--help" | "help" => {
            println!("{}", USAGE);
            0
        }
        "-V" | "--version" | "version" => {
            println!("Arcana {}", VERSION);
            0
        }
        "run" => cmd_run(rest),
        "check" => cmd_check(rest),
        "tokens" => cmd_tokens(rest),
        // `arcana hello.arc` is shorthand for `arcana run hello.arc`
        other if other.ends_with(".arc") => cmd_run(args),
        other => usage_error(&format!("argument command: invalid choice: '{}'", other)),
    }
}
