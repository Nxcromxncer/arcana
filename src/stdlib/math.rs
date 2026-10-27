//! `math`: numbers, rounding, powers, trigonometry, logarithms, random numbers.

use crate::ast::BinOp;
use crate::environment::Env;
use crate::int::Int;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{self, finite, float_arg, int_arg, math_err, need, register as reg, rt, small_int_arg};
use crate::values::*;
use std::cmp::Ordering;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    env.set_local("pi", Value::Dec(std::f64::consts::PI));
    env.set_local("e", Value::Dec(std::f64::consts::E));
    env.set_local("tau", Value::Dec(std::f64::consts::TAU));
    reg(env, "abs", 1, Some(1), runtime::abs);
    reg(env, "min", 1, None, runtime::min);
    reg(env, "max", 1, None, runtime::max);
    reg(env, "round", 1, Some(2), |it, a| {
        // same behaviour as the global round()
        let f = runtime::GLOBALS.iter().find(|e| e.0 == "round").expect("round").3;
        f(it, a)
    });
    reg(env, "sqrt", 1, Some(1), sqrt);
    reg(env, "power", 2, Some(2), power);
    reg(env, "floor", 1, Some(1), |_, a| whole("floor", a, f64::floor));
    reg(env, "ceil", 1, Some(1), |_, a| whole("ceil", a, f64::ceil));
    reg(env, "trunc", 1, Some(1), |_, a| whole("trunc", a, f64::trunc));
    reg(env, "clamp", 3, Some(3), clamp);
    reg(env, "sign", 1, Some(1), sign);
    reg(env, "random", 0, Some(0), |it, _| Ok(Value::Dec(it.next_f64())));
    reg(env, "random_int", 2, Some(2), random_int);
    reg(env, "random_seed", 1, Some(1), random_seed);
    reg(env, "sin", 1, Some(1), |_, a| unary("sin", a, f64::sin));
    reg(env, "cos", 1, Some(1), |_, a| unary("cos", a, f64::cos));
    reg(env, "tan", 1, Some(1), |_, a| unary("tan", a, f64::tan));
    reg(env, "asin", 1, Some(1), |_, a| domain("asin", a, -1.0, 1.0, f64::asin));
    reg(env, "acos", 1, Some(1), |_, a| domain("acos", a, -1.0, 1.0, f64::acos));
    reg(env, "atan", 1, Some(1), |_, a| unary("atan", a, f64::atan));
    reg(env, "atan2", 2, Some(2), |_, a| {
        let y = float_arg("atan2", &a[0], "a number")?;
        let x = float_arg("atan2", &a[1], "a number")?;
        finite(y.atan2(x))
    });
    reg(env, "exp", 1, Some(1), |_, a| unary("exp", a, f64::exp));
    reg(env, "log", 1, Some(2), log);
    reg(env, "log2", 1, Some(1), |_, a| positive("log2", a, f64::log2));
    reg(env, "log10", 1, Some(1), |_, a| positive("log10", a, f64::log10));
    reg(env, "hypot", 2, Some(2), |_, a| {
        let x = float_arg("hypot", &a[0], "a number")?;
        let y = float_arg("hypot", &a[1], "a number")?;
        finite(x.hypot(y))
    });
    reg(env, "degrees", 1, Some(1), |_, a| unary("degrees", a, f64::to_degrees));
    reg(env, "radians", 1, Some(1), |_, a| unary("radians", a, f64::to_radians));
    reg(env, "gcd", 2, Some(2), |_, a| {
        let (x, y) = (int_arg("gcd", &a[0], "an integer")?, int_arg("gcd", &a[1], "an integer")?);
        Ok(Value::Int(Int::from_big(num_integer::Integer::gcd(&x.big(), &y.big()))))
    });
    reg(env, "lcm", 2, Some(2), |_, a| {
        let (x, y) = (int_arg("lcm", &a[0], "an integer")?, int_arg("lcm", &a[1], "an integer")?);
        Ok(Value::Int(Int::from_big(num_integer::Integer::lcm(&x.big(), &y.big()))))
    });
    reg(env, "factorial", 1, Some(1), factorial);
}

fn unary(name: &str, a: &[Value], f: fn(f64) -> f64) -> R<Value> {
    finite(f(float_arg(name, &a[0], "a number")?))
}

