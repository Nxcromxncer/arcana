//! Recursive-descent parser for Arcana v0.1.
//!
//! Newlines end statements.  Inside (), [] and map {} literals newlines are
//! ignored (tracked by `nest`); inside blocks they are significant.

use crate::ast::*;
use crate::errors::{ArcanaError, Span};
use crate::int::Int;
use crate::lexer::{normalize, Lexer};
use crate::tokens::{is_future_reserved, Kind, TokVal, Token};
use crate::values::Value;
use std::rc::Rc;
use std::sync::Arc;

type R<T> = Result<T, Box<ArcanaError>>;

const MAX_NESTING: usize = 20_000;

pub fn parse_source(source: &str, filename: &str) -> R<Program> {
    let source = normalize(source);
    let tokens = Lexer::new(&source, filename).tokenize()?;
    Parser::new(tokens, filename, Arc::from(source.as_str())).parse_program()
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    nest: usize,
    loops: usize,
    spells: usize,
    depth: usize,
    filename: String,
    source: Arc<str>,
}

fn describe(tok: &Token) -> String {
    match tok.kind {
        Kind::Newline => "the end of the line".to_string(),
        Kind::Eof => "the end of the file".to_string(),
        Kind::Text => "a text value".to_string(),
        _ => format!("'{}'", tok.display()),
    }
}

impl Parser {
    pub fn new(toks: Vec<Token>, filename: &str, source: Arc<str>) -> Parser {
        Parser { toks, pos: 0, nest: 0, loops: 0, spells: 0, depth: 0, filename: filename.to_string(), source }
    }

    // ---- helpers -----------------------------------------------------------
    fn error(&self, code: &str, msg: impl Into<String>, span: Span) -> ArcanaError {
        let mut e = ArcanaError::new(code, msg, Some(span));
        e.attach(&self.filename, &self.source);
        e
    }

    fn peek(&mut self) -> Token {
        if self.nest > 0 {
            while self.toks[self.pos].kind == Kind::Newline {
                self.pos += 1;
            }
        }
        self.toks[self.pos].clone()
    }

    fn advance(&mut self) -> Token {
        let t = self.peek();
        if t.kind != Kind::Eof {
            self.pos += 1;
        }
        t
    }

    fn at(&mut self, kind: Kind, value: Option<&str>) -> bool {
        let t = self.peek();
        t.kind == kind && value.map_or(true, |v| t.text() == v)
    }

    fn at_op(&mut self, v: &str) -> bool {
        self.at(Kind::Op, Some(v))
    }

    fn at_kw(&mut self, v: &str) -> bool {
        self.at(Kind::Kw, Some(v))
    }

    fn match_tok(&mut self, kind: Kind, value: &str) -> bool {
        if self.at(kind, Some(value)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: Kind, value: &str, help: Option<&str>) -> R<Token> {
        let t = self.peek();
        if t.kind == kind && t.text() == value {
            return Ok(self.advance());
        }
        let mut e = self.error("E4001", format!("Expected '{}' but found {}", value, describe(&t)), t.span());
        if let Some(h) = help {
            e = e.help(h);
        }
        Err(e.boxed())
    }

    fn skip_newlines(&mut self) {
        while self.toks[self.pos].kind == Kind::Newline {
            self.pos += 1;
        }
    }

    fn end_statement(&mut self) -> R<()> {
        let t = self.peek();
        if t.kind == Kind::Newline {
            self.advance();
            Ok(())
        } else if t.kind == Kind::Eof || (t.kind == Kind::Op && t.text() == "}") {
            Ok(())
        } else {
            Err(self
                .error("E4001", format!("Unexpected {} after the end of a statement", describe(&t)), t.span())
                .help("put each statement on its own line")
                .boxed())
        }
    }

    fn ident(&mut self) -> R<Token> {
        let t = self.peek();
        if t.kind == Kind::Ident {
            return Ok(self.advance());
        }
        if t.kind == Kind::Kw {
            if is_future_reserved(t.text()) {
                return Err(self
                    .error("E4006", format!("'{}' is reserved for future use and cannot be a name", t.text()), t.span())
                    .help("choose a different name")
                    .boxed());
            }
            return Err(self
                .error("E4006", format!("'{}' is a reserved word and cannot be used as a name", t.text()), t.span())
                .help("choose a different name")
                .boxed());
        }
        Err(self.error("E4001", format!("Expected a name but found {}", describe(&t)), t.span()).boxed())
    }

    fn enter(&mut self) -> R<()> {
        self.depth += 1;
        if self.depth > MAX_NESTING {
            return Err(self
                .error("E2007", "Expression or call nesting is too deep", self.toks[self.pos].span())
                .help("simplify the nesting or check for runaway recursion")
                .boxed());
        }
        Ok(())
    }

    // ---- program / blocks ----------------------------------------------------
    pub fn parse_program(&mut self) -> R<Program> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at(Kind::Eof, None) {
            let t = self.peek();
            if t.kind == Kind::Kw && (t.text() == "import" || t.text() == "from") {
                stmts.push(self.parse_import()?);
            } else if t.kind == Kind::Kw && t.text() == "spell" {
                stmts.push(self.parse_spell()?);
            } else if t.kind == Kind::Kw && t.text() == "bind" {
                stmts.push(self.parse_bind()?);
            } else if t.kind == Kind::Op && t.text() == "}" {
                return Err(self.error("E4001", "Unexpected '}'", t.span()).help("there is no open block to close").boxed());
            } else {
                return Err(self
                    .error("E4002", "Only declarations are allowed at the top level of a grimoire", t.span())
                    .help("put this statement inside 'spell main() { ... }'")
                    .note("top-level code may only use 'spell', 'bind' and 'import'")
                    .boxed());
            }
            self.end_statement()?;
            self.skip_newlines();
        }
        Ok(Program { stmts })
    }

