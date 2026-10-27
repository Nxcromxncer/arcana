//! Tree-walking interpreter for Arcana v0.1.

use crate::ast::*;
use crate::environment::Env;
use crate::errors::{name_help, ArcanaError, Span};
use crate::int::Int;
use crate::lexer::normalize;
use crate::parser::parse_source;
use crate::errors::close_match;
use crate::runtime::make_builtins;
use crate::stdlib;
use crate::values::*;
use std::collections::HashMap;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

pub type R<T> = Result<T, Box<ArcanaError>>;

pub enum Flow {
    Normal,
    Break,
    Continue,
    Return(Value),
}

enum LoopCtl {
    Next,
    Break,
    Ret(Value),
}

#[derive(Clone, Debug)]
pub struct Options {
    pub max_steps: Option<i64>,
    pub max_depth: usize,
    pub search_paths: Vec<PathBuf>,
    /// Command-line arguments passed to the program (see `system.args()`).
    pub args: Vec<String>,
}

impl Default for Options {
    fn default() -> Options {
        Options { max_steps: None, max_depth: 1000, search_paths: Vec::new(), args: Vec::new() }
    }
}

pub struct Interpreter {
    pub(crate) out: Box<dyn Write>,
    pub(crate) inp: Box<dyn BufRead>,
    pub(crate) err_out: Box<dyn Write>,
    pub(crate) prog_args: Vec<String>,
    rng: u64,
    started: std::time::Instant,
    max_steps: Option<i64>,
    max_depth: usize,
    search_paths: Vec<PathBuf>,
    steps: i64,
    depth: usize,
    modules: HashMap<String, Rc<ModuleValue>>,
    loading: Vec<String>,
    module_stack: Vec<Rc<ModuleInfo>>,
    builtin_env: Rc<Env>,
}

fn abspath(p: &Path) -> PathBuf {
    let abs = if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(p) };
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn overflow(span: Option<Span>, interp: &Interpreter) -> Box<ArcanaError> {
    interp.err("E2008", "Numeric overflow: the result is too large", span).recoverable("MathError").boxed()
}

fn float_floor_div(vx: f64, wx: f64) -> f64 {
    let mut m = vx % wx;
    let mut div = (vx - m) / wx;
    if m != 0.0 && ((wx < 0.0) != (m < 0.0)) {
        m += wx;
        div -= 1.0;
    }
    let _ = m;
    if div != 0.0 {
        let f = div.floor();
        if div - f > 0.5 {
            f + 1.0
        } else {
            f
        }
    } else {
        0.0f64.copysign(vx / wx)
    }
}

fn float_mod(vx: f64, wx: f64) -> f64 {
    let mut m = vx % wx;
    if m != 0.0 {
        if (wx < 0.0) != (m < 0.0) {
            m += wx;
        }
    } else {
        m = 0.0f64.copysign(wx);
    }
    m
}

impl Interpreter {
    /// An interpreter wired to the process's stdin/stdout.
    pub fn new(opts: Options) -> Interpreter {
        let out: Box<dyn Write> = if std::io::stdout().is_terminal() {
            Box::new(std::io::stdout())
        } else {
            Box::new(std::io::BufWriter::with_capacity(64 * 1024, std::io::stdout()))
        };
        Interpreter::with_io(out, Box::new(std::io::stdin().lock()), opts)
    }

