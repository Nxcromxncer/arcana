//! `logic`: type checks, emptiness, and `all` / `any` over lists of booleans.
//!
//! Arcana has no truthiness, so `all` and `any` need lists of `On`/`Off` values.

use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{list_arg, register as reg, type_err};
use crate::values::*;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    reg(env, "is_number", 1, Some(1), |_, a| Ok(Value::Bool(is_number(&a[0]))));
    reg(env, "is_integer", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::Int(_)))));
    reg(env, "is_decimal", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::Dec(_)))));
    reg(env, "is_text", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::Text(_)))));
    reg(env, "is_boolean", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::Bool(_)))));
    reg(env, "is_list", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::List(_)))));
    reg(env, "is_map", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::Map(_)))));
    reg(env, "is_null", 1, Some(1), |_, a| Ok(Value::Bool(matches!(a[0], Value::Null))));
    reg(env, "is_empty", 1, Some(1), is_empty);
    reg(env, "all", 1, Some(1), |_, a| quantify("all", a, true));
    reg(env, "any", 1, Some(1), |_, a| quantify("any", a, false));
}

fn is_empty(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Ok(Value::Bool(match &a[0] {
        Value::Text(s) => s.is_empty(),
        Value::List(l) => l.borrow().is_empty(),
        Value::Map(m) => m.borrow().is_empty(),
        Value::Range(r) => r.len().is_zero(),
        other => {
            return Err(type_err(format!("is_empty() needs text, a list, a map or a range, but got {}", type_name(other)))
                .help("use is_null(x) to test for null")
                .boxed())
        }
    }))
}

fn quantify(name: &str, a: &[Value], want_all: bool) -> R<Value> {
    let l = list_arg(name, &a[0], "a list of booleans")?;
    for v in l.borrow().iter() {
        match v {
            Value::Bool(b) => {
                if want_all && !*b {
                    return Ok(Value::Bool(false));
                }
                if !want_all && *b {
                    return Ok(Value::Bool(true));
                }
            }
            other => {
                return Err(type_err(format!("{}() needs a list of booleans (On or Off), but found {}", name, type_name(other)))
                    .help("Arcana has no truthiness; compare explicitly first")
                    .boxed())
            }
        }
    }
    Ok(Value::Bool(want_all))
}
