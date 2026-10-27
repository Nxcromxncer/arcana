//! Runtime values and value-level helpers.
//!
//! Arcana type   Rust representation
//! integer       `Int` (arbitrary precision)
//! decimal       `f64`
//! text          `Rc<str>`
//! boolean       `bool` (written On / Off)
//! null          `Value::Null`
//! list          `Rc<RefCell<Vec<Value>>>`   (shared reference semantics)
//! map           `Rc<RefCell<ArcMap>>`       (insertion ordered; keys: text or integer)
//! range         `RangeVal`
//! spell         `Spell` | `Builtin`
//! grimoire      `ModuleValue`

use crate::ast::{Param, Stmt};
use crate::environment::{Env, FxBuild};
use crate::int::Int;
use crate::runtime::NativeFn;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

pub struct ModuleInfo {
    pub filename: String,
    pub source: Arc<str>,
    pub base_dir: PathBuf,
}

pub struct Spell {
    pub name: String,
    pub params: Rc<Vec<Param>>,
    pub body: Rc<Vec<Stmt>>,
    pub closure: Rc<Env>,
    pub module: Rc<ModuleInfo>,
}

impl Spell {
    pub fn min_args(&self) -> usize {
        self.params.iter().filter(|p| p.default.is_none()).count()
    }
}

pub struct Builtin {
    pub name: &'static str,
    pub func: NativeFn,
    pub min_args: usize,
    pub max_args: Option<usize>,
}

pub struct ModuleValue {
    pub name: String,
    pub env: Rc<Env>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeVal {
    pub start: Int,
    pub stop: Int,
    pub step: Int,
}

impl RangeVal {
    pub fn len(&self) -> Int {
        let one = Int::Small(1);
        if self.step.is_pos() {
            if self.start >= self.stop {
                Int::Small(0)
            } else {
                self.stop.sub(&self.start).sub(&one).floor_div(&self.step).add(&one)
            }
        } else if self.start <= self.stop {
            Int::Small(0)
        } else {
            self.start.sub(&self.stop).sub(&one).floor_div(&self.step.neg()).add(&one)
        }
    }

    pub fn get(&self, idx: &Int) -> Int {
        self.start.add(&idx.mul(&self.step))
    }

    pub fn contains(&self, item: &Int) -> bool {
        if self.step.is_pos() {
            *item >= self.start && *item < self.stop && item.sub(&self.start).modulo(&self.step).is_zero()
        } else {
            *item <= self.start && *item > self.stop && self.start.sub(item).modulo(&self.step.neg()).is_zero()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Text(Rc<str>),
    Int(Int),
}

impl Key {
    pub fn from_value(v: &Value) -> Option<Key> {
        match v {
            Value::Text(s) => Some(Key::Text(s.clone())),
            Value::Int(i) => Some(Key::Int(i.clone())),
            _ => None,
        }
    }

    pub fn to_value(&self) -> Value {
        match self {
            Key::Text(s) => Value::Text(s.clone()),
            Key::Int(i) => Value::Int(i.clone()),
        }
    }
}

/// Insertion-ordered map (no removal is needed by the language).
#[derive(Clone, Default)]
pub struct ArcMap {
    entries: Vec<(Key, Value)>,
    index: HashMap<Key, usize, FxBuild>,
}

impl ArcMap {
    pub fn new() -> ArcMap {
        ArcMap::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, k: &Key) -> Option<&Value> {
        self.index.get(k).map(|&i| &self.entries[i].1)
    }

    pub fn contains_key(&self, k: &Key) -> bool {
        self.index.contains_key(k)
    }

    pub fn insert(&mut self, k: Key, v: Value) {
        match self.index.get(&k) {
            Some(&i) => self.entries[i].1 = v,
            None => {
                self.index.insert(k.clone(), self.entries.len());
                self.entries.push((k, v));
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Key, Value)> {
        self.entries.iter()
    }
}

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(Int),
    Dec(f64),
    Text(Rc<str>),
    List(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<ArcMap>>),
    Range(Rc<RangeVal>),
    Spell(Rc<Spell>),
    Builtin(Rc<Builtin>),
    Module(Rc<ModuleValue>),
}

impl Value {
    pub fn text(s: impl AsRef<str>) -> Value {
        Value::Text(Rc::from(s.as_ref()))
    }

    pub fn list(items: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(items)))
    }

    pub fn map(m: ArcMap) -> Value {
        Value::Map(Rc::new(RefCell::new(m)))
    }

    pub fn int(i: i64) -> Value {
        Value::Int(Int::Small(i))
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", to_text(self, true))
    }
}

pub fn is_number(v: &Value) -> bool {
    matches!(v, Value::Int(_) | Value::Dec(_))
}

pub fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Int(_) => "integer",
        Value::Dec(_) => "decimal",
        Value::Text(_) => "text",
        Value::List(_) => "list",
        Value::Map(_) => "map",
        Value::Range(_) => "range",
        Value::Spell(_) | Value::Builtin(_) => "spell",
        Value::Module(_) => "grimoire",
    }
}

