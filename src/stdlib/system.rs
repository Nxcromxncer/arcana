//! `system`: version, platform, clock, program arguments, environment and exit.
//!
//! Only these deliberate capabilities are exposed; there is no way to run other
//! programs or to read arbitrary memory.

use crate::environment::Env;
use crate::errors::ArcanaError;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{register as reg, rt, small_int_arg, text_arg};
use crate::values::*;
use std::io::Write;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    reg(env, "version", 0, Some(0), |_, _| Ok(Value::text(crate::VERSION)));
    reg(env, "platform", 0, Some(0), |_, _| Ok(Value::text(std::env::consts::OS)));
    reg(env, "clock", 0, Some(0), |it, _| Ok(Value::Dec(it.elapsed_secs())));
    reg(env, "args", 0, Some(0), |it, _| Ok(Value::list(it.prog_args.iter().map(Value::text).collect())));
    reg(env, "env", 1, Some(1), |_, a| {
        let name = text_arg("env", &a[0], "the name of an environment variable")?;
        Ok(match std::env::var(name.as_ref()) {
            Ok(v) => Value::text(v),
            Err(_) => Value::Null,
        })
    });
    reg(env, "cwd", 0, Some(0), |_, _| match std::env::current_dir() {
        Ok(p) => Ok(Value::text(p.display().to_string())),
        Err(e) => Err(rt("E2012", "FileError", format!("Cannot read the current directory: {}", e)).boxed()),
    });
    reg(env, "write", 1, Some(1), |it, a| {
        let _ = it.out.write_all(to_text(&a[0], false).as_bytes());
        Ok(Value::Null)
    });
    reg(env, "exit", 0, Some(1), exit);
}

/// Stops the program immediately with the given status (default 0). `finally`
/// blocks still run; `catch` cannot intercept it.
fn exit(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let code = match a.first() {
        Some(v) => small_int_arg("exit", v, "an integer exit code")?,
        None => 0,
    };
    if !(0..=255).contains(&code) {
        return Err(rt("E2006", "ValueError", "exit() code must be between 0 and 255").boxed());
    }
    Err(ArcanaError::new("E0000", "exit", None).with_exit_code(code as i32).boxed())
}
