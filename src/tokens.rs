//! Token definitions for Arcana.

use crate::errors::Span;
use crate::int::Int;
use crate::values::float_repr;

/// Reserved words in v0.1.  Booleans are On / Off (capitalised).
pub const KEYWORDS: [&str; 24] = [
    "spell", "bind", "cast", "if", "else", "while", "for", "in", "return", "break", "continue", "try", "catch",
    "finally", "import", "from", "as", "On", "Off", "null", "and", "or", "not",
    // reserved for possible future syntax, not implemented in v0.1
    "count",
];

pub const FUTURE_RESERVED: [&str; 3] = ["count", "repeat", "times"];

pub fn is_keyword(w: &str) -> bool {
    KEYWORDS.contains(&w) || FUTURE_RESERVED.contains(&w)
}

pub fn is_future_reserved(w: &str) -> bool {
    FUTURE_RESERVED.contains(&w)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Int,
    Decimal,
    Text,
    Ident,
    Kw,
    Op,
    Newline,
    Eof,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Int => "INT",
            Kind::Decimal => "DECIMAL",
            Kind::Text => "TEXT",
            Kind::Ident => "IDENT",
            Kind::Kw => "KW",
            Kind::Op => "OP",
            Kind::Newline => "NEWLINE",
            Kind::Eof => "EOF",
        }
    }
}

#[derive(Clone, Debug)]
pub enum TokVal {
    None,
    Str(String),
    Int(Int),
    Dec(f64),
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: Kind,
    pub value: TokVal,
    pub line: usize,
    pub col: usize,
    pub length: usize,
}

impl Token {
    pub fn span(&self) -> Span {
        Span::new(self.line, self.col, self.length)
    }

    /// The string payload (identifiers, keywords, operators, text); empty otherwise.
    pub fn text(&self) -> &str {
        match &self.value {
            TokVal::Str(s) => s,
            _ => "",
        }
    }

    /// Display form of the value, like Python's `str(value)`.
    pub fn display(&self) -> String {
        match &self.value {
            TokVal::None => "None".to_string(),
            TokVal::Str(s) => s.clone(),
            TokVal::Int(i) => i.to_string(),
            TokVal::Dec(f) => float_repr(*f),
        }
    }
}
