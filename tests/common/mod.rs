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
            let opts = Options { max_steps, max_depth, search_paths: vec![] };
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
