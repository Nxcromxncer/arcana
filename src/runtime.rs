//! Core built-in spells (always in scope) and the helpers shared by every
//! standard-library module (`src/stdlib/`).
//!
//! A built-in is a plain function `fn(&mut Interpreter, &[Value]) -> R<Value>`.
//! Built-ins raise errors without a span; the interpreter fills in the call site.

use crate::environment::Env;
use crate::errors::ArcanaError;
use crate::int::Int;
use crate::interpreter::{Interpreter, R};
use crate::values::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::io::{BufRead, Write};
use std::rc::Rc;

pub type NativeFn = fn(&mut Interpreter, &[Value]) -> R<Value>;

// ---- helpers shared with the standard library --------------------------------------

/// A catchable runtime error (E2xxx) of the given kind.
pub fn rt(code: &str, kind: &str, message: impl Into<String>) -> ArcanaError {
    ArcanaError::new(code, message, None).recoverable(kind)
}

/// A type error (E3003).
pub fn type_err(message: impl Into<String>) -> ArcanaError {
    ArcanaError::new("E3003", message, None)
}

pub fn math_err(message: impl Into<String>) -> ArcanaError {
    rt("E2008", "MathError", message)
}

pub fn need(ok: bool, fname: &str, expected: &str, v: &Value) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err(type_err(format!("{}() needs {}, but got {}", fname, expected, type_name(v))).boxed())
    }
}

pub fn register(env: &Rc<Env>, name: &'static str, min: usize, max: Option<usize>, func: NativeFn) {
    env.set_local(name, Value::Builtin(Rc::new(Builtin { name, func, min_args: min, max_args: max })));
}

pub fn text_arg(fname: &str, v: &Value, what: &str) -> R<Rc<str>> {
    match v {
        Value::Text(s) => Ok(s.clone()),
        _ => Err(type_err(format!("{}() needs {}, but got {}", fname, what, type_name(v))).boxed()),
    }
}

pub fn int_arg(fname: &str, v: &Value, what: &str) -> R<Int> {
    match v {
        Value::Int(i) => Ok(i.clone()),
        _ => Err(type_err(format!("{}() needs {}, but got {}", fname, what, type_name(v))).boxed()),
    }
}

/// An integer that fits in an i64 (otherwise E2006).
pub fn small_int_arg(fname: &str, v: &Value, what: &str) -> R<i64> {
    let i = int_arg(fname, v, what)?;
    i.to_i64().ok_or_else(|| rt("E2006", "ValueError", format!("{}() got an integer that is too large", fname)).boxed())
}

pub fn list_arg(fname: &str, v: &Value, what: &str) -> R<Rc<RefCell<Vec<Value>>>> {
    match v {
        Value::List(l) => Ok(l.clone()),
        _ => Err(type_err(format!("{}() needs {}, but got {}", fname, what, type_name(v))).boxed()),
    }
}

pub fn bool_arg(fname: &str, v: &Value, what: &str) -> R<bool> {
    match v {
        Value::Bool(b) => Ok(*b),
        _ => Err(type_err(format!("{}() needs {}, but got {}", fname, what, type_name(v))).boxed()),
    }
}

/// Any number as an f64 (an integer that is too large is E2008).
pub fn float_arg(fname: &str, v: &Value, what: &str) -> R<f64> {
    match v {
        Value::Dec(f) => Ok(*f),
        Value::Int(i) => i.to_f64().ok_or_else(|| math_err("Number is too large to be a decimal").boxed()),
        _ => Err(type_err(format!("{}() needs {}, but got {}", fname, what, type_name(v))).boxed()),
    }
}

pub fn finite(x: f64) -> R<Value> {
    if x.is_finite() {
        Ok(Value::Dec(x))
    } else {
        Err(math_err("Numeric overflow: the result is too large").boxed())
    }
}

pub fn parse_integer(s: &str) -> Option<Int> {
    Int::parse(s.trim())
}

pub fn parse_decimal(s: &str) -> Option<f64> {
    let t = s.trim();
    let cleaned: String;
    let t = if t.contains('_') {
        let b = t.as_bytes();
        let ok = !t.starts_with('_')
            && !t.ends_with('_')
            && (0..b.len()).all(|i| {
                b[i] != b'_' || (i > 0 && b[i - 1].is_ascii_digit() && i + 1 < b.len() && b[i + 1].is_ascii_digit())
            });
        if !ok {
            return None;
        }
        cleaned = t.replace('_', "");
        cleaned.as_str()
    } else {
        t
    };
    let v: f64 = t.parse().ok()?;
    if v.is_finite() {
        Some(v)
    } else {
        None
    }
}