/// Order two numbers or two texts; `None` for any other pair.
pub fn cmp_values(a: &Value, b: &Value) -> Option<Ordering> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(x.cmp(y)),
        (Value::Int(x), Value::Dec(y)) => Some(x.cmp_f64(*y)),
        (Value::Dec(x), Value::Int(y)) => Some(y.cmp_f64(*x).reverse()),
        (Value::Dec(x), Value::Dec(y)) => x.partial_cmp(y),
        (Value::Text(x), Value::Text(y)) => Some(x.as_ref().cmp(y.as_ref())),
        _ => None,
    }
}

const MAX_EQ_DEPTH: usize = 100_000;

/// Equality without implicit coercion (integer and decimal compare by value).
pub fn arc_equal(a: &Value, b: &Value) -> bool {
    eq_depth(a, b, 0)
}

fn eq_depth(a: &Value, b: &Value, depth: usize) -> bool {
    if depth > MAX_EQ_DEPTH {
        return false;
    }
    match (a, b) {
        (Value::Int(_) | Value::Dec(_), Value::Int(_) | Value::Dec(_)) => cmp_values(a, b) == Some(Ordering::Equal),
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Null, Value::Null) => true,
        (Value::Text(x), Value::Text(y)) => x == y,
        (Value::List(x), Value::List(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let (x, y) = (x.borrow(), y.borrow());
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| eq_depth(p, q, depth + 1))
        }
        (Value::Map(x), Value::Map(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let (x, y) = (x.borrow(), y.borrow());
            x.len() == y.len()
                && x.iter().all(|(k, v)| match y.get(k) {
                    Some(w) => eq_depth(v, w, depth + 1),
                    None => false,
                })
        }
        (Value::Range(x), Value::Range(y)) => {
            let (lx, ly) = (x.len(), y.len());
            if lx != ly {
                return false;
            }
            if lx.is_zero() {
                return true;
            }
            x.start == y.start && (lx == Int::Small(1) || x.step == y.step)
        }
        (Value::Spell(x), Value::Spell(y)) => Rc::ptr_eq(x, y),
        (Value::Builtin(x), Value::Builtin(y)) => Rc::ptr_eq(x, y),
        (Value::Module(x), Value::Module(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

pub fn quote(s: &str) -> String {
    let out = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\t', "\\t");
    format!("\"{}\"", out)
}

/// Python-compatible `repr(float)` (shortest round-trip digits).
pub fn float_repr(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return if x > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if x == 0.0 {
        return if x.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let s = format!("{:e}", x.abs());
    let (mant, exp) = s.split_once('e').expect("exponent form");
    let exp: i32 = exp.parse().expect("exponent");
    let digits: String = mant.chars().filter(|c| *c != '.').collect();
    let decpt = exp + 1;
    let sign = if x < 0.0 { "-" } else { "" };
    let body = if decpt > 16 || decpt < -3 {
        let mut m = digits[..1].to_string();
        if digits.len() > 1 {
            m.push('.');
            m.push_str(&digits[1..]);
        }
        let e = decpt - 1;
        format!("{}e{}{:02}", m, if e < 0 { '-' } else { '+' }, e.abs())
    } else if decpt <= 0 {
        format!("0.{}{}", "0".repeat((-decpt) as usize), digits)
    } else if decpt as usize >= digits.len() {
        format!("{}{}.0", digits, "0".repeat(decpt as usize - digits.len()))
    } else {
        format!("{}.{}", &digits[..decpt as usize], &digits[decpt as usize..])
    };
    format!("{}{}", sign, body)
}

/// Convert any value to the text shown by `cast` / `text()`.
pub fn to_text(v: &Value, nested: bool) -> String {
    let mut seen = Vec::new();
    text_of(v, nested, &mut seen)
}

fn text_of(v: &Value, nested: bool, seen: &mut Vec<usize>) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => (if *b { "On" } else { "Off" }).into(),
        Value::Int(i) => i.to_string(),
        Value::Dec(f) => float_repr(*f),
        Value::Text(s) => {
            if nested {
                quote(s)
            } else {
                s.to_string()
            }
        }
        Value::List(l) => {
            let id = Rc::as_ptr(l) as *const u8 as usize;
            if seen.contains(&id) {
                return "[...]".into();
            }
            seen.push(id);
            let parts: Vec<String> = l.borrow().iter().map(|x| text_of(x, true, seen)).collect();
            seen.pop();
            format!("[{}]", parts.join(", "))
        }
        Value::Map(m) => {
            let id = Rc::as_ptr(m) as *const u8 as usize;
            if seen.contains(&id) {
                return "{...}".into();
            }
            seen.push(id);
            let parts: Vec<String> = m
                .borrow()
                .iter()
                .map(|(k, x)| format!("{}: {}", text_of(&k.to_value(), true, seen), text_of(x, true, seen)))
                .collect();
            seen.pop();
            format!("{{{}}}", parts.join(", "))
        }
        Value::Range(r) => format!("range({}, {}, {})", r.start, r.stop, r.step),
        Value::Spell(s) => format!("<spell {}>", s.name),
        Value::Builtin(b) => format!("<built-in spell {}>", b.name),
        Value::Module(m) => format!("<grimoire {}>", m.name),
    }
}