    pub fn with_io(out: Box<dyn Write>, inp: Box<dyn BufRead>, opts: Options) -> Interpreter {
        let mut search_paths = opts.search_paths.clone();
        if let Some(v) = std::env::var_os("ARCANA_PATH") {
            search_paths.extend(std::env::split_paths(&v).filter(|p| !p.as_os_str().is_empty()));
        }
        let builtin_env = Env::new_builtin();
        make_builtins(&builtin_env);
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5eed)
            ^ ((std::process::id() as u64) << 32);
        Interpreter {
            out,
            inp,
            err_out: Box::new(std::io::stderr()),
            prog_args: opts.args.clone(),
            rng: seed,
            started: std::time::Instant::now(),
            max_steps: opts.max_steps,
            max_depth: opts.max_depth,
            search_paths,
            steps: 0,
            depth: 0,
            modules: HashMap::new(),
            loading: Vec::new(),
            module_stack: Vec::new(),
            builtin_env,
        }
    }

    pub fn flush(&mut self) {
        let _ = self.out.flush();
    }

    /// Redirect what `warn()` / `debug()` write (stderr by default); used by tests.
    pub fn set_err(&mut self, w: Box<dyn Write>) {
        self.err_out = w;
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub(crate) fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub(crate) fn seed_rng(&mut self, seed: u64) {
        self.rng = seed;
    }

    pub(crate) fn elapsed_secs(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    // ---- errors ----------------------------------------------------------------
    pub(crate) fn attach(&self, e: &mut ArcanaError) {
        if let Some(info) = self.module_stack.last() {
            e.attach(&info.filename, &info.source);
        }
    }

    pub(crate) fn err(&self, code: &str, msg: impl Into<String>, span: Option<Span>) -> ArcanaError {
        let mut e = ArcanaError::new(code, msg, span);
        self.attach(&mut e);
        e
    }

    fn unknown_name(&self, name: &str, span: Span, env: &Env, assign: bool) -> Box<ArcanaError> {
        self.err("E1001", format!("Unknown name '{}'", name), Some(span))
            .help(name_help(name, &env.all_names(), assign))
            .note("names are case-sensitive")
            .with_category("name")
            .boxed()
    }

    fn tick(&mut self, span: Span) -> R<()> {
        self.steps += 1;
        if let Some(m) = self.max_steps {
            if self.steps > m {
                return Err(self
                    .err("E2009", format!("Step limit of {} exceeded", m), Some(span))
                    .help("the program ran longer than the configured --max-steps limit")
                    .boxed());
            }
        }
        Ok(())
    }

    // ---- running programs ---------------------------------------------------
    pub fn run_source(&mut self, source: &str, filename: &str, call_main: bool) -> R<Rc<Env>> {
        let program = parse_source(source, filename)?;
        let info = Rc::new(ModuleInfo {
            filename: filename.to_string(),
            source: Arc::from(normalize(source).as_str()),
            base_dir: Self::base_dir(filename),
        });
        let env = Env::new(Some(self.builtin_env.clone()));
        self.module_stack.push(info);
        let result = (|| -> R<()> {
            self.exec_block(&program.stmts, &env)?;
            if call_main {
                self.call_main(&env)?;
            }
            Ok(())
        })();
        self.module_stack.pop();
        result?;
        Ok(env)
    }

    fn base_dir(filename: &str) -> PathBuf {
        let p = Path::new(filename);
        if p.is_file() {
            abspath(p).parent().map(|d| d.to_path_buf()).unwrap_or_default()
        } else {
            std::env::current_dir().unwrap_or_default()
        }
    }

    fn call_main(&mut self, env: &Rc<Env>) -> R<()> {
        match env.get_local("main") {
            None => Err(self
                .err("E2004", "No 'main' spell found", None)
                .help("define the entry point: spell main() { ... }")
                .boxed()),
            Some(m) => {
                self.call_value(&m, Vec::new(), None)?;
                Ok(())
            }
        }
    }

    // ---- statements ------------------------------------------------------------
    pub fn exec_block(&mut self, stmts: &[Stmt], env: &Rc<Env>) -> R<Flow> {
        for s in stmts {
            self.tick(s.span())?;
            match self.exec(s, env)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_scoped(&mut self, stmts: &[Stmt], env: &Rc<Env>) -> R<Flow> {
        let inner = Env::new(Some(env.clone()));
        self.exec_block(stmts, &inner)
    }

    fn exec(&mut self, s: &Stmt, env: &Rc<Env>) -> R<Flow> {
        match s {
            Stmt::Bind { span, name, value } => {
                let v = self.eval(value, env)?;
                self.declare(env, name, v, *span)?;
            }
            Stmt::SpellDecl { span, name, params, body } => {
                let module = self.module_stack.last().cloned().expect("module");
                let sp = Spell { name: name.clone(), params: params.clone(), body: body.clone(), closure: env.clone(), module };
                self.declare(env, name, Value::Spell(Rc::new(sp)), *span)?;
            }
            Stmt::ExprStmt { expr, .. } => {
                self.eval(expr, env)?;
            }
            Stmt::Cast { value, .. } => {
                let v = self.eval(value, env)?;
                let mut text = to_text(&v, false);
                text.push('\n');
                let _ = self.out.write_all(text.as_bytes());
            }
            Stmt::Assign { target, value, .. } => self.x_assign(target, value, env)?,
            Stmt::If { branches, else_body, .. } => {
                for (cond, body) in branches {
                    let c = self.eval(cond, env)?;
                    if self.require_bool(&c, cond.span(), "Condition")? {
                        return self.exec_scoped(body, env);
                    }
                }
                if let Some(b) = else_body {
                    return self.exec_scoped(b, env);
                }
            }
            Stmt::While { span, cond, body } => loop {
                let c = self.eval(cond, env)?;
                if !self.require_bool(&c, cond.span(), "Condition")? {
                    break;
                }
                self.tick(*span)?;
                match self.exec_scoped(body, env)? {
                    Flow::Break => break,
                    Flow::Return(v) => return Ok(Flow::Return(v)),
                    Flow::Continue | Flow::Normal => {}
                }
            },
            Stmt::For { span, var, iterable, body } => return self.x_for(*span, var, iterable, body, env),
            Stmt::Return { value, .. } => {
                let v = match value {
                    None => Value::Null,
                    Some(e) => self.eval(e, env)?,
                };
                return Ok(Flow::Return(v));
            }
            Stmt::Break { .. } => return Ok(Flow::Break),
            Stmt::Continue { .. } => return Ok(Flow::Continue),
            Stmt::Try { body, catch_name, catch_body, finally_body, .. } => {
                let mut result = self.exec_scoped(body, env);
                if let Err(e) = &result {
                    if e.recoverable {
                        if let Some(cb) = catch_body {
                            let e = e.clone();
                            let cenv = Env::new(Some(env.clone()));
                            let mut m = ArcMap::new();
                            m.insert(Key::Text(Rc::from("code")), Value::text(&e.code));
                            m.insert(Key::Text(Rc::from("kind")), Value::text(&e.kind));
                            m.insert(Key::Text(Rc::from("message")), Value::text(&e.message));
                            if let Some(n) = catch_name {
                                cenv.set_local(n, Value::map(m));
                            }
                            result = self.exec_block(cb, &cenv);
                        }
                    }
                }
                if let Some(fb) = finally_body {
                    match self.exec_scoped(fb, env) {
                        Err(e) => return Err(e),
                        Ok(Flow::Normal) => {}
                        Ok(other) => return Ok(other),
                    }
                }
                return result;
            }
            Stmt::Import { span, module, alias, names } => self.x_import(*span, module, alias, names, env)?,
        }
        Ok(Flow::Normal)
    }

    fn declare(&mut self, env: &Rc<Env>, name: &str, value: Value, span: Span) -> R<()> {
        if env.has_local(name) {
            return Err(self
                .err("E3002", format!("'{}' is already declared in this scope", name), Some(span))
                .help(format!("to change its value, assign to it: {} = ...", name))
                .note("to make a separate variable, use a different name")
                .boxed());
        }
        env.set_local(name, value);
        Ok(())
    }

    fn x_assign(&mut self, target: &Expr, value: &Expr, env: &Rc<Env>) -> R<()> {
        let v = self.eval(value, env)?;
        match target {
            Expr::Name { span, name } => {
                match env.find_scope(name) {
                    None => return Err(self.unknown_name(name, *span, env, true)),
                    Some(scope) => {
                        if scope.builtin {
                            return Err(self
                                .err("E3002", format!("Cannot assign to built-in '{}'", name), Some(*span))
                                .help(format!("declare your own with 'bind {} = ...' instead", name))
                                .boxed());
                        }
                        scope.set_local(name, v);
                    }
                }
                Ok(())
            }
            Expr::Index { span, obj, index } => {
                let o = self.eval(obj, env)?;
                let idx = self.eval(index, env)?;
                match &o {
                    Value::List(l) => {
                        let i = {
                            let len = l.borrow().len();
                            self.check_index(&idx, len, "list", *span)?
                        };
                        l.borrow_mut()[i] = v;
                        Ok(())
                    }
                    Value::Map(m) => {
                        let k = self.check_key(&idx, *span)?;
                        m.borrow_mut().insert(k, v);
                        Ok(())
                    }
                    other => Err(self
                        .err("E3003", format!("Cannot assign into a {}", type_name(other)), Some(*span))
                        .help("only lists and maps can be changed in place; text is immutable")
                        .boxed()),
                }
            }
            _ => unreachable!("parser only produces Name/Index targets"),
        }
    }

    fn require_bool(&self, v: &Value, span: Span, what: &str) -> R<bool> {
        if let Value::Bool(b) = v {
            return Ok(*b);
        }
        let tn = type_name(v);
        let hint = match tn {
            "integer" => "compare explicitly, e.g. x != 0",
            "decimal" => "compare explicitly, e.g. x != 0.0",
            "text" | "list" | "map" => "compare explicitly, e.g. length(x) > 0",
            "null" => "compare explicitly, e.g. x != null",
            _ => "use a comparison to produce On or Off",
        };
        Err(self
            .err("E3001", format!("{} must be a boolean (On or Off), but got {}", what, tn), Some(span))
            .help(hint)
            .note("Arcana has no truthiness: only On and Off are accepted")
            .boxed())
    }

    fn loop_body(&mut self, span: Span, var: &str, item: Value, body: &[Stmt], env: &Rc<Env>) -> R<LoopCtl> {
        self.tick(span)?;
        let loop_env = Env::new(Some(env.clone()));
        loop_env.set_local(var, item);
        Ok(match self.exec_block(body, &loop_env)? {
            Flow::Break => LoopCtl::Break,
            Flow::Return(v) => LoopCtl::Ret(v),
            Flow::Continue | Flow::Normal => LoopCtl::Next,
        })
    }

    fn x_for(&mut self, span: Span, var: &str, iterable: &Expr, body: &[Stmt], env: &Rc<Env>) -> R<Flow> {
        let it = self.eval(iterable, env)?;
        macro_rules! run {
            ($item:expr) => {
                match self.loop_body(span, var, $item, body, env)? {
                    LoopCtl::Next => {}
                    LoopCtl::Break => break,
                    LoopCtl::Ret(v) => return Ok(Flow::Return(v)),
                }
            };
        }
        match &it {
            Value::List(l) => {
                let seq: Vec<Value> = l.borrow().clone();
                for item in seq {
                    run!(item);
                }
            }
            Value::Text(s) => {
                let s = s.clone();
                for c in s.chars() {
                    run!(Value::text(c.to_string()));
                }
            }
            Value::Map(m) => {
                let keys: Vec<Value> = m.borrow().iter().map(|(k, _)| k.to_value()).collect();
                for k in keys {
                    run!(k);
                }
            }
            Value::Range(r) => {
                let mut cur = r.start.clone();
                let up = r.step.is_pos();
                loop {
                    let go = if up { cur < r.stop } else { cur > r.stop };
                    if !go {
                        break;
                    }
                    run!(Value::Int(cur.clone()));
                    cur = cur.add(&r.step);
                }
            }
            other => {
                let mut help = "loop over a list, text, map or range(...)".to_string();
                if let Value::Int(i) = other {
                    help = format!("did you mean range({})?", i);
                }
                return Err(self
                    .err("E3003", format!("Cannot loop over a {}", type_name(other)), Some(iterable.span()))
                    .help(help)
                    .boxed());
            }
        }
        Ok(Flow::Normal)
    }

    // ---- imports -----------------------------------------------------------------
    fn find_module(&self, dotted: &str, base_dir: &Path) -> (Option<PathBuf>, Vec<PathBuf>) {
        let mut rel = PathBuf::new();
        for part in dotted.split('.') {
            rel.push(part);
        }
        let mut name = rel.into_os_string();
        name.push(".arc");
        let rel = PathBuf::from(name);
        let mut dirs = vec![base_dir.to_path_buf()];
        dirs.extend(self.search_paths.iter().cloned());
        for d in &dirs {
            let p = d.join(&rel);
            if p.is_file() {
                return (Some(abspath(&p)), dirs);
            }
        }
        (None, dirs)
    }

    fn std_module(&mut self, name: &str) -> Option<Rc<ModuleValue>> {
        let key = format!("std:{}", name);
        if let Some(m) = self.modules.get(&key) {
            return Some(m.clone());
        }
        let m = stdlib::build_module(name)?;
        self.modules.insert(key, m.clone());
        Some(m)
    }

    fn missing_name(&self, m: &ModuleValue, name: &str, span: Span) -> Box<ArcanaError> {
        let candidates = m.env.local_names();
        let help = match close_match(name, &candidates, 0.6) {
            Some(c) => format!("did you mean '{}'?", c),
            None => "check the spelling of the name in that grimoire".to_string(),
        };
        self.err("E5003", format!("Grimoire '{}' has no name '{}'", m.name, name), Some(span)).help(help).boxed()
    }

    fn load_module(&mut self, dotted: &str, span: Span) -> R<Rc<ModuleValue>> {
        if let Some(m) = self.std_module(dotted) {
            return Ok(m);
        }
        let base = self.module_stack.last().map(|m| m.base_dir.clone()).unwrap_or_default();
        let (path, dirs) = self.find_module(dotted, &base);
        let path = match path {
            Some(p) => p,
            None => {
                let searched: Vec<String> = dirs.iter().map(|d| d.display().to_string()).collect();
                return Err(self
                    .err("E5001", format!("Cannot find grimoire '{}'", dotted), Some(span))
                    .help("place the .arc file next to this one, or add its folder to ARCANA_PATH")
                    .note(format!("searched: {}", searched.join(", ")))
                    .boxed());
            }
        };
        let key = path.to_string_lossy().into_owned();
        if self.loading.contains(&key) {
            return Err(self
                .err("E5002", format!("Cyclic import of '{}'", dotted), Some(span))
                .help("move shared code into a third grimoire that both can import")
                .boxed());
        }
        if let Some(m) = self.modules.get(&key) {
            return Ok(m.clone());
        }
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                return Err(self
                    .err("E5001", format!("Cannot read grimoire '{}': {}", dotted, e), Some(span))
                    .boxed())
            }
        };
        let program = parse_source(&source, &key)?;
        let env = Env::new(Some(self.builtin_env.clone()));
        let info = Rc::new(ModuleInfo {
            filename: key.clone(),
            source: Arc::from(normalize(&source).as_str()),
            base_dir: path.parent().map(|p| p.to_path_buf()).unwrap_or_default(),
        });
        self.loading.push(key.clone());
        self.module_stack.push(info);
        let r = self.exec_block(&program.stmts, &env);
        self.module_stack.pop();
        self.loading.pop();
        r?;
        let m = Rc::new(ModuleValue { name: dotted.to_string(), env });
        self.modules.insert(key, m.clone());
        Ok(m)
    }

    fn x_import(
        &mut self,
        span: Span,
        module: &str,
        alias: &Option<String>,
        names: &Option<Vec<(String, Span)>>,
        env: &Rc<Env>,
    ) -> R<()> {
        let m = self.load_module(module, span)?;
        match names {
            None => {
                let name = alias.clone().unwrap_or_else(|| module.rsplit('.').next().unwrap_or(module).to_string());
                self.declare(env, &name, Value::Module(m), span)
            }
            Some(list) => {
                for (name, nspan) in list {
                    let v = match m.env.get_local(name) {
                        Some(v) => v,
                        None => return Err(self.missing_name(&m, name, *nspan)),
                    };
                    self.declare(env, name, v, *nspan)?;
                }
                Ok(())
            }
        }
    }

    // ---- expressions -----------------------------------------------------------
    pub fn eval(&mut self, node: &Expr, env: &Rc<Env>) -> R<Value> {
        match node {
            Expr::Literal { value, .. } => Ok(value.clone()),
            Expr::Name { span, name } => match env.lookup(name) {
                Some(v) => Ok(v),
                None => Err(self.unknown_name(name, *span, env, false)),
            },
            Expr::List { items, .. } => {
                let mut out = Vec::with_capacity(items.len());
                for i in items {
                    out.push(self.eval(i, env)?);
                }
                Ok(Value::list(out))
            }
            Expr::Map { entries, .. } => {
                let mut m = ArcMap::new();
                for (k, v) in entries {
                    let key = self.eval(k, env)?;
                    let key = self.check_key(&key, k.span())?;
                    let val = self.eval(v, env)?;
                    m.insert(key, val);
                }
                Ok(Value::map(m))
            }
            Expr::Unary { span, op, operand } => self.e_unary(*span, *op, operand, env),
            Expr::Logical { and, left, right, .. } => {
                let opname = if *and { "and" } else { "or" };
                let what = format!("Operand of '{}'", opname);
                let l = self.eval(left, env)?;
                let lb = self.require_bool(&l, left.span(), &what)?;
                if *and != lb {
                    return Ok(Value::Bool(lb));
                }
                let r = self.eval(right, env)?;
                let rb = self.require_bool(&r, right.span(), &what)?;
                Ok(Value::Bool(rb))
            }
            Expr::Binary { span, op, left, right } => {
                let l = self.eval(left, env)?;
                let r = self.eval(right, env)?;
                self.e_binary(*span, *op, &l, &r)
            }
            Expr::Index { span, obj, index } => {
                let o = self.eval(obj, env)?;
                let i = self.eval(index, env)?;
                self.e_index(*span, &o, &i)
            }
            Expr::Member { span, obj, name } => {
                let o = self.eval(obj, env)?;
                match &o {
                    Value::Module(m) => match m.env.get_local(name) {
                        Some(v) => Ok(v),
                        None => Err(self.missing_name(m, name, *span)),
                    },
                    other => Err(self
                        .err("E3003", format!("A {} has no members", type_name(other)), Some(*span))
                        .help("member access (a.b) is only used with imported grimoires")
                        .boxed()),
                }
            }
            Expr::Call { span, callee, args } => {
                let c = self.eval(callee, env)?;
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a, env)?);
                }
                self.call_value(&c, vals, Some(*span))
            }
        }
    }

    fn check_key(&self, k: &Value, span: Span) -> R<Key> {
        match Key::from_value(k) {
            Some(key) => Ok(key),
            None => Err(self
                .err("E3003", format!("Map keys must be text or integer, but got {}", type_name(k)), Some(span))
                .boxed()),
        }
    }

    fn e_unary(&mut self, span: Span, op: UnOp, operand: &Expr, env: &Rc<Env>) -> R<Value> {
        let v = self.eval(operand, env)?;
        match op {
            UnOp::Not => match v {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                other => Err(self
                    .err("E3001", format!("'not' needs a boolean (On or Off), but got {}", type_name(&other)), Some(span))
                    .help("use a comparison to produce On or Off")
                    .boxed()),
            },
            UnOp::Neg => match v {
                Value::Int(i) => Ok(Value::Int(i.neg())),
                Value::Dec(f) => Ok(Value::Dec(-f)),
                other => Err(self.err("E3003", format!("Cannot negate a {}", type_name(&other)), Some(span)).boxed()),
            },
        }
    }

    fn contains(&self, item: &Value, coll: &Value, span: Span) -> R<bool> {
        match coll {
            Value::List(l) => Ok(l.borrow().iter().any(|x| arc_equal(item, x))),
            Value::Map(m) => Ok(match Key::from_value(item) {
                Some(k) => m.borrow().contains_key(&k),
                None => false,
            }),
            Value::Text(s) => match item {
                Value::Text(t) => Ok(s.contains(t.as_ref())),
                other => Err(self
                    .err("E3003", format!("Cannot look for a {} inside text", type_name(other)), Some(span))
                    .help("use text(x) to convert it first")
                    .boxed()),
            },
            Value::Range(r) => Ok(match item {
                Value::Int(i) => r.contains(i),
                _ => false,
            }),
            other => Err(self
                .err(
                    "E3003",
                    format!("'in' needs a list, map, text or range on the right, but got {}", type_name(other)),
                    Some(span),
                )
                .boxed()),
        }
    }

    fn e_binary(&mut self, span: Span, op: BinOp, left: &Value, right: &Value) -> R<Value> {
        match op {
            BinOp::Eq => Ok(Value::Bool(arc_equal(left, right))),
            BinOp::Ne => Ok(Value::Bool(!arc_equal(left, right))),
            BinOp::In => Ok(Value::Bool(self.contains(left, right, span)?)),
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => match cmp_values(left, right) {
                Some(ord) => {
                    use std::cmp::Ordering::*;
                    Ok(Value::Bool(match op {
                        BinOp::Lt => ord == Less,
                        BinOp::Le => ord != Greater,
                        BinOp::Gt => ord == Greater,
                        _ => ord != Less,
                    }))
                }
                None => Err(self
                    .err(
                        "E3003",
                        format!("Cannot compare {} with {} using '{}'", type_name(left), type_name(right), op.symbol()),
                        Some(span),
                    )
                    .help("'<' and '>' work on two numbers or two texts")
                    .boxed()),
            },
            _ => self.arith(op, left, right, Some(span)),
        }
    }

    fn math_err(&self, code: &str, msg: &str, span: Option<Span>) -> Box<ArcanaError> {
        self.err(code, msg, span).recoverable("MathError").boxed()
    }

    pub(crate) fn arith(&mut self, op: BinOp, a: &Value, b: &Value, span: Option<Span>) -> R<Value> {
        if op == BinOp::Add {
            if let (Value::Text(x), Value::Text(y)) = (a, b) {
                let mut s = String::with_capacity(x.len() + y.len());
                s.push_str(x);
                s.push_str(y);
                return Ok(Value::Text(Rc::from(s)));
            }
        }
        if !(is_number(a) && is_number(b)) {
            let mut e = self.err(
                "E3003",
                format!("Cannot use '{}' with {} and {}", op.symbol(), type_name(a), type_name(b)),
                span,
            );
            if op == BinOp::Add && (matches!(a, Value::Text(_)) || matches!(b, Value::Text(_))) {
                e = e.help("use text(value) to convert a value to text before joining");
            }
            return Err(e.boxed());
        }
        if let (Value::Int(x), Value::Int(y)) = (a, b) {
            return match op {
                BinOp::Add => Ok(Value::Int(x.add(y))),
                BinOp::Sub => Ok(Value::Int(x.sub(y))),
                BinOp::Mul => Ok(Value::Int(x.mul(y))),
                BinOp::Div | BinOp::FloorDiv | BinOp::Mod => {
                    if y.is_zero() {
                        return Err(self
                            .err("E2001", "Division by zero", span)
                            .recoverable("MathError")
                            .help("check that the divisor is not 0 first")
                            .boxed());
                    }
                    match op {
                        BinOp::Div => match Int::true_div(x, y) {
                            Some(f) => Ok(Value::Dec(f)),
                            None => Err(overflow(span, self)),
                        },
                        BinOp::FloorDiv => Ok(Value::Int(x.floor_div(y))),
                        _ => Ok(Value::Int(x.modulo(y))),
                    }
                }
                BinOp::Pow => self.int_pow(x, y, span),
                _ => unreachable!(),
            };
        }
        // at least one decimal: compute in floating point
        let to_f = |v: &Value| -> Option<f64> {
            match v {
                Value::Int(i) => i.to_f64(),
                Value::Dec(f) => Some(*f),
                _ => None,
            }
        };
        let (x, y) = match (to_f(a), to_f(b)) {
            (Some(x), Some(y)) => (x, y),
            _ => return Err(overflow(span, self)),
        };
        let r = match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div | BinOp::FloorDiv | BinOp::Mod => {
                if y == 0.0 {
                    return Err(self
                        .err("E2001", "Division by zero", span)
                        .recoverable("MathError")
                        .help("check that the divisor is not 0 first")
                        .boxed());
                }
                match op {
                    BinOp::Div => x / y,
                    BinOp::FloorDiv => float_floor_div(x, y),
                    _ => float_mod(x, y),
                }
            }
            BinOp::Pow => self.float_pow(x, y, span)?,
            _ => unreachable!(),
        };
        if !r.is_finite() {
            return Err(overflow(span, self));
        }
        Ok(Value::Dec(r))
    }

    fn float_pow(&self, x: f64, y: f64, span: Option<Span>) -> R<f64> {
        if x == 0.0 && y < 0.0 {
            return Err(self.math_err("E2001", "Zero cannot be raised to a negative power", span));
        }
        if x < 0.0 && y.fract() != 0.0 {
            return Err(self.math_err("E2008", "The result is not a real number", span));
        }
        Ok(x.powf(y))
    }

    fn int_pow(&self, a: &Int, b: &Int, span: Option<Span>) -> R<Value> {
        if b.is_pos() && a.abs() > Int::Small(1) {
            let bits = a.bit_length() as u128;
            let too_big = match b.to_i64() {
                Some(e) => bits * (e as u128) > 1_000_000,
                None => true,
            };
            if too_big {
                return Err(overflow(span, self));
            }
        }
        if a.is_zero() && b.is_neg() {
            return Err(self.math_err("E2001", "Zero cannot be raised to a negative power", span));
        }
        if b.is_neg() {
            return match (a.to_f64(), b.to_f64()) {
                (Some(x), Some(y)) => {
                    let r = self.float_pow(x, y, span)?;
                    if r.is_finite() {
                        Ok(Value::Dec(r))
                    } else {
                        Err(overflow(span, self))
                    }
                }
                _ => Err(overflow(span, self)),
            };
        }
        if b.is_zero() {
            return Ok(Value::int(1));
        }
        // b > 0
        let one = Int::Small(1);
        if a.is_zero() || *a == one {
            return Ok(Value::Int(a.clone()));
        }
        if *a == one.neg() {
            return Ok(Value::Int(if b.is_even() { one } else { one.neg() }));
        }
        let e = b.to_i64().and_then(|e| u32::try_from(e).ok()).ok_or_else(|| overflow(span, self))?;
        Ok(Value::Int(a.pow(e)))
    }

    fn check_index(&self, idx: &Value, len: usize, seq_type: &str, span: Span) -> R<usize> {
        let i = match idx {
            Value::Int(i) => i,
            other => {
                return Err(self
                    .err("E3003", format!("Index must be an integer, but got {}", type_name(other)), Some(span))
                    .boxed())
            }
        };
        match i.to_usize() {
            Some(u) if u < len => Ok(u),
            _ => Err(self.index_error(i, seq_type, &len.to_string(), span)),
        }
    }

    fn index_error(&self, i: &Int, seq_type: &str, len: &str, span: Span) -> Box<ArcanaError> {
        self.err(
            "E2002",
            format!("Index {} is out of range for a {} of length {}", i, seq_type, len),
            Some(span),
        )
        .recoverable("IndexError")
        .help("valid indexes run from 0 to length - 1")
        .boxed()
    }

    fn e_index(&mut self, span: Span, obj: &Value, idx: &Value) -> R<Value> {
        match obj {
            Value::List(l) => {
                let len = l.borrow().len();
                let i = self.check_index(idx, len, "list", span)?;
                let v = l.borrow()[i].clone();
                Ok(v)
            }
            Value::Text(s) => {
                let len = s.chars().count();
                let i = self.check_index(idx, len, "text", span)?;
                Ok(Value::text(s.chars().nth(i).expect("checked").to_string()))
            }
            Value::Range(r) => {
                let len = r.len();
                match idx {
                    Value::Int(i) => {
                        if i.is_neg() || *i >= len {
                            Err(self.index_error(i, "range", &len.to_string(), span))
                        } else {
                            Ok(Value::Int(r.get(i)))
                        }
                    }
                    other => Err(self
                        .err("E3003", format!("Index must be an integer, but got {}", type_name(other)), Some(span))
                        .boxed()),
                }
            }
            Value::Map(m) => {
                let key = self.check_key(idx, span)?;
                let found = m.borrow().get(&key).cloned();
                match found {
                    Some(v) => Ok(v),
                    None => Err(self
                        .err("E2005", format!("Key {} not found in map", to_text(idx, true)), Some(span))
                        .recoverable("KeyError")
                        .help("test first with: key in map")
                        .boxed()),
                }
            }
            other => Err(self
                .err("E3003", format!("Cannot index a {}", type_name(other)), Some(span))
                .help("only lists, text, maps and ranges can be indexed")
                .boxed()),
        }
    }

    // ---- calls -------------------------------------------------------------------
    fn arity_error(&self, name: &str, lo: usize, hi: Option<usize>, got: usize, span: Option<Span>, sig: &str) -> Box<ArcanaError> {
        let plural = |n: usize| if n != 1 { "s" } else { "" };
        let want = match hi {
            None => format!("at least {} argument{}", lo, plural(lo)),
            Some(h) if h == lo => format!("{} argument{}", lo, plural(lo)),
            Some(h) => format!("{} to {} arguments", lo, h),
        };
        self.err("E3004", format!("Spell '{}' expects {} but got {}", name, want, got), span)
            .help(format!("call it like {}", sig))
            .boxed()
    }

    pub fn call_value(&mut self, callee: &Value, args: Vec<Value>, span: Option<Span>) -> R<Value> {
        match callee {
            Value::Spell(s) => self.call_spell(s, args, span),
            Value::Builtin(b) => {
                let n = args.len();
                if n < b.min_args || b.max_args.map_or(false, |m| n > m) {
                    return Err(self.arity_error(b.name, b.min_args, b.max_args, n, span, &format!("{}(...)", b.name)));
                }
                let func = b.func;
                match func(self, &args) {
                    Ok(v) => Ok(v),
                    Err(mut e) => {
                        if e.span.is_none() {
                            e.span = span;
                            self.attach(&mut e);
                        }
                        Err(e)
                    }
                }
            }
            Value::Module(m) => {
                let mut e = self
                    .err("E3005", format!("Cannot call a value of type {}", type_name(callee)), span)
                    .help(format!("'{}' is an imported grimoire; call its functions as {}.name(...)", m.name, m.name));
                if crate::runtime::is_builtin(&m.name) {
                    e = e.note(format!(
                        "this import hides the built-in spell '{}'; import the grimoire under another name: import {} as x",
                        m.name, m.name
                    ));
                }
                Err(e.boxed())
            }
            other => Err(self
                .err("E3005", format!("Cannot call a value of type {}", type_name(other)), span)
                .help("only spells can be called with ( )")
                .boxed()),
        }
    }

    fn call_spell(&mut self, spell: &Rc<Spell>, args: Vec<Value>, span: Option<Span>) -> R<Value> {
        let n_params = spell.params.len();
        if args.len() < spell.min_args() || args.len() > n_params {
            let names: Vec<&str> = spell.params.iter().map(|p| p.name.as_str()).collect();
            let sig = format!("{}({})", spell.name, names.join(", "));
            return Err(self.arity_error(&spell.name, spell.min_args(), Some(n_params), args.len(), span, &sig));
        }
        if self.depth >= self.max_depth {
            return Err(self
                .err("E2007", format!("Maximum call depth ({}) exceeded", self.max_depth), span)
                .help("check that your recursive spell has a base case that stops the recursion")
                .boxed());
        }
        let env = Env::new(Some(spell.closure.clone()));
        self.depth += 1;
        self.module_stack.push(spell.module.clone());
        let result = self.run_spell_body(spell, args, &env);
        self.module_stack.pop();
        self.depth -= 1;
        result
    }

    fn run_spell_body(&mut self, spell: &Rc<Spell>, args: Vec<Value>, env: &Rc<Env>) -> R<Value> {
        let mut args = args.into_iter();
        for p in spell.params.iter() {
            let v = match args.next() {
                Some(v) => v,
                None => match &p.default {
                    Some(d) => self.eval(d, env)?,
                    None => Value::Null,
                },
            };
            env.set_local(&p.name, v);
        }
        match self.exec_block(&spell.body, env)? {
            Flow::Return(v) => Ok(v),
            _ => Ok(Value::Null),
        }
    }
}