fn is_whole(f: f64) -> bool {
    f.is_finite() && f.fract() == 0.0
}

// ---- registry ---------------------------------------------------------------------------

type Entry = (&'static str, usize, Option<usize>, NativeFn);

/// The spells that are always in scope.
pub const GLOBALS: &[Entry] = &[
    ("ask", 0, Some(1), ask),
    ("ask_integer", 0, Some(1), ask_integer),
    ("ask_decimal", 0, Some(1), ask_decimal),
    ("text", 1, Some(1), to_text_fn),
    ("integer", 1, Some(1), integer),
    ("decimal", 1, Some(1), decimal),
    ("to_text", 1, Some(1), to_text_fn),
    ("to_integer", 1, Some(1), integer),
    ("to_decimal", 1, Some(1), decimal),
    ("to_number", 1, Some(1), to_number),
    ("round", 1, Some(2), round),
    ("abs", 1, Some(1), abs),
    ("min", 1, None, min),
    ("max", 1, None, max),
    ("length", 1, Some(1), length),
    ("range", 1, Some(3), range),
    ("append", 2, Some(2), append),
    ("pop", 1, Some(1), pop),
    ("keys", 1, Some(1), keys),
    ("values", 1, Some(1), values),
    ("copy", 1, Some(1), copy),
    ("upper", 1, Some(1), upper),
    ("lower", 1, Some(1), lower),
    ("split", 1, Some(2), split),
    ("join", 1, Some(2), join),
    ("type_of", 1, Some(1), type_of),
    ("fail", 1, Some(1), fail),
    ("assert", 1, Some(2), assert),
];

pub fn make_builtins(env: &Rc<Env>) {
    for (name, lo, hi, f) in GLOBALS {
        register(env, name, *lo, *hi, *f);
    }
}

pub fn builtin_names() -> Vec<String> {
    GLOBALS.iter().map(|e| e.0.to_string()).collect()
}

pub fn is_builtin(name: &str) -> bool {
    GLOBALS.iter().any(|e| e.0 == name)
}

// ---- input ----------------------------------------------------------------------------------

fn read_line(it: &mut Interpreter, args: &[Value]) -> R<String> {
    let empty = Value::text("");
    let prompt = args.first().unwrap_or(&empty);
    need(matches!(prompt, Value::Text(_)), "ask", "text for the prompt", prompt)?;
    if let Value::Text(p) = prompt {
        let _ = it.out.write_all(p.as_bytes());
    }
    let _ = it.out.flush();
    let mut buf = Vec::new();
    let n = it.inp.read_until(b'\n', &mut buf).unwrap_or(0);
    if n == 0 {
        return Err(rt("E2010", "InputError", "No input available (end of input reached)")
            .help("provide input on standard input, or catch the error with try/catch")
            .boxed());
    }
    let line = String::from_utf8_lossy(&buf).into_owned();
    Ok(line.trim_end_matches('\n').trim_end_matches('\r').to_string())
}

fn ask(it: &mut Interpreter, args: &[Value]) -> R<Value> {
    Ok(Value::text(read_line(it, args)?))
}

fn ask_integer(it: &mut Interpreter, args: &[Value]) -> R<Value> {
    let line = read_line(it, args)?;
    match parse_integer(&line) {
        Some(v) => Ok(Value::Int(v)),
        None => Err(rt("E2010", "InputError", format!("{} is not a valid integer", quote(&line)))
            .help("enter a whole number, such as 42")
            .boxed()),
    }
}

fn ask_decimal(it: &mut Interpreter, args: &[Value]) -> R<Value> {
    let line = read_line(it, args)?;
    match parse_decimal(&line) {
        Some(v) => Ok(Value::Dec(v)),
        None => Err(rt("E2010", "InputError", format!("{} is not a valid decimal", quote(&line)))
            .help("enter a number, such as 3.14 or 42")
            .boxed()),
    }
}

// ---- conversion -------------------------------------------------------------------------------

fn to_text_fn(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Ok(Value::text(to_text(&a[0], false)))
}

fn integer(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::Int(_) => Ok(a[0].clone()),
        Value::Dec(x) => {
            if is_whole(*x) {
                Ok(Value::Int(Int::from_f64(*x).expect("finite")))
            } else {
                Err(rt(
                    "E2011",
                    "ConversionError",
                    format!("Cannot convert decimal {} to integer without losing its fraction", to_text(&a[0], false)),
                )
                .help("use round(x) to round to the nearest integer")
                .boxed())
            }
        }
        Value::Text(s) => match parse_integer(s) {
            Some(v) => Ok(Value::Int(v)),
            None => Err(rt("E2011", "ConversionError", format!("Cannot convert {} to integer", quote(s)))
                .help("the text must look like a whole number, e.g. \"42\"")
                .boxed()),
        },
        other => Err(type_err(format!("integer() cannot convert {}", type_name(other))).boxed()),
    }
}

