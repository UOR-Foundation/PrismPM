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

pub fn parse_int(text: Str) -> R<Option<i64>> { match canonical_decimal(&text.0) { None => Ok(None), Some(Some(n)) => match i64::try_from(n) { Ok(value) => Ok(Some(value)), Err(_) => Err(Overflow), }, Some(None) => Err(Overflow), } }

pub fn format_u8(a: u8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u8(text: Str) -> Option<u8> { match canonical_decimal(&text.0) { Some(Some(n)) => u8::try_from(n).ok(), _ => None, } }

pub fn format_u16(a: u16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u16(text: Str) -> Option<u16> { match canonical_decimal(&text.0) { Some(Some(n)) => u16::try_from(n).ok(), _ => None, } }

pub fn format_u32(a: u32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u32(text: Str) -> Option<u32> { match canonical_decimal(&text.0) { Some(Some(n)) => u32::try_from(n).ok(), _ => None, } }

pub fn format_u64(a: u64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_u64(text: Str) -> Option<u64> { match canonical_decimal(&text.0) { Some(Some(n)) => u64::try_from(n).ok(), _ => None, } }

pub fn format_i8(a: i8) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i8(text: Str) -> Option<i8> { match canonical_decimal(&text.0) { Some(Some(n)) => i8::try_from(n).ok(), _ => None, } }

pub fn format_i16(a: i16) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i16(text: Str) -> Option<i16> { match canonical_decimal(&text.0) { Some(Some(n)) => i16::try_from(n).ok(), _ => None, } }

pub fn format_i32(a: i32) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i32(text: Str) -> Option<i32> { match canonical_decimal(&text.0) { Some(Some(n)) => i32::try_from(n).ok(), _ => None, } }

pub fn format_i64(a: i64) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }

pub fn parse_i64(text: Str) -> Option<i64> { match canonical_decimal(&text.0) { Some(Some(n)) => i64::try_from(n).ok(), _ => None, } }

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

#[derive(Clone)]
pub enum Adt0 {
    C0(Str, Adt1, Adt4),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, Str, Str, Str, Str, Str, Str, List<Adt2>, List<Adt2>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, List<Adt3>, Option<Str>, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt4 {
    C0(Str, Str, Str, Option<Str>, Str, Str, Str, Adt5),
}

#[derive(Clone)]
pub enum Adt5 {
    C0,
    C1(Adt6),
}

#[derive(Clone)]
pub enum Adt6 {
    C0(Str, Str, Str, Str, List<Adt7>, List<Adt8>),
}

#[derive(Clone)]
pub enum Adt7 {
    C0(Str, Str, u64),
}

#[derive(Clone)]
pub enum Adt8 {
    C0(Str, Option<Str>),
}

#[derive(Clone)]
pub enum Adt9 {
    C0(List<Adt10>),
    C1(Str, Str),
}

#[derive(Clone)]
pub enum Adt10 {
    C0(Str, Str),
}

pub fn f0(v0: Adt0) -> R<Adt9> {
    Ok(f1(f2(f3(Str::lit("js"), match v0.clone() {
        Adt0::C0(_, h1, _) => {
            h1
        }
    }), f4(Str::lit("js"), false, f5(), {
        let m4 = match v0.clone() {
            Adt0::C0(_, h3, _) => {
                h3
            }
        };
        match m4 {
            Adt1::C0(_, _, _, _, _, _, _, _, h2) => {
                h2
            }
        }
    })?), f6(v0)?))
}

pub fn f1(v0: Option<(Str, Str)>, v1: List<Adt10>) -> Adt9 {
    let m5 = v0;
    match m5 {
        None => {
            Adt9::C0(v1)
        }
        Some(v2) => {
            Adt9::C1({
                let (h6, _) = v2;
                h6
            }, {
                let (_, h7) = v2;
                h7
            })
        }
    }
}

pub fn f2(v0: Option<(Str, Str)>, v1: Option<(Str, Str)>) -> Option<(Str, Str)> {
    let m8 = v0.clone();
    match m8 {
        None => {
            v1
        }
        Some(_) => {
            v0
        }
    }
}

pub fn f3(v0: Str, v1: Adt1) -> Option<(Str, Str)> {
    let a12 = {
        let a10 = match v1.clone() {
            Adt1::C0(h9, _, _, _, _, _, _, _, _) => {
                h9
            }
        };
        let a11 = Str::lit("hologram:guest/core-wasm@1");
        equal(a10, a11)
    };
    if bool_not(a12) {
        Some((Str::lit("generate.world"), f7(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer has no host protocol for "), List::cons(match v1 {
            Adt1::C0(h13, _, _, _, _, _, _, _, _) => {
                h13
            }
        }, List::<Str>::nil())))))))
    } else {
        let m15 = match v1.clone() {
            Adt1::C0(_, _, _, _, _, _, _, h14, _) => {
                h14
            }
        };
        match m15.uncons() {
            None => {
                None::<(Str, Str)>
            }
            Some((_, _)) => {
                Some((Str::lit("generate.world"), f7(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer binds import-free worlds; "), List::cons(match v1 {
                    Adt1::C0(h16, _, _, _, _, _, _, _, _) => {
                        h16
                    }
                }, List::cons(Str::lit(" has imports"), List::<Str>::nil()))))))))
            }
        }
    }
}

pub fn f4(v0: Str, v1: bool, v2: List<(Str, Str)>, v3: List<Adt2>) -> R<Option<(Str, Str)>> {
    let m17 = v3;
    match m17.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v4, v5)) => {
            Ok(f2(if v1 {
                f8(v0.clone(), match v4.clone() {
                    Adt2::C0(h18, _, _, _) => {
                        h18
                    }
                })?
            } else {
                None::<(Str, Str)>
            }, f2(f9(v0.clone(), v1, v2.clone(), match v4.clone() {
                Adt2::C0(_, h19, _, _) => {
                    h19
                }
            })?, f2({
                let m21 = match v4 {
                    Adt2::C0(_, _, h20, _) => {
                        h20
                    }
                };
                match m21 {
                    None => {
                        None::<(Str, Str)>
                    }
                    Some(v6) => {
                        f10(v0.clone(), v6, v2.clone())
                    }
                }
            }, f4(v0, v1, v2, v5)?))))
        }
    }
}

pub fn f5() -> List<(Str, Str)> {
    List::cons((Str::lit("bool"), Str::lit("boolean")), List::cons((Str::lit("u8"), Str::lit("number")), List::cons((Str::lit("u16"), Str::lit("number")), List::cons((Str::lit("u32"), Str::lit("number")), List::cons((Str::lit("s8"), Str::lit("number")), List::cons((Str::lit("s16"), Str::lit("number")), List::cons((Str::lit("s32"), Str::lit("number")), List::cons((Str::lit("char"), Str::lit("number")), List::cons((Str::lit("u64"), Str::lit("bigint")), List::cons((Str::lit("s64"), Str::lit("bigint")), List::cons((Str::lit("f32"), Str::lit("number")), List::cons((Str::lit("f64"), Str::lit("number")), List::<(Str, Str)>::nil()))))))))))))
}

