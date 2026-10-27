//! `date`: calendar dates and times.
//!
//! A **date** is a map `{"year", "month", "day"}`; a **datetime** also has
//! `"hour"`, `"minute"` and `"second"`. They are ordinary maps, so you can read
//! fields (`d["year"]`), print them and compare them with `==`; use
//! `date.compare` for ordering. Calendar arithmetic is exact (proleptic
//! Gregorian calendar, years 1-9999). There is no time-zone database: values
//! are plain wall-clock readings, `utc_now()` gives UTC and `now()` / `today()`
//! use the machine's current UTC offset (UTC on Windows).

use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{register as reg, rt, small_int_arg, text_arg, type_err};
use crate::values::*;
use std::rc::Rc;

const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
const DAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

pub fn register(env: &Rc<Env>) {
    reg(env, "today", 0, Some(0), |_, _| {
        let ts = now_secs();
        Ok(to_value(&from_ts(ts + local_offset_at(ts), false)))
    });
    reg(env, "now", 0, Some(0), |_, _| {
        let ts = now_secs();
        Ok(to_value(&from_ts(ts + local_offset_at(ts), true)))
    });
    reg(env, "utc_now", 0, Some(0), |_, _| Ok(to_value(&from_ts(now_secs(), true))));
    reg(env, "local_offset", 0, Some(0), |_, _| Ok(Value::int(local_offset_at(now_secs()))));
    reg(env, "make_date", 3, Some(3), |_, a| {
        let dt = Dt { y: n("make_date", &a[0])?, mo: n("make_date", &a[1])?, d: n("make_date", &a[2])?, h: 0, mi: 0, s: 0, has_time: false };
        validate(&dt)?;
        Ok(to_value(&dt))
    });
    reg(env, "make_datetime", 3, Some(6), make_datetime);
    reg(env, "parse", 1, Some(1), parse);
    reg(env, "format", 2, Some(2), |_, a| {
        let dt = get("format", &a[0])?;
        let pat = text_arg("format", &a[1], "a format pattern")?;
        Ok(Value::text(strftime(&dt, &pat)?))
    });
    reg(env, "iso", 1, Some(1), |_, a| {
        let dt = get("iso", &a[0])?;
        let pat = if dt.has_time { "%Y-%m-%dT%H:%M:%S" } else { "%Y-%m-%d" };
        Ok(Value::text(strftime(&dt, pat)?))
    });
    reg(env, "weekday", 1, Some(1), |_, a| Ok(Value::int(weekday(&get("weekday", &a[0])?))));
    reg(env, "weekday_name", 1, Some(1), |_, a| {
        let w = match &a[0] {
            Value::Int(_) => {
                let w = small_int_arg("weekday_name", &a[0], "a weekday 1-7 or a date")?;
                if !(1..=7).contains(&w) {
                    return Err(rt("E2006", "ValueError", "weekday_name() needs a weekday from 1 (Monday) to 7 (Sunday)").boxed());
                }
                w
            }
            v => weekday(&get("weekday_name", v)?),
        };
        Ok(Value::text(DAYS[(w - 1) as usize]))
    });
    reg(env, "month_name", 1, Some(1), |_, a| {
        let m = match &a[0] {
            Value::Int(_) => {
                let m = small_int_arg("month_name", &a[0], "a month 1-12 or a date")?;
                if !(1..=12).contains(&m) {
                    return Err(rt("E2006", "ValueError", "month_name() needs a month from 1 to 12").boxed());
                }
                m
            }
            v => get("month_name", v)?.mo,
        };
        Ok(Value::text(MONTHS[(m - 1) as usize]))
    });
    reg(env, "day_of_year", 1, Some(1), |_, a| {
        let dt = get("day_of_year", &a[0])?;
        Ok(Value::int(days_from_civil(dt.y, dt.mo, dt.d) - days_from_civil(dt.y, 1, 1) + 1))
    });
    reg(env, "is_leap_year", 1, Some(1), |_, a| Ok(Value::Bool(is_leap(small_int_arg("is_leap_year", &a[0], "a year")?))));
    reg(env, "days_in_month", 2, Some(2), |_, a| {
        let (y, m) = (small_int_arg("days_in_month", &a[0], "a year")?, small_int_arg("days_in_month", &a[1], "a month")?);
        if !(1..=12).contains(&m) {
            return Err(rt("E2006", "ValueError", "days_in_month() needs a month from 1 to 12").boxed());
        }
        Ok(Value::int(dim(y, m)))
    });
    reg(env, "add_days", 2, Some(2), |_, a| add_days(a));
    reg(env, "add_months", 2, Some(2), |_, a| add_months("add_months", a, 1));
    reg(env, "add_years", 2, Some(2), |_, a| add_months("add_years", a, 12));
    reg(env, "add_hours", 2, Some(2), |_, a| add_seconds("add_hours", a, 3600));
    reg(env, "add_minutes", 2, Some(2), |_, a| add_seconds("add_minutes", a, 60));
    reg(env, "add_seconds", 2, Some(2), |_, a| add_seconds("add_seconds", a, 1));
    reg(env, "diff_days", 2, Some(2), |_, a| {
        let (x, y) = (get("diff_days", &a[0])?, get("diff_days", &a[1])?);
        Ok(Value::int(days_from_civil(x.y, x.mo, x.d) - days_from_civil(y.y, y.mo, y.d)))
    });
    reg(env, "diff_seconds", 2, Some(2), |_, a| {
        let (x, y) = (get("diff_seconds", &a[0])?, get("diff_seconds", &a[1])?);
        Ok(Value::int(timestamp(&x) - timestamp(&y)))
    });
    reg(env, "compare", 2, Some(2), |_, a| {
        let (x, y) = (get("compare", &a[0])?, get("compare", &a[1])?);
        Ok(Value::int(match timestamp(&x).cmp(&timestamp(&y)) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }))
    });
    reg(env, "to_timestamp", 1, Some(1), |_, a| Ok(Value::int(timestamp(&get("to_timestamp", &a[0])?))));
    reg(env, "from_timestamp", 1, Some(2), from_timestamp);
}