fn decimal(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::Dec(_) => Ok(a[0].clone()),
        Value::Int(i) => match i.to_f64() {
            Some(f) => Ok(Value::Dec(f)),
            None => Err(math_err("Number is too large to be a decimal").boxed()),
        },
        Value::Text(s) => match parse_decimal(s) {
            Some(v) => Ok(Value::Dec(v)),
            None => Err(rt("E2011", "ConversionError", format!("Cannot convert {} to decimal", quote(s)))
                .help("the text must look like a number, e.g. \"3.14\"")
                .boxed()),
        },
        other => Err(type_err(format!("decimal() cannot convert {}", type_name(other))).boxed()),
    }
}

fn to_number(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::Int(_) | Value::Dec(_) => Ok(a[0].clone()),
        Value::Text(s) => {
            if let Some(i) = parse_integer(s) {
                return Ok(Value::Int(i));
            }
            match parse_decimal(s) {
                Some(f) => Ok(Value::Dec(f)),
                None => Err(rt("E2011", "ConversionError", format!("Cannot convert {} to a number", quote(s)))
                    .help("the text must look like a number, e.g. \"42\" or \"3.14\"")
                    .boxed()),
            }
        }
        other => Err(type_err(format!("to_number() cannot convert {}", type_name(other))).boxed()),
    }
}

/// `round(x)` gives an integer (halves round away from zero);
/// `round(x, digits)` gives a decimal rounded to 0..=15 places.
fn round(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    need(is_number(&a[0]), "round", "a number", &a[0])?;
    if a.len() == 2 {
        let d = small_int_arg("round", &a[1], "an integer number of digits")?;
        if !(0..=15).contains(&d) {
            return Err(rt("E2006", "ValueError", "round() digits must be between 0 and 15").boxed());
        }
        let x = float_arg("round", &a[0], "a number")?;
        let p = 10f64.powi(d as i32);
        let y = x * p;
        if !y.is_finite() {
            return Ok(Value::Dec(x));
        }
        return finite((y.abs() + 0.5).floor().copysign(y) / p);
    }
    match &a[0] {
        Value::Dec(x) => {
            let r = (x.abs() + 0.5).floor().copysign(*x);
            Ok(Value::Int(Int::from_f64(r).expect("finite")))
        }
        other => Ok(other.clone()),
    }
}

pub fn abs(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    need(is_number(&a[0]), "abs", "a number", &a[0])?;
    match &a[0] {
        Value::Int(i) => Ok(Value::Int(i.abs())),
        Value::Dec(f) => Ok(Value::Dec(f.abs())),
        _ => unreachable!(),
    }
}

fn extremum(name: &str, is_min: bool, args: &[Value]) -> R<Value> {
    let items: Vec<Value> = match args {
        [Value::List(l)] => l.borrow().clone(),
        _ => args.to_vec(),
    };
    if items.is_empty() {
        return Err(type_err(format!("{}() needs at least one value", name)).boxed());
    }
    let all_num = items.iter().all(is_number);
    let all_text = items.iter().all(|i| matches!(i, Value::Text(_)));
    if !(all_num || all_text) {
        return Err(type_err(format!("{}() needs all numbers or all text", name)).boxed());
    }
    let mut best = items[0].clone();
    for it in &items[1..] {
        let ord = cmp_values(it, &best).unwrap_or(Ordering::Equal);
        if (is_min && ord == Ordering::Less) || (!is_min && ord == Ordering::Greater) {
            best = it.clone();
        }
    }
    Ok(best)
}

pub fn min(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    extremum("min", true, a)
}

pub fn max(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    extremum("max", false, a)
}

// ---- collections -------------------------------------------------------------------------------

fn length(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::Text(s) => Ok(Value::int(s.chars().count() as i64)),
        Value::List(l) => Ok(Value::int(l.borrow().len() as i64)),
        Value::Map(m) => Ok(Value::int(m.borrow().len() as i64)),
        Value::Range(r) => Ok(Value::Int(r.len())),
        other => Err(type_err(format!(
            "length() needs text, a list, a map or a range, but got {}",
            type_name(other)
        ))
        .boxed()),
    }
}