    fn parse_block(&mut self) -> R<Vec<Stmt>> {
        self.expect(Kind::Op, "{", Some("blocks are written between '{' and '}'"))?;
        let mut stmts = Vec::new();
        self.skip_newlines();
        while !self.at_op("}") {
            if self.at(Kind::Eof, None) {
                let t = self.peek();
                return Err(self
                    .error("E4001", "Unclosed block: expected '}' before the end of the file", t.span())
                    .help("add a closing '}'")
                    .boxed());
            }
            stmts.push(self.parse_statement()?);
            self.end_statement()?;
            self.skip_newlines();
        }
        self.advance();
        Ok(stmts)
    }

    // ---- statements ----------------------------------------------------------
    fn parse_statement(&mut self) -> R<Stmt> {
        let t = self.peek();
        if t.kind == Kind::Kw {
            let v = t.text().to_string();
            match v.as_str() {
                "bind" => return self.parse_bind(),
                "cast" => {
                    self.advance();
                    let value = self.parse_expression()?;
                    return Ok(Stmt::Cast { span: t.span(), value });
                }
                "if" => return self.parse_if(),
                "while" => return self.parse_while(),
                "for" => return self.parse_for(),
                "return" => return self.parse_return(),
                "break" | "continue" => {
                    self.advance();
                    if self.loops == 0 {
                        return Err(self
                            .error("E2003", format!("'{}' can only be used inside a loop", v), t.span())
                            .help("move it into a 'for' or 'while' block")
                            .boxed());
                    }
                    return Ok(if v == "break" { Stmt::Break { span: t.span() } } else { Stmt::Continue { span: t.span() } });
                }
                "try" => return self.parse_try(),
                "spell" => return self.parse_spell(),
                "import" | "from" => {
                    return Err(self
                        .error("E4001", "Imports must be at the top level of a grimoire", t.span())
                        .help("move this import to the top of the file")
                        .boxed());
                }
                "else" => {
                    return Err(self
                        .error("E4001", "'else' without a matching 'if'", t.span())
                        .help("'else' must follow an 'if' block on the same line as the closing brace: } else {")
                        .boxed());
                }
                "catch" | "finally" => {
                    return Err(self
                        .error("E4001", format!("'{}' without a matching 'try'", v), t.span())
                        .help(format!("'{}' must follow a 'try' block on the same line as the closing brace", v))
                        .boxed());
                }
                _ => {}
            }
        }
        let expr = self.parse_expression()?;
        if self.at_op("=") {
            if !matches!(expr, Expr::Name { .. } | Expr::Index { .. }) {
                let eq = self.peek();
                return Err(self
                    .error("E4004", "Cannot assign to this expression", eq.span())
                    .help("you can assign to a name or to a list/map element, e.g. x = 1 or items[0] = 1")
                    .boxed());
            }
            self.advance();
            self.skip_newlines();
            let span = expr.span();
            let value = self.parse_expression()?;
            return Ok(Stmt::Assign { span, target: expr, value });
        }
        if matches!(expr, Expr::Call { .. }) {
            return Ok(Stmt::ExprStmt { span: expr.span(), expr });
        }
        Err(self
            .error("E4004", "This expression has no effect", expr.span())
            .help("use 'cast' to show a value, 'bind' to declare a variable, or call a spell")
            .boxed())
    }

