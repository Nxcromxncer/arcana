//! Lexer: source text -> tokens.
//!
//! Comments start with '?' and run to the end of the line (a '?' inside a
//! text literal is just a character).

use crate::errors::{ArcanaError, Span};
use crate::int::Int;
use crate::tokens::{is_keyword, Kind, TokVal, Token};
use std::sync::Arc;

type R<T> = Result<T, Box<ArcanaError>>;

const TWO: [&str; 6] = ["==", "!=", "<=", ">=", "//", "**"];
const ONE: &str = "+-*/%<>=()[]{},:.";
const MAX_INT_DIGITS: usize = 4300;

fn char_help(c: char) -> Option<&'static str> {
    Some(match c {
        '!' => "use 'not' for negation and '!=' for inequality",
        '#' => "comments start with '?'",
        ';' => "Arcana does not use semicolons; end a statement with a new line",
        '\'' => "text is written with double quotes, like \"hello\"",
        '&' => "use 'and' for logical conjunction",
        '|' => "use 'or' for logical disjunction",
        _ => return None,
    })
}

pub fn normalize(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n").trim_start_matches('\u{feff}').to_string()
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

pub struct Lexer {
    s: Vec<char>,
    src: Arc<str>,
    filename: String,
    i: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str, filename: &str) -> Lexer {
        let norm = normalize(source);
        Lexer {
            s: norm.chars().collect(),
            src: Arc::from(norm.as_str()),
            filename: filename.to_string(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn error(&self, code: &str, msg: String, line: usize, col: usize, length: usize) -> ArcanaError {
        let mut e = ArcanaError::new(code, msg, Some(Span::new(line, col, length)));
        e.attach(&self.filename, &self.src);
        e
    }

    fn adv(&mut self, n: usize) {
        self.i += n;
        self.col += n;
    }

    pub fn tokenize(&mut self) -> R<Vec<Token>> {
        let mut out = Vec::new();
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if c == ' ' || c == '\t' {
                self.adv(1);
            } else if c == '\n' {
                out.push(Token { kind: Kind::Newline, value: TokVal::Str("\n".into()), line: self.line, col: self.col, length: 1 });
                self.i += 1;
                self.line += 1;
                self.col = 1;
            } else if c == '?' {
                while self.i < self.s.len() && self.s[self.i] != '\n' {
                    self.adv(1);
                }
            } else if c == '"' {
                out.push(self.text()?);
            } else if is_digit(c) {
                out.push(self.number()?);
            } else if c.is_alphabetic() || c == '_' {
                out.push(self.word());
            } else {
                out.push(self.operator()?);
            }
        }
        out.push(Token { kind: Kind::Eof, value: TokVal::None, line: self.line, col: self.col, length: 0 });
        Ok(out)
    }

    fn word(&mut self) -> Token {
        let (start, col) = (self.i, self.col);
        while self.i < self.s.len() && (self.s[self.i].is_alphanumeric() || self.s[self.i] == '_') {
            self.adv(1);
        }
        let word: String = self.s[start..self.i].iter().collect();
        let len = self.i - start;
        let kind = if is_keyword(&word) { Kind::Kw } else { Kind::Ident };
        Token { kind, value: TokVal::Str(word), line: self.line, col, length: len }
    }

    fn number(&mut self) -> R<Token> {
        let (start, col, line) = (self.i, self.col, self.line);
        let n = self.s.len();
        while self.i < n && (is_digit(self.s[self.i]) || self.s[self.i] == '_') {
            self.adv(1);
        }
        let mut is_decimal = false;
        if self.i + 1 < n && self.s[self.i] == '.' && is_digit(self.s[self.i + 1]) {
            self.adv(1);
            while self.i < n && (is_digit(self.s[self.i]) || self.s[self.i] == '_') {
                self.adv(1);
            }
            is_decimal = true;
        }
        if self.i < n && (self.s[self.i] == 'e' || self.s[self.i] == 'E') {
            let mut j = self.i + 1;
            if j < n && (self.s[j] == '+' || self.s[j] == '-') {
                j += 1;
            }
            if j < n && is_digit(self.s[j]) {
                self.adv(j - self.i);
                while self.i < n && is_digit(self.s[self.i]) {
                    self.adv(1);
                }
                is_decimal = true;
            }
        }
        let mut raw: String = self.s[start..self.i].iter().collect();
        let bad_tail = self.i < n && (self.s[self.i].is_alphabetic() || self.s[self.i] == '_');
        if bad_tail || raw.ends_with('_') || raw.contains("__") || raw.contains("_.") || raw.contains("._") {
            while self.i < n && (self.s[self.i].is_alphanumeric() || self.s[self.i] == '_') {
                self.adv(1);
            }
            raw = self.s[start..self.i].iter().collect();
            let len = raw.chars().count();
            return Err(self
                .error("E1005", format!("Invalid number '{}'", raw), line, col, len)
                .help("numbers contain digits, an optional '.', and optional '_' between digits")
                .boxed());
        }
        let clean: String = raw.chars().filter(|c| *c != '_').collect();
        let len = raw.chars().count();
        if is_decimal {
            let value: f64 = clean.parse().unwrap_or(f64::INFINITY);
            if !value.is_finite() {
                return Err(self.error("E1005", format!("Decimal '{}' is too large", raw), line, col, len).boxed());
            }
            Ok(Token { kind: Kind::Decimal, value: TokVal::Dec(value), line, col, length: len })
        } else {
            let digits = clean.len();
            let value = if digits > MAX_INT_DIGITS { None } else { Int::parse(&clean) };
            match value {
                Some(v) => Ok(Token { kind: Kind::Int, value: TokVal::Int(v), line, col, length: len }),
                None => {
                    let head: String = raw.chars().take(20).collect();
                    Err(self.error("E1005", format!("Number '{}...' is too large", head), line, col, len).boxed())
                }
            }
        }
    }

    fn text(&mut self) -> R<Token> {
        let (line, col) = (self.line, self.col);
        self.adv(1);
        let mut buf = String::new();
        loop {
            if self.i >= self.s.len() || self.s[self.i] == '\n' {
                let len = std::cmp::max(1, self.col - col);
                return Err(self
                    .error("E1003", "Unterminated text".into(), line, col, len)
                    .help("add a closing \" to end the text")
                    .boxed());
            }
            let c = self.s[self.i];
            if c == '"' {
                self.adv(1);
                break;
            }
            if c == '\\' {
                let nxt = self.s.get(self.i + 1).copied();
                let simple = match nxt {
                    Some('n') => Some('\n'),
                    Some('t') => Some('\t'),
                    Some('\\') => Some('\\'),
                    Some('"') => Some('"'),
                    _ => None,
                };
                if let Some(ch) = simple {
                    buf.push(ch);
                    self.adv(2);
                    continue;
                }
                if nxt == Some('u') && self.s.get(self.i + 2) == Some(&'{') {
                    let end = (self.i + 3..self.s.len()).find(|&k| self.s[k] == '}');
                    let digits: String = match end {
                        Some(e) => self.s[self.i + 3..e].iter().collect(),
                        None => String::new(),
                    };
                    let parsed = if digits.is_empty() || digits.contains('\n') {
                        None
                    } else {
                        u32::from_str_radix(digits.trim(), 16).ok().and_then(char::from_u32)
                    };
                    match (parsed, end) {
                        (Some(ch), Some(e)) => {
                            buf.push(ch);
                            self.adv(e - self.i + 1);
                            continue;
                        }
                        _ => {
                            let len = match end {
                                Some(e) => std::cmp::max(2, e - self.i + 1),
                                None => 2,
                            };
                            return Err(self
                                .error("E1004", "Invalid unicode escape".into(), self.line, self.col, len)
                                .help("write \\u{...} with a hexadecimal code point, e.g. \\u{2728}")
                                .boxed());
                        }
                    }
                }
                let shown = nxt.map(|c| c.to_string()).unwrap_or_default();
                return Err(self
                    .error("E1004", format!("Invalid escape sequence '\\{}'", shown), self.line, self.col, 2)
                    .help(r#"valid escapes are \n \t \\ \" and \u{...}"#)
                    .boxed());
            }
            buf.push(c);
            self.adv(1);
        }
        let length = self.col - col;
        Ok(Token { kind: Kind::Text, value: TokVal::Str(buf), line, col, length })
    }

    fn operator(&mut self) -> R<Token> {
        let c = self.s[self.i];
        if self.i + 1 < self.s.len() {
            let two: String = [c, self.s[self.i + 1]].iter().collect();
            if TWO.contains(&two.as_str()) {
                let tok = Token { kind: Kind::Op, value: TokVal::Str(two), line: self.line, col: self.col, length: 2 };
                self.adv(2);
                return Ok(tok);
            }
        }
        if ONE.contains(c) {
            let tok = Token { kind: Kind::Op, value: TokVal::Str(c.to_string()), line: self.line, col: self.col, length: 1 };
            self.adv(1);
            return Ok(tok);
        }
        let mut e = self.error("E1002", format!("Unexpected character '{}'", c), self.line, self.col, 1);
        if let Some(h) = char_help(c) {
            e = e.help(h);
        }
        Err(e.boxed())
    }
}