fn range(_: &mut Interpreter, args: &[Value]) -> R<Value> {
    let mut ints = Vec::new();
    for a in args {
        match a {
            Value::Int(i) => ints.push(i.clone()),
            other => {
                return Err(type_err(format!("range() needs integers, but got {}", type_name(other)))
                    .help("use integer(x) or round(x) to convert")
                    .boxed())
            }
        }
    }
    let (start, stop, step) = match ints.len() {
        1 => (Int::Small(0), ints[0].clone(), Int::Small(1)),
        2 => (ints[0].clone(), ints[1].clone(), Int::Small(1)),
        _ => (ints[0].clone(), ints[1].clone(), ints[2].clone()),
    };
    if step.is_zero() {
        return Err(rt("E2006", "ValueError", "range() step cannot be zero")
            .help("use a positive step to count up or a negative step to count down")
            .boxed());
    }
    Ok(Value::Range(Rc::new(RangeVal { start, stop, step })))
}

pub fn append(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("append", &a[0], "a list as its first argument")?;
    l.borrow_mut().push(a[1].clone());
    Ok(Value::Null)
}

pub fn pop(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("pop", &a[0], "a list")?;
    let popped = l.borrow_mut().pop();
    match popped {
        Some(v) => Ok(v),
        None => Err(rt("E2002", "IndexError", "Cannot pop from an empty list").help("check length(list) > 0 first").boxed()),
    }
}

fn keys(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    need(matches!(a[0], Value::Map(_)), "keys", "a map", &a[0])?;
    if let Value::Map(m) = &a[0] {
        return Ok(Value::list(m.borrow().iter().map(|(k, _)| k.to_value()).collect()));
    }
    unreachable!()
}

fn values(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    need(matches!(a[0], Value::Map(_)), "values", "a map", &a[0])?;
    if let Value::Map(m) = &a[0] {
        return Ok(Value::list(m.borrow().iter().map(|(_, v)| v.clone()).collect()));
    }
    unreachable!()
}

fn copy(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::List(l) => Ok(Value::list(l.borrow().clone())),
        Value::Map(m) => Ok(Value::map(m.borrow().clone())),
        other => Ok(other.clone()),
    }
}

// ---- text ---------------------------------------------------------------------------------------

pub fn upper(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Ok(Value::text(text_arg("upper", &a[0], "text")?.to_uppercase()))
}

pub fn lower(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Ok(Value::text(text_arg("lower", &a[0], "text")?.to_lowercase()))
}

pub fn split(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("split", &a[0], "text as its first argument")?;
    match a.get(1) {
        None | Some(Value::Null) => Ok(Value::list(s.split_whitespace().map(Value::text).collect())),
        Some(sep) => {
            let sep = text_arg("split", sep, "text as the separator")?;
            if sep.is_empty() {
                return Err(rt("E2011", "ValueError", "split() separator cannot be empty").boxed());
            }
            Ok(Value::list(s.split(sep.as_ref()).map(Value::text).collect()))
        }
    }
}

pub fn join(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("join", &a[0], "a list as its first argument")?;
    let sep: Rc<str> = match a.get(1) {
        None => Rc::from(""),
        Some(v) => text_arg("join", v, "text as the separator")?,
    };
    let mut parts: Vec<String> = Vec::new();
    for item in l.borrow().iter() {
        match item {
            Value::Text(t) => parts.push(t.to_string()),
            _ => return Err(type_err("join() needs a list of text values").help("convert items with text(x) first").boxed()),
        }
    }
    Ok(Value::text(parts.join(sep.as_ref())))
}

// ---- other ---------------------------------------------------------------------------------------

fn type_of(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Ok(Value::text(type_name(&a[0])))
}

pub fn fail(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Err(ArcanaError::new("E2100", to_text(&a[0], false), None).recoverable("UserError").boxed())
}

/// `assert(condition, message = "Assertion failed")`: raises a catchable E2101 when Off.
fn assert(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let ok = bool_arg("assert", &a[0], "a boolean (On or Off) condition")?;
    if ok {
        return Ok(Value::Null);
    }
    let msg = match a.get(1) {
        Some(m) => to_text(m, false),
        None => "Assertion failed".to_string(),
    };
    Err(rt("E2101", "AssertionError", msg).boxed())
}