pub fn f6(v0: Adt0) -> R<List<Adt10>> {
    let v1: Str = match v0.clone() {
        Adt0::C0(h22, _, _) => {
            h22
        }
    };
    let v2: Adt4 = match v0.clone() {
        Adt0::C0(_, _, h23) => {
            h23
        }
    };
    let v3: Str = match v2.clone() {
        Adt4::C0(_, _, h24, _, _, _, _, _) => {
            h24
        }
    };
    Ok(List::cons(f11(Str::lit("world.js"), f12(v1.clone(), match v0.clone() {
        Adt0::C0(_, h25, _) => {
            h25
        }
    })?), List::cons(f11(Str::lit("world.d.ts"), f13(v1.clone(), match v0 {
        Adt0::C0(_, h26, _) => {
            h26
        }
    })?), {
        let m28 = match v2.clone() {
            Adt4::C0(_, _, _, _, _, _, _, h27) => {
                h27
            }
        };
        match m28 {
            Adt5::C0 => {
                List::cons(f11(f7(List::cons(v3.clone(), List::cons(Str::lit(".js"), List::<Str>::nil()))), f14(v1.clone(), v2.clone())?), List::cons(f11(f7(List::cons(v3, List::cons(Str::lit(".d.ts"), List::<Str>::nil()))), f15(v1, v2)?), List::<Adt10>::nil()))
            }
            Adt5::C1(v4) => {
                List::cons(f11(f7(List::cons(v3.clone(), List::cons(Str::lit(".js"), List::<Str>::nil()))), f16(v1.clone(), v2.clone(), v4.clone())?), List::cons(f11(f7(List::cons(v3, List::cons(Str::lit(".d.ts"), List::<Str>::nil()))), f17(v1, v2, v4)?), List::<Adt10>::nil()))
            }
        }
    })))
}

pub fn f7(v0: List<Str>) -> Str {
    let a29 = v0;
    let a30 = Str::lit("");
    join(a29, a30)
}

pub fn f8(v0: Str, v1: Str) -> R<Option<(Str, Str)>> {
    if f18(v1.clone())? {
        Ok(None::<(Str, Str)>)
    } else {
        Ok(Some((Str::lit("generate.model"), f7(List::cons(f19(v1)?, List::cons(Str::lit(" is not a name the "), List::cons(v0, List::cons(Str::lit(" writer can bind"), List::<Str>::nil()))))))))
    }
}

pub fn f9(v0: Str, v1: bool, v2: List<(Str, Str)>, v3: List<Adt3>) -> R<Option<(Str, Str)>> {
    let m31 = v3;
    match m31.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v4, v5)) => {
            Ok(f2(if v1 {
                f8(v0.clone(), f20(match v4.clone() {
                    Adt3::C0(h32, _) => {
                        h32
                    }
                })?)?
            } else {
                None::<(Str, Str)>
            }, f2(f10(v0.clone(), match v4 {
                Adt3::C0(_, h33) => {
                    h33
                }
            }, v2.clone()), f9(v0, v1, v2, v5)?)))
        }
    }
}

