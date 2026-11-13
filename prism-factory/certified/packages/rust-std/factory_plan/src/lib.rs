#![forbid(unsafe_code)]
use core::cmp::Ordering;

use core::sync::atomic::{AtomicU64, Ordering as Memory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overflow;

pub type R<T> = Result<T, Overflow>;

static WORK: AtomicU64 = AtomicU64::new(0);

pub fn tick(units: u64) { WORK.fetch_add(units, Memory::Relaxed); }

pub fn work() -> u64 { WORK.load(Memory::Relaxed) }

pub trait Same {
    fn same(&self, other: &Self) -> bool;
}

pub trait Key {
    fn key(&self, other: &Self) -> Ordering;
}

impl Same for bool {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for bool {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u8 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u8 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u16 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u16 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u32 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u32 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for u64 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for u64 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i8 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i8 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i16 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i16 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i32 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i32 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for i64 {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl Key for i64 {
    fn key(&self, other: &Self) -> Ordering { self.cmp(other) }
}

impl Same for Ordering {
    fn same(&self, other: &Self) -> bool { self == other }
}

impl<A: Key, B: Key> Key for (A, B) {
    fn key(&self, other: &Self) -> Ordering { match self.0.key(&other.0) { Ordering::Equal => self.1.key(&other.1), decided => decided, } }
}

pub fn equal<T: Same>(a: T, b: T) -> bool { a.same(&b) }

pub fn compare<T: Key>(a: T, b: T) -> Ordering { a.key(&b) }

pub fn nat_add(p0: u64, p1: u64) -> R<u64> {
    match u64::checked_add(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<u64, Overflow>(Overflow)
        }
    }
}

pub fn nat_sub(p0: u64, p1: u64) -> u64 {
    u64::saturating_sub(p0, p1)
}

pub fn nat_mul(p0: u64, p1: u64) -> R<u64> {
    match u64::checked_mul(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<u64, Overflow>(Overflow)
        }
    }
}

pub fn nat_quot(p0: u64, p1: u64, p2: u64) -> u64 {
    u64::checked_div(p0, p1).unwrap_or(p2)
}

pub fn nat_rem(p0: u64, p1: u64, p2: u64) -> u64 {
    u64::checked_rem(p0, p1).unwrap_or(p2)
}

pub fn nat_eq(p0: u64, p1: u64) -> bool {
    core::cmp::PartialEq::eq(&p0, &p1)
}

pub fn nat_le(p0: u64, p1: u64) -> bool {
    core::cmp::PartialOrd::le(&p0, &p1)
}

pub fn nat_lt(p0: u64, p1: u64) -> bool {
    core::cmp::PartialOrd::lt(&p0, &p1)
}

pub fn nat_succ(p0: u64) -> R<u64> {
    match u64::checked_add(p0, 1u64) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<u64, Overflow>(Overflow)
        }
    }
}

pub fn int_add(p0: i64, p1: i64) -> R<i64> {
    match i64::checked_add(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_sub(p0: i64, p1: i64) -> R<i64> {
    match i64::checked_sub(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_mul(p0: i64, p1: i64) -> R<i64> {
    match i64::checked_mul(p0, p1) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_neg(p0: i64) -> R<i64> {
    match i64::checked_neg(p0) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    }
}

pub fn int_quot(p0: i64, p1: i64, p2: i64) -> R<i64> {
    { let value = { let (a, b, z): (i64, i64, i64) = (p0, p1, p2); if b == 0 { Some(z) } else { a.checked_div(b) } }; match value {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<i64, Overflow>(Overflow)
        }
    } }
}

pub fn int_rem(p0: i64, p1: i64, p2: i64) -> i64 {
    { let (a, b, z): (i64, i64, i64) = (p0, p1, p2); if b == 0 { z } else { a.wrapping_rem(b) } }
}

pub fn bool_not(p0: bool) -> bool {
    !p0
}

pub fn bool_and(p0: bool, p1: bool) -> bool {
    if p0 {
        p1
    } else {
        false
    }
}

pub fn bool_or(p0: bool, p1: bool) -> bool {
    if p0 {
        true
    } else {
        p1
    }
}

pub mod fixed_u8 {
    pub fn checked_add(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u8, p1: u8) -> Option<u8> {
        u8::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u8, p1: u8) -> u8 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u8, p1: u8) -> u8 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u8, p1: u8) -> u8 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u8) -> u8 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u8, p1: u32) -> Option<u8> {
        u8::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u8, p1: u32) -> Option<u8> {
        u8::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u8> {
        u8::try_from(p0).ok()
    }
}

pub mod fixed_u16 {
    pub fn checked_add(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u16, p1: u16) -> Option<u16> {
        u16::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u16, p1: u16) -> u16 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u16, p1: u16) -> u16 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u16, p1: u16) -> u16 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u16) -> u16 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u16, p1: u32) -> Option<u16> {
        u16::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u16, p1: u32) -> Option<u16> {
        u16::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u16> {
        u16::try_from(p0).ok()
    }
}

pub mod fixed_u32 {
    pub fn checked_add(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u32, p1: u32) -> u32 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u32, p1: u32) -> u32 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u32, p1: u32) -> u32 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u32) -> u32 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u32, p1: u32) -> Option<u32> {
        u32::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u32> {
        u32::try_from(p0).ok()
    }
}

pub mod fixed_u64 {
    pub fn checked_add(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: u64, p1: u64) -> Option<u64> {
        u64::checked_div(p0, p1)
    }

    pub fn bit_and(p0: u64, p1: u64) -> u64 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: u64, p1: u64) -> u64 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: u64, p1: u64) -> u64 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: u64) -> u64 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: u64, p1: u32) -> Option<u64> {
        u64::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: u64, p1: u32) -> Option<u64> {
        u64::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<u64> {
        u64::try_from(p0).ok()
    }
}

pub mod fixed_i8 {
    pub fn checked_add(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i8, p1: i8) -> Option<i8> {
        i8::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i8, p1: i8) -> i8 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i8, p1: i8) -> i8 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i8, p1: i8) -> i8 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i8) -> i8 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i8, p1: u32) -> Option<i8> {
        i8::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i8, p1: u32) -> Option<i8> {
        i8::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i8> {
        i8::try_from(p0).ok()
    }
}

pub fn checked_neg_i8(p0: i8) -> Option<i8> {
    i8::checked_neg(p0)
}

pub mod fixed_i16 {
    pub fn checked_add(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i16, p1: i16) -> Option<i16> {
        i16::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i16, p1: i16) -> i16 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i16, p1: i16) -> i16 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i16, p1: i16) -> i16 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i16) -> i16 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i16, p1: u32) -> Option<i16> {
        i16::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i16, p1: u32) -> Option<i16> {
        i16::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i16> {
        i16::try_from(p0).ok()
    }
}

pub fn checked_neg_i16(p0: i16) -> Option<i16> {
    i16::checked_neg(p0)
}

pub mod fixed_i32 {
    pub fn checked_add(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i32, p1: i32) -> Option<i32> {
        i32::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i32, p1: i32) -> i32 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i32, p1: i32) -> i32 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i32, p1: i32) -> i32 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i32) -> i32 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i32, p1: u32) -> Option<i32> {
        i32::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i32, p1: u32) -> Option<i32> {
        i32::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i32> {
        i32::try_from(p0).ok()
    }
}

pub fn checked_neg_i32(p0: i32) -> Option<i32> {
    i32::checked_neg(p0)
}

pub mod fixed_i64 {
    pub fn checked_add(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_add(p0, p1)
    }

    pub fn checked_sub(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_sub(p0, p1)
    }

    pub fn checked_mul(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_mul(p0, p1)
    }

    pub fn checked_quot(p0: i64, p1: i64) -> Option<i64> {
        i64::checked_div(p0, p1)
    }

    pub fn bit_and(p0: i64, p1: i64) -> i64 {
        core::ops::BitAnd::bitand(p0, p1)
    }

    pub fn bit_or(p0: i64, p1: i64) -> i64 {
        core::ops::BitOr::bitor(p0, p1)
    }

    pub fn bit_xor(p0: i64, p1: i64) -> i64 {
        core::ops::BitXor::bitxor(p0, p1)
    }

    pub fn bit_not(p0: i64) -> i64 {
        core::ops::Not::not(p0)
    }

    pub fn shift_left(p0: i64, p1: u32) -> Option<i64> {
        i64::checked_shl(p0, p1)
    }

    pub fn shift_right(p0: i64, p1: u32) -> Option<i64> {
        i64::checked_shr(p0, p1)
    }

    pub fn convert(p0: i128) -> Option<i64> {
        i64::try_from(p0).ok()
    }
}

pub fn checked_neg_i64(p0: i64) -> Option<i64> {
    i64::checked_neg(p0)
}
use core::str;

use std::mem;

use std::rc::Rc;

#[derive(Clone)]
pub struct Str(Rc<str>);

impl Str {
    pub fn lit(text: &str) -> Str { Str(Rc::from(text)) }
    pub fn text(&self) -> &str { &self.0 }
    pub fn length(&self) -> u64 { chars(&self.0) }
    pub fn utf8_encode(&self) -> Bytes { tick(self.0.len() as u64); Bytes::lit(self.0.as_bytes()) }
    pub fn split_exact(&self, delimiter: &Str, maximum: u32) -> Option<List<Str>> { if delimiter.0.is_empty() { return None; }; chars(&self.0); let mut fields: Vec<Str> = Vec::new(); for piece in self.0.split(&*delimiter.0) { fields.push(Str::lit(piece)); }; if fields.len() as u128 > u128::from(maximum) { return None; }; Some(List::onto(fields, List::nil())) }
    pub fn join(texts: List<Str>, delimiter: &Str) -> Str { let mut out = String::new(); let mut first = true; let mut cursor = &texts; while let Repr::Cell(cell) = &cursor.0 { tick(1); if !first { chars(&delimiter.0); out.push_str(&delimiter.0); }; first = false; chars(&cell.head.0); out.push_str(&cell.head.0); cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for text in buffer[..*len].iter().rev() { tick(1); if !first { chars(&delimiter.0); out.push_str(&delimiter.0); }; first = false; chars(&text.0); out.push_str(&text.0); } }, _ => { }, }; Str::lit(&out) }
    pub fn parse_int(text: &Str) -> Option<Option<i64>> { match canonical_decimal(&text.0) { None => Some(None), Some(Some(n)) => match i64::try_from(n) { Ok(value) => Some(Some(value)), Err(_) => None, }, Some(None) => None, } }
    pub fn format_u8(a: u8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u8(text: &Str) -> Option<u8> { match canonical_decimal(&text.0) { Some(Some(n)) => u8::try_from(n).ok(), _ => None, } }
    pub fn format_u16(a: u16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u16(text: &Str) -> Option<u16> { match canonical_decimal(&text.0) { Some(Some(n)) => u16::try_from(n).ok(), _ => None, } }
    pub fn format_u32(a: u32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u32(text: &Str) -> Option<u32> { match canonical_decimal(&text.0) { Some(Some(n)) => u32::try_from(n).ok(), _ => None, } }
    pub fn format_u64(a: u64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_u64(text: &Str) -> Option<u64> { match canonical_decimal(&text.0) { Some(Some(n)) => u64::try_from(n).ok(), _ => None, } }
    pub fn format_i8(a: i8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i8(text: &Str) -> Option<i8> { match canonical_decimal(&text.0) { Some(Some(n)) => i8::try_from(n).ok(), _ => None, } }
    pub fn format_i16(a: i16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i16(text: &Str) -> Option<i16> { match canonical_decimal(&text.0) { Some(Some(n)) => i16::try_from(n).ok(), _ => None, } }
    pub fn format_i32(a: i32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i32(text: &Str) -> Option<i32> { match canonical_decimal(&text.0) { Some(Some(n)) => i32::try_from(n).ok(), _ => None, } }
    pub fn format_i64(a: i64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
    pub fn parse_i64(text: &Str) -> Option<i64> { match canonical_decimal(&text.0) { Some(Some(n)) => i64::try_from(n).ok(), _ => None, } }
}

#[derive(Clone)]
pub struct Bytes(Rc<[u8]>);

impl Bytes {
    pub fn lit(octets: &[u8]) -> Bytes { Bytes(Rc::from(octets)) }
    pub fn octets(&self) -> &[u8] { &self.0 }
    pub fn append(a: Bytes, b: Bytes) -> Bytes { tick((a.0.len() + b.0.len()) as u64); let mut out = Vec::with_capacity(a.0.len() + b.0.len()); out.extend_from_slice(&a.0); out.extend_from_slice(&b.0); Bytes(Rc::from(out)) }
    pub fn length(&self) -> u64 { self.0.len() as u64 }
    pub fn index(&self, i: u64) -> Option<u8> { match usize::try_from(i) { Ok(at) => self.0.get(at).copied(), Err(_) => None, } }
    pub fn slice(&self, start: u64, count: u64) -> Option<Bytes> { let end = u128::from(start) + u128::from(count); if end > self.0.len() as u128 { return None; }; tick(count); Some(Bytes::lit(&self.0[start as usize..end as usize])) }
    pub fn utf8_decode(&self) -> Option<Str> { tick(self.0.len() as u64); match str::from_utf8(&self.0) { Ok(text) => Some(Str::lit(text)), Err(_) => None, } }
    pub fn compare(&self, other: &Bytes) -> Ordering { tick(1 + self.0.len().min(other.0.len()) as u64); self.0.cmp(&other.0) }
}

pub struct Node<T> { head: T, tail: List<T> }

pub enum Repr<T> { Nil, Run { buffer: Rc<Vec<T>>, len: usize }, Cell(Rc<Node<T>>) }

pub struct List<T>(Repr<T>);

impl<T> Clone for List<T> {
    fn clone(&self) -> Self { List(match &self.0 { Repr::Nil => Repr::Nil, Repr::Run { buffer, len } => Repr::Run { buffer: buffer.clone(), len: *len }, Repr::Cell(cell) => Repr::Cell(cell.clone()), }) }
}

impl<T> List<T> {
    fn take(&mut self) -> Repr<T> { mem::replace(&mut self.0, Repr::Nil) }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) { let mut next = match self.tail.take() { Repr::Cell(cell) => Some(cell), _ => None, }; while let Some(cell) = next { match Rc::try_unwrap(cell) { Ok(mut node) => next = match node.tail.take() { Repr::Cell(cell) => Some(cell), _ => None, }, Err(_) => break, } } }
}

impl<T: Clone> List<T> {
    pub fn nil() -> Self { List(Repr::Nil) }
    pub fn cons(head: T, mut tail: List<T>) -> Self { List(match tail.take() { Repr::Nil => { let mut buffer = Vec::with_capacity(16); buffer.push(head); Repr::Run { buffer: Rc::new(buffer), len: 1 } }, Repr::Run { mut buffer, len } => { match Rc::get_mut(&mut buffer) { Some(cells) => { cells.truncate(len); cells.push(head); Repr::Run { buffer, len: len + 1 } }, None => Repr::Cell(Rc::new(Node { head, tail: List(Repr::Run { buffer, len }) })), } }, rest => Repr::Cell(Rc::new(Node { head, tail: List(rest) })), }) }
    pub fn uncons(&self) -> Option<(T, List<T>)> { match &self.0 { Repr::Nil => None, Repr::Run { buffer, len } => { let head = buffer[len - 1].clone(); Some((head, List(if *len == 1 { Repr::Nil } else { Repr::Run { buffer: buffer.clone(), len: len - 1 } }))) }, Repr::Cell(cell) => Some((cell.head.clone(), cell.tail.clone())), } }
    pub fn length(&self) -> u64 { let mut count = 0; let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); count += 1; cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for _ in buffer[..*len].iter().rev() { tick(1); count += 1; } }, _ => { }, }; count }
    pub fn append(a: List<T>, b: List<T>) -> List<T> { List::onto(a.items(), b) }
    pub fn index(&self, i: u64) -> Option<T> { let mut position = 0; let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); if position == i { return Some(cell.head.clone()); }; position += 1; cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for item in buffer[..*len].iter().rev() { tick(1); if position == i { return Some(item.clone()); }; position += 1; } }, _ => { }, }; None }
    pub fn slice(&self, start: u64, count: u64) -> Option<List<T>> { let end = u128::from(start) + u128::from(count); let mut part = Vec::new(); let mut position: u128 = 0; let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); if position >= end { return Some(List::onto(part, List::nil())); }; if position >= u128::from(start) { part.push(cell.head.clone()); }; position += 1; cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for item in buffer[..*len].iter().rev() { tick(1); if position >= end { return Some(List::onto(part, List::nil())); }; if position >= u128::from(start) { part.push(item.clone()); }; position += 1; } }, _ => { }, }; if position < end { None } else { Some(List::onto(part, List::nil())) } }
    fn items(&self) -> Vec<T> { let mut out = Vec::new(); let mut cursor = self; while let Repr::Cell(cell) = &cursor.0 { tick(1); out.push(cell.head.clone()); cursor = &cell.tail; }; match &cursor.0 { Repr::Run { buffer, len } => { for item in buffer[..*len].iter().rev() { tick(1); out.push(item.clone()); } }, _ => { }, }; out }
    fn onto(mut items: Vec<T>, tail: List<T>) -> List<T> { let mut out = tail; while let Some(item) = items.pop() { tick(1); out = List::cons(item, out); }; out }
}

