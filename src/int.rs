//! Arbitrary-precision integers with an i64 fast path.
//!
//! Invariant: a value that fits in an i64 is always `Small`, so the derived
//! `Eq`/`Hash` are correct.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{FromPrimitive, Signed, ToPrimitive};
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Int {
    Small(i64),
    Big(BigInt),
}

impl From<i64> for Int {
    fn from(v: i64) -> Int {
        Int::Small(v)
    }
}

impl Int {
    pub fn from_big(b: BigInt) -> Int {
        match b.to_i64() {
            Some(v) => Int::Small(v),
            None => Int::Big(b),
        }
    }

    pub fn big(&self) -> BigInt {
        match self {
            Int::Small(v) => BigInt::from(*v),
            Int::Big(b) => b.clone(),
        }
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Int::Small(0))
    }

    pub fn is_neg(&self) -> bool {
        match self {
            Int::Small(v) => *v < 0,
            Int::Big(b) => b.is_negative(),
        }
    }

    pub fn is_pos(&self) -> bool {
        !self.is_neg() && !self.is_zero()
    }

    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Int::Small(v) => Some(*v),
            Int::Big(_) => None,
        }
    }

    pub fn to_usize(&self) -> Option<usize> {
        match self {
            Int::Small(v) => usize::try_from(*v).ok(),
            Int::Big(_) => None,
        }
    }

    /// Nearest f64, or `None` when the value is too large for a decimal.
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Int::Small(v) => Some(*v as f64),
            Int::Big(b) => b.to_f64().filter(|f| f.is_finite()),
        }
    }

    /// Truncate a finite f64 to an integer.
    pub fn from_f64(f: f64) -> Option<Int> {
        if !f.is_finite() {
            return None;
        }
        BigInt::from_f64(f.trunc()).map(Int::from_big)
    }

    pub fn add(&self, o: &Int) -> Int {
        if let (Int::Small(a), Int::Small(b)) = (self, o) {
            if let Some(r) = a.checked_add(*b) {
                return Int::Small(r);
            }
        }
        Int::from_big(self.big() + o.big())
    }

    pub fn sub(&self, o: &Int) -> Int {
        if let (Int::Small(a), Int::Small(b)) = (self, o) {
            if let Some(r) = a.checked_sub(*b) {
                return Int::Small(r);
            }
        }
        Int::from_big(self.big() - o.big())
    }

    pub fn mul(&self, o: &Int) -> Int {
        if let (Int::Small(a), Int::Small(b)) = (self, o) {
            if let Some(r) = a.checked_mul(*b) {
                return Int::Small(r);
            }
        }
        Int::from_big(self.big() * o.big())
    }

    pub fn neg(&self) -> Int {
        match self {
            Int::Small(v) => match v.checked_neg() {
                Some(r) => Int::Small(r),
                None => Int::from_big(-BigInt::from(*v)),
            },
            Int::Big(b) => Int::from_big(-b.clone()),
        }
    }

    pub fn abs(&self) -> Int {
        if self.is_neg() {
            self.neg()
        } else {
            self.clone()
        }
    }

    /// Floor division; the divisor must not be zero.
    pub fn floor_div(&self, o: &Int) -> Int {
        match (self, o) {
            (Int::Small(a), Int::Small(b)) => {
                if *b == -1 {
                    self.neg()
                } else {
                    Int::Small(a.div_floor(b))
                }
            }
            _ => Int::from_big(self.big().div_floor(&o.big())),
        }
    }

    /// Modulo with the sign of the divisor; the divisor must not be zero.
    pub fn modulo(&self, o: &Int) -> Int {
        match (self, o) {
            (Int::Small(a), Int::Small(b)) => {
                if *b == -1 {
                    Int::Small(0)
                } else {
                    Int::Small(a.mod_floor(b))
                }
            }
            _ => Int::from_big(self.big().mod_floor(&o.big())),
        }
    }

    pub fn pow(&self, e: u32) -> Int {
        match self {
            Int::Small(a) => match a.checked_pow(e) {
                Some(r) => Int::Small(r),
                None => Int::from_big(BigInt::from(*a).pow(e)),
            },
            Int::Big(b) => Int::from_big(b.pow(e)),
        }
    }

    pub fn bit_length(&self) -> u64 {
        match self {
            Int::Small(v) => 64 - v.unsigned_abs().leading_zeros() as u64,
            Int::Big(b) => b.bits(),
        }
    }

    pub fn is_even(&self) -> bool {
        match self {
            Int::Small(v) => v & 1 == 0,
            Int::Big(b) => b.is_even(),
        }
    }

    /// Exact comparison with a finite f64.
    pub fn cmp_f64(&self, f: f64) -> Ordering {
        let fl = f.floor();
        let fi = Int::from_f64(fl).unwrap_or(Int::Small(0));
        match self.cmp(&fi) {
            Ordering::Equal => {
                if f > fl {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            }
            o => o,
        }
    }

    /// Parse like Python's `int(str)` (after the caller has stripped spaces).
    pub fn parse(s: &str) -> Option<Int> {
        let (neg, body) = match s.as_bytes().first()? {
            b'+' => (false, &s[1..]),
            b'-' => (true, &s[1..]),
            _ => (false, s),
        };
        if body.is_empty() || body.starts_with('_') || body.ends_with('_') || body.contains("__") {
            return None;
        }
        if !body.bytes().all(|c| c.is_ascii_digit() || c == b'_') {
            return None;
        }
        let digits: String = body.chars().filter(|c| *c != '_').collect();
        let b = BigInt::parse_bytes(digits.as_bytes(), 10)?;
        Some(Int::from_big(if neg { -b } else { b }))
    }

    /// Exactly representable as an f64 without rounding?
    fn exact_f64(&self) -> Option<f64> {
        match self {
            Int::Small(v) if v.unsigned_abs() <= (1u64 << 53) => Some(*v as f64),
            _ => None,
        }
    }

    /// True division `a / b` (b != 0).  `None` means the result overflows.
    pub fn true_div(a: &Int, b: &Int) -> Option<f64> {
        if let (Some(x), Some(y)) = (a.exact_f64(), b.exact_f64()) {
            let r = x / y;
            return if r.is_finite() { Some(r) } else { None };
        }
        if a.is_zero() {
            return Some(0.0);
        }
        let neg = a.is_neg() != b.is_neg();
        let ab = a.abs().big();
        let bb = b.abs().big();
        let k = 64 + bb.bits() as i64 - ab.bits() as i64;
        let (num, den) = if k >= 0 { (ab << (k as usize), bb) } else { (ab, bb << ((-k) as usize)) };
        let q = num / den;
        let mut r = q.to_f64()?;
        let mut e = -k;
        while e > 1000 {
            r *= 2f64.powi(1000);
            e -= 1000;
        }
        while e < -1000 {
            r *= 2f64.powi(-1000);
            e += 1000;
        }
        r *= 2f64.powi(e as i32);
        if !r.is_finite() {
            return None;
        }
        Some(if neg { -r } else { r })
    }
}

impl PartialOrd for Int {
    fn partial_cmp(&self, other: &Int) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Int {
    fn cmp(&self, other: &Int) -> Ordering {
        match (self, other) {
            (Int::Small(a), Int::Small(b)) => a.cmp(b),
            (Int::Small(_), Int::Big(b)) => {
                if b.is_negative() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (Int::Big(a), Int::Small(_)) => {
                if a.is_negative() {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (Int::Big(a), Int::Big(b)) => a.cmp(b),
        }
    }
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Int::Small(v) => write!(f, "{}", v),
            Int::Big(b) => write!(f, "{}", b),
        }
    }
}