fn domain(name: &str, a: &[Value], lo: f64, hi: f64, f: fn(f64) -> f64) -> R<Value> {
    let x = float_arg(name, &a[0], "a number")?;
    if x < lo || x > hi {
        return Err(math_err("The result is not a real number")
            .help(format!("{}() needs a value between {} and {}", name, lo, hi))
            .boxed());
    }
    finite(f(x))
}

fn positive(name: &str, a: &[Value], f: fn(f64) -> f64) -> R<Value> {
    let x = float_arg(name, &a[0], "a number")?;
    if x <= 0.0 {
        return Err(math_err("The result is not a real number")
            .help(format!("{}() needs a value greater than 0", name))
            .boxed());
    }
    finite(f(x))
}

fn sqrt(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let x = float_arg("sqrt", &a[0], "a number")?;
    if x < 0.0 {
        return Err(math_err("The result is not a real number").help("sqrt() needs a value of 0 or more").boxed());
    }
    finite(x.sqrt())
}

fn power(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    need(is_number(&a[0]), "power", "a number as its first argument", &a[0])?;
    need(is_number(&a[1]), "power", "a number as its exponent", &a[1])?;
    it.arith(BinOp::Pow, &a[0], &a[1], None)
}

/// floor / ceil / trunc: integers stay integers, decimals become integers.
fn whole(name: &str, a: &[Value], f: fn(f64) -> f64) -> R<Value> {
    need(is_number(&a[0]), name, "a number", &a[0])?;
    match &a[0] {
        Value::Dec(x) => Ok(Value::Int(Int::from_f64(f(*x)).expect("finite"))),
        other => Ok(other.clone()),
    }
}

fn clamp(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    for v in a {
        need(is_number(v), "clamp", "numbers", v)?;
    }
    if cmp_values(&a[1], &a[2]) == Some(Ordering::Greater) {
        return Err(rt("E2006", "ValueError", "clamp() needs low <= high").boxed());
    }
    if cmp_values(&a[0], &a[1]) == Some(Ordering::Less) {
        return Ok(a[1].clone());
    }
    if cmp_values(&a[0], &a[2]) == Some(Ordering::Greater) {
        return Ok(a[2].clone());
    }
    Ok(a[0].clone())
}

fn sign(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    need(is_number(&a[0]), "sign", "a number", &a[0])?;
    let ord = cmp_values(&a[0], &Value::int(0)).unwrap_or(Ordering::Equal);
    Ok(Value::int(match ord {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }))
}

fn random_int(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    let lo = small_int_arg("random_int", &a[0], "an integer")?;
    let hi = small_int_arg("random_int", &a[1], "an integer")?;
    if lo > hi {
        return Err(rt("E2006", "ValueError", "random_int() needs low <= high").boxed());
    }
    let span = (hi as i128 - lo as i128 + 1) as u128;
    let r = (it.next_u64() as u128) % span;
    Ok(Value::int((lo as i128 + r as i128) as i64))
}

fn random_seed(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = small_int_arg("random_seed", &a[0], "an integer")?;
    it.seed_rng(s as u64);
    Ok(Value::Null)
}

fn log(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let x = float_arg("log", &a[0], "a number")?;
    if x <= 0.0 {
        return Err(math_err("The result is not a real number").help("log() needs a value greater than 0").boxed());
    }
    match a.get(1) {
        None => finite(x.ln()),
        Some(b) => {
            let base = float_arg("log", b, "a number as the base")?;
            if base <= 0.0 || base == 1.0 {
                return Err(rt("E2006", "ValueError", "log() base must be greater than 0 and not 1").boxed());
            }
            finite(x.ln() / base.ln())
        }
    }
}

fn factorial(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let n = small_int_arg("factorial", &a[0], "an integer")?;
    if n < 0 {
        return Err(rt("E2006", "ValueError", "factorial() needs 0 or more").boxed());
    }
    if n > 20_000 {
        return Err(math_err("Numeric overflow: the result is too large").boxed());
    }
    let mut acc = Int::Small(1);
    for i in 2..=n {
        acc = acc.mul(&Int::Small(i));
    }
    Ok(Value::Int(acc))
}
