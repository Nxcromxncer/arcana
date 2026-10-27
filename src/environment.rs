//! Lexical environments (scopes).

use crate::values::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::rc::Rc;

/// A small, fast, non-cryptographic hasher (FxHash).
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FxHasher {
    #[inline]
    fn add(&mut self, w: u64) {
        self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for c in &mut chunks {
            self.add(u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]));
        }
        for b in chunks.remainder() {
            self.add(*b as u64);
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.add(i as u64);
    }
    fn write_u32(&mut self, i: u32) {
        self.add(i as u64);
    }
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }
    fn finish(&self) -> u64 {
        self.hash
    }
}

pub type FxBuild = BuildHasherDefault<FxHasher>;

pub struct Env {
    pub vars: RefCell<HashMap<String, Value, FxBuild>>,
    pub parent: Option<Rc<Env>>,
    pub builtin: bool,
}

impl Env {
    pub fn new(parent: Option<Rc<Env>>) -> Rc<Env> {
        Rc::new(Env { vars: RefCell::new(HashMap::default()), parent, builtin: false })
    }

    pub fn new_builtin() -> Rc<Env> {
        Rc::new(Env { vars: RefCell::new(HashMap::default()), parent: None, builtin: true })
    }

    pub fn has_local(&self, name: &str) -> bool {
        self.vars.borrow().contains_key(name)
    }

    pub fn get_local(&self, name: &str) -> Option<Value> {
        self.vars.borrow().get(name).cloned()
    }

    pub fn set_local(&self, name: &str, v: Value) {
        self.vars.borrow_mut().insert(name.to_string(), v);
    }

    pub fn lookup(&self, name: &str) -> Option<Value> {
        let mut cur: &Env = self;
        loop {
            if let Some(v) = cur.vars.borrow().get(name) {
                return Some(v.clone());
            }
            match &cur.parent {
                Some(p) => cur = p.as_ref(),
                None => return None,
            }
        }
    }

    pub fn find_scope(&self, name: &str) -> Option<&Env> {
        let mut cur: &Env = self;
        loop {
            if cur.vars.borrow().contains_key(name) {
                return Some(cur);
            }
            match &cur.parent {
                Some(p) => cur = p.as_ref(),
                None => return None,
            }
        }
    }

    /// Names declared directly in this scope (not in parents).
    pub fn local_names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.vars.borrow().keys().cloned().collect();
        v.sort();
        v
    }

    pub fn all_names(&self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        let mut cur: Option<&Env> = Some(self);
        while let Some(e) = cur {
            names.extend(e.vars.borrow().keys().cloned());
            cur = e.parent.as_deref();
        }
        names.sort();
        names.dedup();
        names
    }
}
