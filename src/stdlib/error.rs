//! `error`: raising errors, warnings, debugging output and assertions.
//!
//! `warn` and `debug` write to standard error so they never mix with program output.

use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{self, bool_arg, register as reg, rt};
use crate::values::*;
use std::io::Write;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    reg(env, "fail", 1, Some(1), runtime::fail);
    reg(env, "warn", 1, Some(1), |it, a| {
        stderr_line(it, &format!("warning: {}", to_text(&a[0], false)));
        Ok(Value::Null)
    });
    reg(env, "debug", 1, Some(1), |it, a| {
        stderr_line(it, &format!("debug: {} ({})", to_text(&a[0], true), type_name(&a[0])));
        Ok(Value::Null)
    });
    reg(env, "assert_true", 1, Some(2), |_, a| {
        let ok = bool_arg("assert_true", &a[0], "a boolean (On or Off) condition")?;
        if ok {
            return Ok(Value::Null);
        }
        let msg = a.get(1).map(|m| to_text(m, false)).unwrap_or_else(|| "Assertion failed: the condition is Off".to_string());
        Err(rt("E2101", "AssertionError", msg).boxed())
    });
    reg(env, "assert_equal", 2, Some(3), assert_equal);
}

fn stderr_line(it: &mut Interpreter, line: &str) {
    let _ = it.out.flush();
    let _ = writeln!(it.err_out, "{}", line);
    let _ = it.err_out.flush();
}

fn assert_equal(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    if arc_equal(&a[0], &a[1]) {
        return Ok(Value::Null);
    }
    let msg = match a.get(2) {
        Some(m) => to_text(m, false),
        None => format!("Assertion failed: {} is not equal to {}", to_text(&a[0], true), to_text(&a[1], true)),
    };
    Err(rt("E2101", "AssertionError", msg).boxed())
}