pub fn chars(text: &str) -> u64 { let mut count = 0; for _ in text.chars() { tick(1); count += 1; }; count }

impl Same for Str {
    fn same(&self, other: &Self) -> bool { let mut left = self.0.chars(); let mut right = other.0.chars(); loop { tick(1); match (left.next(), right.next()) { (None, None) => return true, (a, b) if a != b => return false, _ => { }, } } }
}

impl Same for Bytes {
    fn same(&self, other: &Self) -> bool { tick(1 + self.0.len().min(other.0.len()) as u64); self.0 == other.0 }
}

impl Key for Str {
    fn key(&self, other: &Self) -> Ordering { let mut left = self.0.chars(); let mut right = other.0.chars(); loop { tick(1); match (left.next(), right.next()) { (None, None) => return Ordering::Equal, (None, Some(_)) => return Ordering::Less, (Some(_), None) => return Ordering::Greater, (Some(a), Some(b)) => match a.cmp(&b) { Ordering::Equal => { }, decided => return decided, }, } } }
}

pub fn canonical_decimal(text: &str) -> Option<Option<i128>> { let digits = match text.strip_prefix('-') { Some(rest) => rest, None => text, }; let mut canonical = !digits.is_empty() && (digits == "0" || !digits.starts_with('0')) && text != "-0"; for c in digits.chars() { tick(1); canonical = canonical && c.is_ascii_digit(); }; if !canonical { return None; }; Some(text.parse::<i128>().ok()) }

pub fn append_list<T: Clone>(p0: List<T>, p1: List<T>) -> List<T> {
    List::append(p0, p1)
}

pub fn length_list<T: Clone>(p0: List<T>) -> u64 {
    List::length(&p0)
}

pub fn index_list<T: Clone>(p0: List<T>, p1: u64) -> Option<T> {
    List::index(&p0, p1)
}

pub fn slice_list<T: Clone>(p0: List<T>, p1: u64, p2: u64) -> Option<List<T>> {
    List::slice(&p0, p1, p2)
}

pub fn append_bytes(p0: Bytes, p1: Bytes) -> Bytes {
    Bytes::append(p0, p1)
}

pub fn length_bytes(p0: Bytes) -> u64 {
    Bytes::length(&p0)
}

pub fn length_string(p0: Str) -> u64 {
    Str::length(&p0)
}

pub fn index_bytes(p0: Bytes, p1: u64) -> Option<u8> {
    Bytes::index(&p0, p1)
}

pub fn slice_bytes(p0: Bytes, p1: u64, p2: u64) -> Option<Bytes> {
    Bytes::slice(&p0, p1, p2)
}

pub fn utf8_encode(p0: Str) -> Bytes {
    Str::utf8_encode(&p0)
}

pub fn utf8_decode(p0: Bytes) -> Option<Str> {
    Bytes::utf8_decode(&p0)
}

pub fn compare_bytes(p0: Bytes, p1: Bytes) -> Ordering {
    Bytes::compare(&p0, &p1)
}

pub fn split_exact(p0: Str, p1: Str, p2: u32) -> Option<List<Str>> {
    Str::split_exact(&p0, &p1, p2)
}

pub fn join(p0: List<Str>, p1: Str) -> Str {
    Str::join(p0, &p1)
}

pub fn format_u8(p0: u8) -> Str {
    Str::format_u8(p0)
}

pub fn parse_u8(p0: Str) -> Option<u8> {
    Str::parse_u8(&p0)
}

pub fn format_u16(p0: u16) -> Str {
    Str::format_u16(p0)
}

pub fn parse_u16(p0: Str) -> Option<u16> {
    Str::parse_u16(&p0)
}

pub fn format_u32(p0: u32) -> Str {
    Str::format_u32(p0)
}

pub fn parse_u32(p0: Str) -> Option<u32> {
    Str::parse_u32(&p0)
}

pub fn format_u64(p0: u64) -> Str {
    Str::format_u64(p0)
}

pub fn parse_u64(p0: Str) -> Option<u64> {
    Str::parse_u64(&p0)
}

pub fn format_i8(p0: i8) -> Str {
    Str::format_i8(p0)
}

pub fn parse_i8(p0: Str) -> Option<i8> {
    Str::parse_i8(&p0)
}

pub fn format_i16(p0: i16) -> Str {
    Str::format_i16(p0)
}

pub fn parse_i16(p0: Str) -> Option<i16> {
    Str::parse_i16(&p0)
}

pub fn format_i32(p0: i32) -> Str {
    Str::format_i32(p0)
}

pub fn parse_i32(p0: Str) -> Option<i32> {
    Str::parse_i32(&p0)
}

pub fn format_i64(p0: i64) -> Str {
    Str::format_i64(p0)
}

pub fn parse_i64(p0: Str) -> Option<i64> {
    Str::parse_i64(&p0)
}

pub fn parse_int(p0: Str) -> R<Option<i64>> {
    match Str::parse_int(&p0) {
        Some(v0) => {
            Ok(v0)
        }
        None => {
            Err::<Option<i64>, Overflow>(Overflow)
        }
    }
}

pub fn format_int(p0: i64) -> Str {
    Str::format_i64(p0)
}

#[derive(Clone)]
pub enum Adt0 {
    C0(List<(Bytes, Bytes)>, List<(Str, bool)>, List<(Str, Str)>, List<(Str, bool)>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Adt2, Str, Adt4),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, Adt3, Str, Str, List<Str>, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0,
    C1,
}

#[derive(Clone)]
pub enum Adt4 {
    C0,
    C1,
    C2,
    C3,
}

#[derive(Clone)]
pub enum Adt5 {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,
    C13,
    C14,
    C15,
}

#[derive(Clone)]
pub enum Adt6 {
    C0(u32, u32, u32, u32, u32, u32, u32, u32),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32, u32),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(u32, u32, u32, u32, u32, u32, u32, u32, Adt7, u64),
}

#[derive(Clone)]
pub enum Fn0 {
    F6,
}