    fn parse_bind(&mut self) -> R<Stmt> {
        self.advance();
        let name = self.ident()?;
        self.expect(Kind::Op, "=", Some("give the binding a value, e.g. bind mana = 100"))?;
        self.skip_newlines();
        let value = self.parse_expression()?;
        Ok(Stmt::Bind { span: name.span(), name: name.text().to_string(), value })
    }

    fn parse_if(&mut self) -> R<Stmt> {
        let kw = self.advance();
        let cond = self.parse_expression()?;
        let body = self.parse_block()?;
        let mut branches = vec![(cond, body)];
        let mut else_body = None;
        while self.at_kw("else") {
            self.advance();
            if self.match_tok(Kind::Kw, "if") {
                let c = self.parse_expression()?;
                let b = self.parse_block()?;
                branches.push((c, b));
            } else {
                else_body = Some(self.parse_block()?);
                break;
            }
        }
        Ok(Stmt::If { span: kw.span(), branches, else_body })
    }

    fn parse_while(&mut self) -> R<Stmt> {
        let kw = self.advance();
        let cond = self.parse_expression()?;
        self.loops += 1;
        let body = self.parse_block()?;
        self.loops -= 1;
        Ok(Stmt::While { span: kw.span(), cond, body })
    }

    fn parse_for(&mut self) -> R<Stmt> {
        self.advance();
        let var = self.ident()?;
        self.expect(Kind::Kw, "in", Some("loops are written: for item in collection { ... }"))?;
        let iterable = self.parse_expression()?;
        self.loops += 1;
        let body = self.parse_block()?;
        self.loops -= 1;
        Ok(Stmt::For { span: var.span(), var: var.text().to_string(), iterable, body })
    }

    fn parse_return(&mut self) -> R<Stmt> {
        let kw = self.advance();
        if self.spells == 0 {
            return Err(self
                .error("E4005", "'return' can only be used inside a spell", kw.span())
                .help("put the return statement inside a 'spell' body")
                .boxed());
        }
        let t = self.peek();
        if t.kind == Kind::Newline || t.kind == Kind::Eof || (t.kind == Kind::Op && t.text() == "}") {
            return Ok(Stmt::Return { span: kw.span(), value: None });
        }
        let value = self.parse_expression()?;
        Ok(Stmt::Return { span: kw.span(), value: Some(value) })
    }

    fn parse_try(&mut self) -> R<Stmt> {
        let kw = self.advance();
        let body = self.parse_block()?;
        let (mut cname, mut cbody, mut fbody) = (None, None, None);
        if self.match_tok(Kind::Kw, "catch") {
            cname = Some(self.ident()?.text().to_string());
            cbody = Some(self.parse_block()?);
        }
        if self.match_tok(Kind::Kw, "finally") {
            fbody = Some(self.parse_block()?);
        }
        if cbody.is_none() && fbody.is_none() {
            return Err(self
                .error("E4001", "'try' needs a 'catch' or 'finally' block", kw.span())
                .help("add: catch error { ... }")
                .boxed());
        }
        Ok(Stmt::Try { span: kw.span(), body, catch_name: cname, catch_body: cbody, finally_body: fbody })
    }

    fn parse_spell(&mut self) -> R<Stmt> {
        self.advance();
        let name = self.ident()?;
        self.expect(Kind::Op, "(", None)?;
        self.nest += 1;
        let mut params: Vec<Param> = Vec::new();
        let mut seen: Vec<String> = Vec::new();
        let mut has_default = false;
        while !self.at_op(")") {
            let p = self.ident()?;
            if seen.iter().any(|s| s == p.text()) {
                return Err(self
                    .error("E3002", format!("Duplicate parameter '{}'", p.text()), p.span())
                    .help("give each parameter a unique name")
                    .boxed());
            }
            seen.push(p.text().to_string());
            let mut default = None;
            if self.match_tok(Kind::Op, "=") {
                default = Some(self.parse_expression()?);
                has_default = true;
            } else if has_default {
                return Err(self
                    .error("E4001", "A parameter without a default cannot follow one with a default", p.span())
                    .help("move required parameters before optional ones")
                    .boxed());
            }
            params.push(Param { span: p.span(), name: p.text().to_string(), default });
            if !self.match_tok(Kind::Op, ",") {
                break;
            }
        }
        self.expect(Kind::Op, ")", None)?;
        self.nest -= 1;
        let saved_loops = std::mem::replace(&mut self.loops, 0);
        self.spells += 1;
        let body = self.parse_block()?;
        self.spells -= 1;
        self.loops = saved_loops;
        Ok(Stmt::SpellDecl { span: name.span(), name: name.text().to_string(), params: Rc::new(params), body: Rc::new(body) })
    }