pub fn f10(v0: Str, v1: Str, v2: List<(Str, Str)>) -> Option<(Str, Str)> {
    let m34 = f21(v1.clone(), v2);
    match m34 {
        None => {
            Some((Str::lit("generate.world"), f7(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer has no type for WIT "), List::cons(v1, List::<Str>::nil())))))))
        }
        Some(_) => {
            None::<(Str, Str)>
        }
    }
}

pub fn f11(v0: Str, v1: Str) -> Adt10 {
    Adt10::C0(v0, v1)
}

pub fn f12(v0: Str, v1: Adt1) -> R<Str> {
    Ok(f22({
        let a49 = List::cons(f23(Str::lit("//"), v0, f24(v1.clone())), List::cons(Str::lit("/** The world these bindings were generated from. It equals `inputs.world` in prismpm.lock. */\u{a}export const WORLD = Object.freeze({"), List::cons(f7(List::cons(Str::lit("  contract: "), List::cons(f25(match v1.clone() {
            Adt1::C0(h35, _, _, _, _, _, _, _, _) => {
                h35
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f7(List::cons(Str::lit("  blake3: "), List::cons(f25(match v1.clone() {
            Adt1::C0(_, _, _, _, h36, _, _, _, _) => {
                h36
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f7(List::cons(Str::lit("  sha256: "), List::cons(f25(match v1.clone() {
            Adt1::C0(_, _, _, _, _, h37, _, _, _) => {
                h37
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit("});\u{a}\u{a}/** Every export the world requires, and its kind. */\u{a}const EXPORTS = Object.freeze({"), List::<Str>::nil()))))));
        let a50 = {
            let a47 = f26(f27(List::cons((match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, h38, _, _) => {
                    h38
                }
            }, Str::lit("memory")), f28(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, _, _, h39) => {
                    h39
                }
            }))))?;
            let a48 = {
                let a45 = List::cons(Str::lit("});\u{a}\u{a}/**\u{a} * Instantiate a compiled guest and check that it offers exactly the world's exports.\u{a} * The guest imports nothing.\u{a} * @param {WebAssembly.Module} module\u{a} */\u{a}export function instantiate(module) {\u{a}  const { exports } = new WebAssembly.Instance(module, {});\u{a}  const names = Object.keys(exports).sort();\u{a}  const expected = Object.keys(EXPORTS).sort();\u{a}  if (names.join() !== expected.join() || names.some((name) => kindOf(exports[name]) !== EXPORTS[name])) {\u{a}    throw new TypeError(`the guest exports ${names.join(', ')}; ${WORLD.contract} requires ${expected.join(', ')}`);\u{a}  }\u{a}  return {"), List::cons(f7(List::cons(Str::lit("    "), List::cons(match v1.clone() {
                    Adt1::C0(_, _, _, _, _, _, h40, _, _) => {
                        h40
                    }
                }, List::cons(Str::lit(": exports."), List::cons(match v1.clone() {
                    Adt1::C0(_, _, _, _, _, _, h41, _, _) => {
                        h41
                    }
                }, List::cons(Str::lit(","), List::<Str>::nil())))))), List::<Str>::nil()));
                let a46 = {
                    let a43 = f29(match v1 {
                        Adt1::C0(_, _, _, _, _, _, _, _, h42) => {
                            h42
                        }
                    })?;
                    let a44 = List::cons(Str::lit("  };\u{a}}\u{a}\u{a}function kindOf(value) {\u{a}  return value instanceof WebAssembly.Memory ? 'memory' : typeof value === 'function' ? 'function' : 'other';\u{a}}\u{a}"), List::cons(f30(), List::<Str>::nil()));
                    append_list(a43, a44)
                };
                append_list(a45, a46)
            };
            append_list(a47, a48)
        };
        append_list(a49, a50)
    }))
}

pub fn f13(v0: Str, v1: Adt1) -> R<Str> {
    Ok(f22({
        let a57 = List::cons(f23(Str::lit("//"), v0, f24(v1.clone())), List::cons(Str::lit("/** The world these bindings were generated from. It equals `inputs.world` in prismpm.lock. */\u{a}export declare const WORLD: {"), List::cons(f7(List::cons(Str::lit("  readonly contract: "), List::cons(f25(match v1.clone() {
            Adt1::C0(h51, _, _, _, _, _, _, _, _) => {
                h51
            }
        })?, List::cons(Str::lit(";"), List::<Str>::nil())))), List::cons(Str::lit("  readonly blake3: string;\u{a}  readonly sha256: string;\u{a}};\u{a}"), List::cons(f7(List::cons(Str::lit("/** A guest of "), List::cons(match v1.clone() {
            Adt1::C0(h52, _, _, _, _, _, _, _, _) => {
                h52
            }
        }, List::cons(Str::lit(", instantiated. */\u{a}export interface Guest {"), List::<Str>::nil())))), List::cons(f7(List::cons(Str::lit("  readonly "), List::cons(match v1.clone() {
            Adt1::C0(_, _, _, _, _, _, h53, _, _) => {
                h53
            }
        }, List::cons(Str::lit(": WebAssembly.Memory;"), List::<Str>::nil())))), List::<Str>::nil()))))));
        let a58 = {
            let a55 = f31(match v1 {
                Adt1::C0(_, _, _, _, _, _, _, _, h54) => {
                    h54
                }
            })?;
            let a56 = List::cons(Str::lit("}\u{a}\u{a}/** Instantiate a compiled guest and check that it offers exactly the world's exports. */\u{a}export declare function instantiate(module: WebAssembly.Module): Guest;\u{a}"), List::cons(f32(), List::<Str>::nil()));
            append_list(a55, a56)
        };
        append_list(a57, a58)
    }))
}

pub fn f14(v0: Str, v1: Adt4) -> R<Str> {
    let v2: Str = match v1.clone() {
        Adt4::C0(h59, _, _, _, _, _, _, _) => {
            h59
        }
    };
    Ok(f22({
        let a65 = List::cons(f23(Str::lit("//"), v0, f33(v1.clone())), List::cons(f34(v1.clone())?, List::cons(Str::lit("\u{a}/**"), List::<Str>::nil())));
        let a66 = {
            let a63 = f35({
                let a60 = f36(v1.clone(), f7(List::cons(Str::lit("A client for "), List::cons(v2.clone(), List::cons(Str::lit(", over request and response bytes."), List::<Str>::nil())))));
                let a61 = List::cons(Str::lit(""), List::<Str>::nil());
                append_list(a60, a61)
            });
            let a64 = List::cons(f7(List::cons(Str::lit(" * "), List::cons(v2, List::cons(Str::lit("'s model describes no typed operations, so the client sends bytes.\u{a} * `dispatch` is `app.dispatch` of an application prism-code's `load()` opened, or\u{a} * `(request) => dispatch(module, request)` with `dispatch` from ./world.js.\u{a} */\u{a}"), List::cons(f37(match v1 {
                Adt4::C0(_, h62, _, _, _, _, _, _) => {
                    h62
                }
            }), List::cons(Str::lit("\u{a}\u{a}  /** Send one request and resolve to the response bytes. */\u{a}  async invoke(request) {\u{a}    return this.#dispatch(request);\u{a}  }\u{a}}"), List::<Str>::nil())))))), List::<Str>::nil());
            append_list(a63, a64)
        };
        append_list(a65, a66)
    }))
}

pub fn f15(v0: Str, v1: Adt4) -> R<Str> {
    Ok(f22({
        let a71 = List::cons(f23(Str::lit("//"), v0, f33(v1.clone())), List::cons(f38(v1.clone())?, List::cons(Str::lit("\u{a}/** Sends request bytes to the guest and returns its response bytes. */\u{a}export type Dispatch = (request: Uint8Array) => Uint8Array | Promise<Uint8Array>;\u{a}"), List::<Str>::nil())));
        let a72 = {
            let a69 = f39(Str::lit(""), f36(v1.clone(), f7(List::cons(Str::lit("A client for "), List::cons(match v1.clone() {
                Adt4::C0(h67, _, _, _, _, _, _, _) => {
                    h67
                }
            }, List::cons(Str::lit(", over request and response bytes."), List::<Str>::nil()))))));
            let a70 = List::cons(f7(List::cons(Str::lit("export declare class "), List::cons(match v1 {
                Adt4::C0(_, h68, _, _, _, _, _, _) => {
                    h68
                }
            }, List::cons(Str::lit(" {\u{a}  constructor(dispatch: Dispatch);\u{a}  /** Send one request and resolve to the response bytes. */\u{a}  invoke(request: Uint8Array): Promise<Uint8Array>;\u{a}}"), List::<Str>::nil())))), List::<Str>::nil());
            append_list(a69, a70)
        };
        append_list(a71, a72)
    }))
}

pub fn f16(v0: Str, v1: Adt4, v2: Adt6) -> R<Str> {
    let v3: Str = match v1.clone() {
        Adt4::C0(h73, _, _, _, _, _, _, _) => {
            h73
        }
    };
    let v4: Str = match v2.clone() {
        Adt6::C0(_, _, _, h74, _, _) => {
            h74
        }
    };
    let v5: Str = match v2.clone() {
        Adt6::C0(_, h75, _, _, _, _) => {
            h75
        }
    };
    Ok(f22({
        let a97 = List::cons(f23(Str::lit("//"), v0, f33(v1.clone())), List::cons(f34(v1.clone())?, List::cons(f7(List::cons(Str::lit("\u{a}/** Every operation "), List::cons(v3.clone(), List::cons(Str::lit(" accepts, by its request name. */\u{a}export const OPERATIONS = Object.freeze(["), List::cons({
            let a77 = f40(match v2.clone() {
                Adt6::C0(_, _, _, _, h76, _) => {
                    h76
                }
            })?;
            let a78 = Str::lit(", ");
            join(a77, a78)
        }, List::cons(Str::lit("]);\u{a}\u{a}const VERSION = "), List::cons(f25(match v2.clone() {
            Adt6::C0(h79, _, _, _, _, _) => {
                h79
            }
        })?, List::cons(Str::lit(";\u{a}const MIN = -(2n ** 63n);\u{a}const MAX = 2n ** 63n - 1n;\u{a}const MESSAGES = Object.freeze({"), List::<Str>::nil())))))))), List::<Str>::nil())));
        let a98 = {
            let a95 = f41(v1.clone(), match v2.clone() {
                Adt6::C0(h80, _, _, _, _, _) => {
                    h80
                }
            }, match v2.clone() {
                Adt6::C0(_, _, _, _, _, h81) => {
                    h81
                }
            })?;
            let a96 = {
                let a93 = List::cons(f7(List::cons(Str::lit("});\u{a}\u{a}const encoder = new TextEncoder();\u{a}const decoder = new TextDecoder('utf-8', { fatal: true });\u{a}\u{a}/** A request "), List::cons(v3.clone(), List::cons(Str::lit(" refused. `code` is the guest's answer, such as `"), List::cons(f42(match v2.clone() {
                    Adt6::C0(_, _, _, _, _, h82) => {
                        h82
                    }
                }), List::cons(Str::lit("`. */\u{a}export class "), List::cons(v4.clone(), List::cons(Str::lit(" extends Error {\u{a}  constructor(code) {\u{a}    super(MESSAGES[code] ?? `"), List::cons(v3.clone(), List::cons(Str::lit(" refused the request: ${code}.`);\u{a}    this.name = "), List::cons(f25(v4.clone())?, List::cons(Str::lit(";\u{a}    this.code = code;\u{a}  }\u{a}}\u{a}\u{a}/**"), List::<Str>::nil())))))))))))), List::<Str>::nil());
                let a94 = {
                    let a91 = f35({
                        let a83 = f36(v1.clone(), f7(List::cons(Str::lit("A client for "), List::cons(v3.clone(), List::cons(Str::lit("."), List::<Str>::nil())))));
                        let a84 = List::cons(Str::lit(""), List::<Str>::nil());
                        append_list(a83, a84)
                    });
                    let a92 = {
                        let a89 = List::cons(f7(List::cons(Str::lit(" * `dispatch` sends request bytes to the guest and returns its response bytes:\u{a} * `app.dispatch` of an application prism-code's `load()` opened, or\u{a} * `(request) => dispatch(module, request)` with `dispatch` from ./world.js.\u{a} */\u{a}"), List::cons(f37(match v1 {
                            Adt4::C0(_, h85, _, _, _, _, _, _) => {
                                h85
                            }
                        }), List::<Str>::nil()))), List::<Str>::nil());
                        let a90 = {
                            let a87 = f43(v5.clone(), match v2 {
                                Adt6::C0(_, _, _, _, h86, _) => {
                                    h86
                                }
                            })?;
                            let a88 = List::cons(f7(List::cons(Str::lit("\u{a}  /**\u{a}   * Apply `operation` to two signed 64-bit integers. Resolves to the result;\u{a}   * rejects with "), List::cons(v4.clone(), List::cons(Str::lit(" when "), List::cons(v3, List::cons(Str::lit(" refuses, for example on overflow.\u{a}   */\u{a}  async "), List::cons(v5, List::cons(Str::lit("(operation, left, right) {\u{a}    if (!OPERATIONS.includes(operation)) throw new TypeError(`unknown operation ${operation}`);\u{a}    const request = encoder.encode(`${VERSION}\\t${operation}\\t${int64(left)}\\t${int64(right)}`);\u{a}    const [version, status, value] = decoder.decode(await this.#dispatch(request)).split('\\t');\u{a}    if (version !== VERSION) throw new "), List::cons(v4.clone(), List::cons(Str::lit("('unsupported-version');\u{a}    if (status === 'ok') return BigInt(value);\u{a}    throw new "), List::cons(v4, List::cons(Str::lit("(value);\u{a}  }\u{a}}\u{a}\u{a}function int64(value) {\u{a}  const n = typeof value === 'bigint' ? value : Number.isSafeInteger(value) ? BigInt(value) : null;\u{a}  if (n === null || n < MIN || n > MAX) throw new RangeError(`${value} is not a signed 64-bit integer`);\u{a}  return n.toString();\u{a}}"), List::<Str>::nil())))))))))))), List::<Str>::nil());
                            append_list(a87, a88)
                        };
                        append_list(a89, a90)
                    };
                    append_list(a91, a92)
                };
                append_list(a93, a94)
            };
            append_list(a95, a96)
        };
        append_list(a97, a98)
    }))
}

pub fn f17(v0: Str, v1: Adt4, v2: Adt6) -> R<Str> {
    let v3: Str = match v1.clone() {
        Adt4::C0(h99, _, _, _, _, _, _, _) => {
            h99
        }
    };
    let v4: Str = match v2.clone() {
        Adt6::C0(_, _, _, h100, _, _) => {
            h100
        }
    };
    let v5: Str = f7(List::cons(v4.clone(), List::cons(Str::lit("Code"), List::<Str>::nil())));
    Ok(f22({
        let a120 = List::cons(f23(Str::lit("//"), v0, f33(v1.clone())), List::cons(f38(v1.clone())?, List::cons(f7(List::cons(Str::lit("\u{a}/** An operation "), List::cons(v3.clone(), List::cons(Str::lit(" accepts, by its request name. */\u{a}export type "), List::cons(match v2.clone() {
            Adt6::C0(_, _, h101, _, _, _) => {
                h101
            }
        }, List::cons(Str::lit(" = "), List::cons({
            let a103 = f40(match v2.clone() {
                Adt6::C0(_, _, _, _, h102, _) => {
                    h102
                }
            })?;
            let a104 = Str::lit(" | ");
            join(a103, a104)
        }, List::cons(Str::lit(";\u{a}export declare const OPERATIONS: readonly "), List::cons(match v2.clone() {
            Adt6::C0(_, _, h105, _, _, _) => {
                h105
            }
        }, List::cons(Str::lit("[];\u{a}\u{a}/** A signed 64-bit integer: a bigint, or a number that is a safe integer. */\u{a}export type Int64 = bigint | number;\u{a}\u{a}/** Every answer with which "), List::cons(v3.clone(), List::cons(Str::lit(" refuses a request. */\u{a}export type "), List::cons(v5.clone(), List::cons(Str::lit(" = "), List::cons({
            let a107 = f44(match v2.clone() {
                Adt6::C0(_, _, _, _, _, h106) => {
                    h106
                }
            })?;
            let a108 = Str::lit(" | ");
            join(a107, a108)
        }, List::cons(Str::lit(";\u{a}\u{a}/** A request "), List::cons(v3.clone(), List::cons(Str::lit(" refused. `code` is the guest's answer. */\u{a}export declare class "), List::cons(v4.clone(), List::cons(Str::lit(" extends Error {\u{a}  constructor(code: "), List::cons(v5.clone(), List::cons(Str::lit(");\u{a}  readonly code: "), List::cons(v5, List::cons(Str::lit(";\u{a}}\u{a}\u{a}/** Sends request bytes to the guest and returns its response bytes. */\u{a}export type Dispatch = (request: Uint8Array) => Uint8Array | Promise<Uint8Array>;\u{a}"), List::<Str>::nil())))))))))))))))))))))))), List::<Str>::nil())));
        let a121 = {
            let a118 = f39(Str::lit(""), f36(v1.clone(), f7(List::cons(Str::lit("A client for "), List::cons(v3.clone(), List::cons(Str::lit("."), List::<Str>::nil()))))));
            let a119 = {
                let a116 = List::cons(f7(List::cons(Str::lit("export declare class "), List::cons(match v1 {
                    Adt4::C0(_, h109, _, _, _, _, _, _) => {
                        h109
                    }
                }, List::cons(Str::lit(" {\u{a}  constructor(dispatch: Dispatch);"), List::<Str>::nil())))), List::<Str>::nil());
                let a117 = {
                    let a114 = f45(match v2.clone() {
                        Adt6::C0(_, h110, _, _, _, _) => {
                            h110
                        }
                    }, match v2.clone() {
                        Adt6::C0(_, _, _, _, h111, _) => {
                            h111
                        }
                    })?;
                    let a115 = List::cons(f7(List::cons(Str::lit("  /** Apply `operation` to two signed 64-bit integers; rejects with "), List::cons(v4, List::cons(Str::lit(" when "), List::cons(v3, List::cons(Str::lit(" refuses. */\u{a}  "), List::cons(match v2.clone() {
                        Adt6::C0(_, h112, _, _, _, _) => {
                            h112
                        }
                    }, List::cons(Str::lit("(operation: "), List::cons(match v2 {
                        Adt6::C0(_, _, h113, _, _, _) => {
                            h113
                        }
                    }, List::cons(Str::lit(", left: Int64, right: Int64): Promise<bigint>;\u{a}}"), List::<Str>::nil())))))))))), List::<Str>::nil());
                    append_list(a114, a115)
                };
                append_list(a116, a117)
            };
            append_list(a118, a119)
        };
        append_list(a120, a121)
    }))
}

pub fn f18(v0: Str) -> R<bool> {
    let v1: Bytes = {
        let a122 = v0;
        utf8_encode(a122)
    };
    let a126 = {
        let a124 = {
            let a123 = v1.clone();
            length_bytes(a123)
        };
        let a125 = 0u64;
        nat_eq(a124, a125)
    };
    if bool_not(a126) {
        let a127 = f46(f47(v1.clone(), 0u64, 1u64), f48(), 10u64);
        if bool_not(a127) {
            f49(v1.clone(), 0u64, {
                let a128 = v1;
                length_bytes(a128)
            })
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub fn f19(v0: Str) -> R<Str> {
    f50(v0, Bytes::lit(&[0x22]))
}

pub fn f20(v0: Str) -> R<Str> {
    Ok(f51(f52({
        let a129 = v0.clone();
        utf8_encode(a129)
    }, 0u64, false, {
        let a131 = {
            let a130 = v0.clone();
            utf8_encode(a130)
        };
        length_bytes(a131)
    })?, v0))
}

pub fn f21(v0: Str, p138: List<(Str, Str)>) -> Option<Str> {
    let m137 = p138;
    {
        let mut m137 = m137;
        loop {
            let step = {
                let v1 = m137;
                let m132 = v1;
                match m132.uncons() {
                    None => {
                        Err::<List<(Str, Str)>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a134 = {
                            let (h133, _) = v2;
                            h133
                        };
                        let a135 = v0.clone();
                        if equal(a134, a135) {
                            Err::<List<(Str, Str)>, Option<Str>>(Some({
                                let (_, h136) = v2;
                                h136
                            }))
                        } else {
                            Ok::<List<(Str, Str)>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m137 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f22(v0: List<Str>) -> Str {
    let a141 = {
        let a139 = v0;
        let a140 = List::cons(Str::lit(""), List::<Str>::nil());
        append_list(a139, a140)
    };
    let a142 = Str::lit("\u{a}");
    join(a141, a142)
}

pub fn f23(v0: Str, v1: Str, v2: Str) -> Str {
    f7(List::cons(v0.clone(), List::cons(Str::lit(" Generated by `"), List::cons(v1.clone(), List::cons(Str::lit("` from "), List::cons(v2, List::cons(Str::lit(".\u{a}"), List::cons(v0, List::cons(Str::lit(" Do not edit: run `"), List::cons(v1, List::cons(Str::lit("` again.\u{a}"), List::<Str>::nil())))))))))))
}

pub fn f24(v0: Adt1) -> Str {
    f7(List::cons(Str::lit("the WIT world "), List::cons(match v0.clone() {
        Adt1::C0(h143, _, _, _, _, _, _, _, _) => {
            h143
        }
    }, List::cons(Str::lit(" ("), List::cons(match v0 {
        Adt1::C0(_, _, _, h144, _, _, _, _, _) => {
            h144
        }
    }, List::cons(Str::lit(")"), List::<Str>::nil()))))))
}

pub fn f25(v0: Str) -> R<Str> {
    f50(v0, Bytes::lit(&[0x27]))
}

pub fn f26(v0: List<(Str, Str)>) -> R<List<Str>> {
    let m145 = v0;
    match m145.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f7(List::cons(Str::lit("  "), List::cons({
                let (h146, _) = v1;
                h146
            }, List::cons(Str::lit(": "), List::cons(f25({
                let (_, h147) = v1;
                h147
            })?, List::cons(Str::lit(","), List::<Str>::nil())))))), f26(v2)?))
        }
    }
}

pub fn f27(v0: List<(Str, Str)>) -> List<(Str, Str)> {
    let m148 = v0;
    match m148.uncons() {
        None => {
            List::<(Str, Str)>::nil()
        }
        Some((v1, v2)) => {
            f53(v1, f27(v2))
        }
    }
}

pub fn f28(v0: List<Adt2>) -> List<(Str, Str)> {
    let m149 = v0;
    match m149.uncons() {
        None => {
            List::<(Str, Str)>::nil()
        }
        Some((v1, v2)) => {
            List::cons((match v1 {
                Adt2::C0(h150, _, _, _) => {
                    h150
                }
            }, Str::lit("function")), f28(v2))
        }
    }
}

pub fn f29(v0: List<Adt2>) -> R<List<Str>> {
    let m151 = v0;
    match m151.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a152 = f54(v1)?;
            let a153 = f29(v2)?;
            Ok(append_list(a152, a153))
        }
    }
}

pub fn f30() -> Str {
    Str::lit("/**\u{a} * Send one request to a fresh instance of the guest and return a copy of its response.\u{a} * The guest allocator never frees, so every request gets its own instance.\u{a} * @param {WebAssembly.Module} module\u{a} * @param {Uint8Array} request\u{a} * @returns {Uint8Array}\u{a} */\u{a}export function dispatch(module, request) {\u{a}  const guest = instantiate(module);\u{a}  const offset = guest.holoAlloc(request.length);\u{a}  new Uint8Array(guest.memory.buffer, offset, request.length).set(request);\u{a}  const packed = guest.holoRun(offset, request.length);\u{a}  const at = Number(packed >> 32n);\u{a}  const length = Number(packed & 0xffffffffn);\u{a}  if (at + length > guest.memory.buffer.byteLength) {\u{a}    throw new RangeError(`the guest answered ${length} bytes at ${at}, outside its memory`);\u{a}  }\u{a}  return new Uint8Array(guest.memory.buffer.slice(at, at + length));\u{a}}")
}

pub fn f31(v0: List<Adt2>) -> R<List<Str>> {
    let m154 = v0;
    match m154.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a155 = f55(v1)?;
            let a156 = f31(v2)?;
            Ok(append_list(a155, a156))
        }
    }
}

pub fn f32() -> Str {
    Str::lit("/**\u{a} * Send one request to a fresh instance of the guest and return a copy of its response.\u{a} * The guest allocator never frees, so every request gets its own instance.\u{a} */\u{a}export declare function dispatch(module: WebAssembly.Module, request: Uint8Array): Uint8Array;")
}

pub fn f33(v0: Adt4) -> Str {
    f7(List::cons(match v0.clone() {
        Adt4::C0(h157, _, _, _, _, _, _, _) => {
            h157
        }
    }, List::cons(Str::lit("'s model ("), List::cons(match v0.clone() {
        Adt4::C0(_, _, _, _, h158, _, _, _) => {
            h158
        }
    }, List::cons(Str::lit(", blake3:"), List::cons(match v0 {
        Adt4::C0(_, _, _, _, _, h159, _, _) => {
            h159
        }
    }, List::cons(Str::lit(")"), List::<Str>::nil())))))))
}

pub fn f34(v0: Adt4) -> R<Str> {
    Ok(f7(List::cons(Str::lit("/** The model this client was generated from. It equals `inputs.model` in prismpm.lock. */\u{a}export const MODEL = Object.freeze({\u{a}  name: "), List::cons(f25(match v0.clone() {
        Adt4::C0(h160, _, _, _, _, _, _, _) => {
            h160
        }
    })?, List::cons(Str::lit(",\u{a}  blake3: "), List::cons(f25(match v0.clone() {
        Adt4::C0(_, _, _, _, _, h161, _, _) => {
            h161
        }
    })?, List::cons(Str::lit(",\u{a}  sha256: "), List::cons(f25(match v0 {
        Adt4::C0(_, _, _, _, _, _, h162, _) => {
            h162
        }
    })?, List::cons(Str::lit(",\u{a}});"), List::<Str>::nil())))))))))
}

pub fn f35(v0: List<Str>) -> List<Str> {
    let m163 = v0;
    match m163.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f56(f7(List::cons(Str::lit(" * "), List::cons(v1, List::<Str>::nil())))), f35(v2))
        }
    }
}

pub fn f36(v0: Adt4, v1: Str) -> List<Str> {
    let m165 = match v0 {
        Adt4::C0(_, _, _, h164, _, _, _, _) => {
            h164
        }
    };
    match m165 {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some(v2) => {
            List::cons(v1, List::cons(Str::lit(""), List::cons(v2, List::<Str>::nil())))
        }
    }
}

pub fn f37(v0: Str) -> Str {
    f7(List::cons(Str::lit("export class "), List::cons(v0.clone(), List::cons(Str::lit(" {\u{a}  #dispatch;\u{a}\u{a}  /** @param {(request: Uint8Array) => Uint8Array | Promise<Uint8Array>} dispatch */\u{a}  constructor(dispatch) {\u{a}    if (typeof dispatch !== 'function') throw new TypeError('"), List::cons(v0, List::cons(Str::lit(" needs a dispatch function');\u{a}    this.#dispatch = dispatch;\u{a}  }"), List::<Str>::nil()))))))
}

pub fn f38(v0: Adt4) -> R<Str> {
    Ok(f7(List::cons(Str::lit("/** The model this client was generated from. It equals `inputs.model` in prismpm.lock. */\u{a}export declare const MODEL: {\u{a}  readonly name: "), List::cons(f25(match v0 {
        Adt4::C0(h166, _, _, _, _, _, _, _) => {
            h166
        }
    })?, List::cons(Str::lit(";\u{a}  readonly blake3: string;\u{a}  readonly sha256: string;\u{a}};"), List::<Str>::nil())))))
}

pub fn f39(v0: Str, v1: List<Str>) -> List<Str> {
    let m167 = v1.clone();
    match m167.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            let m168 = v3;
            match m168.uncons() {
                None => {
                    List::cons(f7(List::cons(v0, List::cons(Str::lit("/** "), List::cons(v2, List::cons(Str::lit(" */"), List::<Str>::nil()))))), List::<Str>::nil())
                }
                Some((_, _)) => {
                    let a169 = List::cons(f7(List::cons(v0.clone(), List::cons(Str::lit("/**"), List::<Str>::nil()))), f57(v0.clone(), v1));
                    let a170 = List::cons(f7(List::cons(v0, List::cons(Str::lit(" */"), List::<Str>::nil()))), List::<Str>::nil());
                    append_list(a169, a170)
                }
            }
        }
    }
}

pub fn f40(v0: List<Adt7>) -> R<List<Str>> {
    let m171 = v0;
    match m171.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f25(match v1 {
                Adt7::C0(h172, _, _) => {
                    h172
                }
            })?, f40(v2)?))
        }
    }
}

pub fn f41(v0: Adt4, v1: Str, v2: List<Adt8>) -> R<List<Str>> {
    let m173 = v2;
    match m173.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v3, v4)) => {
            Ok(List::cons(f7(List::cons(Str::lit("  "), List::cons(if f58(match v3.clone() {
                Adt8::C0(h174, _) => {
                    h174
                }
            })? {
                match v3.clone() {
                    Adt8::C0(h175, _) => {
                        h175
                    }
                }
            } else {
                f25(match v3.clone() {
                    Adt8::C0(h176, _) => {
                        h176
                    }
                })?
            }, List::cons(Str::lit(": "), List::cons(f25(f59(v0.clone(), v1.clone(), v3))?, List::cons(Str::lit(","), List::<Str>::nil())))))), f41(v0, v1, v4)?))
        }
    }
}

pub fn f42(v0: List<Adt8>) -> Str {
    let m177 = v0;
    match m177.uncons() {
        None => {
            Str::lit("an error code")
        }
        Some((v1, _)) => {
            match v1 {
                Adt8::C0(h178, _) => {
                    h178
                }
            }
        }
    }
}

pub fn f43(v0: Str, v1: List<Adt7>) -> R<List<Str>> {
    let m179 = v1;
    match m179.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(f7(List::cons(Str::lit("\u{a}  "), List::cons(f60(v0.clone(), v2.clone())?, List::cons(Str::lit("\u{a}  "), List::cons(f61(match v2.clone() {
                Adt7::C0(h180, _, _) => {
                    h180
                }
            })?, List::cons(Str::lit("(left, right) {\u{a}    return this."), List::cons(v0.clone(), List::cons(Str::lit("("), List::cons(f25(match v2 {
                Adt7::C0(h181, _, _) => {
                    h181
                }
            })?, List::cons(Str::lit(", left, right);\u{a}  }"), List::<Str>::nil())))))))))), f43(v0, v3)?))
        }
    }
}

pub fn f44(v0: List<Adt8>) -> R<List<Str>> {
    let m182 = v0;
    match m182.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f25(match v1 {
                Adt8::C0(h183, _) => {
                    h183
                }
            })?, f44(v2)?))
        }
    }
}