// ---- calendar arithmetic (Howard Hinnant's algorithms) -------------------------------------

#[derive(Clone, Copy)]
struct Dt {
    y: i64,
    mo: i64,
    d: i64,
    h: i64,
    mi: i64,
    s: i64,
    has_time: bool,
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn dim(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
    }
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn timestamp(dt: &Dt) -> i64 {
    days_from_civil(dt.y, dt.mo, dt.d) * 86_400 + dt.h * 3600 + dt.mi * 60 + dt.s
}

fn from_ts(ts: i64, has_time: bool) -> Dt {
    let days = ts.div_euclid(86_400);
    let rem = ts.rem_euclid(86_400);
    let (y, mo, d) = civil_from_days(days);
    Dt { y, mo, d, h: rem / 3600, mi: rem % 3600 / 60, s: rem % 60, has_time }
}

/// ISO weekday: 1 = Monday ... 7 = Sunday.
fn weekday(dt: &Dt) -> i64 {
    (days_from_civil(dt.y, dt.mo, dt.d) + 3).rem_euclid(7) + 1
}

fn invalid(msg: impl Into<String>) -> Box<crate::errors::ArcanaError> {
    rt("E2006", "ValueError", msg).boxed()
}

fn validate(dt: &Dt) -> R<()> {
    if !(1..=9999).contains(&dt.y) {
        return Err(invalid(format!("Invalid date: year {} must be between 1 and 9999", dt.y)));
    }
    if !(1..=12).contains(&dt.mo) {
        return Err(invalid(format!("Invalid date: month {} must be between 1 and 12", dt.mo)));
    }
    if dt.d < 1 || dt.d > dim(dt.y, dt.mo) {
        return Err(invalid(format!("Invalid date: day {} must be between 1 and {} for {}-{:02}", dt.d, dim(dt.y, dt.mo), dt.y, dt.mo)));
    }
    if dt.has_time {
        if !(0..=23).contains(&dt.h) {
            return Err(invalid(format!("Invalid time: hour {} must be between 0 and 23", dt.h)));
        }
        if !(0..=59).contains(&dt.mi) {
            return Err(invalid(format!("Invalid time: minute {} must be between 0 and 59", dt.mi)));
        }
        if !(0..=59).contains(&dt.s) {
            return Err(invalid(format!("Invalid time: second {} must be between 0 and 59", dt.s)));
        }
    }
    Ok(())
}

fn n(fname: &str, v: &Value) -> R<i64> {
    small_int_arg(fname, v, "an integer")
}

// ---- conversion between maps and Dt -----------------------------------------------------------

fn to_value(dt: &Dt) -> Value {
    let mut m = ArcMap::new();
    let mut put = |k: &str, v: i64| m.insert(Key::Text(Rc::from(k)), Value::int(v));
    put("year", dt.y);
    put("month", dt.mo);
    put("day", dt.d);
    if dt.has_time {
        put("hour", dt.h);
        put("minute", dt.mi);
        put("second", dt.s);
    }
    Value::map(m)
}

fn get(fname: &str, v: &Value) -> R<Dt> {
    let bad = || {
        type_err(format!("{}() needs a date or datetime map with year, month and day, but got {}", fname, type_name(v)))
            .help("create one with date.make_date(y, m, d) or date.parse(\"2026-10-25\")")
            .boxed()
    };
    let m = match v {
        Value::Map(m) => m.borrow(),
        _ => return Err(bad()),
    };
    let field = |name: &str| -> R<Option<i64>> {
        match m.get(&Key::Text(Rc::from(name))) {
            None => Ok(None),
            Some(Value::Int(i)) => i.to_i64().map(Some).ok_or_else(|| invalid(format!("Invalid date: {} is too large", name))),
            Some(_) => Err(type_err(format!("{}() needs integer fields, but '{}' is not an integer", fname, name)).boxed()),
        }
    };
    let (y, mo, d) = match (field("year")?, field("month")?, field("day")?) {
        (Some(y), Some(mo), Some(d)) => (y, mo, d),
        _ => return Err(bad()),
    };
    let has_time = m.contains_key(&Key::Text(Rc::from("hour")));
    let dt = Dt {
        y,
        mo,
        d,
        h: field("hour")?.unwrap_or(0),
        mi: field("minute")?.unwrap_or(0),
        s: field("second")?.unwrap_or(0),
        has_time,
    };
    validate(&dt)?;
    Ok(dt)
}

fn make_datetime(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let g = |i: usize| -> R<i64> {
        match a.get(i) {
            Some(v) => n("make_datetime", v),
            None => Ok(0),
        }
    };
    let dt = Dt { y: g(0)?, mo: g(1)?, d: g(2)?, h: g(3)?, mi: g(4)?, s: g(5)?, has_time: true };
    validate(&dt)?;
    Ok(to_value(&dt))
}

// ---- parsing and formatting -------------------------------------------------------------------

fn digits(s: &str) -> Option<i64> {
    if s.is_empty() || s.len() > 9 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn parse_text(s: &str) -> Option<Dt> {
    let s = s.trim().trim_end_matches('Z');
    let (dpart, tpart) = match s.find([' ', 'T']) {
        Some(i) => (&s[..i], Some(s[i + 1..].trim())),
        None => (s, None),
    };
    let d: Vec<&str> = dpart.split('-').collect();
    if d.len() != 3 {
        return None;
    }
    let (y, mo, da) = (digits(d[0])?, digits(d[1])?, digits(d[2])?);
    match tpart {
        None => Some(Dt { y, mo, d: da, h: 0, mi: 0, s: 0, has_time: false }),
        Some(t) => {
            let p: Vec<&str> = t.split(':').collect();
            if p.len() < 2 || p.len() > 3 {
                return None;
            }
            let secs = if p.len() == 3 { digits(p[2].split('.').next()?)? } else { 0 };
            Some(Dt { y, mo, d: da, h: digits(p[0])?, mi: digits(p[1])?, s: secs, has_time: true })
        }
    }
}

fn parse(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let s = text_arg("parse", &a[0], "text")?;
    match parse_text(&s) {
        Some(dt) => {
            validate(&dt)?;
            Ok(to_value(&dt))
        }
        None => Err(rt("E2011", "ConversionError", format!("Cannot parse {} as a date", quote(&s)))
            .help("use YYYY-MM-DD or YYYY-MM-DD HH:MM[:SS], e.g. \"2026-10-25 14:30\"")
            .boxed()),
    }
}

fn strftime(dt: &Dt, pat: &str) -> R<String> {
    let mut out = String::new();
    let mut it = pat.chars();
    while let Some(c) = it.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let code = it.next().ok_or_else(|| invalid("Format pattern ends with a lone '%'"))?;
        match code {
            'Y' => out.push_str(&format!("{:04}", dt.y)),
            'y' => out.push_str(&format!("{:02}", dt.y % 100)),
            'm' => out.push_str(&format!("{:02}", dt.mo)),
            'd' => out.push_str(&format!("{:02}", dt.d)),
            'e' => out.push_str(&format!("{:>2}", dt.d)),
            'H' => out.push_str(&format!("{:02}", dt.h)),
            'I' => out.push_str(&format!("{:02}", if dt.h % 12 == 0 { 12 } else { dt.h % 12 })),
            'M' => out.push_str(&format!("{:02}", dt.mi)),
            'S' => out.push_str(&format!("{:02}", dt.s)),
            'p' => out.push_str(if dt.h < 12 { "AM" } else { "PM" }),
            'B' => out.push_str(MONTHS[(dt.mo - 1) as usize]),
            'b' => out.push_str(&MONTHS[(dt.mo - 1) as usize][..3]),
            'A' => out.push_str(DAYS[(weekday(dt) - 1) as usize]),
            'a' => out.push_str(&DAYS[(weekday(dt) - 1) as usize][..3]),
            'u' => out.push_str(&weekday(dt).to_string()),
            'j' => out.push_str(&format!("{:03}", days_from_civil(dt.y, dt.mo, dt.d) - days_from_civil(dt.y, 1, 1) + 1)),
            '%' => out.push('%'),
            other => {
                return Err(invalid(format!("Unknown format code '%{}'", other))
                    .into_help("supported: %Y %y %m %d %e %H %I %M %S %p %B %b %A %a %u %j %%"))
            }
        }
    }
    Ok(out)
}

trait IntoHelp {
    fn into_help(self, h: &str) -> Self;
}

impl IntoHelp for Box<crate::errors::ArcanaError> {
    fn into_help(mut self, h: &str) -> Self {
        self.help = Some(h.to_string());
        self
    }
}

// ---- arithmetic -------------------------------------------------------------------------------

fn checked(dt: Dt) -> R<Value> {
    if !(1..=9999).contains(&dt.y) {
        return Err(rt("E2008", "MathError", "Numeric overflow: the date is outside the years 1-9999").boxed());
    }
    Ok(to_value(&dt))
}

fn add_days(a: &[Value]) -> R<Value> {
    let dt = get("add_days", &a[0])?;
    let k = n("add_days", &a[1])?;
    let days = days_from_civil(dt.y, dt.mo, dt.d)
        .checked_add(k)
        .filter(|d| d.abs() < 4_000_000)
        .ok_or_else(|| rt("E2008", "MathError", "Numeric overflow: the date is outside the years 1-9999").boxed())?;
    let (y, mo, d) = civil_from_days(days);
    checked(Dt { y, mo, d, ..dt })
}

fn add_months(name: &str, a: &[Value], factor: i64) -> R<Value> {
    let dt = get(name, &a[0])?;
    let k = n(name, &a[1])?
        .checked_mul(factor)
        .filter(|m| m.abs() < 1_000_000)
        .ok_or_else(|| rt("E2008", "MathError", "Numeric overflow: the date is outside the years 1-9999").boxed())?;
    let total = dt.y * 12 + (dt.mo - 1) + k;
    let (y, mo) = (total.div_euclid(12), total.rem_euclid(12) + 1);
    if !(1..=9999).contains(&y) {
        return Err(rt("E2008", "MathError", "Numeric overflow: the date is outside the years 1-9999").boxed());
    }
    checked(Dt { y, mo, d: dt.d.min(dim(y, mo)), ..dt })
}

/// Adding hours/minutes/seconds always gives a datetime.
fn add_seconds(name: &str, a: &[Value], unit: i64) -> R<Value> {
    let dt = get(name, &a[0])?;
    let k = n(name, &a[1])?;
    let delta = k
        .checked_mul(unit)
        .filter(|d| d.abs() < 400_000_000_000)
        .ok_or_else(|| rt("E2008", "MathError", "Numeric overflow: the date is outside the years 1-9999").boxed())?;
    checked(from_ts(timestamp(&dt) + delta, true))
}

fn from_timestamp(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let ts = match &a[0] {
        Value::Int(i) => i.to_i64(),
        Value::Dec(f) if f.is_finite() && f.abs() < 4e11 => Some(f.floor() as i64),
        other => return Err(type_err(format!("from_timestamp() needs a number, but got {}", type_name(other))).boxed()),
    }
    .filter(|t| t.abs() < 400_000_000_000)
    .ok_or_else(|| rt("E2008", "MathError", "Numeric overflow: the date is outside the years 1-9999").boxed())?;
    let off = match a.get(1) {
        Some(v) => n("from_timestamp", v)?,
        None => 0,
    };
    checked(from_ts(ts + off, true))
}

// ---- the clock ---------------------------------------------------------------------------------

fn now_secs() -> i64 {
    crate::stdlib::time::epoch_seconds().floor() as i64
}

#[cfg(unix)]
fn local_offset_at(ts: i64) -> i64 {
    use std::os::raw::{c_char, c_int, c_long};
    #[repr(C)]
    struct Tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        tm_gmtoff: c_long,
        tm_zone: *const c_char,
    }
    extern "C" {
        fn localtime_r(t: *const c_long, out: *mut Tm) -> *mut Tm;
    }
    // SAFETY: `tm` is a zeroed, correctly laid out `struct tm` (glibc, musl and the BSDs/macOS
    // share this layout) and `localtime_r` only writes into it.
    unsafe {
        let mut tm: Tm = std::mem::zeroed();
        let t: c_long = ts as c_long;
        if localtime_r(&t, &mut tm).is_null() {
            0
        } else {
            tm.tm_gmtoff as i64
        }
    }
}

#[cfg(not(unix))]
fn local_offset_at(_ts: i64) -> i64 {
    0
}