    fn dotted(&mut self) -> R<(Token, String)> {
        let first = self.ident()?;
        let mut parts = vec![first.text().to_string()];
        while self.match_tok(Kind::Op, ".") {
            parts.push(self.ident()?.text().to_string());
        }
        Ok((first, parts.join(".")))
    }

    fn parse_import(&mut self) -> R<Stmt> {
        let kw = self.advance();
        if kw.text() == "import" {
            let (first, module) = self.dotted()?;
            let mut alias = None;
            if self.match_tok(Kind::Kw, "as") {
                alias = Some(self.ident()?.text().to_string());
            }
            return Ok(Stmt::Import { span: first.span(), module, alias, names: None });
        }
        let (first, module) = self.dotted()?;
        self.expect(Kind::Kw, "import", Some("write: from grimoire import name"))?;
        let mut names = Vec::new();
        loop {
            let n = self.ident()?;
            names.push((n.text().to_string(), n.span()));
            if !self.match_tok(Kind::Op, ",") {
                break;
            }
        }
        Ok(Stmt::Import { span: first.span(), module, alias: None, names: Some(names) })
    }

    // ---- expressions -----------------------------------------------------------
    pub fn parse_expression(&mut self) -> R<Expr> {
        self.enter()?;
        let r = self.parse_or();
        self.depth -= 1;
        r
    }