pub fn f45(v0: Str, v1: List<Adt7>) -> R<List<Str>> {
    let m184 = v1;
    match m184.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons(f7(List::cons(Str::lit("  "), List::cons(f60(v0.clone(), v2.clone())?, List::cons(Str::lit("\u{a}  "), List::cons(f61(match v2 {
                Adt7::C0(h185, _, _) => {
                    h185
                }
            })?, List::cons(Str::lit("(left: Int64, right: Int64): Promise<bigint>;"), List::<Str>::nil())))))), f45(v0, v3)?))
        }
    }
}

pub fn f46(v0: Bytes, v1: Bytes, p188: u64) -> bool {
    let m187 = p188;
    {
        let mut m187 = m187;
        loop {
            let step = {
                let v2 = m187;
                let m186 = v2;
                if m186 == 0 {
                    Err::<u64, bool>(false)
                } else {
                    let v3 = m186 - 1;
                    if f62(f47(v1.clone(), v3, 1u64), v0.clone()) {
                        Err::<u64, bool>(true)
                    } else {
                        Ok::<u64, bool>(v3)
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

pub fn f47(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m192 = {
        let a189 = v0;
        let a190 = v1;
        let a191 = v2;
        slice_bytes(a189, a190, a191)
    };
    match m192 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f48() -> Bytes {
    Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39])
}

pub fn f49(v0: Bytes, p197: u64, p198: u64) -> R<bool> {
    let m196 = (p197, p198);
    {
        let mut m196 = m196;
        loop {
            let step = {
                let (v1, v2) = m196;
                let m193 = v2;
                if m193 == 0 {
                    Err::<(u64, u64), R<bool>>(Ok(true))
                } else {
                    let v3 = m193 - 1;
                    if if f63(f47(v0.clone(), v1, 1u64)) {
                        true
                    } else {
                        f62(f47(v0.clone(), v1, 1u64), Bytes::lit(&[0x5f]))
                    } {
                        Ok::<(u64, u64), R<bool>>(({
                            let a194 = v1;
                            let a195 = 1u64;
                            nat_add(a194, a195)?
                        }, v3))
                    } else {
                        Err::<(u64, u64), R<bool>>(Ok(false))
                    }
                }
            };
            match step {
                Ok(next) => { m196 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f50(v0: Str, v1: Bytes) -> R<Str> {
    Ok(f51({
        let a204 = {
            let a202 = v1.clone();
            let a203 = f64({
                let a199 = v0.clone();
                utf8_encode(a199)
            }, 0u64, v1.clone(), {
                let a201 = {
                    let a200 = v0.clone();
                    utf8_encode(a200)
                };
                length_bytes(a201)
            })?;
            append_bytes(a202, a203)
        };
        let a205 = v1;
        append_bytes(a204, a205)
    }, v0))
}

pub fn f51(v0: Bytes, v1: Str) -> Str {
    let m207 = {
        let a206 = v0;
        utf8_decode(a206)
    };
    match m207 {
        None => {
            v1
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f52(v0: Bytes, v1: u64, v2: bool, v3: u64) -> R<Bytes> {
    let m208 = v3;
    if m208 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m208 - 1;
        let v5: Bytes = f47(v0.clone(), v1, 1u64);
        if f62(v5.clone(), Bytes::lit(&[0x2d])) {
            let a211 = Bytes::lit(&[0x5f]);
            let a212 = f52(v0, {
                let a209 = v1;
                let a210 = 1u64;
                nat_add(a209, a210)?
            }, false, v4)?;
            Ok(append_bytes(a211, a212))
        } else {
            if f65(v5.clone()) {
                let a217 = if v2 {
                    let a213 = Bytes::lit(&[0x5f]);
                    let a214 = f66(v5);
                    append_bytes(a213, a214)
                } else {
                    f66(v5)
                };
                let a218 = f52(v0, {
                    let a215 = v1;
                    let a216 = 1u64;
                    nat_add(a215, a216)?
                }, false, v4)?;
                Ok(append_bytes(a217, a218))
            } else {
                let a221 = v5.clone();
                let a222 = f52(v0, {
                    let a219 = v1;
                    let a220 = 1u64;
                    nat_add(a219, a220)?
                }, f63(v5), v4)?;
                Ok(append_bytes(a221, a222))
            }
        }
    }
}

pub fn f53(v0: (Str, Str), v1: List<(Str, Str)>) -> List<(Str, Str)> {
    let m223 = v1;
    match m223.uncons() {
        None => {
            List::cons(v0, List::<(Str, Str)>::nil())
        }
        Some((v2, v3)) => {
            if f67({
                let (h224, _) = v0.clone();
                h224
            }, {
                let (h225, _) = v2.clone();
                h225
            }) {
                List::cons(v0, List::cons(v2, v3))
            } else {
                List::cons(v2, f53(v0, v3))
            }
        }
    }
}

pub fn f54(v0: Adt2) -> R<List<Str>> {
    let v1: Str = f7(List::cons(Str::lit("exports."), List::cons(match v0.clone() {
        Adt2::C0(h226, _, _, _) => {
            h226
        }
    }, List::cons(Str::lit("("), List::cons({
        let a228 = f68(match v0.clone() {
            Adt2::C0(_, h227, _, _) => {
                h227
            }
        });
        let a229 = Str::lit(", ");
        join(a228, a229)
    }, List::cons(Str::lit(")"), List::<Str>::nil()))))));
    let a237 = f39(Str::lit("    "), match v0.clone() {
        Adt2::C0(_, _, _, h230) => {
            h230
        }
    });
    let a238 = List::cons(f7(List::cons(Str::lit("    "), List::cons(f61(match v0.clone() {
        Adt2::C0(h231, _, _, _) => {
            h231
        }
    })?, List::cons(Str::lit(": ("), List::cons({
        let a233 = f69(match v0.clone() {
            Adt2::C0(_, h232, _, _) => {
                h232
            }
        });
        let a234 = Str::lit(", ");
        join(a233, a234)
    }, List::cons(Str::lit(") => "), List::cons({
        let m236 = match v0 {
            Adt2::C0(_, _, h235, _) => {
                h235
            }
        };
        match m236 {
            None => {
                f7(List::cons(Str::lit("void "), List::cons(v1, List::<Str>::nil())))
            }
            Some(v2) => {
                f70(v2, v1)
            }
        }
    }, List::cons(Str::lit(","), List::<Str>::nil())))))))), List::<Str>::nil());
    Ok(append_list(a237, a238))
}

pub fn f55(v0: Adt2) -> R<List<Str>> {
    let a246 = f39(Str::lit("  "), match v0.clone() {
        Adt2::C0(_, _, _, h239) => {
            h239
        }
    });
    let a247 = List::cons(f7(List::cons(Str::lit("  "), List::cons(f61(match v0.clone() {
        Adt2::C0(h240, _, _, _) => {
            h240
        }
    })?, List::cons(Str::lit("("), List::cons({
        let a242 = f71(match v0.clone() {
            Adt2::C0(_, h241, _, _) => {
                h241
            }
        });
        let a243 = Str::lit(", ");
        join(a242, a243)
    }, List::cons(Str::lit("): "), List::cons({
        let m245 = match v0 {
            Adt2::C0(_, _, h244, _) => {
                h244
            }
        };
        match m245 {
            None => {
                Str::lit("void")
            }
            Some(v1) => {
                f72(v1)
            }
        }
    }, List::cons(Str::lit(";"), List::<Str>::nil())))))))), List::<Str>::nil());
    Ok(append_list(a246, a247))
}

pub fn f56(v0: Str) -> Str {
    f51(f73({
        let a248 = v0.clone();
        utf8_encode(a248)
    }, {
        let a250 = {
            let a249 = v0.clone();
            utf8_encode(a249)
        };
        length_bytes(a250)
    }), v0)
}

pub fn f57(v0: Str, v1: List<Str>) -> List<Str> {
    let m251 = v1;
    match m251.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            List::cons(f56(f7(List::cons(v0.clone(), List::cons(Str::lit(" * "), List::cons(v2, List::<Str>::nil()))))), f57(v0, v3))
        }
    }
}

pub fn f58(v0: Str) -> R<bool> {
    let a252 = f61(v0.clone())?;
    let a253 = v0;
    Ok(equal(a252, a253))
}

pub fn f59(v0: Adt4, v1: Str, v2: Adt8) -> Str {
    let m255 = match v2.clone() {
        Adt8::C0(_, h254) => {
            h254
        }
    };
    match m255 {
        None => {
            let a257 = match v2.clone() {
                Adt8::C0(h256, _) => {
                    h256
                }
            };
            let a258 = Str::lit("unsupported-version");
            if equal(a257, a258) {
                f7(List::cons(match v0 {
                    Adt4::C0(h259, _, _, _, _, _, _, _) => {
                        h259
                    }
                }, List::cons(Str::lit(" does not answer request version "), List::cons(v1, List::cons(Str::lit("."), List::<Str>::nil())))))
            } else {
                let a261 = match v2.clone() {
                    Adt8::C0(h260, _) => {
                        h260
                    }
                };
                let a262 = Str::lit("unknown-operation");
                if equal(a261, a262) {
                    f7(List::cons(match v0 {
                        Adt4::C0(h263, _, _, _, _, _, _, _) => {
                            h263
                        }
                    }, List::cons(Str::lit(" does not know this operation."), List::<Str>::nil())))
                } else {
                    f7(List::cons(match v0 {
                        Adt4::C0(h264, _, _, _, _, _, _, _) => {
                            h264
                        }
                    }, List::cons(Str::lit(" refused the request: "), List::cons(match v2 {
                        Adt8::C0(h265, _) => {
                            h265
                        }
                    }, List::cons(Str::lit("."), List::<Str>::nil())))))
                }
            }
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f60(v0: Str, v1: Adt7) -> R<Str> {
    Ok(f7(List::cons(Str::lit("/** "), List::cons(match v1.clone() {
        Adt7::C0(_, h266, _) => {
            h266
        }
    }, List::cons(Str::lit(": `"), List::cons(v0, List::cons(Str::lit("("), List::cons(f25(match v1 {
        Adt7::C0(h267, _, _) => {
            h267
        }
    })?, List::cons(Str::lit(", left, right)`. */"), List::<Str>::nil())))))))))
}

pub fn f61(v0: Str) -> R<Str> {
    Ok(f51(f74({
        let a268 = v0.clone();
        utf8_encode(a268)
    }, 0u64, false, {
        let a270 = {
            let a269 = v0.clone();
            utf8_encode(a269)
        };
        length_bytes(a270)
    })?, v0))
}

pub fn f62(v0: Bytes, v1: Bytes) -> bool {
    let a275 = {
        let a271 = v0;
        let a272 = v1;
        compare_bytes(a271, a272)
    };
    let a276 = {
        let a273 = Bytes::lit(&[0x00]);
        let a274 = Bytes::lit(&[0x00]);
        compare_bytes(a273, a274)
    };
    equal(a275, a276)
}

pub fn f63(v0: Bytes) -> bool {
    if f46(v0.clone(), f75(), 26u64) {
        true
    } else {
        f46(v0, f48(), 10u64)
    }
}

pub fn f64(v0: Bytes, v1: u64, v2: Bytes, v3: u64) -> R<Bytes> {
    let m277 = v3;
    if m277 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m277 - 1;
        let a282 = if f62(f47(v0.clone(), v1, 1u64), Bytes::lit(&[0x5c])) {
            Bytes::lit(&[0x5c, 0x5c])
        } else {
            if f62(f47(v0.clone(), v1, 1u64), v2.clone()) {
                let a278 = Bytes::lit(&[0x5c]);
                let a279 = v2.clone();
                append_bytes(a278, a279)
            } else {
                if f62(f47(v0.clone(), v1, 1u64), Bytes::lit(&[0x0a])) {
                    Bytes::lit(&[0x5c, 0x6e])
                } else {
                    f47(v0.clone(), v1, 1u64)
                }
            }
        };
        let a283 = f64(v0, {
            let a280 = v1;
            let a281 = 1u64;
            nat_add(a280, a281)?
        }, v2, v4)?;
        Ok(append_bytes(a282, a283))
    }
}

pub fn f65(v0: Bytes) -> bool {
    let a284 = f62(f66(v0.clone()), v0);
    bool_not(a284)
}

pub fn f66(v0: Bytes) -> Bytes {
    f76(v0, f77(), f75(), 26u64)
}

pub fn f67(v0: Str, v1: Str) -> bool {
    let a291 = {
        let a287 = {
            let a285 = v0;
            utf8_encode(a285)
        };
        let a288 = {
            let a286 = v1;
            utf8_encode(a286)
        };
        compare_bytes(a287, a288)
    };
    let a292 = {
        let a289 = Bytes::lit(&[0x00]);
        let a290 = Bytes::lit(&[0x01]);
        compare_bytes(a289, a290)
    };
    equal(a291, a292)
}

pub fn f68(v0: List<Adt3>) -> List<Str> {
    let m293 = v0;
    match m293.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons({
                let a295 = match v1.clone() {
                    Adt3::C0(_, h294) => {
                        h294
                    }
                };
                let a296 = Str::lit("bool");
                if equal(a295, a296) {
                    f7(List::cons(match v1 {
                        Adt3::C0(h297, _) => {
                            h297
                        }
                    }, List::cons(Str::lit(" ? 1 : 0"), List::<Str>::nil())))
                } else {
                    match v1 {
                        Adt3::C0(h298, _) => {
                            h298
                        }
                    }
                }
            }, f68(v2))
        }
    }
}

pub fn f69(v0: List<Adt3>) -> List<Str> {
    let m299 = v0;
    match m299.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt3::C0(h300, _) => {
                    h300
                }
            }, f69(v2))
        }
    }
}

pub fn f70(v0: Str, v1: Str) -> Str {
    let a301 = v0.clone();
    let a302 = Str::lit("bool");
    if equal(a301, a302) {
        f7(List::cons(v1, List::cons(Str::lit(" !== 0"), List::<Str>::nil())))
    } else {
        let a303 = v0.clone();
        let a304 = Str::lit("u8");
        if if equal(a303, a304) {
            true
        } else {
            let a305 = v0.clone();
            let a306 = Str::lit("u16");
            if equal(a305, a306) {
                true
            } else {
                let a307 = v0.clone();
                let a308 = Str::lit("u32");
                if equal(a307, a308) {
                    true
                } else {
                    let a309 = v0.clone();
                    let a310 = Str::lit("char");
                    equal(a309, a310)
                }
            }
        } {
            f7(List::cons(v1, List::cons(Str::lit(" >>> 0"), List::<Str>::nil())))
        } else {
            let a311 = v0.clone();
            let a312 = Str::lit("s8");
            if if equal(a311, a312) {
                true
            } else {
                let a313 = v0.clone();
                let a314 = Str::lit("s16");
                if equal(a313, a314) {
                    true
                } else {
                    let a315 = v0.clone();
                    let a316 = Str::lit("s32");
                    equal(a315, a316)
                }
            } {
                f7(List::cons(v1, List::cons(Str::lit(" | 0"), List::<Str>::nil())))
            } else {
                let a317 = v0.clone();
                let a318 = Str::lit("u64");
                if equal(a317, a318) {
                    f7(List::cons(Str::lit("BigInt.asUintN(64, "), List::cons(v1, List::cons(Str::lit(")"), List::<Str>::nil()))))
                } else {
                    let a319 = v0;
                    let a320 = Str::lit("s64");
                    if equal(a319, a320) {
                        f7(List::cons(Str::lit("BigInt.asIntN(64, "), List::cons(v1, List::cons(Str::lit(")"), List::<Str>::nil()))))
                    } else {
                        v1
                    }
                }
            }
        }
    }
}

pub fn f71(v0: List<Adt3>) -> List<Str> {
    let m321 = v0;
    match m321.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f7(List::cons(match v1.clone() {
                Adt3::C0(h322, _) => {
                    h322
                }
            }, List::cons(Str::lit(": "), List::cons(f72(match v1 {
                Adt3::C0(_, h323) => {
                    h323
                }
            }), List::<Str>::nil())))), f71(v2))
        }
    }
}

pub fn f72(v0: Str) -> Str {
    let a324 = v0.clone();
    let a325 = Str::lit("bool");
    if equal(a324, a325) {
        Str::lit("boolean")
    } else {
        let a326 = v0.clone();
        let a327 = Str::lit("u64");
        if if equal(a326, a327) {
            true
        } else {
            let a328 = v0;
            let a329 = Str::lit("s64");
            equal(a328, a329)
        } {
            Str::lit("bigint")
        } else {
            Str::lit("number")
        }
    }
}

pub fn f73(p332: Bytes, p333: u64) -> Bytes {
    let m331 = (p332, p333);
    {
        let mut m331 = m331;
        loop {
            let step = {
                let (v0, v1) = m331;
                let m330 = v1;
                if m330 == 0 {
                    Err::<(Bytes, u64), Bytes>(Bytes::lit(&[]))
                } else {
                    let v2 = m330 - 1;
                    if f62(f47(v0.clone(), v2, 1u64), Bytes::lit(&[0x20])) {
                        Ok::<(Bytes, u64), Bytes>((f47(v0, 0u64, v2), v2))
                    } else {
                        Err::<(Bytes, u64), Bytes>(v0)
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

pub fn f74(v0: Bytes, v1: u64, v2: bool, v3: u64) -> R<Bytes> {
    let m334 = v3;
    if m334 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m334 - 1;
        if f78(f47(v0.clone(), v1, 1u64)) {
            f74(v0, {
                let a335 = v1;
                let a336 = 1u64;
                nat_add(a335, a336)?
            }, true, v4)
        } else {
            let a339 = if v2 {
                f79(f47(v0.clone(), v1, 1u64))
            } else {
                f47(v0.clone(), v1, 1u64)
            };
            let a340 = f74(v0, {
                let a337 = v1;
                let a338 = 1u64;
                nat_add(a337, a338)?
            }, false, v4)?;
            Ok(append_bytes(a339, a340))
        }
    }
}

pub fn f75() -> Bytes {
    Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a])
}

pub fn f76(v0: Bytes, v1: Bytes, v2: Bytes, p343: u64) -> Bytes {
    let m342 = p343;
    {
        let mut m342 = m342;
        loop {
            let step = {
                let v3 = m342;
                let m341 = v3;
                if m341 == 0 {
                    Err::<u64, Bytes>(v0.clone())
                } else {
                    let v4 = m341 - 1;
                    if f62(f47(v1.clone(), v4, 1u64), v0.clone()) {
                        Err::<u64, Bytes>(f47(v2.clone(), v4, 1u64))
                    } else {
                        Ok::<u64, Bytes>(v4)
                    }
                }
            };
            match step {
                Ok(next) => { m342 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f77() -> Bytes {
    Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a])
}

pub fn f78(v0: Bytes) -> bool {
    if f62(v0.clone(), Bytes::lit(&[0x2d])) {
        true
    } else {
        f62(v0, Bytes::lit(&[0x5f]))
    }
}

pub fn f79(v0: Bytes) -> Bytes {
    f76(v0, f75(), f77(), 26u64)
}

pub fn run(p0: Adt0) -> R<Adt9> {
    f0(p0)
}
