//! `list`: sorting, searching, slicing and changing lists.
//!
//! Rule of thumb: a **verb** (`sort`, `reverse`, `shuffle`, `remove`, `insert`,
//! `extend`, `clear`, `append`) changes the list **in place** and returns null;
//! `pop` removes and returns the last item; everything else leaves the list alone
//! and returns a new value (`sorted`, `reversed`, `unique`, `slice`, `concat`,
//! `first`, `last`, `sum`, `index_of`, `contains`).

use crate::ast::BinOp;
use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{self, bool_arg, int_arg, list_arg, need, register as reg, rt, type_err};
use crate::values::*;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    reg(env, "append", 2, Some(2), runtime::append);
    reg(env, "pop", 1, Some(1), runtime::pop);
    reg(env, "remove", 2, Some(2), remove);
    reg(env, "insert", 3, Some(3), insert);
    reg(env, "extend", 2, Some(2), extend);
    reg(env, "clear", 1, Some(1), |_, a| {
        list_arg("clear", &a[0], "a list")?.borrow_mut().clear();
        Ok(Value::Null)
    });
    reg(env, "contains", 2, Some(2), |_, a| {
        let l = list_arg("contains", &a[0], "a list")?;
        let found = l.borrow().iter().any(|x| arc_equal(x, &a[1]));
        Ok(Value::Bool(found))
    });
    reg(env, "sort", 1, Some(2), sort);
    reg(env, "sorted", 1, Some(2), sorted);
    reg(env, "reverse", 1, Some(1), |_, a| {
        list_arg("reverse", &a[0], "a list")?.borrow_mut().reverse();
        Ok(Value::Null)
    });
    reg(env, "reversed", 1, Some(1), |_, a| {
        let mut v = list_arg("reversed", &a[0], "a list")?.borrow().clone();
        v.reverse();
        Ok(Value::list(v))
    });
    reg(env, "first", 1, Some(1), |_, a| end_item("first", a, true));
    reg(env, "last", 1, Some(1), |_, a| end_item("last", a, false));
    reg(env, "unique", 1, Some(1), unique);
    reg(env, "index_of", 2, Some(2), |_, a| {
        let l = list_arg("index_of", &a[0], "a list")?;
        let pos = l.borrow().iter().position(|x| arc_equal(x, &a[1]));
        Ok(match pos {
            Some(i) => Value::int(i as i64),
            None => Value::Null,
        })
    });
    reg(env, "slice", 2, Some(3), slice);
    reg(env, "concat", 2, Some(2), |_, a| {
        let mut v = list_arg("concat", &a[0], "a list")?.borrow().clone();
        v.extend(list_arg("concat", &a[1], "a list")?.borrow().iter().cloned());
        Ok(Value::list(v))
    });
    reg(env, "sum", 1, Some(1), sum);
    reg(env, "shuffle", 1, Some(1), shuffle);
    reg(env, "choice", 1, Some(1), choice);
}

fn remove(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("remove", &a[0], "a list")?;
    let pos = l.borrow().iter().position(|x| arc_equal(x, &a[1]));
    match pos {
        Some(i) => {
            l.borrow_mut().remove(i);
            Ok(Value::Null)
        }
        None => Err(rt("E2006", "ValueError", format!("{} is not in the list", to_text(&a[1], true)))
            .help("check with: value in list")
            .boxed()),
    }
}

fn insert(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("insert", &a[0], "a list")?;
    let idx = int_arg("insert", &a[1], "an integer index")?;
    let len = l.borrow().len();
    match idx.to_usize() {
        Some(i) if i <= len => {
            l.borrow_mut().insert(i, a[2].clone());
            Ok(Value::Null)
        }
        _ => Err(rt("E2002", "IndexError", format!("Index {} is out of range for a list of length {}", idx, len))
            .help("insert at 0 to length (inclusive)")
            .boxed()),
    }
}