    fn parse_or(&mut self) -> R<Expr> {
        let mut left = self.parse_and()?;
        while self.at_kw("or") {
            let op = self.advance();
            self.skip_newlines();
            let right = self.parse_and()?;
            left = Expr::Logical { span: op.span(), and: false, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> R<Expr> {
        let mut left = self.parse_not()?;
        while self.at_kw("and") {
            let op = self.advance();
            self.skip_newlines();
            let right = self.parse_not()?;
            left = Expr::Logical { span: op.span(), and: true, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> R<Expr> {
        if self.at_kw("not") {
            let op = self.advance();
            self.enter()?;
            let operand = self.parse_not();
            self.depth -= 1;
            return Ok(Expr::Unary { span: op.span(), op: UnOp::Not, operand: Box::new(operand?) });
        }
        self.parse_comparison()
    }

    fn is_comp(&mut self) -> bool {
        let t = self.peek();
        (t.kind == Kind::Op && matches!(t.text(), "==" | "!=" | "<" | "<=" | ">" | ">="))
            || (t.kind == Kind::Kw && t.text() == "in")
    }

    fn parse_comparison(&mut self) -> R<Expr> {
        let left = self.parse_additive()?;
        if self.is_comp() {
            let op = self.advance();
            self.skip_newlines();
            let right = self.parse_additive()?;
            if self.is_comp() {
                let t = self.peek();
                return Err(self
                    .error("E4003", "Comparisons cannot be chained", t.span())
                    .help("combine them with 'and', e.g. a < b and b < c")
                    .boxed());
            }
            let bop = BinOp::from_symbol(op.text()).expect("comparison operator");
            return Ok(Expr::Binary { span: op.span(), op: bop, left: Box::new(left), right: Box::new(right) });
        }
        Ok(left)
    }

    fn binary_loop(&mut self, sub: fn(&mut Parser) -> R<Expr>, ops: &[&str]) -> R<Expr> {
        let mut left = sub(self)?;
        loop {
            let t = self.peek();
            if !(t.kind == Kind::Op && ops.contains(&t.text())) {
                return Ok(left);
            }
            let op = self.advance();
            self.skip_newlines();
            let right = sub(self)?;
            let bop = BinOp::from_symbol(op.text()).expect("binary operator");
            left = Expr::Binary { span: op.span(), op: bop, left: Box::new(left), right: Box::new(right) };
        }
    }

    fn parse_additive(&mut self) -> R<Expr> {
        self.binary_loop(Parser::parse_term, &["+", "-"])
    }

    fn parse_term(&mut self) -> R<Expr> {
        self.binary_loop(Parser::parse_unary, &["*", "/", "//", "%"])
    }

    fn parse_unary(&mut self) -> R<Expr> {
        if self.at_op("-") {
            let op = self.advance();
            self.enter()?;
            let operand = self.parse_unary();
            self.depth -= 1;
            return Ok(Expr::Unary { span: op.span(), op: UnOp::Neg, operand: Box::new(operand?) });
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> R<Expr> {
        let base = self.parse_postfix()?;
        if self.at_op("**") {
            let op = self.advance();
            self.skip_newlines();
            let right = self.parse_unary()?;
            return Ok(Expr::Binary { span: op.span(), op: BinOp::Pow, left: Box::new(base), right: Box::new(right) });
        }
        Ok(base)
    }

    fn parse_postfix(&mut self) -> R<Expr> {
        let mut node = self.parse_primary()?;
        loop {
            let t = self.peek();
            if t.kind != Kind::Op {
                return Ok(node);
            }
            match t.text() {
                "(" => {
                    self.advance();
                    self.nest += 1;
                    let mut args = Vec::new();
                    while !self.at_op(")") {
                        args.push(self.parse_expression()?);
                        if !self.match_tok(Kind::Op, ",") {
                            break;
                        }
                    }
                    self.expect(Kind::Op, ")", Some("close the call with ')'"))?;
                    self.nest -= 1;
                    node = Expr::Call { span: node.span(), callee: Box::new(node), args };
                }
                "[" => {
                    self.advance();
                    self.nest += 1;
                    let idx = self.parse_expression()?;
                    self.expect(Kind::Op, "]", Some("close the index with ']'"))?;
                    self.nest -= 1;
                    node = Expr::Index { span: t.span(), obj: Box::new(node), index: Box::new(idx) };
                }
                "." => {
                    self.advance();
                    let name = self.ident()?;
                    node = Expr::Member { span: name.span(), obj: Box::new(node), name: name.text().to_string() };
                }
                _ => return Ok(node),
            }
        }
    }

    fn parse_primary(&mut self) -> R<Expr> {
        let t = self.peek();
        match t.kind {
            Kind::Int | Kind::Decimal | Kind::Text => {
                self.advance();
                let value = match &t.value {
                    TokVal::Int(i) => Value::Int(i.clone()),
                    TokVal::Dec(f) => Value::Dec(*f),
                    TokVal::Str(s) => Value::Text(Rc::from(s.as_str())),
                    TokVal::None => Value::Null,
                };
                return Ok(Expr::Literal { span: t.span(), value });
            }
            Kind::Kw if matches!(t.text(), "On" | "Off" | "null") => {
                self.advance();
                let value = match t.text() {
                    "On" => Value::Bool(true),
                    "Off" => Value::Bool(false),
                    _ => Value::Null,
                };
                return Ok(Expr::Literal { span: t.span(), value });
            }
            Kind::Ident => {
                self.advance();
                return Ok(Expr::Name { span: t.span(), name: t.text().to_string() });
            }
            Kind::Op if t.text() == "(" => {
                self.advance();
                self.nest += 1;
                let e = self.parse_expression()?;
                self.expect(Kind::Op, ")", Some("close the parenthesis with ')'"))?;
                self.nest -= 1;
                return Ok(e);
            }
            Kind::Op if t.text() == "[" => {
                self.advance();
                self.nest += 1;
                let mut items = Vec::new();
                while !self.at_op("]") {
                    items.push(self.parse_expression()?);
                    if !self.match_tok(Kind::Op, ",") {
                        break;
                    }
                }
                self.expect(Kind::Op, "]", Some("close the list with ']'"))?;
                self.nest -= 1;
                return Ok(Expr::List { span: t.span(), items });
            }
            Kind::Op if t.text() == "{" => {
                self.advance();
                self.nest += 1;
                let mut entries = Vec::new();
                while !self.at_op("}") {
                    let k = self.parse_expression()?;
                    self.expect(Kind::Op, ":", Some("map entries are written key: value, e.g. {\"mana\": 10}"))?;
                    let v = self.parse_expression()?;
                    entries.push((k, v));
                    if !self.match_tok(Kind::Op, ",") {
                        break;
                    }
                }
                self.expect(Kind::Op, "}", Some("close the map with '}'"))?;
                self.nest -= 1;
                return Ok(Expr::Map { span: t.span(), entries });
            }
            Kind::Kw => {
                if is_future_reserved(t.text()) {
                    return Err(self.error("E4006", format!("'{}' is reserved for future use", t.text()), t.span()).boxed());
                }
                return Err(self.error("E4001", format!("Expected an expression but found '{}'", t.text()), t.span()).boxed());
            }
            _ => {}
        }
        Err(self.error("E4001", format!("Expected an expression but found {}", describe(&t)), t.span()).boxed())
    }
}

#[allow(dead_code)]
fn _unused(_: Int) {}
