//! Built-in spells.
//!
//! Built-ins raise errors without a span; the interpreter fills in the call site.

use crate::environment::Env;
use crate::errors::ArcanaError;
use crate::int::Int;
use crate::interpreter::{Interpreter, R};
use crate::values::*;
use std::cmp::Ordering;
use std::io::BufRead;
use std::io::Write;
use std::rc::Rc;

pub const BUILTIN_NAMES: [&str; 23] = [
    "ask", "ask_integer", "ask_decimal", "text", "integer", "decimal", "round", "abs", "min", "max", "length", "range",
    "append", "pop", "keys", "values", "copy", "upper", "lower", "split", "join", "type_of", "fail",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinId {
    Ask,
    AskInteger,
    AskDecimal,
    Text,
    Integer,
    Decimal,
    Round,
    Abs,
    Min,
    Max,
    Length,
    Range,
    Append,
    Pop,
    Keys,
    Values,
    Copy,
    Upper,
    Lower,
    Split,
    Join,
    TypeOf,
    Fail,
}

pub fn make_builtins(env: &Rc<Env>) {
    use BuiltinId::*;
    let table: [(&'static str, BuiltinId, usize, Option<usize>); 23] = [
        ("ask", Ask, 0, Some(1)),
        ("ask_integer", AskInteger, 0, Some(1)),
        ("ask_decimal", AskDecimal, 0, Some(1)),
        ("text", Text, 1, Some(1)),
        ("integer", Integer, 1, Some(1)),
        ("decimal", Decimal, 1, Some(1)),
        ("round", Round, 1, Some(1)),
        ("abs", Abs, 1, Some(1)),
        ("min", Min, 1, None),
        ("max", Max, 1, None),
        ("length", Length, 1, Some(1)),
        ("range", Range, 1, Some(3)),
        ("append", Append, 2, Some(2)),
        ("pop", Pop, 1, Some(1)),
        ("keys", Keys, 1, Some(1)),
        ("values", Values, 1, Some(1)),
        ("copy", Copy, 1, Some(1)),
        ("upper", Upper, 1, Some(1)),
        ("lower", Lower, 1, Some(1)),
        ("split", Split, 1, Some(2)),
        ("join", Join, 1, Some(2)),
        ("type_of", TypeOf, 1, Some(1)),
        ("fail", Fail, 1, Some(1)),
    ];
    for (name, id, lo, hi) in table {
        env.set_local(name, Value::Builtin(Rc::new(Builtin { name, id, min_args: lo, max_args: hi })));
    }
}

fn rt(code: &str, kind: &str, message: impl Into<String>) -> ArcanaError {
    ArcanaError::new(code, message, None).recoverable(kind)
}

fn type_err(message: impl Into<String>) -> ArcanaError {
    ArcanaError::new("E3003", message, None)
}

fn need(ok: bool, fname: &str, expected: &str, v: &Value) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err(type_err(format!("{}() needs {}, but got {}", fname, expected, type_name(v))).boxed())
    }
}

fn parse_integer(s: &str) -> Option<Int> {
    Int::parse(s.trim())
}

