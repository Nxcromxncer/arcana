//! `file`: reading and writing text files and inspecting directories.
//!
//! Paths are relative to the directory the program was started from. Failures
//! raise a catchable `E2012` (kind `FileError`).

use crate::environment::Env;
use crate::interpreter::{Interpreter, R};
use crate::runtime::{list_arg, register as reg, rt, text_arg, type_err};
use crate::values::*;
use std::io::Write as _;
use std::rc::Rc;

pub fn register(env: &Rc<Env>) {
    reg(env, "read_text", 1, Some(1), read_text);
    reg(env, "read_lines", 1, Some(1), |it, a| {
        let t = read_text(it, a)?;
        match t {
            Value::Text(s) => Ok(Value::list(s.lines().map(Value::text).collect())),
            _ => unreachable!(),
        }
    });
    reg(env, "write_text", 2, Some(2), |_, a| write("write_text", a, false));
    reg(env, "append_text", 2, Some(2), |_, a| write("append_text", a, true));
    reg(env, "write_lines", 2, Some(2), write_lines);
    reg(env, "exists", 1, Some(1), |_, a| Ok(Value::Bool(std::path::Path::new(&*text_arg("exists", &a[0], "a path")?).exists())));
    reg(env, "is_file", 1, Some(1), |_, a| Ok(Value::Bool(std::path::Path::new(&*text_arg("is_file", &a[0], "a path")?).is_file())));
    reg(env, "is_dir", 1, Some(1), |_, a| Ok(Value::Bool(std::path::Path::new(&*text_arg("is_dir", &a[0], "a path")?).is_dir())));
    reg(env, "size", 1, Some(1), |_, a| {
        let p = text_arg("size", &a[0], "a path")?;
        match std::fs::metadata(p.as_ref()) {
            Ok(m) => Ok(Value::int(m.len() as i64)),
            Err(e) => Err(io_err("read", &p, e)),
        }
    });
    reg(env, "remove", 1, Some(1), remove);
    reg(env, "make_dir", 1, Some(1), |_, a| {
        let p = text_arg("make_dir", &a[0], "a path")?;
        std::fs::create_dir_all(p.as_ref()).map_err(|e| io_err("create the directory", &p, e))?;
        Ok(Value::Null)
    });
    reg(env, "list_dir", 1, Some(1), list_dir);
}

fn io_err(action: &str, path: &str, e: std::io::Error) -> Box<crate::errors::ArcanaError> {
    let msg = e.to_string();
    let msg = msg.split(" (os error").next().unwrap_or(&msg).to_string();
    rt("E2012", "FileError", format!("Cannot {} '{}': {}", action, path, msg)).boxed()
}

fn read_text(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let p = text_arg("read_text", &a[0], "a path")?;
    match std::fs::read_to_string(p.as_ref()) {
        Ok(s) => Ok(Value::text(s)),
        Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            Err(rt("E2012", "FileError", format!("Cannot read '{}': the file is not valid UTF-8 text", p)).boxed())
        }
        Err(e) => Err(io_err("read", &p, e)),
    }
}

fn write(name: &str, a: &[Value], append: bool) -> R<Value> {
    let p = text_arg(name, &a[0], "a path")?;
    let content = text_arg(name, &a[1], "text to write")?;
    let mut opts = std::fs::OpenOptions::new();
    opts.create(true);
    if append {
        opts.append(true);
    } else {
        opts.write(true).truncate(true);
    }
    let mut f = opts.open(p.as_ref()).map_err(|e| io_err("write", &p, e))?;
    f.write_all(content.as_bytes()).map_err(|e| io_err("write", &p, e))?;
    Ok(Value::Null)
}

fn write_lines(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let p = text_arg("write_lines", &a[0], "a path")?;
    let l = list_arg("write_lines", &a[1], "a list of text lines")?;
    let mut out = String::new();
    for item in l.borrow().iter() {
        match item {
            Value::Text(t) => {
                out.push_str(t);
                out.push('\n');
            }
            other => return Err(type_err(format!("write_lines() needs a list of text, but found {}", type_name(other))).boxed()),
        }
    }
    std::fs::write(p.as_ref(), out).map_err(|e| io_err("write", &p, e))?;
    Ok(Value::Null)
}

fn remove(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let p = text_arg("remove", &a[0], "a path")?;
    let path = std::path::Path::new(&*p);
    let r = if path.is_dir() { std::fs::remove_dir(path) } else { std::fs::remove_file(path) };
    r.map_err(|e| io_err("remove", &p, e))?;
    Ok(Value::Null)
}

fn list_dir(_: &mut Interpreter, a: &[Value]) -> R<Value> {
    let p = text_arg("list_dir", &a[0], "a path")?;
    let rd = std::fs::read_dir(p.as_ref()).map_err(|e| io_err("list", &p, e))?;
    let mut names: Vec<String> = rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    Ok(Value::list(names.into_iter().map(Value::text).collect()))
}
