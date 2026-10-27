//! The Arcana standard library: modules written in Rust.
//!
//! `import math` (or `from math import sqrt`) loads one of these instead of a file.
//! A standard module always wins over a file of the same name.

use crate::environment::Env;
use crate::values::ModuleValue;
use std::rc::Rc;

pub mod convert;
pub mod date;
pub mod error;
pub mod file;
pub mod list;
pub mod logic;
pub mod math;
pub mod system;
pub mod text;
pub mod time;

/// Names of the standard modules, in documentation order.
pub const MODULES: [&str; 10] = ["math", "text", "list", "logic", "convert", "error", "system", "file", "time", "date"];

pub fn is_std_module(name: &str) -> bool {
    MODULES.contains(&name)
}

/// Build a fresh module value for a standard module (None if `name` is not one).
pub fn build_module(name: &str) -> Option<Rc<ModuleValue>> {
    let env = Env::new(None);
    match name {
        "math" => math::register(&env),
        "text" => text::register(&env),
        "list" => list::register(&env),
        "logic" => logic::register(&env),
        "convert" => convert::register(&env),
        "error" => error::register(&env),
        "system" => system::register(&env),
        "file" => file::register(&env),
        "time" => time::register(&env),
        "date" => date::register(&env),
        _ => return None,
    }
    Some(Rc::new(ModuleValue { name: name.to_string(), env }))
}
