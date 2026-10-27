//! `time`: the wall clock, sleeping and human-readable durations.
//!
//! Timestamps are seconds since 1970-01-01 00:00:00 UTC (the Unix epoch).
//! Use the `date` module to turn them into calendar dates.

use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{float_arg, register as reg, rt};
use crate::values::*;
use std::io::Write;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    reg(env, "now", 0, Some(0), |_, _| Ok(Value::Dec(epoch_seconds())));
    reg(env, "sleep", 1, Some(1), sleep);
    reg(env, "format_duration", 1, Some(1), format_duration);
}

pub fn epoch_seconds() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn sleep(it: &mut Interpreter, a: &[Value]) -> R<Value> {
    let secs = float_arg("sleep", &a[0], "a number of seconds")?;
    if !(0.0..=1e9).contains(&secs) {
        return Err(rt("E2006", "ValueError", "sleep() needs a number of seconds between 0 and 1,000,000,000").boxed());
    }
    let _ = it.out.flush();
    std::thread::sleep(std::time::Duration::from_secs_f64(secs));
    Ok(Value::Null)
}

/// 3725 -> "1h 2m 5s", 90061 -> "1d 1h 1m 1s", 0.25 -> "250ms".
fn format_duration(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let x = float_arg("format_duration", &a[0], "a number of seconds")?;
    if x < 0.0 {
        return Err(rt("E2006", "ValueError", "format_duration() needs 0 or more seconds").boxed());
    }
    if x < 1.0 {
        return Ok(Value::text(format!("{}ms", (x * 1000.0).round() as i64)));
    }
    let total = x.floor() as i64;
    let (d, h, m, s) = (total / 86_400, total % 86_400 / 3600, total % 3600 / 60, total % 60);
    let mut parts = Vec::new();
    for (n, unit) in [(d, "d"), (h, "h"), (m, "m"), (s, "s")] {
        if n > 0 {
            parts.push(format!("{}{}", n, unit));
        }
    }
    Ok(Value::text(parts.join(" ")))
}
