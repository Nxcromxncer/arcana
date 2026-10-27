//! `convert`: explicit conversions between types, number bases and formatting.

use crate::environment::Env;
use crate::int::Int;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{int_arg, math_err, parse_decimal, parse_integer, register as reg, rt, small_int_arg, text_arg, type_err};
use crate::values::*;
use std::rc::Rc;

const MAX_LIST: usize = 10_000_000;

pub fn register(env: &Rc<Env>) {
    reg(env, "to_boolean", 1, Some(1), to_boolean);
    reg(env, "to_list", 1, Some(1), to_list);
    reg(env, "to_map", 1, Some(1), to_map);
    reg(env, "to_binary", 1, Some(1), |_, a| radix("to_binary", a, 2, "0b"));
    reg(env, "to_octal", 1, Some(1), |_, a| radix("to_octal", a, 8, "0o"));
    reg(env, "to_hex", 1, Some(1), |_, a| radix("to_hex", a, 16, "0x"));
    reg(env, "from_binary", 1, Some(1), |_, a| parse_radix("from_binary", a, 2, "0b"));
    reg(env, "from_octal", 1, Some(1), |_, a| parse_radix("from_octal", a, 8, "0o"));
    reg(env, "from_hex", 1, Some(1), |_, a| parse_radix("from_hex", a, 16, "0x"));
    reg(env, "parse_number", 1, Some(1), parse_number);
    reg(env, "format_number", 2, Some(2), format_number);
}

fn to_boolean(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::Bool(_) => Ok(a[0].clone()),
        Value::Null => Ok(Value::Bool(false)),
        Value::Int(i) => Ok(Value::Bool(!i.is_zero())),
        Value::Dec(f) => Ok(Value::Bool(*f != 0.0)),
        Value::Text(s) => match s.trim().to_lowercase().as_str() {
            "on" | "true" | "yes" | "1" => Ok(Value::Bool(true)),
            "off" | "false" | "no" | "0" => Ok(Value::Bool(false)),
            _ => Err(rt("E2011", "ConversionError", format!("Cannot convert {} to a boolean", quote(s)))
                .help("use text such as \"On\", \"Off\", \"yes\", \"no\", \"true\", \"false\", \"1\" or \"0\"")
                .boxed()),
        },
        other => Err(type_err(format!("to_boolean() cannot convert {}", type_name(other)))
            .help("compare explicitly instead, e.g. length(x) > 0")
            .boxed()),
    }
}

/// Always returns a new list: text -> characters, map -> [key, value] pairs, range -> integers.
fn to_list(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::List(l) => Ok(Value::list(l.borrow().clone())),
        Value::Text(s) => Ok(Value::list(s.chars().map(|c| Value::text(c.to_string())).collect())),
        Value::Map(m) => Ok(Value::list(
            m.borrow().iter().map(|(k, v)| Value::list(vec![k.to_value(), v.clone()])).collect(),
        )),
        Value::Range(r) => {
            let len = r.len();
            match len.to_usize() {
                Some(n) if n <= MAX_LIST => {
                    let mut out = Vec::with_capacity(n);
                    let mut cur = r.start.clone();
                    for _ in 0..n {
                        out.push(Value::Int(cur.clone()));
                        cur = cur.add(&r.step);
                    }
                    Ok(Value::list(out))
                }
                _ => Err(math_err("Numeric overflow: the result is too large").boxed()),
            }
        }
        other => Err(type_err(format!("to_list() cannot convert {}", type_name(other)))
            .help("to_list() accepts a list, text, map or range")
            .boxed()),
    }
}

/// The reverse of `to_list(map)`: a list of [key, value] pairs becomes a map.
fn to_map(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    match &a[0] {
        Value::Map(m) => Ok(Value::map(m.borrow().clone())),
        Value::List(l) => {
            let mut out = ArcMap::new();
            for item in l.borrow().iter() {
                let pair = match item {
                    Value::List(p) if p.borrow().len() == 2 => p.borrow().clone(),
                    _ => return Err(type_err("to_map() needs a list of [key, value] pairs").boxed()),
                };
                match Key::from_value(&pair[0]) {
                    Some(k) => out.insert(k, pair[1].clone()),
                    None => {
                        return Err(type_err(format!("Map keys must be text or integer, but got {}", type_name(&pair[0]))).boxed())
                    }
                }
            }
            Ok(Value::map(out))
        }
        other => Err(type_err(format!("to_map() cannot convert {}", type_name(other))).boxed()),
    }
}

fn radix(name: &str, a: &[Value], base: u32, prefix: &str) -> R<Value> {
    let i = int_arg(name, &a[0], "an integer")?;
    let digits = i.abs().big().to_str_radix(base);
    Ok(Value::text(format!("{}{}{}", if i.is_neg() { "-" } else { "" }, prefix, digits)))
}

fn parse_radix(name: &str, a: &[Value], base: u32, prefix: &str) -> R<Value> {
    let s = text_arg(name, &a[0], "text")?;
    let t = s.trim();
    let (neg, body) = match t.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let body = body.strip_prefix(prefix).or_else(|| body.strip_prefix(&prefix.to_uppercase())).unwrap_or(body);
    match num_bigint::BigInt::parse_bytes(body.as_bytes(), base) {
        Some(v) if !body.is_empty() && !body.starts_with(['+', '-']) => Ok(Value::Int(Int::from_big(if neg { -v } else { v }))),
        _ => Err(rt("E2011", "ConversionError", format!("Cannot convert {} to a base-{} integer", quote(&s), base))
            .help(format!("the text may start with {} and must contain only valid base-{} digits", prefix, base))
            .boxed()),
    }
}

/// Like `to_number`, but returns null instead of raising an error.
fn parse_number(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("parse_number", &a[0], "text")?;
    if let Some(i) = parse_integer(&s) {
        return Ok(Value::Int(i));
    }
    Ok(match parse_decimal(&s) {
        Some(f) => Value::Dec(f),
        None => Value::Null,
    })
}

/// `format_number(x, digits)`: text with exactly `digits` decimal places.
fn format_number(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let d = small_int_arg("format_number", &a[1], "an integer number of digits")?;
    if !(0..=100).contains(&d) {
        return Err(rt("E2006", "ValueError", "format_number() digits must be between 0 and 100").boxed());
    }
    let d = d as usize;
    match &a[0] {
        Value::Int(i) => {
            let mut s = i.to_string();
            if d > 0 {
                s.push('.');
                s.push_str(&"0".repeat(d));
            }
            Ok(Value::text(s))
        }
        Value::Dec(f) => Ok(Value::text(format!("{:.*}", d, f))),
        other => Err(type_err(format!("format_number() needs a number, but got {}", type_name(other))).boxed()),
    }
}