fn parse_decimal(s: &str) -> Option<f64> {
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

impl Interpreter {
    fn read_line(&mut self, prompt: &Value) -> R<String> {
        need(matches!(prompt, Value::Text(_)), "ask", "text for the prompt", prompt)?;
        if let Value::Text(p) = prompt {
            let _ = self.out.write_all(p.as_bytes());
        }
        let _ = self.out.flush();
        let mut buf = Vec::new();
        let n = self.inp.read_until(b'\n', &mut buf).unwrap_or(0);
        if n == 0 {
            return Err(rt("E2010", "InputError", "No input available (end of input reached)")
                .help("provide input on standard input, or catch the error with try/catch")
                .boxed());
        }
        let line = String::from_utf8_lossy(&buf).into_owned();
        Ok(line.trim_end_matches('\n').trim_end_matches('\r').to_string())
    }

    pub(crate) fn call_builtin(&mut self, id: BuiltinId, args: Vec<Value>) -> R<Value> {
        use BuiltinId::*;
        let empty = Value::text("");
        match id {
            Ask => {
                let p = args.first().unwrap_or(&empty).clone();
                Ok(Value::text(self.read_line(&p)?))
            }
            AskInteger => {
                let p = args.first().unwrap_or(&empty).clone();
                let line = self.read_line(&p)?;
                match parse_integer(&line) {
                    Some(v) => Ok(Value::Int(v)),
                    None => Err(rt("E2010", "InputError", format!("{} is not a valid integer", quote(&line)))
                        .help("enter a whole number, such as 42")
                        .boxed()),
                }
            }
            AskDecimal => {
                let p = args.first().unwrap_or(&empty).clone();
                let line = self.read_line(&p)?;
                match parse_decimal(&line) {
                    Some(v) => Ok(Value::Dec(v)),
                    None => Err(rt("E2010", "InputError", format!("{} is not a valid decimal", quote(&line)))
                        .help("enter a number, such as 3.14 or 42")
                        .boxed()),
                }
            }
            Text => Ok(Value::text(to_text(&args[0], false))),
            Integer => match &args[0] {
                Value::Int(_) => Ok(args[0].clone()),
                Value::Dec(x) => {
                    if is_whole(*x) {
                        Ok(Value::Int(Int::from_f64(*x).expect("finite")))
                    } else {
                        Err(rt(
                            "E2011",
                            "ConversionError",
                            format!(
                                "Cannot convert decimal {} to integer without losing its fraction",
                                to_text(&args[0], false)
                            ),
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
            },
            Decimal => match &args[0] {
                Value::Dec(_) => Ok(args[0].clone()),
                Value::Int(i) => match i.to_f64() {
                    Some(f) => Ok(Value::Dec(f)),
                    None => Err(rt("E2008", "MathError", "Number is too large to be a decimal").boxed()),
                },
                Value::Text(s) => match parse_decimal(s) {
                    Some(v) => Ok(Value::Dec(v)),
                    None => Err(rt("E2011", "ConversionError", format!("Cannot convert {} to decimal", quote(s)))
                        .help("the text must look like a number, e.g. \"3.14\"")
                        .boxed()),
                },
                other => Err(type_err(format!("decimal() cannot convert {}", type_name(other))).boxed()),
            },
            Round => {
                need(is_number(&args[0]), "round", "a number", &args[0])?;
                match &args[0] {
                    Value::Dec(x) => {
                        let r = (x.abs() + 0.5).floor().copysign(*x);
                        Ok(Value::Int(Int::from_f64(r).expect("finite")))
                    }
                    other => Ok(other.clone()),
                }
            }
            Abs => {
                need(is_number(&args[0]), "abs", "a number", &args[0])?;
                match &args[0] {
                    Value::Int(i) => Ok(Value::Int(i.abs())),
                    Value::Dec(f) => Ok(Value::Dec(f.abs())),
                    _ => unreachable!(),
                }
            }
            Min => extremum("min", true, &args),
            Max => extremum("max", false, &args),
            Length => match &args[0] {
                Value::Text(s) => Ok(Value::int(s.chars().count() as i64)),
                Value::List(l) => Ok(Value::int(l.borrow().len() as i64)),
                Value::Map(m) => Ok(Value::int(m.borrow().len() as i64)),
                Value::Range(r) => Ok(Value::Int(r.len())),
                other => Err(type_err(format!(
                    "length() needs text, a list, a map or a range, but got {}",
                    type_name(other)
                ))
                .boxed()),
            },
            Range => {
                let mut ints = Vec::new();
                for a in &args {
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
            Append => {
                need(matches!(args[0], Value::List(_)), "append", "a list as its first argument", &args[0])?;
                if let Value::List(l) = &args[0] {
                    l.borrow_mut().push(args[1].clone());
                }
                Ok(Value::Null)
            }
            Pop => {
                need(matches!(args[0], Value::List(_)), "pop", "a list", &args[0])?;
                if let Value::List(l) = &args[0] {
                    let popped = l.borrow_mut().pop();
                    return match popped {
                        Some(v) => Ok(v),
                        None => Err(rt("E2002", "IndexError", "Cannot pop from an empty list")
                            .help("check length(list) > 0 first")
                            .boxed()),
                    };
                }
                unreachable!()
            }
            Keys => {
                need(matches!(args[0], Value::Map(_)), "keys", "a map", &args[0])?;
                if let Value::Map(m) = &args[0] {
                    return Ok(Value::list(m.borrow().iter().map(|(k, _)| k.to_value()).collect()));
                }
                unreachable!()
            }
            Values => {
                need(matches!(args[0], Value::Map(_)), "values", "a map", &args[0])?;
                if let Value::Map(m) = &args[0] {
                    return Ok(Value::list(m.borrow().iter().map(|(_, v)| v.clone()).collect()));
                }
                unreachable!()
            }
            Copy => match &args[0] {
                Value::List(l) => Ok(Value::list(l.borrow().clone())),
                Value::Map(m) => Ok(Value::map(m.borrow().clone())),
                other => Ok(other.clone()),
            },
            Upper => {
                need(matches!(args[0], Value::Text(_)), "upper", "text", &args[0])?;
                if let Value::Text(s) = &args[0] {
                    return Ok(Value::text(s.to_uppercase()));
                }
                unreachable!()
            }
            Lower => {
                need(matches!(args[0], Value::Text(_)), "lower", "text", &args[0])?;
                if let Value::Text(s) = &args[0] {
                    return Ok(Value::text(s.to_lowercase()));
                }
                unreachable!()
            }
            Split => {
                need(matches!(args[0], Value::Text(_)), "split", "text as its first argument", &args[0])?;
                let s = match &args[0] {
                    Value::Text(s) => s.clone(),
                    _ => unreachable!(),
                };
                match args.get(1) {
                    None | Some(Value::Null) => Ok(Value::list(s.split_whitespace().map(Value::text).collect())),
                    Some(sep) => {
                        need(matches!(sep, Value::Text(_)), "split", "text as the separator", sep)?;
                        let sep = match sep {
                            Value::Text(t) => t.clone(),
                            _ => unreachable!(),
                        };
                        if sep.is_empty() {
                            return Err(rt("E2011", "ValueError", "split() separator cannot be empty").boxed());
                        }
                        Ok(Value::list(s.split(sep.as_ref()).map(Value::text).collect()))
                    }
                }
            }
            Join => {
                need(matches!(args[0], Value::List(_)), "join", "a list as its first argument", &args[0])?;
                let sep = match args.get(1) {
                    None => Rc::from(""),
                    Some(Value::Text(t)) => t.clone(),
                    Some(other) => {
                        need(false, "join", "text as the separator", other)?;
                        unreachable!()
                    }
                };
                if let Value::List(l) = &args[0] {
                    let mut parts: Vec<String> = Vec::new();
                    for item in l.borrow().iter() {
                        match item {
                            Value::Text(t) => parts.push(t.to_string()),
                            _ => {
                                return Err(type_err("join() needs a list of text values")
                                    .help("convert items with text(x) first")
                                    .boxed())
                            }
                        }
                    }
                    return Ok(Value::text(parts.join(sep.as_ref())));
                }
                unreachable!()
            }
            TypeOf => Ok(Value::text(type_name(&args[0]))),
            Fail => Err(ArcanaError::new("E2100", to_text(&args[0], false), None).recoverable("UserError").boxed()),
        }
    }
}
