//! Static checks run by `arcana check` (before execution).
//!
//! Detects unknown names (E1001) and same-scope redeclarations (E3002)
//! without running the program.  Spell bodies are checked after the
//! enclosing block has been fully scanned, so spells may call each other
//! regardless of order.

use crate::ast::*;
use crate::errors::{name_help, ArcanaError, Span};
use crate::lexer::normalize;
use crate::parser::parse_source;
use crate::runtime::{builtin_names, is_builtin};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

type Deferred = (Rc<Vec<Param>>, Rc<Vec<Stmt>>);

pub struct Checker {
    filename: String,
    source: Arc<str>,
    scopes: Vec<HashSet<String>>,
    pub errors: Vec<ArcanaError>,
}

impl Checker {
    pub fn new(filename: &str, source: &str) -> Checker {
        Checker {
            filename: filename.to_string(),
            source: Arc::from(normalize(source).as_str()),
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn push_err(&mut self, e: ArcanaError) {
        let mut e = e;
        e.attach(&self.filename, &self.source);
        self.errors.push(e);
    }

    pub fn check(mut self, program: &Program) -> Vec<ArcanaError> {
        self.run_stmts(&program.stmts, HashSet::new());
        self.errors
    }

    fn visible(&self) -> Vec<String> {
        let mut names: HashSet<String> = builtin_names().into_iter().collect();
        for s in &self.scopes {
            names.extend(s.iter().cloned());
        }
        names.into_iter().collect()
    }

    fn declare(&mut self, name: &str, span: Span) {
        let exists = self.scopes.last().map_or(false, |s| s.contains(name));
        if exists {
            self.push_err(
                ArcanaError::new("E3002", format!("'{}' is already declared in this scope", name), Some(span))
                    .help(format!("to change its value, assign to it: {} = ...", name))
                    .note("to make a separate variable, use a different name"),
            );
        }
        if let Some(s) = self.scopes.last_mut() {
            s.insert(name.to_string());
        }
    }

    fn block(&mut self, stmts: &[Stmt], names: &[&str]) {
        self.run_stmts(stmts, names.iter().map(|s| s.to_string()).collect());
    }

    fn run_stmts(&mut self, stmts: &[Stmt], scope: HashSet<String>) {
        self.scopes.push(scope);
        let mut deferred: Vec<Deferred> = Vec::new();
        for s in stmts {
            self.stmt(s, &mut deferred);
        }
        for (params, body) in deferred {
            self.spell_body(&params, &body);
        }
        self.scopes.pop();
    }

    fn spell_body(&mut self, params: &Rc<Vec<Param>>, body: &Rc<Vec<Stmt>>) {
        self.scopes.push(HashSet::new());
        for p in params.iter() {
            if let Some(d) = &p.default {
                self.expr(d);
            }
            if let Some(s) = self.scopes.last_mut() {
                s.insert(p.name.clone());
            }
        }
        self.scopes.pop();
        let names: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
        self.run_stmts(body, names);
    }

    fn stmt(&mut self, s: &Stmt, deferred: &mut Vec<Deferred>) {
        match s {
            Stmt::Bind { span, name, value } => {
                self.expr(value);
                self.declare(name, *span);
            }
            Stmt::SpellDecl { span, name, params, body } => {
                self.declare(name, *span);
                deferred.push((params.clone(), body.clone()));
            }
            Stmt::Import { span, module, alias, names } => match names {
                None => {
                    let n = alias.clone().unwrap_or_else(|| module.rsplit('.').next().unwrap_or(module).to_string());
                    self.declare(&n, *span);
                }
                Some(list) => {
                    for (name, nspan) in list {
                        self.declare(name, *nspan);
                    }
                }
            },
            Stmt::Assign { target, value, .. } => {
                self.expr(value);
                match target {
                    Expr::Name { span, name } => self.resolve(name, *span, true),
                    Expr::Index { obj, index, .. } => {
                        self.expr(obj);
                        self.expr(index);
                    }
                    _ => {}
                }
            }
            Stmt::Cast { value, .. } => self.expr(value),
            Stmt::ExprStmt { expr, .. } => self.expr(expr),
            Stmt::If { branches, else_body, .. } => {
                for (cond, body) in branches {
                    self.expr(cond);
                    self.block(body, &[]);
                }
                if let Some(b) = else_body {
                    self.block(b, &[]);
                }
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body, &[]);
            }
            Stmt::For { var, iterable, body, .. } => {
                self.expr(iterable);
                self.block(body, &[var.as_str()]);
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.expr(v);
                }
            }
            Stmt::Try { body, catch_name, catch_body, finally_body, .. } => {
                self.block(body, &[]);
                if let Some(cb) = catch_body {
                    let n: Vec<&str> = catch_name.iter().map(|s| s.as_str()).collect();
                    self.block(cb, &n);
                }
                if let Some(fb) = finally_body {
                    self.block(fb, &[]);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn resolve(&mut self, name: &str, span: Span, assign: bool) {
        if is_builtin(name) || self.scopes.iter().any(|s| s.contains(name)) {
            return;
        }
        let help = name_help(name, &self.visible(), assign);
        self.push_err(
            ArcanaError::new("E1001", format!("Unknown name '{}'", name), Some(span))
                .help(help)
                .note("names are case-sensitive")
                .with_category("name"),
        );
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Name { span, name } => self.resolve(name, *span, false),
            Expr::List { items, .. } => items.iter().for_each(|i| self.expr(i)),
            Expr::Map { entries, .. } => {
                for (k, v) in entries {
                    self.expr(k);
                    self.expr(v);
                }
            }
            Expr::Unary { operand, .. } => self.expr(operand),
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                self.expr(left);
                self.expr(right);
            }
            Expr::Call { callee, args, .. } => {
                self.expr(callee);
                args.iter().for_each(|a| self.expr(a));
            }
            Expr::Index { obj, index, .. } => {
                self.expr(obj);
                self.expr(index);
            }
            Expr::Member { obj, .. } => self.expr(obj),
            Expr::Literal { .. } => {}
        }
    }
}

/// Return a list of errors (empty when the program is clean).
pub fn check_source(source: &str, filename: &str) -> Vec<ArcanaError> {
    match parse_source(source, filename) {
        Err(e) => vec![*e],
        Ok(program) => Checker::new(filename, source).check(&program),
    }
}