impl Fn0 {
    pub fn apply(&self, p0: Adt0, p1: Bytes) -> R<Adt0> {
        match self {
            Fn0::F6 => f6(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn1 {
    F12(List<(Bytes, Bytes)>),
    F117,
}

impl Fn1 {
    pub fn apply(&self, p0: List<Bytes>, p1: Adt1) -> R<List<Bytes>> {
        match self {
            Fn1::F12(k0) => f12(k0.clone(), p0, p1),
            Fn1::F117 => Ok(f117(p0, p1)),
        }
    }
}

#[derive(Clone)]
pub enum Fn2 {
    F14,
}

impl Fn2 {
    pub fn apply(&self, p0: Bytes, p1: (Str, u64)) -> Bytes {
        match self {
            Fn2::F14 => f14(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn3 {
    F19(List<(Str, bool)>, List<(Bytes, Bytes)>),
}

impl Fn3 {
    pub fn apply(&self, p0: (List<Adt1>, List<(List<Str>, Str)>), p1: Adt2) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
        match self {
            Fn3::F19(k0, k1) => f19(k0.clone(), k1.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn4 {
    F23(Bytes),
}

impl Fn4 {
    pub fn apply(&self, p0: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
        match self {
            Fn4::F23(k0) => f23(k0.clone(), p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn5 {
    F27,
}

impl Fn5 {
    pub fn apply(&self, p0: List<(Bytes, Bytes)>, p1: (Bytes, Bytes)) -> List<(Bytes, Bytes)> {
        match self {
            Fn5::F27 => f27(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn6 {
    F29,
}

impl Fn6 {
    pub fn apply(&self, p0: List<(Str, Str)>, p1: (Str, Str)) -> List<(Str, Str)> {
        match self {
            Fn6::F29 => f29(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn7 {
    F33(List<(Str, Str)>),
}

impl Fn7 {
    pub fn apply(&self, p0: List<(Str, u64)>, p1: Adt1) -> List<(Str, u64)> {
        match self {
            Fn7::F33(k0) => f33(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn8 {
    F36,
    F75(Str, Str),
    F116(Str),
}

impl Fn8 {
    pub fn apply(&self, p0: List<Bytes>, p1: Str) -> List<Bytes> {
        match self {
            Fn8::F36 => f36(p0, p1),
            Fn8::F75(k0, k1) => f75(k0.clone(), k1.clone(), p0, p1),
            Fn8::F116(k0) => f116(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn9 {
    F46,
}

impl Fn9 {
    pub fn apply(&self, p0: List<Bytes>, p1: Bytes) -> List<Bytes> {
        match self {
            Fn9::F46 => f46(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn10 {
    F48,
}

impl Fn10 {
    pub fn apply(&self, p0: List<Bytes>) -> Option<List<Bytes>> {
        match self {
            Fn10::F48 => f48(p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn11 {
    F60,
}

impl Fn11 {
    pub fn apply(&self, p0: bool, p1: Adt4) -> bool {
        match self {
            Fn11::F60 => f60(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn12 {
    F62,
}

impl Fn12 {
    pub fn apply(&self, p0: List<Adt4>, p1: Adt1) -> List<Adt4> {
        match self {
            Fn12::F62 => f62(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn13 {
    F74(List<Str>, Str),
    F102(List<Str>),
}

impl Fn13 {
    pub fn apply(&self, p0: List<Bytes>, p1: (Bytes, Bytes)) -> R<List<Bytes>> {
        match self {
            Fn13::F74(k0, k1) => f74(k0.clone(), k1.clone(), p0, p1),
            Fn13::F102(k0) => f102(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn14 {
    F78(Str),
}

impl Fn14 {
    pub fn apply(&self, p0: Str, p1: (Str, Str)) -> Str {
        match self {
            Fn14::F78(k0) => f78(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn15 {
    F80(Bytes),
}

impl Fn15 {
    pub fn apply(&self, p0: Option<Bytes>, p1: Bytes) -> Option<Bytes> {
        match self {
            Fn15::F80(k0) => f80(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn16 {
    F83(List<Str>),
}

impl Fn16 {
    pub fn apply(&self, p0: Option<Str>, p1: (List<Str>, Str)) -> Option<Str> {
        match self {
            Fn16::F83(k0) => f83(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn17 {
    F87(List<Adt1>),
}

impl Fn17 {
    pub fn apply(&self, p0: List<Adt1>, p1: Str) -> List<Adt1> {
        match self {
            Fn17::F87(k0) => f87(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn18 {
    F90(List<Adt1>),
}

impl Fn18 {
    pub fn apply(&self, p0: bool, p1: Str) -> bool {
        match self {
            Fn18::F90(k0) => f90(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn19 {
    F93(Bytes, u64, Bytes, u64),
}

impl Fn19 {
    pub fn apply(&self, p0: u64) -> R<Option<u64>> {
        match self {
            Fn19::F93(k0, k1, k2, k3) => f93(k0.clone(), *k1, k2.clone(), *k3, p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn20 {
    F97,
}

impl Fn20 {
    pub fn apply(&self, p0: (Option<Bytes>, List<Bytes>), p1: Bytes) -> (Option<Bytes>, List<Bytes>) {
        match self {
            Fn20::F97 => f97(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn21 {
    F107(Str),
}

impl Fn21 {
    pub fn apply(&self, p0: Option<bool>, p1: (Str, bool)) -> Option<bool> {
        match self {
            Fn21::F107(k0) => f107(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn22 {
    F112(Bytes, u64, u64),
}

impl Fn22 {
    pub fn apply(&self, p0: (u64, Adt6)) -> R<(u64, Adt6)> {
        match self {
            Fn22::F112(k0, k1, k2) => f112(k0.clone(), *k1, *k2, p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn23 {
    F115(Str),
}

impl Fn23 {
    pub fn apply(&self, p0: Option<Adt1>, p1: Adt1) -> Option<Adt1> {
        match self {
            Fn23::F115(k0) => f115(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn24 {
    F119(bool, Bytes),
}

impl Fn24 {
    pub fn apply(&self, p0: u64, p1: Str) -> R<u64> {
        match self {
            Fn24::F119(k0, k1) => f119(*k0, k1.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn25 {
    F127,
}

impl Fn25 {
    pub fn apply(&self, p0: Adt8) -> R<Adt8> {
        match self {
            Fn25::F127 => f127(p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn26 {
    F145,
}

impl Fn26 {
    pub fn apply(&self, p0: (u64, (u32, u32))) -> (u64, (u32, u32)) {
        match self {
            Fn26::F145 => f145(p0),
        }
    }
}

pub fn f0(v0: Bytes) -> R<Bytes> {
    let v1: Adt0 = f1(v0)?;
    f2(f3(f4(), match v1.clone() {
        Adt0::C0(h1, _, _, _) => {
            h1
        }
    }, match v1.clone() {
        Adt0::C0(_, h2, _, _) => {
            h2
        }
    })?, match v1.clone() {
        Adt0::C0(_, _, h3, _) => {
            h3
        }
    }, match v1 {
        Adt0::C0(h4, _, _, _) => {
            h4
        }
    })
}

pub fn f1(v0: Bytes) -> R<Adt0> {
    let v1: Adt0 = f5(Fn0::F6, Adt0::C0(List::<(Bytes, Bytes)>::nil(), List::<(Str, bool)>::nil(), List::<(Str, Str)>::nil(), List::<(Str, bool)>::nil()), f7(v0)?)?;
    Ok(Adt0::C0(f8(match v1.clone() {
        Adt0::C0(h5, _, _, _) => {
            h5
        }
    }), match v1.clone() {
        Adt0::C0(_, h6, _, _) => {
            h6
        }
    }, f9(match v1.clone() {
        Adt0::C0(_, _, h7, _) => {
            h7
        }
    }), match v1 {
        Adt0::C0(_, _, _, h8) => {
            h8
        }
    }))
}

pub fn f2(v0: List<Adt1>, v1: List<(Str, Str)>, v2: List<(Bytes, Bytes)>) -> R<Bytes> {
    let v3: Bytes = f10(f11(Fn1::F12(v2), List::<Bytes>::nil(), v0.clone())?);
    let v4: Bytes = f13(Fn2::F14, Bytes::lit(&[]), f15(v0.clone(), v1));
    let a11 = {
        let a9 = v3;
        let a10 = v4;
        append_bytes(a9, a10)
    };
    let a12 = f16(List::cons(Str::lit("releasable"), List::cons(if f17(v0) {
        Str::lit("yes")
    } else {
        Str::lit("no")
    }, List::<Str>::nil())));
    Ok(append_bytes(a11, a12))
}

pub fn f3(v0: List<Adt2>, v1: List<(Bytes, Bytes)>, v2: List<(Str, bool)>) -> R<List<Adt1>> {
    let (h13, _) = f18(Fn3::F19(v2, v1), (List::<Adt1>::nil(), List::<(List<Str>, Str)>::nil()), v0)?;
    Ok(h13)
}

pub fn f4() -> List<Adt2> {
    List::cons(f20(Str::lit("policy"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/policy"), List::cons(Str::lit(".github/workflows/release.yml"), List::cons(Str::lit("Cargo.toml"), List::cons(Str::lit("model/errors.toml"), List::cons(Str::lit("scripts/"), List::cons(Str::lit("@mode"), List::cons(Str::lit("@run"), List::<Str>::nil())))))), List::<Str>::nil()), List::cons(f20(Str::lit("factory-model"), Adt3::C0, Str::lit("host"), Str::lit("bash prism-factory/check.sh"), List::cons(Str::lit(".github/workflows/factory.yml"), List::cons(Str::lit(".github/workflows/release.yml"), List::cons(Str::lit("prism-factory/"), List::cons(Str::lit("vendor/lexlean/"), List::<Str>::nil())))), List::<Str>::nil()), List::cons(f20(Str::lit("plumbing"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/devcontainer-image"), List::cons(Str::lit(""), List::cons(Str::lit("@run"), List::<Str>::nil())), List::cons(Str::lit("policy"), List::<Str>::nil())), List::cons(f20(Str::lit("vv-sdk"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/vv-sdk"), List::cons(Str::lit(""), List::cons(Str::lit("!.github/workflows/release-qualify.yml"), List::cons(Str::lit("!.github/workflows/release.yml"), List::cons(Str::lit("!factory/"), List::cons(Str::lit("!scripts/browser-api-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/ci-capacity.test.mjs"), List::cons(Str::lit("!scripts/factory.mjs"), List::cons(Str::lit("!scripts/factory.test.mjs"), List::cons(Str::lit("!scripts/library-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/product-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/release-gate-evidence.mjs"), List::cons(Str::lit("!scripts/release-gate-evidence.test.mjs"), List::cons(Str::lit("!scripts/release-native-contract.test.mjs"), List::cons(Str::lit("!scripts/release-phases.md"), List::cons(Str::lit("!scripts/release-phases.mjs"), List::cons(Str::lit("!scripts/release-phases.test.mjs"), List::cons(Str::lit("!scripts/release-publication.mjs"), List::cons(Str::lit("!scripts/release-publication.test.mjs"), List::cons(Str::lit("!scripts/release-rebuild.sh"), List::cons(Str::lit("!scripts/rust-oracle-tools-inputs.test.mjs"), List::cons(Str::lit("!scripts/sdk-candidate-sbom.test.mjs"), List::cons(Str::lit("!scripts/sdk-construction-handoff.test.mjs"), List::cons(Str::lit("!scripts/sdk-image-inputs.test.mjs"), List::cons(Str::lit("!scripts/sdk-release-evidence.mjs"), List::cons(Str::lit("!scripts/sdk-release-evidence.test.mjs"), List::cons(Str::lit("!scripts/sdk-vv-check.test.mjs"), List::cons(Str::lit("!scripts/unpublished-sdk-qualification.test.mjs"), List::cons(Str::lit("!prism-factory/"), List::cons(Str::lit("prism-factory/jobs/header.yml"), List::cons(Str::lit("prism-factory/jobs/vv-sdk.yml"), List::<Str>::nil())))))))))))))))))))))))))))))), List::<Str>::nil()), List::cons(f20(Str::lit("source-vv"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/source-vv"), List::cons(Str::lit(""), List::cons(Str::lit("!.github/workflows/release-qualify.yml"), List::cons(Str::lit("!.github/workflows/release.yml"), List::cons(Str::lit("!factory/"), List::cons(Str::lit("!scripts/browser-api-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/ci-capacity.test.mjs"), List::cons(Str::lit("!scripts/factory.mjs"), List::cons(Str::lit("!scripts/factory.test.mjs"), List::cons(Str::lit("!scripts/library-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/product-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/release-gate-evidence.mjs"), List::cons(Str::lit("!scripts/release-gate-evidence.test.mjs"), List::cons(Str::lit("!scripts/release-native-contract.test.mjs"), List::cons(Str::lit("!scripts/release-phases.md"), List::cons(Str::lit("!scripts/release-phases.mjs"), List::cons(Str::lit("!scripts/release-phases.test.mjs"), List::cons(Str::lit("!scripts/release-publication.mjs"), List::cons(Str::lit("!scripts/release-publication.test.mjs"), List::cons(Str::lit("!scripts/release-rebuild.sh"), List::cons(Str::lit("!scripts/rust-oracle-tools-inputs.test.mjs"), List::cons(Str::lit("!scripts/sdk-candidate-sbom.test.mjs"), List::cons(Str::lit("!scripts/sdk-construction-handoff.test.mjs"), List::cons(Str::lit("!scripts/sdk-image-inputs.test.mjs"), List::cons(Str::lit("!scripts/sdk-release-evidence.mjs"), List::cons(Str::lit("!scripts/sdk-release-evidence.test.mjs"), List::cons(Str::lit("!scripts/sdk-vv-check.test.mjs"), List::cons(Str::lit("!scripts/unpublished-sdk-qualification.test.mjs"), List::cons(Str::lit("!prism-factory/"), List::cons(Str::lit("prism-factory/jobs/header.yml"), List::cons(Str::lit("prism-factory/jobs/source-vv.yml"), List::<Str>::nil())))))))))))))))))))))))))))))), List::cons(Str::lit("vv-sdk"), List::<Str>::nil())), List::cons(f20(Str::lit("build"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/build"), List::cons(Str::lit(""), List::cons(Str::lit("!.github/workflows/release-qualify.yml"), List::cons(Str::lit("!.github/workflows/release.yml"), List::cons(Str::lit("!factory/"), List::cons(Str::lit("!scripts/browser-api-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/ci-capacity.test.mjs"), List::cons(Str::lit("!scripts/factory.mjs"), List::cons(Str::lit("!scripts/factory.test.mjs"), List::cons(Str::lit("!scripts/library-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/product-sdk-check.test.mjs"), List::cons(Str::lit("!scripts/release-gate-evidence.test.mjs"), List::cons(Str::lit("!scripts/release-native-contract.test.mjs"), List::cons(Str::lit("!scripts/release-phases.md"), List::cons(Str::lit("!scripts/release-phases.test.mjs"), List::cons(Str::lit("!scripts/release-publication.mjs"), List::cons(Str::lit("!scripts/release-publication.test.mjs"), List::cons(Str::lit("!scripts/rust-oracle-tools-inputs.test.mjs"), List::cons(Str::lit("!scripts/sdk-candidate-sbom.test.mjs"), List::cons(Str::lit("!scripts/sdk-construction-handoff.test.mjs"), List::cons(Str::lit("!scripts/sdk-image-inputs.test.mjs"), List::cons(Str::lit("!scripts/sdk-release-evidence.mjs"), List::cons(Str::lit("!scripts/sdk-release-evidence.test.mjs"), List::cons(Str::lit("!scripts/sdk-vv-check.test.mjs"), List::cons(Str::lit("!scripts/unpublished-sdk-qualification.test.mjs"), List::cons(Str::lit("!prism-factory/"), List::cons(Str::lit("scripts/release-gate-evidence.mjs"), List::cons(Str::lit("scripts/release-phases.mjs"), List::cons(Str::lit("scripts/release-rebuild.sh"), List::cons(Str::lit("prism-factory/jobs/header.yml"), List::cons(Str::lit("prism-factory/jobs/image-platforms.yml"), List::cons(Str::lit("prism-factory/jobs/images.yml"), List::cons(Str::lit("prism-factory/jobs/reproducibility.yml"), List::cons(Str::lit("prism-factory/jobs/native.yml"), List::cons(Str::lit("prism-factory/jobs/installed-sdk.yml"), List::<Str>::nil())))))))))))))))))))))))))))))))))), List::<Str>::nil()), List::cons(f20(Str::lit("gate"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/gate"), List::cons(Str::lit(""), List::cons(Str::lit("@run"), List::<Str>::nil())), List::cons(Str::lit("plumbing"), List::cons(Str::lit("source-vv"), List::<Str>::nil()))), List::cons(f20(Str::lit("oci-native"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/oci-native"), List::cons(Str::lit(""), List::cons(Str::lit("@mode"), List::cons(Str::lit("@run"), List::<Str>::nil()))), List::cons(Str::lit("gate"), List::cons(Str::lit("build"), List::<Str>::nil()))), List::cons(f20(Str::lit("crates"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/crates"), List::cons(Str::lit(""), List::cons(Str::lit("@mode"), List::cons(Str::lit("@run"), List::<Str>::nil()))), List::cons(Str::lit("oci-native"), List::<Str>::nil())), List::cons(f20(Str::lit("release"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/release"), List::cons(Str::lit(""), List::cons(Str::lit("@mode"), List::cons(Str::lit("@run"), List::<Str>::nil()))), List::cons(Str::lit("factory-model"), List::cons(Str::lit("crates"), List::<Str>::nil()))), List::<Adt2>::nil()))))))))))
}

pub fn f5(v0: Fn0, p17: Adt0, p18: List<Bytes>) -> R<Adt0> {
    let m16 = (p17, p18);
    {
        let mut m16 = m16;
        loop {
            let step = {
                let (v1, v2) = m16;
                let m14 = v2;
                match m14.uncons() {
                    None => {
                        Err::<(Adt0, List<Bytes>), R<Adt0>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(Adt0, List<Bytes>), R<Adt0>>(({
                            let c15: Fn0 = v0.clone();
                            c15.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m16 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: Adt0, v1: Bytes) -> R<Adt0> {
    f21(v0, v1)
}

pub fn f7(v0: Bytes) -> R<List<Bytes>> {
    let v1: u64 = {
        let a19 = v0.clone();
        length_bytes(a19)
    };
    let v2: (u64, (u64, List<Bytes>)) = {
        let (h20, _) = f22(Fn4::F23(v0.clone()), v1, (0u64, (0u64, List::<Bytes>::nil())))?;
        h20
    };
    let v3: u64 = {
        let (h21, _) = {
            let (_, h22) = v2.clone();
            h22
        };
        h21
    };
    let v4: List<Bytes> = {
        let (_, h23) = {
            let (_, h24) = v2;
            h24
        };
        h23
    };
    let a25 = v3;
    let a26 = v1;
    if nat_lt(a25, a26) {
        Ok(f24(List::cons(f25(v0, v3, {
            let a27 = v1;
            let a28 = v3;
            nat_sub(a27, a28)
        }), v4)))
    } else {
        Ok(f24(v4))
    }
}

pub fn f8(v0: List<(Bytes, Bytes)>) -> List<(Bytes, Bytes)> {
    f26(Fn5::F27, List::<(Bytes, Bytes)>::nil(), v0)
}

pub fn f9(v0: List<(Str, Str)>) -> List<(Str, Str)> {
    f28(Fn6::F29, List::<(Str, Str)>::nil(), v0)
}

pub fn f10(v0: List<Bytes>) -> Bytes {
    f30(f24(v0))
}

pub fn f11(v0: Fn1, p32: List<Bytes>, p33: List<Adt1>) -> R<List<Bytes>> {
    let m31 = (p32, p33);
    {
        let mut m31 = m31;
        loop {
            let step = {
                let (v1, v2) = m31;
                let m29 = v2;
                match m29.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Adt1>), R<List<Bytes>>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Adt1>), R<List<Bytes>>>(({
                            let c30: Fn1 = v0.clone();
                            c30.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m31 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f12(v0: List<(Bytes, Bytes)>, v1: List<Bytes>, v2: Adt1) -> R<List<Bytes>> {
    Ok(List::cons(f31(v2, v0)?, v1))
}

pub fn f13(v0: Fn2, p37: Bytes, p38: List<(Str, u64)>) -> Bytes {
    let m36 = (p37, p38);
    {
        let mut m36 = m36;
        loop {
            let step = {
                let (v1, v2) = m36;
                let m34 = v2;
                match m34.uncons() {
                    None => {
                        Err::<(Bytes, List<(Str, u64)>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<(Str, u64)>), Bytes>(({
                            let c35: Fn2 = v0.clone();
                            c35.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m36 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14(v0: Bytes, v1: (Str, u64)) -> Bytes {
    let a40 = v0;
    let a41 = f16(List::cons(Str::lit("schedule"), List::cons({
        let (h39, _) = v1;
        h39
    }, List::<Str>::nil())));
    append_bytes(a40, a41)
}

pub fn f15(v0: List<Adt1>, v1: List<(Str, Str)>) -> List<(Str, u64)> {
    f32(Fn7::F33(v1), List::<(Str, u64)>::nil(), v0)
}

pub fn f16(v0: List<Str>) -> Bytes {
    f34(f35(Fn8::F36, List::<Bytes>::nil(), v0))
}

pub fn f17(v0: List<Adt1>) -> bool {
    f37(f38(v0))
}

pub fn f18(v0: Fn3, p45: (List<Adt1>, List<(List<Str>, Str)>), p46: List<Adt2>) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
    let m44 = (p45, p46);
    {
        let mut m44 = m44;
        loop {
            let step = {
                let (v1, v2) = m44;
                let m42 = v2;
                match m42.uncons() {
                    None => {
                        Err::<((List<Adt1>, List<(List<Str>, Str)>), List<Adt2>), R<(List<Adt1>, List<(List<Str>, Str)>)>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<((List<Adt1>, List<(List<Str>, Str)>), List<Adt2>), R<(List<Adt1>, List<(List<Str>, Str)>)>>(({
                            let c43: Fn3 = v0.clone();
                            c43.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m44 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f19(v0: List<(Str, bool)>, v1: List<(Bytes, Bytes)>, v2: (List<Adt1>, List<(List<Str>, Str)>), v3: Adt2) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
    f39(v1, v0, v2, v3)
}

pub fn f20(v0: Str, v1: Adt3, v2: Str, v3: Str, v4: List<Str>, v5: List<Str>) -> Adt2 {
    Adt2::C0(v0, v1, v2, v3, v4, v5)
}

pub fn f21(v0: Adt0, v1: Bytes) -> R<Adt0> {
    let v2: (Bytes, Bytes) = f40(v1)?;
    let v3: Bytes = {
        let (h47, _) = v2;
        h47
    };
    let v4: (Bytes, Bytes) = f40({
        let (_, h48) = v2;
        h48
    })?;
    let a49 = v3.clone();
    let a50 = f41(Str::lit("tree"));
    if equal(a49, a50) {
        Ok(Adt0::C0(List::cons(({
            let (h51, _) = v4;
            h51
        }, {
            let (_, h52) = v4;
            h52
        }), match v0.clone() {
            Adt0::C0(h53, _, _, _) => {
                h53
            }
        }), match v0.clone() {
            Adt0::C0(_, h54, _, _) => {
                h54
            }
        }, match v0.clone() {
            Adt0::C0(_, _, h55, _) => {
                h55
            }
        }, match v0 {
            Adt0::C0(_, _, _, h56) => {
                h56
            }
        }))
    } else {
        let a57 = v3.clone();
        let a58 = f41(Str::lit("receipt"));
        if equal(a57, a58) {
            Ok(Adt0::C0(match v0.clone() {
                Adt0::C0(h59, _, _, _) => {
                    h59
                }
            }, List::cons((f42({
                let (h60, _) = v4;
                h60
            }), f43({
                let (_, h61) = v4;
                h61
            })), match v0.clone() {
                Adt0::C0(_, h62, _, _) => {
                    h62
                }
            }), match v0.clone() {
                Adt0::C0(_, _, h63, _) => {
                    h63
                }
            }, match v0 {
                Adt0::C0(_, _, _, h64) => {
                    h64
                }
            }))
        } else {
            let a65 = v3.clone();
            let a66 = f41(Str::lit("weight"));
            if equal(a65, a66) {
                Ok(Adt0::C0(match v0.clone() {
                    Adt0::C0(h67, _, _, _) => {
                        h67
                    }
                }, match v0.clone() {
                    Adt0::C0(_, h68, _, _) => {
                        h68
                    }
                }, List::cons((f42({
                    let (h69, _) = v4;
                    h69
                }), f42({
                    let (_, h70) = v4;
                    h70
                })), match v0.clone() {
                    Adt0::C0(_, _, h71, _) => {
                        h71
                    }
                }), match v0 {
                    Adt0::C0(_, _, _, h72) => {
                        h72
                    }
                }))
            } else {
                let a73 = v3;
                let a74 = f41(Str::lit("outcome"));
                if equal(a73, a74) {
                    Ok(Adt0::C0(match v0.clone() {
                        Adt0::C0(h75, _, _, _) => {
                            h75
                        }
                    }, match v0.clone() {
                        Adt0::C0(_, h76, _, _) => {
                            h76
                        }
                    }, match v0.clone() {
                        Adt0::C0(_, _, h77, _) => {
                            h77
                        }
                    }, List::cons((f42({
                        let (h78, _) = v4;
                        h78
                    }), f43({
                        let (_, h79) = v4;
                        h79
                    })), match v0 {
                        Adt0::C0(_, _, _, h80) => {
                            h80
                        }
                    })))
                } else {
                    Ok(v0)
                }
            }
        }
    }
}

pub fn f22(v0: Fn4, p85: u64, p86: (u64, (u64, List<Bytes>))) -> R<((u64, (u64, List<Bytes>)), bool)> {
    let m84 = (p85, p86);
    {
        let mut m84 = m84;
        loop {
            let step = {
                let (v1, v2) = m84;
                let m81 = v1;
                if m81 == 0 {
                    Err::<(u64, (u64, (u64, List<Bytes>))), R<((u64, (u64, List<Bytes>)), bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m81 - 1;
                    let m83 = {
                        let c82: Fn4 = v0.clone();
                        c82.apply(v2.clone())?
                    };
                    match m83 {
                        None => {
                            Err::<(u64, (u64, (u64, List<Bytes>))), R<((u64, (u64, List<Bytes>)), bool)>>(Ok((v2, true)))
                        }
                        Some(v4) => {
                            Ok::<(u64, (u64, (u64, List<Bytes>))), R<((u64, (u64, List<Bytes>)), bool)>>((v3, v4))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m84 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f23(v0: Bytes, v1: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
    f44(v0, v1)
}

pub fn f24(v0: List<Bytes>) -> List<Bytes> {
    f45(Fn9::F46, List::<Bytes>::nil(), v0)
}

pub fn f25(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m90 = {
        let a87 = v0;
        let a88 = v1;
        let a89 = v2;
        slice_bytes(a87, a88, a89)
    };
    match m90 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f26(v0: Fn5, p94: List<(Bytes, Bytes)>, p95: List<(Bytes, Bytes)>) -> List<(Bytes, Bytes)> {
    let m93 = (p94, p95);
    {
        let mut m93 = m93;
        loop {
            let step = {
                let (v1, v2) = m93;
                let m91 = v2;
                match m91.uncons() {
                    None => {
                        Err::<(List<(Bytes, Bytes)>, List<(Bytes, Bytes)>), List<(Bytes, Bytes)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Bytes, Bytes)>, List<(Bytes, Bytes)>), List<(Bytes, Bytes)>>(({
                            let c92: Fn5 = v0.clone();
                            c92.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m93 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f27(v0: List<(Bytes, Bytes)>, v1: (Bytes, Bytes)) -> List<(Bytes, Bytes)> {
    List::cons(v1, v0)
}

pub fn f28(v0: Fn6, p99: List<(Str, Str)>, p100: List<(Str, Str)>) -> List<(Str, Str)> {
    let m98 = (p99, p100);
    {
        let mut m98 = m98;
        loop {
            let step = {
                let (v1, v2) = m98;
                let m96 = v2;
                match m96.uncons() {
                    None => {
                        Err::<(List<(Str, Str)>, List<(Str, Str)>), List<(Str, Str)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Str, Str)>, List<(Str, Str)>), List<(Str, Str)>>(({
                            let c97: Fn6 = v0.clone();
                            c97.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m98 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f29(v0: List<(Str, Str)>, v1: (Str, Str)) -> List<(Str, Str)> {
    List::cons(v1, v0)
}

pub fn f30(v0: List<Bytes>) -> Bytes {
    let m103 = {
        let (h101, _) = f47(Fn10::F48, {
            let a102 = v0.clone();
            length_list(a102)
        }, v0);
        h101
    };
    match m103.uncons() {
        None => {
            Bytes::lit(&[])
        }
        Some((v1, _)) => {
            v1
        }
    }
}

pub fn f31(v0: Adt1, v1: List<(Bytes, Bytes)>) -> R<Bytes> {
    let v2: Adt2 = match v0.clone() {
        Adt1::C0(h104, _, _) => {
            h104
        }
    };
    let v3: Str = match v2.clone() {
        Adt2::C0(h105, _, _, _, _, _) => {
            h105
        }
    };
    let v4: Bytes = f16(List::cons(Str::lit("obligation"), List::cons(v3.clone(), List::cons(f49(match v0.clone() {
        Adt1::C0(_, _, h106) => {
            h106
        }
    }), List::cons(match v0.clone() {
        Adt1::C0(_, h107, _) => {
            h107
        }
    }, List::<Str>::nil())))));
    let v5: Bytes = {
        let a113 = {
            let a110 = f16(List::cons(Str::lit("kind"), List::cons(v3.clone(), List::cons(f50(match v2.clone() {
                Adt2::C0(_, h108, _, _, _, _) => {
                    h108
                }
            }), List::<Str>::nil()))));
            let a111 = f16(List::cons(Str::lit("environment"), List::cons(v3.clone(), List::cons(match v2.clone() {
                Adt2::C0(_, _, h109, _, _, _) => {
                    h109
                }
            }, List::<Str>::nil()))));
            append_bytes(a110, a111)
        };
        let a114 = f16(List::cons(Str::lit("run"), List::cons(v3.clone(), List::cons(match v2.clone() {
            Adt2::C0(_, _, _, h112, _, _) => {
                h112
            }
        }, List::<Str>::nil()))));
        append_bytes(a113, a114)
    };
    let v6: Bytes = if f51(v0) {
        f52(v3.clone(), match v2.clone() {
            Adt2::C0(_, _, _, _, h115, _) => {
                h115
            }
        }, v1)?
    } else {
        Bytes::lit(&[])
    };
    let a124 = {
        let a116 = v4;
        let a117 = v5;
        append_bytes(a116, a117)
    };
    let a125 = {
        let a122 = {
            let a120 = f53(Str::lit("input"), v3.clone(), match v2.clone() {
                Adt2::C0(_, _, _, _, h118, _) => {
                    h118
                }
            });
            let a121 = f53(Str::lit("needs"), v3, match v2 {
                Adt2::C0(_, _, _, _, _, h119) => {
                    h119
                }
            });
            append_bytes(a120, a121)
        };
        let a123 = v6;
        append_bytes(a122, a123)
    };
    Ok(append_bytes(a124, a125))
}

pub fn f32(v0: Fn7, p129: List<(Str, u64)>, p130: List<Adt1>) -> List<(Str, u64)> {
    let m128 = (p129, p130);
    {
        let mut m128 = m128;
        loop {
            let step = {
                let (v1, v2) = m128;
                let m126 = v2;
                match m126.uncons() {
                    None => {
                        Err::<(List<(Str, u64)>, List<Adt1>), List<(Str, u64)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Str, u64)>, List<Adt1>), List<(Str, u64)>>(({
                            let c127: Fn7 = v0.clone();
                            c127.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m128 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f33(v0: List<(Str, Str)>, v1: List<(Str, u64)>, v2: Adt1) -> List<(Str, u64)> {
    if f54(match v2.clone() {
        Adt1::C0(_, _, h131) => {
            h131
        }
    }) {
        f55(({
            let m134 = match v2.clone() {
                Adt1::C0(h133, _, _) => {
                    h133
                }
            };
            match m134 {
                Adt2::C0(h132, _, _, _, _, _) => {
                    h132
                }
            }
        }, f56(f57({
            let m137 = match v2 {
                Adt1::C0(h136, _, _) => {
                    h136
                }
            };
            match m137 {
                Adt2::C0(h135, _, _, _, _, _) => {
                    h135
                }
            }
        }, v0))), v1)
    } else {
        v1
    }
}

pub fn f34(v0: List<Bytes>) -> Bytes {
    let a138 = f58(v0, Bytes::lit(&[0x20]));
    let a139 = Bytes::lit(&[0x0a]);
    append_bytes(a138, a139)
}

pub fn f35(v0: Fn8, p143: List<Bytes>, p144: List<Str>) -> List<Bytes> {
    let m142 = (p143, p144);
    {
        let mut m142 = m142;
        loop {
            let step = {
                let (v1, v2) = m142;
                let m140 = v2;
                match m140.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Str>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Str>), List<Bytes>>(({
                            let c141: Fn8 = v0.clone();
                            c141.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m142 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f36(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a145 = v0;
    let a146 = List::cons(f41(v1), List::<Bytes>::nil());
    append_list(a145, a146)
}

pub fn f37(v0: List<Adt4>) -> bool {
    f59(Fn11::F60, true, v0)
}

pub fn f38(v0: List<Adt1>) -> List<Adt4> {
    f61(Fn12::F62, List::<Adt4>::nil(), v0)
}

pub fn f39(v0: List<(Bytes, Bytes)>, v1: List<(Str, bool)>, v2: (List<Adt1>, List<(List<Str>, Str)>), v3: Adt2) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
    let v4: List<Adt1> = {
        let (h147, _) = v2;
        h147
    };
    let v5: List<(List<Str>, Str)> = {
        let (_, h148) = v2;
        h148
    };
    let v6: List<Str> = match v3.clone() {
        Adt2::C0(_, _, _, _, h149, _) => {
            h149
        }
    };
    let v8: Str = {
        let m150 = f63(v6.clone(), v5.clone());
        match m150 {
            None => {
                f64(v6.clone(), v0)?
            }
            Some(v7) => {
                v7
            }
        }
    };
    let v9: List<Adt1> = f65(match v3.clone() {
        Adt2::C0(_, _, _, _, _, h151) => {
            h151
        }
    }, v4.clone());
    let v10: Str = f66(v3.clone(), v8.clone(), v9.clone())?;
    let v11: List<Adt4> = if f67(match v3.clone() {
        Adt2::C0(_, _, _, _, _, h152) => {
            h152
        }
    }, v4.clone()) {
        f38(v9)
    } else {
        List::cons(Adt4::C3, List::<Adt4>::nil())
    };
    Ok(({
        let a154 = v4;
        let a155 = List::cons(Adt1::C0(v3.clone(), v10.clone(), f68(match v3 {
            Adt2::C0(_, h153, _, _, _, _) => {
                h153
            }
        }, v10, v1, v11)), List::<Adt1>::nil());
        append_list(a154, a155)
    }, List::cons((v6, v8), v5)))
}

pub fn f40(v0: Bytes) -> R<(Bytes, Bytes)> {
    let m156 = f69(v0.clone(), Bytes::lit(&[0x20]))?;
    match m156 {
        None => {
            Ok((v0, Bytes::lit(&[])))
        }
        Some(v1) => {
            Ok((f25(v0.clone(), 0u64, v1), f25(v0.clone(), {
                let a157 = v1;
                let a158 = 1u64;
                nat_add(a157, a158)?
            }, {
                let a162 = {
                    let a159 = v0;
                    length_bytes(a159)
                };
                let a163 = {
                    let a160 = v1;
                    let a161 = 1u64;
                    nat_add(a160, a161)?
                };
                nat_sub(a162, a163)
            })))
        }
    }
}

pub fn f41(v0: Str) -> Bytes {
    let a164 = v0;
    utf8_encode(a164)
}

pub fn f42(v0: Bytes) -> Str {
    let m166 = {
        let a165 = v0;
        utf8_decode(a165)
    };
    match m166 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f43(v0: Bytes) -> bool {
    let a167 = v0;
    let a168 = f41(Str::lit("pass"));
    equal(a167, a168)
}

pub fn f44(v0: Bytes, v1: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
    let v2: u64 = {
        let (h169, _) = v1.clone();
        h169
    };
    let v3: u64 = {
        let (h170, _) = {
            let (_, h171) = v1.clone();
            h171
        };
        h170
    };
    let v4: List<Bytes> = {
        let (_, h172) = {
            let (_, h173) = v1;
            h173
        };
        h172
    };
    let m174 = f70(v0.clone(), v2);
    match m174 {
        None => {
            Ok(None::<(u64, (u64, List<Bytes>))>)
        }
        Some(v5) => {
            let a175 = v5;
            let a176 = 10u8;
            if equal(a175, a176) {
                Ok(Some(({
                    let a177 = v2;
                    let a178 = 1u64;
                    nat_add(a177, a178)?
                }, ({
                    let a179 = v2;
                    let a180 = 1u64;
                    nat_add(a179, a180)?
                }, List::cons(f71(f25(v0, v3, {
                    let a181 = v2;
                    let a182 = v3;
                    nat_sub(a181, a182)
                })), v4)))))
            } else {
                Ok(Some(({
                    let a183 = v2;
                    let a184 = 1u64;
                    nat_add(a183, a184)?
                }, (v3, v4))))
            }
        }
    }
}

pub fn f45(v0: Fn9, p188: List<Bytes>, p189: List<Bytes>) -> List<Bytes> {
    let m187 = (p188, p189);
    {
        let mut m187 = m187;
        loop {
            let step = {
                let (v1, v2) = m187;
                let m185 = v2;
                match m185.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Bytes>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Bytes>), List<Bytes>>(({
                            let c186: Fn9 = v0.clone();
                            c186.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m187 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f46(v0: List<Bytes>, v1: Bytes) -> List<Bytes> {
    List::cons(v1, v0)
}

pub fn f47(v0: Fn10, p194: u64, p195: List<Bytes>) -> (List<Bytes>, bool) {
    let m193 = (p194, p195);
    {
        let mut m193 = m193;
        loop {
            let step = {
                let (v1, v2) = m193;
                let m190 = v1;
                if m190 == 0 {
                    Err::<(u64, List<Bytes>), (List<Bytes>, bool)>((v2, false))
                } else {
                    let v3 = m190 - 1;
                    let m192 = {
                        let c191: Fn10 = v0.clone();
                        c191.apply(v2.clone())
                    };
                    match m192 {
                        None => {
                            Err::<(u64, List<Bytes>), (List<Bytes>, bool)>((v2, true))
                        }
                        Some(v4) => {
                            Ok::<(u64, List<Bytes>), (List<Bytes>, bool)>((v3, v4))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m193 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f48(v0: List<Bytes>) -> Option<List<Bytes>> {
    f72(v0)
}

pub fn f49(v0: Adt4) -> Str {
    let m196 = v0;
    match m196 {
        Adt4::C0 => {
            Str::lit("proved")
        }
        Adt4::C1 => {
            Str::lit("observe")
        }
        Adt4::C2 => {
            Str::lit("refuted")
        }
        Adt4::C3 => {
            Str::lit("blocked")
        }
    }
}

pub fn f50(v0: Adt3) -> Str {
    let m197 = v0;
    match m197 {
        Adt3::C0 => {
            Str::lit("hermetic")
        }
        Adt3::C1 => {
            Str::lit("fetch")
        }
    }
}

pub fn f51(v0: Adt1) -> bool {
    if f54(match v0.clone() {
        Adt1::C0(_, _, h198) => {
            h198
        }
    }) {
        let a202 = {
            let m201 = match v0 {
                Adt1::C0(h200, _, _) => {
                    h200
                }
            };
            match m201 {
                Adt2::C0(_, _, h199, _, _, _) => {
                    h199
                }
            }
        };
        let a203 = Str::lit("host");
        equal(a202, a203)
    } else {
        false
    }
}

pub fn f52(v0: Str, v1: List<Str>, v2: List<(Bytes, Bytes)>) -> R<Bytes> {
    Ok(f10(f73(Fn13::F74(v1, v0), List::<Bytes>::nil(), v2)?))
}

pub fn f53(v0: Str, v1: Str, v2: List<Str>) -> Bytes {
    f10(f35(Fn8::F75(v1, v0), List::<Bytes>::nil(), v2))
}

pub fn f54(v0: Adt4) -> bool {
    let m204 = v0;
    match m204 {
        Adt4::C0 => {
            false
        }
        Adt4::C1 => {
            true
        }
        Adt4::C2 => {
            false
        }
        Adt4::C3 => {
            false
        }
    }
}

pub fn f55(v0: (Str, u64), v1: List<(Str, u64)>) -> List<(Str, u64)> {
    let m205 = v1;
    match m205.uncons() {
        None => {
            List::cons(v0, List::<(Str, u64)>::nil())
        }
        Some((v2, v3)) => {
            let a208 = {
                let (_, h206) = v2.clone();
                h206
            };
            let a209 = {
                let (_, h207) = v0.clone();
                h207
            };
            if nat_lt(a208, a209) {
                List::cons(v0, List::cons(v2, v3))
            } else {
                List::cons(v2, f55(v0, v3))
            }
        }
    }
}

pub fn f56(v0: Adt5) -> u64 {
    let m210 = v0;
    match m210 {
        Adt5::C0 => {
            1u64
        }
        Adt5::C1 => {
            2u64
        }
        Adt5::C2 => {
            3u64
        }
        Adt5::C3 => {
            4u64
        }
        Adt5::C4 => {
            5u64
        }
        Adt5::C5 => {
            6u64
        }
        Adt5::C6 => {
            7u64
        }
        Adt5::C7 => {
            8u64
        }
        Adt5::C8 => {
            9u64
        }
        Adt5::C9 => {
            10u64
        }
        Adt5::C10 => {
            11u64
        }
        Adt5::C11 => {
            12u64
        }
        Adt5::C12 => {
            13u64
        }
        Adt5::C13 => {
            14u64
        }
        Adt5::C14 => {
            15u64
        }
        Adt5::C15 => {
            16u64
        }
    }
}

pub fn f57(v0: Str, v1: List<(Str, Str)>) -> Adt5 {
    f76(f77(Fn14::F78(v0), Str::lit("4"), v1))
}

pub fn f58(v0: List<Bytes>, v1: Bytes) -> Bytes {
    let m211 = f79(Fn15::F80(v1), None::<Bytes>, v0);
    match m211 {
        None => {
            Bytes::lit(&[])
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f59(v0: Fn11, p215: bool, p216: List<Adt4>) -> bool {
    let m214 = (p215, p216);
    {
        let mut m214 = m214;
        loop {
            let step = {
                let (v1, v2) = m214;
                let m212 = v2;
                match m212.uncons() {
                    None => {
                        Err::<(bool, List<Adt4>), bool>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(bool, List<Adt4>), bool>(({
                            let c213: Fn11 = v0.clone();
                            c213.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m214 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f60(v0: bool, v1: Adt4) -> bool {
    if v0 {
        f81(v1)
    } else {
        false
    }
}

pub fn f61(v0: Fn12, p220: List<Adt4>, p221: List<Adt1>) -> List<Adt4> {
    let m219 = (p220, p221);
    {
        let mut m219 = m219;
        loop {
            let step = {
                let (v1, v2) = m219;
                let m217 = v2;
                match m217.uncons() {
                    None => {
                        Err::<(List<Adt4>, List<Adt1>), List<Adt4>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Adt4>, List<Adt1>), List<Adt4>>(({
                            let c218: Fn12 = v0.clone();
                            c218.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m219 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f62(v0: List<Adt4>, v1: Adt1) -> List<Adt4> {
    let a223 = v0;
    let a224 = List::cons(match v1 {
        Adt1::C0(_, _, h222) => {
            h222
        }
    }, List::<Adt4>::nil());
    append_list(a223, a224)
}

pub fn f63(v0: List<Str>, v1: List<(List<Str>, Str)>) -> Option<Str> {
    f82(Fn16::F83(v0), None::<Str>, v1)
}

pub fn f64(v0: List<Str>, v1: List<(Bytes, Bytes)>) -> R<Str> {
    Ok(f42(f84(f85(v0, v1)?)?))
}

pub fn f65(v0: List<Str>, v1: List<Adt1>) -> List<Adt1> {
    f86(Fn17::F87(v1), List::<Adt1>::nil(), v0)
}

pub fn f66(v0: Adt2, v1: Str, v2: List<Adt1>) -> R<Str> {
    Ok(f42(f84(f88(v0, v1, v2)?)?))
}

pub fn f67(v0: List<Str>, v1: List<Adt1>) -> bool {
    f89(Fn18::F90(v1), true, v0)
}

pub fn f68(v0: Adt3, v1: Str, v2: List<(Str, bool)>, v3: List<Adt4>) -> Adt4 {
    if f37(v3) {
        let m225 = f91(v1, v2);
        match m225 {
            None => {
                Adt4::C1
            }
            Some(v4) => {
                if v4 {
                    Adt4::C0
                } else {
                    let m226 = v0;
                    match m226 {
                        Adt3::C0 => {
                            Adt4::C2
                        }
                        Adt3::C1 => {
                            Adt4::C1
                        }
                    }
                }
            }
        }
    } else {
        Adt4::C3
    }
}

pub fn f69(v0: Bytes, v1: Bytes) -> R<Option<u64>> {
    let v2: u64 = {
        let a227 = v0.clone();
        length_bytes(a227)
    };
    let v3: u64 = {
        let a228 = v1.clone();
        length_bytes(a228)
    };
    let v4: u64 = {
        let (h229, _) = f92(Fn19::F93(v1.clone(), v3, v0.clone(), v2), {
            let a230 = v2;
            let a231 = 1u64;
            nat_add(a230, a231)?
        }, 0u64)?;
        h229
    };
    if f94(v0, v1, v4) {
        Ok(Some(v4))
    } else {
        Ok(None::<u64>)
    }
}

pub fn f70(v0: Bytes, v1: u64) -> Option<u8> {
    let a232 = v0;
    let a233 = v1;
    index_bytes(a232, a233)
}

pub fn f71(v0: Bytes) -> Bytes {
    if f95(v0.clone(), Bytes::lit(&[0x0d])) {
        f25(v0.clone(), 0u64, {
            let a235 = {
                let a234 = v0;
                length_bytes(a234)
            };
            let a236 = 1u64;
            nat_sub(a235, a236)
        })
    } else {
        v0
    }
}

pub fn f72(v0: List<Bytes>) -> Option<List<Bytes>> {
    let m237 = v0.clone();
    match m237.uncons() {
        None => {
            None::<List<Bytes>>
        }
        Some((_, v2)) => {
            let m238 = v2;
            match m238.uncons() {
                None => {
                    None::<List<Bytes>>
                }
                Some((_, _)) => {
                    let v5: (Option<Bytes>, List<Bytes>) = f96(Fn20::F97, (None::<Bytes>, List::<Bytes>::nil()), v0);
                    let m240 = {
                        let (h239, _) = v5;
                        h239
                    };
                    match m240 {
                        None => {
                            Some(f24({
                                let (_, h242) = v5;
                                h242
                            }))
                        }
                        Some(v6) => {
                            Some(f24(List::cons(v6, {
                                let (_, h241) = v5;
                                h241
                            })))
                        }
                    }
                }
            }
        }
    }
}

pub fn f73(v0: Fn13, p246: List<Bytes>, p247: List<(Bytes, Bytes)>) -> R<List<Bytes>> {
    let m245 = (p246, p247);
    {
        let mut m245 = m245;
        loop {
            let step = {
                let (v1, v2) = m245;
                let m243 = v2;
                match m243.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<(Bytes, Bytes)>), R<List<Bytes>>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<(Bytes, Bytes)>), R<List<Bytes>>>(({
                            let c244: Fn13 = v0.clone();
                            c244.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m245 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f74(v0: List<Str>, v1: Str, v2: List<Bytes>, v3: (Bytes, Bytes)) -> R<List<Bytes>> {
    if f98(v0, {
        let (_, h248) = v3.clone();
        h248
    })? {
        Ok(List::cons(f34(List::cons(f41(Str::lit("file")), List::cons(f41(v1), List::cons({
            let (_, h249) = v3;
            h249
        }, List::<Bytes>::nil())))), v2))
    } else {
        Ok(v2)
    }
}

pub fn f75(v0: Str, v1: Str, v2: List<Bytes>, v3: Str) -> List<Bytes> {
    List::cons(f16(List::cons(v1, List::cons(v0, List::cons(v3, List::<Str>::nil())))), v2)
}

pub fn f76(v0: Str) -> Adt5 {
    let a250 = v0.clone();
    let a251 = Str::lit("1");
    if equal(a250, a251) {
        Adt5::C0
    } else {
        let a252 = v0.clone();
        let a253 = Str::lit("2");
        if equal(a252, a253) {
            Adt5::C1
        } else {
            let a254 = v0.clone();
            let a255 = Str::lit("3");
            if equal(a254, a255) {
                Adt5::C2
            } else {
                let a256 = v0.clone();
                let a257 = Str::lit("4");
                if equal(a256, a257) {
                    Adt5::C3
                } else {
                    let a258 = v0.clone();
                    let a259 = Str::lit("5");
                    if equal(a258, a259) {
                        Adt5::C4
                    } else {
                        let a260 = v0.clone();
                        let a261 = Str::lit("6");
                        if equal(a260, a261) {
                            Adt5::C5
                        } else {
                            let a262 = v0.clone();
                            let a263 = Str::lit("7");
                            if equal(a262, a263) {
                                Adt5::C6
                            } else {
                                let a264 = v0.clone();
                                let a265 = Str::lit("8");
                                if equal(a264, a265) {
                                    Adt5::C7
                                } else {
                                    let a266 = v0.clone();
                                    let a267 = Str::lit("9");
                                    if equal(a266, a267) {
                                        Adt5::C8
                                    } else {
                                        let a268 = v0.clone();
                                        let a269 = Str::lit("10");
                                        if equal(a268, a269) {
                                            Adt5::C9
                                        } else {
                                            let a270 = v0.clone();
                                            let a271 = Str::lit("11");
                                            if equal(a270, a271) {
                                                Adt5::C10
                                            } else {
                                                let a272 = v0.clone();
                                                let a273 = Str::lit("12");
                                                if equal(a272, a273) {
                                                    Adt5::C11
                                                } else {
                                                    let a274 = v0.clone();
                                                    let a275 = Str::lit("13");
                                                    if equal(a274, a275) {
                                                        Adt5::C12
                                                    } else {
                                                        let a276 = v0.clone();
                                                        let a277 = Str::lit("14");
                                                        if equal(a276, a277) {
                                                            Adt5::C13
                                                        } else {
                                                            let a278 = v0.clone();
                                                            let a279 = Str::lit("15");
                                                            if equal(a278, a279) {
                                                                Adt5::C14
                                                            } else {
                                                                let a280 = v0;
                                                                let a281 = Str::lit("16");
                                                                if equal(a280, a281) {
                                                                    Adt5::C15
                                                                } else {
                                                                    Adt5::C3
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn f77(v0: Fn14, p285: Str, p286: List<(Str, Str)>) -> Str {
    let m284 = (p285, p286);
    {
        let mut m284 = m284;
        loop {
            let step = {
                let (v1, v2) = m284;
                let m282 = v2;
                match m282.uncons() {
                    None => {
                        Err::<(Str, List<(Str, Str)>), Str>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Str, List<(Str, Str)>), Str>(({
                            let c283: Fn14 = v0.clone();
                            c283.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m284 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f78(v0: Str, v1: Str, v2: (Str, Str)) -> Str {
    let a288 = {
        let (h287, _) = v2;
        h287
    };
    let a289 = v0;
    if equal(a288, a289) {
        let (_, h290) = v2;
        h290
    } else {
        v1
    }
}

pub fn f79(v0: Fn15, p294: Option<Bytes>, p295: List<Bytes>) -> Option<Bytes> {
    let m293 = (p294, p295);
    {
        let mut m293 = m293;
        loop {
            let step = {
                let (v1, v2) = m293;
                let m291 = v2;
                match m291.uncons() {
                    None => {
                        Err::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(({
                            let c292: Fn15 = v0.clone();
                            c292.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m293 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f80(v0: Bytes, v1: Option<Bytes>, v2: Bytes) -> Option<Bytes> {
    let m296 = v1;
    match m296 {
        None => {
            Some(v2)
        }
        Some(v3) => {
            Some({
                let a299 = {
                    let a297 = v3;
                    let a298 = v0;
                    append_bytes(a297, a298)
                };
                let a300 = v2;
                append_bytes(a299, a300)
            })
        }
    }
}

pub fn f81(v0: Adt4) -> bool {
    let m301 = v0;
    match m301 {
        Adt4::C0 => {
            true
        }
        Adt4::C1 => {
            false
        }
        Adt4::C2 => {
            false
        }
        Adt4::C3 => {
            false
        }
    }
}

pub fn f82(v0: Fn16, p305: Option<Str>, p306: List<(List<Str>, Str)>) -> Option<Str> {
    let m304 = (p305, p306);
    {
        let mut m304 = m304;
        loop {
            let step = {
                let (v1, v2) = m304;
                let m302 = v2;
                match m302.uncons() {
                    None => {
                        Err::<(Option<Str>, List<(List<Str>, Str)>), Option<Str>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Str>, List<(List<Str>, Str)>), Option<Str>>(({
                            let c303: Fn16 = v0.clone();
                            c303.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m304 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f83(v0: List<Str>, v1: Option<Str>, v2: (List<Str>, Str)) -> Option<Str> {
    if f99({
        let (h307, _) = v2;
        h307
    }, v0) {
        Some({
            let (_, h308) = v2;
            h308
        })
    } else {
        v1
    }
}

pub fn f84(v0: Bytes) -> R<Bytes> {
    Ok(f100(f101(v0)?))
}

pub fn f85(v0: List<Str>, v1: List<(Bytes, Bytes)>) -> R<Bytes> {
    Ok(f10(f73(Fn13::F102(v0), List::<Bytes>::nil(), v1)?))
}

pub fn f86(v0: Fn17, p312: List<Adt1>, p313: List<Str>) -> List<Adt1> {
    let m311 = (p312, p313);
    {
        let mut m311 = m311;
        loop {
            let step = {
                let (v1, v2) = m311;
                let m309 = v2;
                match m309.uncons() {
                    None => {
                        Err::<(List<Adt1>, List<Str>), List<Adt1>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Adt1>, List<Str>), List<Adt1>>(({
                            let c310: Fn17 = v0.clone();
                            c310.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m311 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f87(v0: List<Adt1>, v1: List<Adt1>, v2: Str) -> List<Adt1> {
    let m314 = f103(v2, v0);
    match m314 {
        None => {
            v1
        }
        Some(v3) => {
            let a315 = v1;
            let a316 = List::cons(v3, List::<Adt1>::nil());
            append_list(a315, a316)
        }
    }
}

pub fn f88(v0: Adt2, v1: Str, v2: List<Adt1>) -> R<Bytes> {
    Ok(f30(List::cons(f34(List::cons(f41(Str::lit("prism-factory/obligation/1")), List::<Bytes>::nil())), List::cons(f34(List::cons(f41(Str::lit("name")), List::cons(f41(match v0.clone() {
        Adt2::C0(h317, _, _, _, _, _) => {
            h317
        }
    }), List::<Bytes>::nil()))), List::cons(f34(List::cons(f41(Str::lit("kind")), List::cons(f41(f50(match v0.clone() {
        Adt2::C0(_, h318, _, _, _, _) => {
            h318
        }
    })), List::<Bytes>::nil()))), List::cons(f34(List::cons(f41(Str::lit("environment")), List::cons(f41(match v0.clone() {
        Adt2::C0(_, _, h319, _, _, _) => {
            h319
        }
    }), List::<Bytes>::nil()))), List::cons(f34(List::cons(f41(Str::lit("run")), List::cons(f41(match v0.clone() {
        Adt2::C0(_, _, _, h320, _, _) => {
            h320
        }
    }), List::<Bytes>::nil()))), List::cons(f104(Str::lit("input"), match v0 {
        Adt2::C0(_, _, _, _, h321, _) => {
            h321
        }
    }), List::cons(f105(v2)?, List::cons(f34(List::cons(f41(Str::lit("closure")), List::cons(f41(v1), List::<Bytes>::nil()))), List::<Bytes>::nil()))))))))))
}

pub fn f89(v0: Fn18, p325: bool, p326: List<Str>) -> bool {
    let m324 = (p325, p326);
    {
        let mut m324 = m324;
        loop {
            let step = {
                let (v1, v2) = m324;
                let m322 = v2;
                match m322.uncons() {
                    None => {
                        Err::<(bool, List<Str>), bool>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(bool, List<Str>), bool>(({
                            let c323: Fn18 = v0.clone();
                            c323.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m324 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f90(v0: List<Adt1>, v1: bool, v2: Str) -> bool {
    let m327 = f103(v2, v0);
    match m327 {
        None => {
            false
        }
        Some(_) => {
            v1
        }
    }
}

pub fn f91(v0: Str, v1: List<(Str, bool)>) -> Option<bool> {
    f106(Fn21::F107(v0), None::<bool>, v1)
}

pub fn f92(v0: Fn19, p332: u64, p333: u64) -> R<(u64, bool)> {
    let m331 = (p332, p333);
    {
        let mut m331 = m331;
        loop {
            let step = {
                let (v1, v2) = m331;
                let m328 = v1;
                if m328 == 0 {
                    Err::<(u64, u64), R<(u64, bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m328 - 1;
                    let m330 = {
                        let c329: Fn19 = v0.clone();
                        c329.apply(v2)?
                    };
                    match m330 {
                        None => {
                            Err::<(u64, u64), R<(u64, bool)>>(Ok((v2, true)))
                        }
                        Some(v4) => {
                            Ok::<(u64, u64), R<(u64, bool)>>((v3, v4))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m331 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f93(v0: Bytes, v1: u64, v2: Bytes, v3: u64, v4: u64) -> R<Option<u64>> {
    if f94(v2, v0, v4) {
        Ok(None::<u64>)
    } else {
        let a336 = {
            let a334 = v4;
            let a335 = v1;
            nat_add(a334, a335)?
        };
        let a337 = v3;
        if nat_lt(a336, a337) {
            Ok(Some({
                let a338 = v4;
                let a339 = 1u64;
                nat_add(a338, a339)?
            }))
        } else {
            Ok(None::<u64>)
        }
    }
}

pub fn f94(v0: Bytes, v1: Bytes, v2: u64) -> bool {
    let m344 = {
        let a341 = v0;
        let a342 = v2;
        let a343 = {
            let a340 = v1.clone();
            length_bytes(a340)
        };
        slice_bytes(a341, a342, a343)
    };
    match m344 {
        None => {
            false
        }
        Some(v3) => {
            let a345 = v3;
            let a346 = v1;
            equal(a345, a346)
        }
    }
}

pub fn f95(v0: Bytes, v1: Bytes) -> bool {
    let v2: u64 = {
        let a347 = v0.clone();
        length_bytes(a347)
    };
    let v3: u64 = {
        let a348 = v1.clone();
        length_bytes(a348)
    };
    let a349 = v3;
    let a350 = v2;
    if nat_le(a349, a350) {
        let m356 = {
            let a353 = v0;
            let a354 = {
                let a351 = v2;
                let a352 = v3;
                nat_sub(a351, a352)
            };
            let a355 = v3;
            slice_bytes(a353, a354, a355)
        };
        match m356 {
            None => {
                false
            }
            Some(v4) => {
                let a357 = v4;
                let a358 = v1;
                equal(a357, a358)
            }
        }
    } else {
        false
    }
}

pub fn f96(v0: Fn20, p362: (Option<Bytes>, List<Bytes>), p363: List<Bytes>) -> (Option<Bytes>, List<Bytes>) {
    let m361 = (p362, p363);
    {
        let mut m361 = m361;
        loop {
            let step = {
                let (v1, v2) = m361;
                let m359 = v2;
                match m359.uncons() {
                    None => {
                        Err::<((Option<Bytes>, List<Bytes>), List<Bytes>), (Option<Bytes>, List<Bytes>)>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<((Option<Bytes>, List<Bytes>), List<Bytes>), (Option<Bytes>, List<Bytes>)>(({
                            let c360: Fn20 = v0.clone();
                            c360.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m361 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f97(v0: (Option<Bytes>, List<Bytes>), v1: Bytes) -> (Option<Bytes>, List<Bytes>) {
    f108(v0, v1)
}

pub fn f98(v0: List<Str>, v1: Bytes) -> R<bool> {
    let a364 = f109(v0.clone(), v1.clone(), true)?;
    let a365 = f109(v0, v1, false)?;
    Ok(nat_lt(a364, a365))
}

pub fn f99(p372: List<Str>, p373: List<Str>) -> bool {
    let m371 = (p372, p373);
    {
        let mut m371 = m371;
        loop {
            let step = {
                let (v0, v1) = m371;
                let m366 = v0;
                match m366.uncons() {
                    None => {
                        let m367 = v1;
                        match m367.uncons() {
                            None => {
                                Err::<(List<Str>, List<Str>), bool>(true)
                            }
                            Some((_, _)) => {
                                Err::<(List<Str>, List<Str>), bool>(false)
                            }
                        }
                    }
                    Some((v4, v5)) => {
                        let m368 = v1;
                        match m368.uncons() {
                            None => {
                                Err::<(List<Str>, List<Str>), bool>(false)
                            }
                            Some((v6, v7)) => {
                                let a369 = v4;
                                let a370 = v6;
                                if equal(a369, a370) {
                                    Ok::<(List<Str>, List<Str>), bool>((v5, v7))
                                } else {
                                    Err::<(List<Str>, List<Str>), bool>(false)
                                }
                            }
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m371 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f100(v0: Adt6) -> Bytes {
    let a394 = f110(match v0.clone() {
        Adt6::C0(h374, _, _, _, _, _, _, _) => {
            h374
        }
    });
    let a395 = {
        let a392 = f110(match v0.clone() {
            Adt6::C0(_, h375, _, _, _, _, _, _) => {
                h375
            }
        });
        let a393 = {
            let a390 = f110(match v0.clone() {
                Adt6::C0(_, _, h376, _, _, _, _, _) => {
                    h376
                }
            });
            let a391 = {
                let a388 = f110(match v0.clone() {
                    Adt6::C0(_, _, _, h377, _, _, _, _) => {
                        h377
                    }
                });
                let a389 = {
                    let a386 = f110(match v0.clone() {
                        Adt6::C0(_, _, _, _, h378, _, _, _) => {
                            h378
                        }
                    });
                    let a387 = {
                        let a384 = f110(match v0.clone() {
                            Adt6::C0(_, _, _, _, _, h379, _, _) => {
                                h379
                            }
                        });
                        let a385 = {
                            let a382 = f110(match v0.clone() {
                                Adt6::C0(_, _, _, _, _, _, h380, _) => {
                                    h380
                                }
                            });
                            let a383 = f110(match v0 {
                                Adt6::C0(_, _, _, _, _, _, _, h381) => {
                                    h381
                                }
                            });
                            append_bytes(a382, a383)
                        };
                        append_bytes(a384, a385)
                    };
                    append_bytes(a386, a387)
                };
                append_bytes(a388, a389)
            };
            append_bytes(a390, a391)
        };
        append_bytes(a392, a393)
    };
    append_bytes(a394, a395)
}

pub fn f101(v0: Bytes) -> R<Adt6> {
    let v1: u64 = {
        let a396 = v0.clone();
        length_bytes(a396)
    };
    let v2: u64 = {
        let a402 = {
            let a399 = {
                let a397 = v1;
                let a398 = 72u64;
                nat_add(a397, a398)?
            };
            let a400 = 64u64;
            let a401 = 0u64;
            nat_quot(a399, a400, a401)
        };
        let a403 = 64u64;
        nat_mul(a402, a403)?
    };
    let (_, h404) = f111(Fn22::F112(v0, v1, v2), {
        let a405 = v2;
        let a406 = 64u64;
        let a407 = 0u64;
        nat_quot(a405, a406, a407)
    }, (0u64, f113()))?;
    Ok(h404)
}

pub fn f102(v0: List<Str>, v1: List<Bytes>, v2: (Bytes, Bytes)) -> R<List<Bytes>> {
    if f98(v0, {
        let (_, h408) = v2.clone();
        h408
    })? {
        Ok(List::cons(f34(List::cons(f41(Str::lit("file")), List::cons({
            let (h409, _) = v2;
            h409
        }, List::cons({
            let (_, h410) = v2;
            h410
        }, List::<Bytes>::nil())))), v1))
    } else {
        Ok(v1)
    }
}

pub fn f103(v0: Str, v1: List<Adt1>) -> Option<Adt1> {
    f114(Fn23::F115(v0), None::<Adt1>, v1)
}

pub fn f104(v0: Str, v1: List<Str>) -> Bytes {
    f10(f35(Fn8::F116(v0), List::<Bytes>::nil(), v1))
}

pub fn f105(v0: List<Adt1>) -> R<Bytes> {
    Ok(f10(f11(Fn1::F117, List::<Bytes>::nil(), v0)?))
}

pub fn f106(v0: Fn21, p414: Option<bool>, p415: List<(Str, bool)>) -> Option<bool> {
    let m413 = (p414, p415);
    {
        let mut m413 = m413;
        loop {
            let step = {
                let (v1, v2) = m413;
                let m411 = v2;
                match m411.uncons() {
                    None => {
                        Err::<(Option<bool>, List<(Str, bool)>), Option<bool>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<bool>, List<(Str, bool)>), Option<bool>>(({
                            let c412: Fn21 = v0.clone();
                            c412.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m413 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f107(v0: Str, v1: Option<bool>, v2: (Str, bool)) -> Option<bool> {
    let a417 = {
        let (h416, _) = v2;
        h416
    };
    let a418 = v0;
    if equal(a417, a418) {
        let m419 = v1;
        match m419 {
            None => {
                Some({
                    let (_, h421) = v2;
                    h421
                })
            }
            Some(v3) => {
                Some(if v3 {
                    true
                } else {
                    let (_, h420) = v2;
                    h420
                })
            }
        }
    } else {
        v1
    }
}

pub fn f108(v0: (Option<Bytes>, List<Bytes>), v1: Bytes) -> (Option<Bytes>, List<Bytes>) {
    let m423 = {
        let (h422, _) = v0;
        h422
    };
    match m423 {
        None => {
            (Some(v1), {
                let (_, h424) = v0;
                h424
            })
        }
        Some(v2) => {
            (None::<Bytes>, List::cons({
                let a425 = v2;
                let a426 = v1;
                append_bytes(a425, a426)
            }, {
                let (_, h427) = v0;
                h427
            }))
        }
    }
}

pub fn f109(v0: List<Str>, v1: Bytes, v2: bool) -> R<u64> {
    f118(Fn24::F119(v2, v1), 0u64, v0)
}

pub fn f110(v0: u32) -> Bytes {
    let a456 = f120({
        let a428 = f121(v0, 28u32);
        let a429 = 15u32;
        fixed_u32::bit_and(a428, a429)
    });
    let a457 = {
        let a454 = f120({
            let a430 = f121(v0, 24u32);
            let a431 = 15u32;
            fixed_u32::bit_and(a430, a431)
        });
        let a455 = {
            let a452 = f120({
                let a432 = f121(v0, 20u32);
                let a433 = 15u32;
                fixed_u32::bit_and(a432, a433)
            });
            let a453 = {
                let a450 = f120({
                    let a434 = f121(v0, 16u32);
                    let a435 = 15u32;
                    fixed_u32::bit_and(a434, a435)
                });
                let a451 = {
                    let a448 = f120({
                        let a436 = f121(v0, 12u32);
                        let a437 = 15u32;
                        fixed_u32::bit_and(a436, a437)
                    });
                    let a449 = {
                        let a446 = f120({
                            let a438 = f121(v0, 8u32);
                            let a439 = 15u32;
                            fixed_u32::bit_and(a438, a439)
                        });
                        let a447 = {
                            let a444 = f120({
                                let a440 = f121(v0, 4u32);
                                let a441 = 15u32;
                                fixed_u32::bit_and(a440, a441)
                            });
                            let a445 = f120({
                                let a442 = v0;
                                let a443 = 15u32;
                                fixed_u32::bit_and(a442, a443)
                            });
                            append_bytes(a444, a445)
                        };
                        append_bytes(a446, a447)
                    };
                    append_bytes(a448, a449)
                };
                append_bytes(a450, a451)
            };
            append_bytes(a452, a453)
        };
        append_bytes(a454, a455)
    };
    append_bytes(a456, a457)
}

pub fn f111(v0: Fn22, p461: u64, p462: (u64, Adt6)) -> R<(u64, Adt6)> {
    let m460 = (p461, p462);
    {
        let mut m460 = m460;
        loop {
            let step = {
                let (v1, v2) = m460;
                let m458 = v1;
                if m458 == 0 {
                    Err::<(u64, (u64, Adt6)), R<(u64, Adt6)>>(Ok(v2))
                } else {
                    let v3 = m458 - 1;
                    Ok::<(u64, (u64, Adt6)), R<(u64, Adt6)>>((v3, {
                        let c459: Fn22 = v0.clone();
                        c459.apply(v2)?
                    }))
                }
            };
            match step {
                Ok(next) => { m460 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f112(v0: Bytes, v1: u64, v2: u64, v3: (u64, Adt6)) -> R<(u64, Adt6)> {
    Ok(({
        let a464 = {
            let (h463, _) = v3.clone();
            h463
        };
        let a465 = 64u64;
        nat_add(a464, a465)?
    }, f122({
        let (_, h466) = v3;
        h466
    }, f123(v0, v1, v2, {
        let (h467, _) = v3;
        h467
    })?)?))
}

pub fn f113() -> Adt6 {
    Adt6::C0(1779033703u32, 3144134277u32, 1013904242u32, 2773480762u32, 1359893119u32, 2600822924u32, 528734635u32, 1541459225u32)
}

pub fn f114(v0: Fn23, p471: Option<Adt1>, p472: List<Adt1>) -> Option<Adt1> {
    let m470 = (p471, p472);
    {
        let mut m470 = m470;
        loop {
            let step = {
                let (v1, v2) = m470;
                let m468 = v2;
                match m468.uncons() {
                    None => {
                        Err::<(Option<Adt1>, List<Adt1>), Option<Adt1>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Adt1>, List<Adt1>), Option<Adt1>>(({
                            let c469: Fn23 = v0.clone();
                            c469.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m470 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f115(v0: Str, v1: Option<Adt1>, v2: Adt1) -> Option<Adt1> {
    let a476 = {
        let m475 = match v2.clone() {
            Adt1::C0(h474, _, _) => {
                h474
            }
        };
        match m475 {
            Adt2::C0(h473, _, _, _, _, _) => {
                h473
            }
        }
    };
    let a477 = v0;
    if equal(a476, a477) {
        Some(v2)
    } else {
        v1
    }
}

pub fn f116(v0: Str, v1: List<Bytes>, v2: Str) -> List<Bytes> {
    List::cons(f34(List::cons(f41(v0), List::cons(f41(v2), List::<Bytes>::nil()))), v1)
}

pub fn f117(v0: List<Bytes>, v1: Adt1) -> List<Bytes> {
    List::cons(f34(List::cons(f41(Str::lit("needs")), List::cons(f41({
        let m480 = match v1.clone() {
            Adt1::C0(h479, _, _) => {
                h479
            }
        };
        match m480 {
            Adt2::C0(h478, _, _, _, _, _) => {
                h478
            }
        }
    }), List::cons(f41(match v1 {
        Adt1::C0(_, h481, _) => {
            h481
        }
    }), List::<Bytes>::nil())))), v0)
}

pub fn f118(v0: Fn24, p485: u64, p486: List<Str>) -> R<u64> {
    let m484 = (p485, p486);
    {
        let mut m484 = m484;
        loop {
            let step = {
                let (v1, v2) = m484;
                let m482 = v2;
                match m482.uncons() {
                    None => {
                        Err::<(u64, List<Str>), R<u64>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(u64, List<Str>), R<u64>>(({
                            let c483: Fn24 = v0.clone();
                            c483.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m484 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f119(v0: bool, v1: Bytes, v2: u64, v3: Str) -> R<u64> {
    let a487 = f124(v3.clone(), Bytes::lit(&[0x21]));
    let a488 = v0;
    if equal(a487, a488) {
        let v4: u64 = f125(v3, v1)?;
        let a489 = v2;
        let a490 = v4;
        if nat_lt(a489, a490) {
            Ok(v4)
        } else {
            Ok(v2)
        }
    } else {
        Ok(v2)
    }
}

pub fn f120(v0: u32) -> Bytes {
    let a493 = {
        let a491 = v0;
        let a492 = 8u32;
        fixed_u32::bit_and(a491, a492)
    };
    let a494 = 0u32;
    if equal(a493, a494) {
        let a497 = {
            let a495 = v0;
            let a496 = 4u32;
            fixed_u32::bit_and(a495, a496)
        };
        let a498 = 0u32;
        if equal(a497, a498) {
            let a501 = {
                let a499 = v0;
                let a500 = 2u32;
                fixed_u32::bit_and(a499, a500)
            };
            let a502 = 0u32;
            if equal(a501, a502) {
                let a505 = {
                    let a503 = v0;
                    let a504 = 1u32;
                    fixed_u32::bit_and(a503, a504)
                };
                let a506 = 0u32;
                if equal(a505, a506) {
                    Bytes::lit(&[0x30])
                } else {
                    Bytes::lit(&[0x31])
                }
            } else {
                let a509 = {
                    let a507 = v0;
                    let a508 = 1u32;
                    fixed_u32::bit_and(a507, a508)
                };
                let a510 = 0u32;
                if equal(a509, a510) {
                    Bytes::lit(&[0x32])
                } else {
                    Bytes::lit(&[0x33])
                }
            }
        } else {
            let a513 = {
                let a511 = v0;
                let a512 = 2u32;
                fixed_u32::bit_and(a511, a512)
            };
            let a514 = 0u32;
            if equal(a513, a514) {
                let a517 = {
                    let a515 = v0;
                    let a516 = 1u32;
                    fixed_u32::bit_and(a515, a516)
                };
                let a518 = 0u32;
                if equal(a517, a518) {
                    Bytes::lit(&[0x34])
                } else {
                    Bytes::lit(&[0x35])
                }
            } else {
                let a521 = {
                    let a519 = v0;
                    let a520 = 1u32;
                    fixed_u32::bit_and(a519, a520)
                };
                let a522 = 0u32;
                if equal(a521, a522) {
                    Bytes::lit(&[0x36])
                } else {
                    Bytes::lit(&[0x37])
                }
            }
        }
    } else {
        let a525 = {
            let a523 = v0;
            let a524 = 4u32;
            fixed_u32::bit_and(a523, a524)
        };
        let a526 = 0u32;
        if equal(a525, a526) {
            let a529 = {
                let a527 = v0;
                let a528 = 2u32;
                fixed_u32::bit_and(a527, a528)
            };
            let a530 = 0u32;
            if equal(a529, a530) {
                let a533 = {
                    let a531 = v0;
                    let a532 = 1u32;
                    fixed_u32::bit_and(a531, a532)
                };
                let a534 = 0u32;
                if equal(a533, a534) {
                    Bytes::lit(&[0x38])
                } else {
                    Bytes::lit(&[0x39])
                }
            } else {
                let a537 = {
                    let a535 = v0;
                    let a536 = 1u32;
                    fixed_u32::bit_and(a535, a536)
                };
                let a538 = 0u32;
                if equal(a537, a538) {
                    Bytes::lit(&[0x61])
                } else {
                    Bytes::lit(&[0x62])
                }
            }
        } else {
            let a541 = {
                let a539 = v0;
                let a540 = 2u32;
                fixed_u32::bit_and(a539, a540)
            };
            let a542 = 0u32;
            if equal(a541, a542) {
                let a545 = {
                    let a543 = v0;
                    let a544 = 1u32;
                    fixed_u32::bit_and(a543, a544)
                };
                let a546 = 0u32;
                if equal(a545, a546) {
                    Bytes::lit(&[0x63])
                } else {
                    Bytes::lit(&[0x64])
                }
            } else {
                let a549 = {
                    let a547 = v0;
                    let a548 = 1u32;
                    fixed_u32::bit_and(a547, a548)
                };
                let a550 = 0u32;
                if equal(a549, a550) {
                    Bytes::lit(&[0x65])
                } else {
                    Bytes::lit(&[0x66])
                }
            }
        }
    }
}

pub fn f121(v0: u32, v1: u32) -> u32 {
    let m553 = {
        let a551 = v0;
        let a552 = v1;
        fixed_u32::shift_right(a551, a552)
    };
    match m553 {
        None => {
            0u32
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f122(v0: Adt6, v1: Adt7) -> R<Adt6> {
    let v2: Adt8 = f126(Fn25::F127, 64u64, Adt8::C0(match v0.clone() {
        Adt6::C0(h554, _, _, _, _, _, _, _) => {
            h554
        }
    }, match v0.clone() {
        Adt6::C0(_, h555, _, _, _, _, _, _) => {
            h555
        }
    }, match v0.clone() {
        Adt6::C0(_, _, h556, _, _, _, _, _) => {
            h556
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, h557, _, _, _, _) => {
            h557
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, h558, _, _, _) => {
            h558
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, _, h559, _, _) => {
            h559
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, _, _, h560, _) => {
            h560
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, _, _, _, h561) => {
            h561
        }
    }, v1, 0u64))?;
    Ok(Adt6::C0(f128(match v0.clone() {
        Adt6::C0(h562, _, _, _, _, _, _, _) => {
            h562
        }
    }, match v2.clone() {
        Adt8::C0(h563, _, _, _, _, _, _, _, _, _) => {
            h563
        }
    }), f128(match v0.clone() {
        Adt6::C0(_, h564, _, _, _, _, _, _) => {
            h564
        }
    }, match v2.clone() {
        Adt8::C0(_, h565, _, _, _, _, _, _, _, _) => {
            h565
        }
    }), f128(match v0.clone() {
        Adt6::C0(_, _, h566, _, _, _, _, _) => {
            h566
        }
    }, match v2.clone() {
        Adt8::C0(_, _, h567, _, _, _, _, _, _, _) => {
            h567
        }
    }), f128(match v0.clone() {
        Adt6::C0(_, _, _, h568, _, _, _, _) => {
            h568
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, h569, _, _, _, _, _, _) => {
            h569
        }
    }), f128(match v0.clone() {
        Adt6::C0(_, _, _, _, h570, _, _, _) => {
            h570
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, _, h571, _, _, _, _, _) => {
            h571
        }
    }), f128(match v0.clone() {
        Adt6::C0(_, _, _, _, _, h572, _, _) => {
            h572
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, _, _, h573, _, _, _, _) => {
            h573
        }
    }), f128(match v0.clone() {
        Adt6::C0(_, _, _, _, _, _, h574, _) => {
            h574
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, _, _, _, h575, _, _, _) => {
            h575
        }
    }), f128(match v0 {
        Adt6::C0(_, _, _, _, _, _, _, h576) => {
            h576
        }
    }, match v2 {
        Adt8::C0(_, _, _, _, _, _, _, h577, _, _) => {
            h577
        }
    })))
}

pub fn f123(v0: Bytes, v1: u64, v2: u64, v3: u64) -> R<Adt7> {
    Ok(Adt7::C0(f129(v0.clone(), v1, v2, v3)?, f129(v0.clone(), v1, v2, {
        let a578 = v3;
        let a579 = 4u64;
        nat_add(a578, a579)?
    })?, f129(v0.clone(), v1, v2, {
        let a580 = v3;
        let a581 = 8u64;
        nat_add(a580, a581)?
    })?, f129(v0.clone(), v1, v2, {
        let a582 = v3;
        let a583 = 12u64;
        nat_add(a582, a583)?
    })?, f129(v0.clone(), v1, v2, {
        let a584 = v3;
        let a585 = 16u64;
        nat_add(a584, a585)?
    })?, f129(v0.clone(), v1, v2, {
        let a586 = v3;
        let a587 = 20u64;
        nat_add(a586, a587)?
    })?, f129(v0.clone(), v1, v2, {
        let a588 = v3;
        let a589 = 24u64;
        nat_add(a588, a589)?
    })?, f129(v0.clone(), v1, v2, {
        let a590 = v3;
        let a591 = 28u64;
        nat_add(a590, a591)?
    })?, f129(v0.clone(), v1, v2, {
        let a592 = v3;
        let a593 = 32u64;
        nat_add(a592, a593)?
    })?, f129(v0.clone(), v1, v2, {
        let a594 = v3;
        let a595 = 36u64;
        nat_add(a594, a595)?
    })?, f129(v0.clone(), v1, v2, {
        let a596 = v3;
        let a597 = 40u64;
        nat_add(a596, a597)?
    })?, f129(v0.clone(), v1, v2, {
        let a598 = v3;
        let a599 = 44u64;
        nat_add(a598, a599)?
    })?, f129(v0.clone(), v1, v2, {
        let a600 = v3;
        let a601 = 48u64;
        nat_add(a600, a601)?
    })?, f129(v0.clone(), v1, v2, {
        let a602 = v3;
        let a603 = 52u64;
        nat_add(a602, a603)?
    })?, f129(v0.clone(), v1, v2, {
        let a604 = v3;
        let a605 = 56u64;
        nat_add(a604, a605)?
    })?, f129(v0, v1, v2, {
        let a606 = v3;
        let a607 = 60u64;
        nat_add(a606, a607)?
    })?))
}

pub fn f124(v0: Str, v1: Bytes) -> bool {
    f130(f41(v0), v1)
}

pub fn f125(v0: Str, v1: Bytes) -> R<u64> {
    let v2: Bytes = if f124(v0.clone(), Bytes::lit(&[0x21])) {
        f131(v0)
    } else {
        f41(v0)
    };
    if f130(v1.clone(), v2.clone()) {
        if f130(v1, Bytes::lit(&[0x40])) {
            if f130(v2.clone(), Bytes::lit(&[0x40])) {
                let a609 = {
                    let a608 = v2;
                    length_bytes(a608)
                };
                let a610 = 1u64;
                nat_add(a609, a610)
            } else {
                Ok(0u64)
            }
        } else {
            let a612 = {
                let a611 = v2;
                length_bytes(a611)
            };
            let a613 = 1u64;
            nat_add(a612, a613)
        }
    } else {
        Ok(0u64)
    }
}

pub fn f126(v0: Fn25, p617: u64, p618: Adt8) -> R<Adt8> {
    let m616 = (p617, p618);
    {
        let mut m616 = m616;
        loop {
            let step = {
                let (v1, v2) = m616;
                let m614 = v1;
                if m614 == 0 {
                    Err::<(u64, Adt8), R<Adt8>>(Ok(v2))
                } else {
                    let v3 = m614 - 1;
                    Ok::<(u64, Adt8), R<Adt8>>((v3, {
                        let c615: Fn25 = v0.clone();
                        c615.apply(v2)?
                    }))
                }
            };
            match step {
                Ok(next) => { m616 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f127(v0: Adt8) -> R<Adt8> {
    f132(v0)
}

pub fn f128(v0: u32, v1: u32) -> u32 {
    let m621 = {
        let a619 = f133(v0);
        let a620 = f133(v1);
        fixed_u64::checked_add(a619, a620)
    };
    match m621 {
        None => {
            0u32
        }
        Some(v2) => {
            f134(v2)
        }
    }
}

pub fn f129(v0: Bytes, v1: u64, v2: u64, v3: u64) -> R<u32> {
    let a624 = {
        let a622 = v3;
        let a623 = 8u64;
        nat_add(a622, a623)?
    };
    let a625 = v2;
    if nat_eq(a624, a625) {
        Ok(f135({
            let a628 = {
                let a626 = v1;
                let a627 = 8u64;
                nat_mul(a626, a627)?
            };
            let a629 = 4294967296u64;
            let a630 = 0u64;
            nat_quot(a628, a629, a630)
        }))
    } else {
        let a633 = {
            let a631 = v3;
            let a632 = 4u64;
            nat_add(a631, a632)?
        };
        let a634 = v2;
        if nat_eq(a633, a634) {
            Ok(f135({
                let a637 = {
                    let a635 = v1;
                    let a636 = 8u64;
                    nat_mul(a635, a636)?
                };
                let a638 = 4294967296u64;
                let a639 = 0u64;
                nat_rem(a637, a638, a639)
            }))
        } else {
            Ok(f136(f137(v0.clone(), v1, v3), f137(v0.clone(), v1, {
                let a640 = v3;
                let a641 = 1u64;
                nat_add(a640, a641)?
            }), f137(v0.clone(), v1, {
                let a642 = v3;
                let a643 = 2u64;
                nat_add(a642, a643)?
            }), f137(v0, v1, {
                let a644 = v3;
                let a645 = 3u64;
                nat_add(a644, a645)?
            })))
        }
    }
}

pub fn f130(v0: Bytes, v1: Bytes) -> bool {
    let m650 = {
        let a647 = v0;
        let a648 = 0u64;
        let a649 = {
            let a646 = v1.clone();
            length_bytes(a646)
        };
        slice_bytes(a647, a648, a649)
    };
    match m650 {
        None => {
            false
        }
        Some(v2) => {
            let a651 = v2;
            let a652 = v1;
            equal(a651, a652)
        }
    }
}

pub fn f131(v0: Str) -> Bytes {
    let v1: Bytes = f41(v0);
    f25(v1.clone(), 1u64, {
        let a654 = {
            let a653 = v1;
            length_bytes(a653)
        };
        let a655 = 1u64;
        nat_sub(a654, a655)
    })
}

pub fn f132(v0: Adt8) -> R<Adt8> {
    let v1: u32 = f138(match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, _, _, _, h656) => {
            h656
        }
    });
    let v2: u32 = f128(match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, _, h657, _, _) => {
            h657
        }
    }, f128(f139(match v0.clone() {
        Adt8::C0(_, _, _, _, h658, _, _, _, _, _) => {
            h658
        }
    }), f128(f140(match v0.clone() {
        Adt8::C0(_, _, _, _, h659, _, _, _, _, _) => {
            h659
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, h660, _, _, _, _) => {
            h660
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, h661, _, _, _) => {
            h661
        }
    }), f128(v1, {
        let m664 = match v0.clone() {
            Adt8::C0(_, _, _, _, _, _, _, _, h663, _) => {
                h663
            }
        };
        match m664 {
            Adt7::C0(h662, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                h662
            }
        }
    }))));
    let v3: u32 = f128(f141(match v0.clone() {
        Adt8::C0(h665, _, _, _, _, _, _, _, _, _) => {
            h665
        }
    }), f142(match v0.clone() {
        Adt8::C0(h666, _, _, _, _, _, _, _, _, _) => {
            h666
        }
    }, match v0.clone() {
        Adt8::C0(_, h667, _, _, _, _, _, _, _, _) => {
            h667
        }
    }, match v0.clone() {
        Adt8::C0(_, _, h668, _, _, _, _, _, _, _) => {
            h668
        }
    }));
    Ok(Adt8::C0(f128(v2, v3), match v0.clone() {
        Adt8::C0(h669, _, _, _, _, _, _, _, _, _) => {
            h669
        }
    }, match v0.clone() {
        Adt8::C0(_, h670, _, _, _, _, _, _, _, _) => {
            h670
        }
    }, match v0.clone() {
        Adt8::C0(_, _, h671, _, _, _, _, _, _, _) => {
            h671
        }
    }, f128(match v0.clone() {
        Adt8::C0(_, _, _, h672, _, _, _, _, _, _) => {
            h672
        }
    }, v2), match v0.clone() {
        Adt8::C0(_, _, _, _, h673, _, _, _, _, _) => {
            h673
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, h674, _, _, _, _) => {
            h674
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, h675, _, _, _) => {
            h675
        }
    }, f143(match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, _, _, h676, _) => {
            h676
        }
    }), {
        let a678 = match v0 {
            Adt8::C0(_, _, _, _, _, _, _, _, _, h677) => {
                h677
            }
        };
        let a679 = 1u64;
        nat_add(a678, a679)?
    }))
}

pub fn f133(v0: u32) -> u64 {
    let m681 = {
        let a680 = v0;
        fixed_u64::convert(i128::from(a680))
    };
    match m681 {
        None => {
            0u64
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f134(v0: u64) -> u32 {
    let m685 = {
        let a684 = {
            let a682 = v0;
            let a683 = 4294967295u64;
            fixed_u64::bit_and(a682, a683)
        };
        fixed_u32::convert(i128::from(a684))
    };
    match m685 {
        None => {
            0u32
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f135(v0: u64) -> u32 {
    let (_, h686) = {
        let (_, h687) = f144(Fn26::F145, 32u64, (v0, (1u32, 0u32)));
        h687
    };
    h686
}

pub fn f136(v0: u32, v1: u32, v2: u32, v3: u32) -> u32 {
    let a692 = f146(v0, 24u32);
    let a693 = {
        let a690 = f146(v1, 16u32);
        let a691 = {
            let a688 = f146(v2, 8u32);
            let a689 = v3;
            fixed_u32::bit_or(a688, a689)
        };
        fixed_u32::bit_or(a690, a691)
    };
    fixed_u32::bit_or(a692, a693)
}

pub fn f137(v0: Bytes, v1: u64, v2: u64) -> u32 {
    let a694 = v2;
    let a695 = v1;
    if nat_lt(a694, a695) {
        let m698 = {
            let a696 = v0;
            let a697 = v2;
            index_bytes(a696, a697)
        };
        match m698 {
            None => {
                0u32
            }
            Some(v3) => {
                let m700 = {
                    let a699 = v3;
                    fixed_u32::convert(i128::from(a699))
                };
                match m700 {
                    None => {
                        0u32
                    }
                    Some(v4) => {
                        v4
                    }
                }
            }
        }
    } else {
        let a701 = v2;
        let a702 = v1;
        if nat_eq(a701, a702) {
            128u32
        } else {
            0u32
        }
    }
}

pub fn f138(v0: u64) -> u32 {
    let a703 = v0;
    let a704 = 32u64;
    if nat_lt(a703, a704) {
        let a705 = v0;
        let a706 = 16u64;
        if nat_lt(a705, a706) {
            let a707 = v0;
            let a708 = 8u64;
            if nat_lt(a707, a708) {
                let a709 = v0;
                let a710 = 4u64;
                if nat_lt(a709, a710) {
                    let a711 = v0;
                    let a712 = 2u64;
                    if nat_lt(a711, a712) {
                        let a713 = v0;
                        let a714 = 1u64;
                        if nat_lt(a713, a714) {
                            1116352408u32
                        } else {
                            1899447441u32
                        }
                    } else {
                        let a715 = v0;
                        let a716 = 3u64;
                        if nat_lt(a715, a716) {
                            3049323471u32
                        } else {
                            3921009573u32
                        }
                    }
                } else {
                    let a717 = v0;
                    let a718 = 6u64;
                    if nat_lt(a717, a718) {
                        let a719 = v0;
                        let a720 = 5u64;
                        if nat_lt(a719, a720) {
                            961987163u32
                        } else {
                            1508970993u32
                        }
                    } else {
                        let a721 = v0;
                        let a722 = 7u64;
                        if nat_lt(a721, a722) {
                            2453635748u32
                        } else {
                            2870763221u32
                        }
                    }
                }
            } else {
                let a723 = v0;
                let a724 = 12u64;
                if nat_lt(a723, a724) {
                    let a725 = v0;
                    let a726 = 10u64;
                    if nat_lt(a725, a726) {
                        let a727 = v0;
                        let a728 = 9u64;
                        if nat_lt(a727, a728) {
                            3624381080u32
                        } else {
                            310598401u32
                        }
                    } else {
                        let a729 = v0;
                        let a730 = 11u64;
                        if nat_lt(a729, a730) {
                            607225278u32
                        } else {
                            1426881987u32
                        }
                    }
                } else {
                    let a731 = v0;
                    let a732 = 14u64;
                    if nat_lt(a731, a732) {
                        let a733 = v0;
                        let a734 = 13u64;
                        if nat_lt(a733, a734) {
                            1925078388u32
                        } else {
                            2162078206u32
                        }
                    } else {
                        let a735 = v0;
                        let a736 = 15u64;
                        if nat_lt(a735, a736) {
                            2614888103u32
                        } else {
                            3248222580u32
                        }
                    }
                }
            }
        } else {
            let a737 = v0;
            let a738 = 24u64;
            if nat_lt(a737, a738) {
                let a739 = v0;
                let a740 = 20u64;
                if nat_lt(a739, a740) {
                    let a741 = v0;
                    let a742 = 18u64;
                    if nat_lt(a741, a742) {
                        let a743 = v0;
                        let a744 = 17u64;
                        if nat_lt(a743, a744) {
                            3835390401u32
                        } else {
                            4022224774u32
                        }
                    } else {
                        let a745 = v0;
                        let a746 = 19u64;
                        if nat_lt(a745, a746) {
                            264347078u32
                        } else {
                            604807628u32
                        }
                    }
                } else {
                    let a747 = v0;
                    let a748 = 22u64;
                    if nat_lt(a747, a748) {
                        let a749 = v0;
                        let a750 = 21u64;
                        if nat_lt(a749, a750) {
                            770255983u32
                        } else {
                            1249150122u32
                        }
                    } else {
                        let a751 = v0;
                        let a752 = 23u64;
                        if nat_lt(a751, a752) {
                            1555081692u32
                        } else {
                            1996064986u32
                        }
                    }
                }
            } else {
                let a753 = v0;
                let a754 = 28u64;
                if nat_lt(a753, a754) {
                    let a755 = v0;
                    let a756 = 26u64;
                    if nat_lt(a755, a756) {
                        let a757 = v0;
                        let a758 = 25u64;
                        if nat_lt(a757, a758) {
                            2554220882u32
                        } else {
                            2821834349u32
                        }
                    } else {
                        let a759 = v0;
                        let a760 = 27u64;
                        if nat_lt(a759, a760) {
                            2952996808u32
                        } else {
                            3210313671u32
                        }
                    }
                } else {
                    let a761 = v0;
                    let a762 = 30u64;
                    if nat_lt(a761, a762) {
                        let a763 = v0;
                        let a764 = 29u64;
                        if nat_lt(a763, a764) {
                            3336571891u32
                        } else {
                            3584528711u32
                        }
                    } else {
                        let a765 = v0;
                        let a766 = 31u64;
                        if nat_lt(a765, a766) {
                            113926993u32
                        } else {
                            338241895u32
                        }
                    }
                }
            }
        }
    } else {
        let a767 = v0;
        let a768 = 48u64;
        if nat_lt(a767, a768) {
            let a769 = v0;
            let a770 = 40u64;
            if nat_lt(a769, a770) {
                let a771 = v0;
                let a772 = 36u64;
                if nat_lt(a771, a772) {
                    let a773 = v0;
                    let a774 = 34u64;
                    if nat_lt(a773, a774) {
                        let a775 = v0;
                        let a776 = 33u64;
                        if nat_lt(a775, a776) {
                            666307205u32
                        } else {
                            773529912u32
                        }
                    } else {
                        let a777 = v0;
                        let a778 = 35u64;
                        if nat_lt(a777, a778) {
                            1294757372u32
                        } else {
                            1396182291u32
                        }
                    }
                } else {
                    let a779 = v0;
                    let a780 = 38u64;
                    if nat_lt(a779, a780) {
                        let a781 = v0;
                        let a782 = 37u64;
                        if nat_lt(a781, a782) {
                            1695183700u32
                        } else {
                            1986661051u32
                        }
                    } else {
                        let a783 = v0;
                        let a784 = 39u64;
                        if nat_lt(a783, a784) {
                            2177026350u32
                        } else {
                            2456956037u32
                        }
                    }
                }
            } else {
                let a785 = v0;
                let a786 = 44u64;
                if nat_lt(a785, a786) {
                    let a787 = v0;
                    let a788 = 42u64;
                    if nat_lt(a787, a788) {
                        let a789 = v0;
                        let a790 = 41u64;
                        if nat_lt(a789, a790) {
                            2730485921u32
                        } else {
                            2820302411u32
                        }
                    } else {
                        let a791 = v0;
                        let a792 = 43u64;
                        if nat_lt(a791, a792) {
                            3259730800u32
                        } else {
                            3345764771u32
                        }
                    }
                } else {
                    let a793 = v0;
                    let a794 = 46u64;
                    if nat_lt(a793, a794) {
                        let a795 = v0;
                        let a796 = 45u64;
                        if nat_lt(a795, a796) {
                            3516065817u32
                        } else {
                            3600352804u32
                        }
                    } else {
                        let a797 = v0;
                        let a798 = 47u64;
                        if nat_lt(a797, a798) {
                            4094571909u32
                        } else {
                            275423344u32
                        }
                    }
                }
            }
        } else {
            let a799 = v0;
            let a800 = 56u64;
            if nat_lt(a799, a800) {
                let a801 = v0;
                let a802 = 52u64;
                if nat_lt(a801, a802) {
                    let a803 = v0;
                    let a804 = 50u64;
                    if nat_lt(a803, a804) {
                        let a805 = v0;
                        let a806 = 49u64;
                        if nat_lt(a805, a806) {
                            430227734u32
                        } else {
                            506948616u32
                        }
                    } else {
                        let a807 = v0;
                        let a808 = 51u64;
                        if nat_lt(a807, a808) {
                            659060556u32
                        } else {
                            883997877u32
                        }
                    }
                } else {
                    let a809 = v0;
                    let a810 = 54u64;
                    if nat_lt(a809, a810) {
                        let a811 = v0;
                        let a812 = 53u64;
                        if nat_lt(a811, a812) {
                            958139571u32
                        } else {
                            1322822218u32
                        }
                    } else {
                        let a813 = v0;
                        let a814 = 55u64;
                        if nat_lt(a813, a814) {
                            1537002063u32
                        } else {
                            1747873779u32
                        }
                    }
                }
            } else {
                let a815 = v0;
                let a816 = 60u64;
                if nat_lt(a815, a816) {
                    let a817 = v0;
                    let a818 = 58u64;
                    if nat_lt(a817, a818) {
                        let a819 = v0;
                        let a820 = 57u64;
                        if nat_lt(a819, a820) {
                            1955562222u32
                        } else {
                            2024104815u32
                        }
                    } else {
                        let a821 = v0;
                        let a822 = 59u64;
                        if nat_lt(a821, a822) {
                            2227730452u32
                        } else {
                            2361852424u32
                        }
                    }
                } else {
                    let a823 = v0;
                    let a824 = 62u64;
                    if nat_lt(a823, a824) {
                        let a825 = v0;
                        let a826 = 61u64;
                        if nat_lt(a825, a826) {
                            2428436474u32
                        } else {
                            2756734187u32
                        }
                    } else {
                        let a827 = v0;
                        let a828 = 63u64;
                        if nat_lt(a827, a828) {
                            3204031479u32
                        } else {
                            3329325298u32
                        }
                    }
                }
            }
        }
    }
}

pub fn f139(v0: u32) -> u32 {
    let a831 = {
        let a829 = f147(v0, 6u32, 26u32);
        let a830 = f147(v0, 11u32, 21u32);
        fixed_u32::bit_xor(a829, a830)
    };
    let a832 = f147(v0, 25u32, 7u32);
    fixed_u32::bit_xor(a831, a832)
}

pub fn f140(v0: u32, v1: u32, v2: u32) -> u32 {
    let a838 = {
        let a833 = v0;
        let a834 = v1;
        fixed_u32::bit_and(a833, a834)
    };
    let a839 = {
        let a836 = {
            let a835 = v0;
            fixed_u32::bit_not(a835)
        };
        let a837 = v2;
        fixed_u32::bit_and(a836, a837)
    };
    fixed_u32::bit_xor(a838, a839)
}

pub fn f141(v0: u32) -> u32 {
    let a842 = {
        let a840 = f147(v0, 2u32, 30u32);
        let a841 = f147(v0, 13u32, 19u32);
        fixed_u32::bit_xor(a840, a841)
    };
    let a843 = f147(v0, 22u32, 10u32);
    fixed_u32::bit_xor(a842, a843)
}

pub fn f142(v0: u32, v1: u32, v2: u32) -> u32 {
    let a852 = {
        let a848 = {
            let a844 = v0;
            let a845 = v1;
            fixed_u32::bit_and(a844, a845)
        };
        let a849 = {
            let a846 = v0;
            let a847 = v2;
            fixed_u32::bit_and(a846, a847)
        };
        fixed_u32::bit_xor(a848, a849)
    };
    let a853 = {
        let a850 = v1;
        let a851 = v2;
        fixed_u32::bit_and(a850, a851)
    };
    fixed_u32::bit_xor(a852, a853)
}

pub fn f143(v0: Adt7) -> Adt7 {
    Adt7::C0(match v0.clone() {
        Adt7::C0(_, h854, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h854
        }
    }, match v0.clone() {
        Adt7::C0(_, _, h855, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h855
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, h856, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h856
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, h857, _, _, _, _, _, _, _, _, _, _, _) => {
            h857
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, h858, _, _, _, _, _, _, _, _, _, _) => {
            h858
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, h859, _, _, _, _, _, _, _, _, _) => {
            h859
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, h860, _, _, _, _, _, _, _, _) => {
            h860
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, h861, _, _, _, _, _, _, _) => {
            h861
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, h862, _, _, _, _, _, _) => {
            h862
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, h863, _, _, _, _, _) => {
            h863
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, h864, _, _, _, _) => {
            h864
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, h865, _, _, _) => {
            h865
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, h866, _, _) => {
            h866
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h867, _) => {
            h867
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h868) => {
            h868
        }
    }, f128(f148(match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h869, _) => {
            h869
        }
    }), f128(match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, h870, _, _, _, _, _, _) => {
            h870
        }
    }, f128(f149(match v0.clone() {
        Adt7::C0(_, h871, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h871
        }
    }), match v0 {
        Adt7::C0(h872, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h872
        }
    }))))
}

pub fn f144(v0: Fn26, p876: u64, p877: (u64, (u32, u32))) -> (u64, (u32, u32)) {
    let m875 = (p876, p877);
    {
        let mut m875 = m875;
        loop {
            let step = {
                let (v1, v2) = m875;
                let m873 = v1;
                if m873 == 0 {
                    Err::<(u64, (u64, (u32, u32))), (u64, (u32, u32))>(v2)
                } else {
                    let v3 = m873 - 1;
                    Ok::<(u64, (u64, (u32, u32))), (u64, (u32, u32))>((v3, {
                        let c874: Fn26 = v0.clone();
                        c874.apply(v2)
                    }))
                }
            };
            match step {
                Ok(next) => { m875 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f145(v0: (u64, (u32, u32))) -> (u64, (u32, u32)) {
    let v1: u64 = {
        let (h878, _) = v0;
        h878
    };
    let v2: u32 = {
        let (h879, _) = {
            let (_, h880) = v0;
            h880
        };
        h879
    };
    let v3: u32 = {
        let (_, h881) = {
            let (_, h882) = v0;
            h882
        };
        h881
    };
    ({
        let a883 = v1;
        let a884 = 2u64;
        let a885 = 0u64;
        nat_quot(a883, a884, a885)
    }, (f146(v2, 1u32), {
        let a889 = {
            let a886 = v1;
            let a887 = 2u64;
            let a888 = 0u64;
            nat_rem(a886, a887, a888)
        };
        let a890 = 1u64;
        if nat_eq(a889, a890) {
            let a891 = v3;
            let a892 = v2;
            fixed_u32::bit_or(a891, a892)
        } else {
            v3
        }
    }))
}

pub fn f146(v0: u32, v1: u32) -> u32 {
    let m895 = {
        let a893 = v0;
        let a894 = v1;
        fixed_u32::shift_left(a893, a894)
    };
    match m895 {
        None => {
            0u32
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f147(v0: u32, v1: u32, v2: u32) -> u32 {
    let a896 = f121(v0, v1);
    let a897 = f146(v0, v2);
    fixed_u32::bit_or(a896, a897)
}

pub fn f148(v0: u32) -> u32 {
    let a900 = {
        let a898 = f147(v0, 17u32, 15u32);
        let a899 = f147(v0, 19u32, 13u32);
        fixed_u32::bit_xor(a898, a899)
    };
    let a901 = f121(v0, 10u32);
    fixed_u32::bit_xor(a900, a901)
}

pub fn f149(v0: u32) -> u32 {
    let a904 = {
        let a902 = f147(v0, 7u32, 25u32);
        let a903 = f147(v0, 18u32, 14u32);
        fixed_u32::bit_xor(a902, a903)
    };
    let a905 = f121(v0, 3u32);
    fixed_u32::bit_xor(a904, a905)
}

pub fn run(p0: Bytes) -> R<Bytes> {
    f0(p0)
}