fn extend(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("extend", &a[0], "a list")?;
    let other = list_arg("extend", &a[1], "a list to add")?;
    let items: Vec<Value> = other.borrow().clone();
    l.borrow_mut().extend(items);
    Ok(Value::Null)
}

fn sorted_items(fname: &str, a: &[Value]) -> R<Vec<Value>> {
    let l = list_arg(fname, &a[0], "a list")?;
    let descending = match a.get(1) {
        Some(v) => bool_arg(fname, v, "On or Off for descending")?,
        None => false,
    };
    let mut items: Vec<Value> = l.borrow().clone();
    let all_num = items.iter().all(is_number);
    let all_text = items.iter().all(|i| matches!(i, Value::Text(_)));
    if !(all_num || all_text) {
        return Err(type_err(format!("{}() needs all numbers or all text", fname)).boxed());
    }
    items.sort_by(|x, y| cmp_values(x, y).unwrap_or(Ordering::Equal));
    if descending {
        items.reverse();
    }
    Ok(items)
}

fn sort(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let items = sorted_items("sort", a)?;
    *list_arg("sort", &a[0], "a list")?.borrow_mut() = items;
    Ok(Value::Null)
}

fn sorted(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    Ok(Value::list(sorted_items("sorted", a)?))
}

fn end_item(name: &str, a: &[Value], first: bool) -> R<Value> {
    let l = list_arg(name, &a[0], "a list")?;
    let l = l.borrow();
    let item = if first { l.first() } else { l.last() };
    match item {
        Some(v) => Ok(v.clone()),
        None => Err(rt("E2002", "IndexError", format!("{}() cannot be used on an empty list", name))
            .help("check length(list) > 0 first")
            .boxed()),
    }
}

fn unique(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("unique", &a[0], "a list")?;
    let items = l.borrow().clone();
    let mut out: Vec<Value> = Vec::new();
    if items.iter().all(|v| Key::from_value(v).is_some()) {
        let mut seen: HashSet<Key> = HashSet::new();
        for v in items {
            if seen.insert(Key::from_value(&v).expect("key")) {
                out.push(v);
            }
        }
    } else {
        for v in items {
            if !out.iter().any(|x| arc_equal(x, &v)) {
                out.push(v);
            }
        }
    }
    Ok(Value::list(out))
}

fn slice(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("slice", &a[0], "a list")?;
    let len = l.borrow().len();
    let s = int_arg("slice", &a[1], "an integer start index")?;
    let e = match a.get(2) {
        Some(v) => int_arg("slice", v, "an integer end index")?,
        None => crate::int::Int::Small(len as i64),
    };
    match (s.to_usize(), e.to_usize()) {
        (Some(su), Some(eu)) if su <= eu && eu <= len => Ok(Value::list(l.borrow()[su..eu].to_vec())),
        _ => Err(rt("E2002", "IndexError", format!("slice() range {}..{} is out of range for a list of length {}", s, e, len))
            .help("indexes run from 0 to the length; the end index is not included")
            .boxed()),
    }
}

fn sum(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("sum", &a[0], "a list of numbers")?;
    let items = l.borrow().clone();
    let mut acc = Value::int(0);
    for v in &items {
        need(is_number(v), "sum", "a list of numbers", v)?;
        acc = it.arith(BinOp::Add, &acc, v, None)?;
    }
    Ok(acc)
}

fn shuffle(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("shuffle", &a[0], "a list")?;
    let n = l.borrow().len();
    for i in (1..n).rev() {
        let j = (it.next_u64() % (i as u64 + 1)) as usize;
        l.borrow_mut().swap(i, j);
    }
    Ok(Value::Null)
}

fn choice(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    let l = list_arg("choice", &a[0], "a list")?;
    let n = l.borrow().len();
    if n == 0 {
        return Err(rt("E2002", "IndexError", "choice() cannot be used on an empty list").boxed());
    }
    let i = (it.next_u64() % n as u64) as usize;
    let v = l.borrow()[i].clone();
    Ok(v)
}
