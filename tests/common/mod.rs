#![allow(dead_code)]
use arcana::errors::ArcanaError;
use arcana::interpreter::{Interpreter, Options};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct SharedBuf(pub Rc<RefCell<Vec<u8>>>);

impl Write for SharedBuf {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Run a program on a big-stack thread and capture its output.
pub fn run_with(src: &str, stdin: &str, max_depth: usize, max_steps: Option<i64>) -> Result<String, Box<ArcanaError>> {
    let (src, stdin) = (src.to_string(), stdin.to_string());
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let buf = SharedBuf::default();
            let opts = Options { max_steps, max_depth, ..Options::default() };
            let mut it = Interpreter::with_io(Box::new(buf.clone()), Box::new(std::io::Cursor::new(stdin.into_bytes())), opts);
            match it.run_source(&src, "test.arc", true) {
                Ok(_) => Ok(String::from_utf8(buf.0.borrow().clone()).unwrap()),
                Err(e) => Err(e),
            }
        })
        .unwrap()
        .join()
        .unwrap()
}

pub fn run(src: &str) -> String {
    run_with(src, "", 100, None).unwrap_or_else(|e| panic!("unexpected error:\n{}", e.render()))
}

pub fn error_of(src: &str) -> Box<ArcanaError> {
    match run_with(src, "", 100, None) {
        Ok(out) => panic!("expected an error, got output {:?}", out),
        Err(e) => e,
    }
}

fn wrap(body: &str) -> String {
    format!("spell main() {{\n{}\n}}\n", body)
}

pub fn main_body(body: &str) -> String {
    run(&wrap(body))
}

pub fn main_body_in(body: &str, stdin: &str) -> String {
    run_with(&wrap(body), stdin, 100, None).unwrap_or_else(|e| panic!("unexpected error:\n{}", e.render()))
}

pub fn body_error(body: &str) -> Box<ArcanaError> {
    error_of(&wrap(body))
}

/// Run a program capturing stdout, stderr (`warn`/`debug`) and returning program arguments.
pub fn run_full(src: &str, stdin: &str, args: &[&str]) -> (Result<String, Box<ArcanaError>>, String) {
    let (src, stdin) = (src.to_string(), stdin.to_string());
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let (out, err) = (SharedBuf::default(), SharedBuf::default());
            let opts = Options { max_depth: 100, args, ..Options::default() };
            let mut it = Interpreter::with_io(Box::new(out.clone()), Box::new(std::io::Cursor::new(stdin.into_bytes())), opts);
            it.set_err(Box::new(err.clone()));
            let r = it.run_source(&src, "test.arc", true);
            let stdout = String::from_utf8(out.0.borrow().clone()).unwrap();
            let stderr = String::from_utf8(err.0.borrow().clone()).unwrap();
            (r.map(|_| stdout), stderr)
        })
        .unwrap()
        .join()
        .unwrap()
}

/// `import`s every standard module under its own name, then runs `body` inside `main`.
pub fn std_body(body: &str) -> String {
    let src = format!(
        "import math\nimport text as t\nimport list as l\nimport logic\nimport convert as c\nimport error as err\nimport system\nimport file\nimport time\nimport date\nspell main() {{\n{}\n}}\n",
        body
    );
    run(&src)
}

pub fn std_error(body: &str) -> Box<ArcanaError> {
    let src = format!(
        "import math\nimport text as t\nimport list as l\nimport logic\nimport convert as c\nimport error as err\nimport system\nimport file\nimport time\nimport date\nspell main() {{\n{}\n}}\n",
        body
    );
    error_of(&src)
}
