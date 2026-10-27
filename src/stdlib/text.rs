//! `text`: searching, trimming, replacing, slicing and padding text.
//! Indexes count characters (not bytes) and start at 0.

use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{self, int_arg, math_err, register as reg, rt, small_int_arg, text_arg};
use crate::values::*;
use std::rc::Rc;

const MAX_TEXT: usize = 50_000_000;

pub fn register(env: &Rc<Env>) {
    reg(env, "upper", 1, Some(1), runtime::upper);
    reg(env, "lower", 1, Some(1), runtime::lower);
    reg(env, "split", 1, Some(2), runtime::split);
    reg(env, "join", 1, Some(2), runtime::join);
    reg(env, "trim", 1, Some(1), |_, a| Ok(Value::text(text_arg("trim", &a[0], "text")?.trim())));
    reg(env, "trim_start", 1, Some(1), |_, a| Ok(Value::text(text_arg("trim_start", &a[0], "text")?.trim_start())));
    reg(env, "trim_end", 1, Some(1), |_, a| Ok(Value::text(text_arg("trim_end", &a[0], "text")?.trim_end())));
    reg(env, "contains", 2, Some(2), |_, a| {
        let (s, p) = (text_arg("contains", &a[0], "text")?, text_arg("contains", &a[1], "text to look for")?);
        Ok(Value::Bool(s.contains(p.as_ref())))
    });
    reg(env, "starts_with", 2, Some(2), |_, a| {
        let (s, p) = (text_arg("starts_with", &a[0], "text")?, text_arg("starts_with", &a[1], "text to look for")?);
        Ok(Value::Bool(s.starts_with(p.as_ref())))
    });
    reg(env, "ends_with", 2, Some(2), |_, a| {
        let (s, p) = (text_arg("ends_with", &a[0], "text")?, text_arg("ends_with", &a[1], "text to look for")?);
        Ok(Value::Bool(s.ends_with(p.as_ref())))
    });
    reg(env, "replace", 3, Some(3), replace);
    reg(env, "substring", 2, Some(3), substring);
    reg(env, "repeat_text", 2, Some(2), repeat_text);
    reg(env, "char_at", 2, Some(2), char_at);
    reg(env, "find", 2, Some(2), find);
    reg(env, "lines", 1, Some(1), |_, a| {
        Ok(Value::list(text_arg("lines", &a[0], "text")?.lines().map(Value::text).collect()))
    });
    reg(env, "chars", 1, Some(1), |_, a| {
        Ok(Value::list(text_arg("chars", &a[0], "text")?.chars().map(|c| Value::text(c.to_string())).collect()))
    });
    reg(env, "pad_left", 2, Some(3), |_, a| pad("pad_left", a, true));
    reg(env, "pad_right", 2, Some(3), |_, a| pad("pad_right", a, false));
    reg(env, "reverse_text", 1, Some(1), |_, a| {
        Ok(Value::text(text_arg("reverse_text", &a[0], "text")?.chars().rev().collect::<String>()))
    });
    reg(env, "code_of", 1, Some(1), |_, a| {
        let s = text_arg("code_of", &a[0], "a one-character text")?;
        let mut it = s.chars();
        match (it.next(), it.next()) {
            (Some(c), None) => Ok(Value::int(c as i64)),
            _ => Err(rt("E2006", "ValueError", "code_of() needs text with exactly one character").boxed()),
        }
    });
    reg(env, "from_code", 1, Some(1), |_, a| {
        let n = small_int_arg("from_code", &a[0], "an integer code point")?;
        match u32::try_from(n).ok().and_then(char::from_u32) {
            Some(c) => Ok(Value::text(c.to_string())),
            None => Err(rt("E2006", "ValueError", format!("{} is not a valid Unicode code point", n)).boxed()),
        }
    });
}

fn replace(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("replace", &a[0], "text")?;
    let old = text_arg("replace", &a[1], "text to replace")?;
    let new = text_arg("replace", &a[2], "replacement text")?;
    if old.is_empty() {
        return Err(rt("E2006", "ValueError", "replace() cannot search for empty text").boxed());
    }
    Ok(Value::text(s.replace(old.as_ref(), new.as_ref())))
}

fn char_bounds(fname: &str, len: usize, start: &Value, end: Option<&Value>) -> R<(usize, usize)> {
    let s = int_arg(fname, start, "an integer start index")?;
    let e = match end {
        Some(v) => int_arg(fname, v, "an integer end index")?,
        None => Int_from(len),
    };
    let (su, eu) = (s.to_usize(), e.to_usize());
    match (su, eu) {
        (Some(su), Some(eu)) if su <= eu && eu <= len => Ok((su, eu)),
        _ => Err(rt("E2002", "IndexError", format!("{}() range {}..{} is out of range for a length of {}", fname, s, e, len))
            .help("indexes run from 0 to the length; the end index is not included")
            .boxed()),
    }
}

#[allow(non_snake_case)]
fn Int_from(n: usize) -> crate::int::Int {
    crate::int::Int::Small(n as i64)
}

fn substring(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("substring", &a[0], "text")?;
    let len = s.chars().count();
    let (st, en) = char_bounds("substring", len, &a[1], a.get(2))?;
    Ok(Value::text(s.chars().skip(st).take(en - st).collect::<String>()))
}

fn repeat_text(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("repeat_text", &a[0], "text")?;
    let n = small_int_arg("repeat_text", &a[1], "an integer count")?;
    if n < 0 {
        return Err(rt("E2006", "ValueError", "repeat_text() count cannot be negative").boxed());
    }
    if (s.len() as u128) * (n as u128) > MAX_TEXT as u128 {
        return Err(math_err("Numeric overflow: the result is too large").boxed());
    }
    Ok(Value::text(s.repeat(n as usize)))
}

fn char_at(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("char_at", &a[0], "text")?;
    let i = int_arg("char_at", &a[1], "an integer index")?;
    let len = s.chars().count();
    match i.to_usize() {
        Some(u) if u < len => Ok(Value::text(s.chars().nth(u).expect("checked").to_string())),
        _ => Err(rt("E2002", "IndexError", format!("Index {} is out of range for a text of length {}", i, len))
            .help("valid indexes run from 0 to length - 1")
            .boxed()),
    }
}

/// Character index of the first occurrence, or null.
fn find(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("find", &a[0], "text")?;
    let p = text_arg("find", &a[1], "text to look for")?;
    Ok(match s.find(p.as_ref()) {
        Some(byte) => Value::int(s[..byte].chars().count() as i64),
        None => Value::Null,
    })
}

fn pad(name: &str, a: &[Value], left: bool) -> R<Value> {
    let s = text_arg(name, &a[0], "text")?;
    let width = small_int_arg(name, &a[1], "an integer width")?;
    let fill = match a.get(2) {
        None => ' ',
        Some(v) => {
            let f = text_arg(name, v, "a one-character fill text")?;
            let mut it = f.chars();
            match (it.next(), it.next()) {
                (Some(c), None) => c,
                _ => return Err(rt("E2006", "ValueError", format!("{}() fill must be exactly one character", name)).boxed()),
            }
        }
    };
    let len = s.chars().count();
    if width < 0 || width as usize > MAX_TEXT {
        return Err(rt("E2006", "ValueError", format!("{}() width is out of range", name)).boxed());
    }
    let missing = (width as usize).saturating_sub(len);
    let padding: String = std::iter::repeat(fill).take(missing).collect();
    Ok(Value::text(if left { format!("{}{}", padding, s) } else { format!("{}{}", s, padding) }))
}
