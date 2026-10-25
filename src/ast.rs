//! AST node types.  Every node carries a Span for diagnostics.

use crate::errors::Span;
use crate::values::Value;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::In => "in",
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::FloorDiv => "//",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
        }
    }

    pub fn from_symbol(s: &str) -> Option<BinOp> {
        Some(match s {
            "==" => BinOp::Eq,
            "!=" => BinOp::Ne,
            "<" => BinOp::Lt,
            "<=" => BinOp::Le,
            ">" => BinOp::Gt,
            ">=" => BinOp::Ge,
            "in" => BinOp::In,
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            "//" => BinOp::FloorDiv,
            "%" => BinOp::Mod,
            "**" => BinOp::Pow,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub enum Expr {
    Literal { span: Span, value: Value },
    Name { span: Span, name: String },
    List { span: Span, items: Vec<Expr> },
    Map { span: Span, entries: Vec<(Expr, Expr)> },
    Unary { span: Span, op: UnOp, operand: Box<Expr> },
    Binary { span: Span, op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Logical { span: Span, and: bool, left: Box<Expr>, right: Box<Expr> },
    Call { span: Span, callee: Box<Expr>, args: Vec<Expr> },
    Index { span: Span, obj: Box<Expr>, index: Box<Expr> },
    Member { span: Span, obj: Box<Expr>, name: String },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal { span, .. }
            | Expr::Name { span, .. }
            | Expr::List { span, .. }
            | Expr::Map { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Logical { span, .. }
            | Expr::Call { span, .. }
            | Expr::Index { span, .. }
            | Expr::Member { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub struct Param {
    pub span: Span,
    pub name: String,
    pub default: Option<Expr>,
}

#[derive(Debug)]
pub enum Stmt {
    Import { span: Span, module: String, alias: Option<String>, names: Option<Vec<(String, Span)>> },
    SpellDecl { span: Span, name: String, params: Rc<Vec<Param>>, body: Rc<Vec<Stmt>> },
    Bind { span: Span, name: String, value: Expr },
    Assign { span: Span, target: Expr, value: Expr },
    Cast { span: Span, value: Expr },
    If { span: Span, branches: Vec<(Expr, Vec<Stmt>)>, else_body: Option<Vec<Stmt>> },
    While { span: Span, cond: Expr, body: Vec<Stmt> },
    For { span: Span, var: String, iterable: Expr, body: Vec<Stmt> },
    Return { span: Span, value: Option<Expr> },
    Break { span: Span },
    Continue { span: Span },
    Try { span: Span, body: Vec<Stmt>, catch_name: Option<String>, catch_body: Option<Vec<Stmt>>, finally_body: Option<Vec<Stmt>> },
    ExprStmt { span: Span, expr: Expr },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Import { span, .. }
            | Stmt::SpellDecl { span, .. }
            | Stmt::Bind { span, .. }
            | Stmt::Assign { span, .. }
            | Stmt::Cast { span, .. }
            | Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Break { span }
            | Stmt::Continue { span }
            | Stmt::Try { span, .. }
            | Stmt::ExprStmt { span, .. } => *span,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
