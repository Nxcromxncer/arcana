//! Arcana diagnostics.
//!
//! Every error has an identifier (E1xxx..E6xxx), a category, a source
//! location and optional help/note text.  Errors with `recoverable == true`
//! can be caught by a running program with `try`/`catch`.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub length: usize,
}

impl Span {
    pub fn new(line: usize, col: usize, length: usize) -> Span {
        Span { line, col, length }
    }
}

#[derive(Clone, Debug)]
pub struct ArcanaError {
    pub code: String,
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
    pub note: Option<String>,
    pub kind: String,
    pub category_override: Option<String>,
    pub recoverable: bool,
    pub filename: Option<String>,
    pub source: Option<Arc<str>>,
    /// Set by `system.exit(code)`: stops the program quietly with this exit status.
    pub exit_code: Option<i32>,
}

impl ArcanaError {
    pub fn new(code: &str, message: impl Into<String>, span: Option<Span>) -> ArcanaError {
        ArcanaError {
            code: code.to_string(),
            message: message.into(),
            span,
            help: None,
            note: None,
            kind: "Error".to_string(),
            category_override: None,
            recoverable: false,
            filename: None,
            source: None,
            exit_code: None,
        }
    }

    pub fn help(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }

    pub fn note(mut self, n: impl Into<String>) -> Self {
        self.note = Some(n.into());
        self
    }

    pub fn with_category(mut self, c: &str) -> Self {
        self.category_override = Some(c.to_string());
        self
    }

    /// Mark the error as catchable by `try`/`catch`, with the given kind.
    pub fn recoverable(mut self, kind: &str) -> Self {
        self.recoverable = true;
        self.kind = kind.to_string();
        self
    }

    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = Some(code);
        self
    }

    pub fn boxed(self) -> Box<ArcanaError> {
        Box::new(self)
    }

    pub fn category(&self) -> String {
        if let Some(c) = &self.category_override {
            return c.clone();
        }
        match self.code.get(..2) {
            Some("E1") => "lexical",
            Some("E2") => "runtime",
            Some("E3") => "type",
            Some("E4") => "syntax",
            Some("E5") => "import",
            Some("E6") => "permission",
            _ => "error",
        }
        .to_string()
    }

    /// Remember which file the error belongs to (first writer wins).
    pub fn attach(&mut self, filename: &str, source: &Arc<str>) {
        if self.filename.is_none() {
            self.filename = Some(filename.to_string());
            self.source = Some(source.clone());
        }
    }

    pub fn render(&self) -> String {
        let mut out = vec![format!("error {}: {}", self.code, self.message)];
        let fname = self.filename.clone().unwrap_or_else(|| "<unknown>".to_string());
        if let Some(sp) = self.span {
            let ln = sp.line;
            let w = ln.to_string().len();
            let pad = " ".repeat(w + 1);
            out.push(format!("{}--> {}:{}:{}", pad, fname, ln, sp.col));
            let mut text: Option<String> = None;
            if let Some(src) = &self.source {
                if let Some(l) = src.split('\n').nth(ln.wrapping_sub(1)) {
                    if ln >= 1 {
                        text = Some(l.trim_end_matches('\r').replace('\t', " "));
                    }
                }
            }
            if let Some(t) = text {
                out.push(format!("{}|", pad));
                out.push(format!("{:>w$} | {}", ln, t, w = w));
                out.push(format!(
                    "{}| {}{}",
                    pad,
                    " ".repeat(sp.col.saturating_sub(1)),
                    "^".repeat(sp.length.max(1))
                ));
            }
        } else {
            out.push(format!("  --> {}", fname));
        }
        if let Some(h) = &self.help {
            out.push(format!("help: {}", h));
        }
        if let Some(n) = &self.note {
            out.push(format!("note: {}", n));
        }
        out.join("\n")
    }
}

impl fmt::Display for ArcanaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

impl std::error::Error for ArcanaError {}

// ---- difflib-compatible fuzzy matching (for "did you mean ...?") -----------------

fn longest_match(a: &[char], b: &[char], alo: usize, ahi: usize, blo: usize, bhi: usize) -> (usize, usize, usize) {
    let (mut besti, mut bestj, mut bestsize) = (alo, blo, 0usize);
    let mut prev: HashMap<usize, usize> = HashMap::new();
    for i in alo..ahi {
        let mut cur: HashMap<usize, usize> = HashMap::new();
        for j in blo..bhi {
            if a[i] == b[j] {
                let k = if j > 0 { prev.get(&(j - 1)).copied().unwrap_or(0) } else { 0 } + 1;
                cur.insert(j, k);
                if k > bestsize {
                    besti = i + 1 - k;
                    bestj = j + 1 - k;
                    bestsize = k;
                }
            }
        }
        prev = cur;
    }
    (besti, bestj, bestsize)
}

fn matching_total(a: &[char], b: &[char], alo: usize, ahi: usize, blo: usize, bhi: usize) -> usize {
    let (i, j, k) = longest_match(a, b, alo, ahi, blo, bhi);
    if k == 0 {
        return 0;
    }
    let mut total = k;
    if alo < i && blo < j {
        total += matching_total(a, b, alo, i, blo, j);
    }
    if i + k < ahi && j + k < bhi {
        total += matching_total(a, b, i + k, ahi, j + k, bhi);
    }
    total
}

/// Same score as `difflib.SequenceMatcher(None, a, b).ratio()`.
pub fn ratio(a: &str, b: &str) -> f64 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let t = a.len() + b.len();
    if t == 0 {
        return 1.0;
    }
    2.0 * matching_total(&a, &b, 0, a.len(), 0, b.len()) as f64 / t as f64
}

/// Best candidate with ratio >= cutoff (ties broken like `difflib`).
pub fn close_match(word: &str, candidates: &[String], cutoff: f64) -> Option<String> {
    let mut best: Option<(f64, &String)> = None;
    for c in candidates {
        let r = ratio(c, word);
        if r >= cutoff {
            best = match best {
                None => Some((r, c)),
                Some((br, bc)) => {
                    if r > br || (r == br && c > bc) {
                        Some((r, c))
                    } else {
                        Some((br, bc))
                    }
                }
            };
        }
    }
    best.map(|(_, c)| c.clone())
}

/// Suggest a fix for an unknown name, based on real language rules.
pub fn name_help(name: &str, candidates: &[String], assign: bool) -> String {
    let low = name.to_lowercase();
    if name == "true" || name == "false" {
        return format!("Arcana's boolean values are On and Off, not '{}'", name);
    }
    if low == "on" || low == "off" {
        return "Arcana's boolean values are written On and Off (capitalised)".to_string();
    }
    let mut sorted: Vec<String> = candidates.to_vec();
    sorted.sort();
    sorted.dedup();
    if let Some(m) = close_match(name, &sorted, 0.7) {
        return format!("did you mean '{}'?", m);
    }
    if assign {
        format!("declare it first with 'bind {} = ...'", name)
    } else {
        "declare the variable with 'bind'".to_string()
    }
}
