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
    Ok(f1(f2(v0.clone())?, f3(v0)?))
}

pub fn f1(v0: Option<(Str, Str)>, v1: List<Adt10>) -> Adt9 {
    let m1 = v0;
    match m1 {
        None => {
            Adt9::C0(v1)
        }
        Some(v2) => {
            Adt9::C1({
                let (h2, _) = v2;
                h2
            }, {
                let (_, h3) = v2;
                h3
            })
        }
    }
}

pub fn f2(v0: Adt0) -> R<Option<(Str, Str)>> {
    let v1: Adt4 = match v0.clone() {
        Adt0::C0(_, _, h4) => {
            h4
        }
    };
    let v2: Str = f4(match v1.clone() {
        Adt4::C0(_, _, h5, _, _, _, _, _) => {
            h5
        }
    })?;
    Ok(f5(f6(Str::lit("python"), f7(match v1.clone() {
        Adt4::C0(_, _, h6, _, _, _, _, _) => {
            h6
        }
    })?)?, f5(if f8(v2.clone(), List::cons(Str::lit("world"), List::cons(Str::lit("identity"), List::<Str>::nil()))) {
        Some((Str::lit("generate.model"), f9(List::cons(Str::lit("an application named "), List::cons(v2.clone(), List::cons(Str::lit(" collides with the "), List::cons(v2, List::cons(Str::lit(" module"), List::<Str>::nil()))))))))
    } else {
        None::<(Str, Str)>
    }, f5(f10(v1)?, f5(f11(Str::lit("python"), match v0.clone() {
        Adt0::C0(_, h7, _) => {
            h7
        }
    }), f12(Str::lit("python"), true, f13(), {
        let m10 = match v0 {
            Adt0::C0(_, h9, _) => {
                h9
            }
        };
        match m10 {
            Adt1::C0(_, _, _, _, _, _, _, _, h8) => {
                h8
            }
        }
    })?)))))
}

pub fn f3(v0: Adt0) -> R<List<Adt10>> {
    let v1: Str = match v0.clone() {
        Adt0::C0(h11, _, _) => {
            h11
        }
    };
    let v2: Adt1 = match v0.clone() {
        Adt0::C0(_, h12, _) => {
            h12
        }
    };
    let v3: Adt4 = match v0 {
        Adt0::C0(_, _, h13) => {
            h13
        }
    };
    let v4: Str = f4(match v3.clone() {
        Adt4::C0(_, _, h14, _, _, _, _, _) => {
            h14
        }
    })?;
    Ok(List::cons(f14(Str::lit("__init__.py"), f15(v1.clone(), v2.clone(), v3.clone(), v4.clone())?), List::cons(f14(Str::lit("identity.py"), f16(v1.clone(), v2.clone(), v3.clone())), List::cons(f14(Str::lit("world.py"), f17(v1.clone(), v2)?), List::cons(f14(f9(List::cons(v4, List::cons(Str::lit(".py"), List::<Str>::nil()))), {
        let m16 = match v3.clone() {
            Adt4::C0(_, _, _, _, _, _, _, h15) => {
                h15
            }
        };
        match m16 {
            Adt5::C0 => {
                f18(v1, v3)?
            }
            Adt5::C1(v5) => {
                f19(v1, v3, v5)?
            }
        }
    }), List::<Adt10>::nil())))))
}

pub fn f4(v0: Str) -> R<Str> {
    Ok(f20(f7(v0)?))
}

pub fn f5(v0: Option<(Str, Str)>, v1: Option<(Str, Str)>) -> Option<(Str, Str)> {
    let m17 = v0.clone();
    match m17 {
        None => {
            v1
        }
        Some(_) => {
            v0
        }
    }
}

pub fn f6(v0: Str, v1: Str) -> R<Option<(Str, Str)>> {
    if f21(v1.clone())? {
        Ok(None::<(Str, Str)>)
    } else {
        Ok(Some((Str::lit("generate.model"), f9(List::cons(f22(v1)?, List::cons(Str::lit(" is not a name the "), List::cons(v0, List::cons(Str::lit(" writer can bind"), List::<Str>::nil()))))))))
    }
}

pub fn f7(v0: Str) -> R<Str> {
    Ok(f23(f24({
        let a18 = v0.clone();
        utf8_encode(a18)
    }, 0u64, false, {
        let a20 = {
            let a19 = v0.clone();
            utf8_encode(a19)
        };
        length_bytes(a20)
    })?, v0))
}

pub fn f8(v0: Str, p25: List<Str>) -> bool {
    let m24 = p25;
    {
        let mut m24 = m24;
        loop {
            let step = {
                let v1 = m24;
                let m21 = v1;
                match m21.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a22 = v2;
                        let a23 = v0.clone();
                        if equal(a22, a23) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m24 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f9(v0: List<Str>) -> Str {
    let a26 = v0;
    let a27 = Str::lit("");
    join(a26, a27)
}

pub fn f10(v0: Adt4) -> R<Option<(Str, Str)>> {
    let m29 = match v0.clone() {
        Adt4::C0(_, _, _, _, _, _, _, h28) => {
            h28
        }
    };
    match m29 {
        Adt5::C0 => {
            Ok(None::<(Str, Str)>)
        }
        Adt5::C1(v1) => {
            Ok(f5(f25(Str::lit("python"), match v1.clone() {
                Adt6::C0(_, _, _, _, h30, _) => {
                    h30
                }
            })?, f5(f6(Str::lit("python"), f7(match v1.clone() {
                Adt6::C0(_, h31, _, _, _, _) => {
                    h31
                }
            })?)?, f5(f26(f4(match v1.clone() {
                Adt6::C0(_, h32, _, _, _, _) => {
                    h32
                }
            })?, match v1.clone() {
                Adt6::C0(_, _, _, _, h33, _) => {
                    h33
                }
            })?, f27(v0, v1)))))
        }
    }
}

pub fn f11(v0: Str, v1: Adt1) -> Option<(Str, Str)> {
    let a37 = {
        let a35 = match v1.clone() {
            Adt1::C0(h34, _, _, _, _, _, _, _, _) => {
                h34
            }
        };
        let a36 = Str::lit("hologram:guest/core-wasm@1");
        equal(a35, a36)
    };
    if bool_not(a37) {
        Some((Str::lit("generate.world"), f9(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer has no host protocol for "), List::cons(match v1 {
            Adt1::C0(h38, _, _, _, _, _, _, _, _) => {
                h38
            }
        }, List::<Str>::nil())))))))
    } else {
        let m40 = match v1.clone() {
            Adt1::C0(_, _, _, _, _, _, _, h39, _) => {
                h39
            }
        };
        match m40.uncons() {
            None => {
                None::<(Str, Str)>
            }
            Some((_, _)) => {
                Some((Str::lit("generate.world"), f9(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer binds import-free worlds; "), List::cons(match v1 {
                    Adt1::C0(h41, _, _, _, _, _, _, _, _) => {
                        h41
                    }
                }, List::cons(Str::lit(" has imports"), List::<Str>::nil()))))))))
            }
        }
    }
}

pub fn f12(v0: Str, v1: bool, v2: List<(Str, Str)>, v3: List<Adt2>) -> R<Option<(Str, Str)>> {
    let m42 = v3;
    match m42.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v4, v5)) => {
            Ok(f5(if v1 {
                f6(v0.clone(), match v4.clone() {
                    Adt2::C0(h43, _, _, _) => {
                        h43
                    }
                })?
            } else {
                None::<(Str, Str)>
            }, f5(f28(v0.clone(), v1, v2.clone(), match v4.clone() {
                Adt2::C0(_, h44, _, _) => {
                    h44
                }
            })?, f5({
                let m46 = match v4 {
                    Adt2::C0(_, _, h45, _) => {
                        h45
                    }
                };
                match m46 {
                    None => {
                        None::<(Str, Str)>
                    }
                    Some(v6) => {
                        f29(v0.clone(), v6, v2.clone())
                    }
                }
            }, f12(v0, v1, v2, v5)?))))
        }
    }
}

pub fn f13() -> List<(Str, Str)> {
    List::cons((Str::lit("bool"), Str::lit("bool")), List::cons((Str::lit("u8"), Str::lit("int")), List::cons((Str::lit("u16"), Str::lit("int")), List::cons((Str::lit("u32"), Str::lit("int")), List::cons((Str::lit("u64"), Str::lit("int")), List::cons((Str::lit("s8"), Str::lit("int")), List::cons((Str::lit("s16"), Str::lit("int")), List::cons((Str::lit("s32"), Str::lit("int")), List::cons((Str::lit("s64"), Str::lit("int")), List::cons((Str::lit("f32"), Str::lit("float")), List::cons((Str::lit("f64"), Str::lit("float")), List::cons((Str::lit("char"), Str::lit("int")), List::<(Str, Str)>::nil()))))))))))))
}

pub fn f14(v0: Str, v1: Str) -> Adt10 {
    Adt10::C0(v0, v1)
}

pub fn f15(v0: Str, v1: Adt1, v2: Adt4, v3: Str) -> R<Str> {
    let v4: List<Str> = f30(f31(v2.clone()))?;
    let v5: List<Str> = f32(f9(List::cons(Str::lit("from ."), List::cons(v3.clone(), List::cons(Str::lit(" import "), List::<Str>::nil())))), v4.clone(), Str::lit("("), Str::lit(")"));
    let v6: List<Str> = List::cons(Str::lit("from .identity import Identity"), List::<Str>::nil());
    Ok(f33({
        let a70 = List::cons(f34(Str::lit("#"), v0.clone(), f9(List::cons(match v2.clone() {
            Adt4::C0(h47, _, _, _, _, _, _, _) => {
                h47
            }
        }, List::cons(Str::lit("'s model and the WIT world "), List::cons(match v1.clone() {
            Adt1::C0(h48, _, _, _, _, _, _, _, _) => {
                h48
            }
        }, List::<Str>::nil()))))), List::cons(f9(List::cons(Str::lit("\"\"\"A client for "), List::cons(match v2.clone() {
            Adt4::C0(h49, _, _, _, _, _, _, _) => {
                h49
            }
        }, List::cons(Str::lit(", generated by "), List::cons(v0, List::cons(Str::lit("."), List::<Str>::nil())))))), List::<Str>::nil()));
        let a71 = {
            let a68 = {
                let m51 = match v2.clone() {
                    Adt4::C0(_, _, _, h50, _, _, _, _) => {
                        h50
                    }
                };
                match m51 {
                    None => {
                        List::<Str>::nil()
                    }
                    Some(v7) => {
                        List::cons(Str::lit(""), List::cons(v7, List::<Str>::nil()))
                    }
                }
            };
            let a69 = {
                let a66 = List::cons(Str::lit(""), List::cons(f9(List::cons(Str::lit("`world` is the world "), List::cons(match v1 {
                    Adt1::C0(h52, _, _, _, _, _, _, _, _) => {
                        h52
                    }
                }, List::cons(Str::lit(": implement `world.Guest` over your WebAssembly engine,"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("and `world.dispatch` sends it a request. `"), List::cons(v3.clone(), List::cons(Str::lit("` is the client of "), List::cons(match v2 {
                    Adt4::C0(h53, _, _, _, _, _, _, _) => {
                        h53
                    }
                }, List::cons(Str::lit("."), List::<Str>::nil())))))), List::cons(Str::lit("\"\"\"\u{a}"), List::cons(f9(List::cons(Str::lit("from . import "), List::cons({
                    let a54 = f35(List::cons(v3.clone(), List::cons(Str::lit("world"), List::<Str>::nil())));
                    let a55 = Str::lit(", ");
                    join(a54, a55)
                }, List::<Str>::nil()))), List::<Str>::nil())))));
                let a67 = {
                    let a64 = if f36(f9(List::cons(Str::lit("."), List::cons(v3.clone(), List::<Str>::nil()))), Str::lit(".identity")) {
                        let a56 = v5;
                        let a57 = v6;
                        append_list(a56, a57)
                    } else {
                        let a58 = v6;
                        let a59 = v5;
                        append_list(a58, a59)
                    };
                    let a65 = {
                        let a62 = List::cons(Str::lit(""), List::<Str>::nil());
                        let a63 = f32(Str::lit("__all__ = "), f37(f35({
                            let a60 = v4;
                            let a61 = List::cons(Str::lit("Identity"), List::cons(v3, List::cons(Str::lit("world"), List::<Str>::nil())));
                            append_list(a60, a61)
                        }))?, Str::lit("["), Str::lit("]"));
                        append_list(a62, a63)
                    };
                    append_list(a64, a65)
                };
                append_list(a66, a67)
            };
            append_list(a68, a69)
        };
        append_list(a70, a71)
    }))
}

pub fn f16(v0: Str, v1: Adt1, v2: Adt4) -> Str {
    f33(List::cons(f34(Str::lit("#"), v0, f9(List::cons(match v2 {
        Adt4::C0(h72, _, _, _, _, _, _, _) => {
            h72
        }
    }, List::cons(Str::lit("'s model and the WIT world "), List::cons(match v1 {
        Adt1::C0(h73, _, _, _, _, _, _, _, _) => {
            h73
        }
    }, List::<Str>::nil()))))), List::cons(Str::lit("\"\"\"What a set of bindings was generated from.\"\"\"\u{a}\u{a}from typing import NamedTuple\u{a}\u{a}\u{a}class Identity(NamedTuple):\u{a}    \"\"\"A name and both digests, as prismpm.lock records them.\"\"\"\u{a}\u{a}    name: str\u{a}    \"\"\"The world's contract, or the application's name.\"\"\"\u{a}    blake3: str\u{a}    \"\"\"The BLAKE3 digest, lowercase hex.\"\"\"\u{a}    sha256: str\u{a}    \"\"\"The SHA-256 digest, lowercase hex.\"\"\""), List::<Str>::nil())))
}

pub fn f17(v0: Str, v1: Adt1) -> R<Str> {
    Ok(f33({
        let a89 = List::cons(f34(Str::lit("#"), v0, f38(v1.clone())), List::cons(f9(List::cons(Str::lit("\"\"\"The world "), List::cons(match v1.clone() {
            Adt1::C0(h74, _, _, _, _, _, _, _, _) => {
                h74
            }
        }, List::cons(Str::lit(": its identity, its exports, and how a host sends it a request.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit("\u{a}from typing import Protocol\u{a}\u{a}from .identity import Identity\u{a}\u{a}WORLD = Identity("), List::cons(f9(List::cons(Str::lit("    name="), List::cons(f22(match v1.clone() {
            Adt1::C0(h75, _, _, _, _, _, _, _, _) => {
                h75
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    blake3="), List::cons(f22(match v1.clone() {
            Adt1::C0(_, _, _, _, h76, _, _, _, _) => {
                h76
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    sha256="), List::cons(f22(match v1.clone() {
            Adt1::C0(_, _, _, _, _, h77, _, _, _) => {
                h77
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit(")\u{a}\"\"\"The world these bindings were generated from. It equals `inputs.world` in prismpm.lock.\"\"\"\u{a}"), List::cons(f9(List::cons(Str::lit("EXPORTS = ("), List::cons({
            let a80 = f37(f35(List::cons(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, h78, _, _) => {
                    h78
                }
            }, f39(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, _, _, h79) => {
                    h79
                }
            }))))?;
            let a81 = Str::lit(", ");
            join(a80, a81)
        }, List::cons(Str::lit(")"), List::<Str>::nil())))), List::cons(Str::lit("\"\"\"Every export the world requires, sorted. A guest that offers any other is not of this world.\"\"\"\u{a}\u{a}\u{a}class Guest(Protocol):"), List::cons(f9(List::cons(Str::lit("    \"\"\"A guest of "), List::cons(match v1.clone() {
            Adt1::C0(h82, _, _, _, _, _, _, _, _) => {
                h82
            }
        }, List::cons(Str::lit(", instantiated by the host's WebAssembly engine."), List::<Str>::nil())))), List::cons(Str::lit(""), List::cons(f9(List::cons(Str::lit("    The guest imports nothing. Implement this over the engine's instance: each export, and `"), List::cons(match v1.clone() {
            Adt1::C0(_, _, _, _, _, _, h83, _, _) => {
                h83
            }
        }, List::cons(Str::lit("` as a read and a write."), List::<Str>::nil())))), List::cons(Str::lit("    A failed call into the guest, such as a trap, raises the engine's own exception.\u{a}    \"\"\""), List::<Str>::nil())))))))))))));
        let a90 = {
            let a87 = f40(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, _, _, h84) => {
                    h84
                }
            })?;
            let a88 = List::cons(Str::lit("\u{a}    def write_memory(self, offset: int, data: bytes) -> None:"), List::cons(f9(List::cons(Str::lit("        \"\"\"Copy `data` into the guest's `"), List::cons(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, h85, _, _) => {
                    h85
                }
            }, List::cons(Str::lit("` at `offset`.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit("        ...\u{a}\u{a}    def read_memory(self, offset: int, length: int) -> bytes:"), List::cons(f9(List::cons(Str::lit("        \"\"\"Copy `length` bytes out of the guest's `"), List::cons(match v1 {
                Adt1::C0(_, _, _, _, _, _, h86, _, _) => {
                    h86
                }
            }, List::cons(Str::lit("` at `offset`. Raises when they are outside it.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit("        ...\u{a}\u{a}"), List::cons(f41(), List::<Str>::nil()))))));
            append_list(a87, a88)
        };
        append_list(a89, a90)
    }))
}

pub fn f18(v0: Str, v1: Adt4) -> R<Str> {
    let v2: Str = match v1.clone() {
        Adt4::C0(h91, _, _, _, _, _, _, _) => {
            h91
        }
    };
    Ok(f33({
        let a100 = List::cons(f34(Str::lit("#"), v0, f42(v1.clone())), List::cons(f9(List::cons(Str::lit("\"\"\"The client of "), List::cons(v2.clone(), List::cons(Str::lit(", over request and response bytes.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit("\u{a}from typing import Callable\u{a}\u{a}from .identity import Identity\u{a}\u{a}MODEL = Identity("), List::cons(f9(List::cons(Str::lit("    name="), List::cons(f22(match v1.clone() {
            Adt4::C0(h92, _, _, _, _, _, _, _) => {
                h92
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    blake3="), List::cons(f22(match v1.clone() {
            Adt4::C0(_, _, _, _, _, h93, _, _) => {
                h93
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    sha256="), List::cons(f22(match v1.clone() {
            Adt4::C0(_, _, _, _, _, _, h94, _) => {
                h94
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit(")\u{a}\"\"\"The model this client was generated from. It equals `inputs.model` in prismpm.lock.\"\"\"\u{a}\u{a}"), List::cons(f9(List::cons(Str::lit("class "), List::cons(match v1.clone() {
            Adt4::C0(_, h95, _, _, _, _, _, _) => {
                h95
            }
        }, List::cons(Str::lit(":"), List::<Str>::nil())))), List::<Str>::nil()))))))));
        let a101 = {
            let a98 = f43(Str::lit("    "), {
                let a96 = f44(v1, f9(List::cons(Str::lit("A client for "), List::cons(v2.clone(), List::cons(Str::lit(", over request and response bytes."), List::<Str>::nil())))));
                let a97 = List::cons(Str::lit(""), List::cons(f9(List::cons(v2, List::cons(Str::lit("'s model describes no typed operations, so the client sends bytes."), List::<Str>::nil()))), List::<Str>::nil()));
                append_list(a96, a97)
            })?;
            let a99 = List::cons(Str::lit("\u{a}    def __init__(self, dispatch: Callable[[bytes], bytes]) -> None:\u{a}        \"\"\"A client over this dispatch function.\"\"\"\u{a}        self._dispatch = dispatch\u{a}\u{a}    def invoke(self, request: bytes) -> bytes:\u{a}        \"\"\"Send one request and return the response bytes.\"\"\"\u{a}        return bytes(self._dispatch(bytes(request)))"), List::<Str>::nil());
            append_list(a98, a99)
        };
        append_list(a100, a101)
    }))
}

pub fn f19(v0: Str, v1: Adt4, v2: Adt6) -> R<Str> {
    let v3: Str = match v1.clone() {
        Adt4::C0(h102, _, _, _, _, _, _, _) => {
            h102
        }
    };
    let v4: Str = match v2.clone() {
        Adt6::C0(h103, _, _, _, _, _) => {
            h103
        }
    };
    let v5: Str = match v2.clone() {
        Adt6::C0(_, _, _, h104, _, _) => {
            h104
        }
    };
    let v6: Str = f9(List::cons(v5.clone(), List::cons(Str::lit("Code"), List::<Str>::nil())));
    let v7: Str = match v2.clone() {
        Adt6::C0(_, _, h105, _, _, _) => {
            h105
        }
    };
    let v8: Str = f4(match v2.clone() {
        Adt6::C0(_, h106, _, _, _, _) => {
            h106
        }
    })?;
    Ok(f33({
        let a135 = List::cons(f34(Str::lit("#"), v0, f42(v1.clone())), List::cons(f9(List::cons(Str::lit("\"\"\"The client of "), List::cons(v3.clone(), List::cons(Str::lit(": one method per operation, over any dispatch function.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit("\u{a}from enum import Enum\u{a}from typing import Callable, Optional\u{a}\u{a}from .identity import Identity\u{a}\u{a}MODEL = Identity("), List::cons(f9(List::cons(Str::lit("    name="), List::cons(f22(match v1.clone() {
            Adt4::C0(h107, _, _, _, _, _, _, _) => {
                h107
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    blake3="), List::cons(f22(match v1.clone() {
            Adt4::C0(_, _, _, _, _, h108, _, _) => {
                h108
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    sha256="), List::cons(f22(match v1.clone() {
            Adt4::C0(_, _, _, _, _, _, h109, _) => {
                h109
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit(")\u{a}\"\"\"The model this client was generated from. It equals `inputs.model` in prismpm.lock.\"\"\"\u{a}"), List::cons(f9(List::cons(Str::lit("APPLICATION = "), List::cons(f22(v3.clone())?, List::<Str>::nil()))), List::cons(f9(List::cons(Str::lit("VERSION = "), List::cons(f22(v4.clone())?, List::<Str>::nil()))), List::cons(Str::lit("MINIMUM = -(2**63)\u{a}MAXIMUM = 2**63 - 1\u{a}\u{a}"), List::cons(f9(List::cons(Str::lit("class "), List::cons(v7.clone(), List::cons(Str::lit("(str, Enum):"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    \"\"\"An operation "), List::cons(v3.clone(), List::cons(Str::lit(" accepts; its value is the name the guest reads in a request.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit(""), List::<Str>::nil())))))))))))));
        let a136 = {
            let a133 = f45(match v2.clone() {
                Adt6::C0(_, _, _, _, h110, _) => {
                    h110
                }
            })?;
            let a134 = {
                let a131 = List::cons(Str::lit("\u{a}"), List::cons(f9(List::cons(Str::lit("class "), List::cons(v6.clone(), List::cons(Str::lit("(str, Enum):"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    \"\"\"An answer with which "), List::cons(v3.clone(), List::cons(Str::lit(" refuses a request; its value is the code as the guest wrote it.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit(""), List::<Str>::nil()))));
                let a132 = {
                    let a129 = f46(v1.clone(), v4.clone(), match v2.clone() {
                        Adt6::C0(_, _, _, _, _, h111) => {
                            h111
                        }
                    })?;
                    let a130 = {
                        let a127 = List::cons(Str::lit("\u{a}    @property\u{a}    def message(self) -> str:\u{a}        \"\"\"What the model says this answer means, for a person.\"\"\"\u{a}        return _MESSAGES[self]\u{a}\u{a}\u{a}_MESSAGES = {"), List::<Str>::nil());
                        let a128 = {
                            let a125 = f47(v1.clone(), v4, v6.clone(), match v2.clone() {
                                Adt6::C0(_, _, _, _, _, h112) => {
                                    h112
                                }
                            })?;
                            let a126 = {
                                let a123 = List::cons(Str::lit("}\u{a}\u{a}"), List::cons(f9(List::cons(Str::lit("class "), List::cons(v5.clone(), List::cons(Str::lit("(Exception):"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("    \"\"\""), List::cons(v3.clone(), List::cons(Str::lit(" refused a request, for example on overflow.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit(""), List::cons(f9(List::cons(Str::lit("    def __init__(self, code: str, known: Optional["), List::cons(v6.clone(), List::cons(Str::lit("]) -> None:"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("        message = known.message if known is not None else "), List::cons(f22(f9(List::cons(v3.clone(), List::cons(Str::lit(" refused the request."), List::<Str>::nil()))))?, List::<Str>::nil()))), List::cons(Str::lit("        super().__init__(f\"{message} ({code})\")\u{a}        self.code = code\u{a}        \"\"\"The code as the guest wrote it.\"\"\"\u{a}        self.known = known\u{a}        \"\"\"The code, when this client was generated with it; otherwise None.\"\"\"\u{a}        self.message = message\u{a}        \"\"\"What the model says the code means, for a person.\"\"\"\u{a}\u{a}\u{a}class ResponseError(Exception):"), List::cons(f9(List::cons(Str::lit("    \"\"\""), List::cons(v3.clone(), List::cons(Str::lit(" sent an answer this client cannot read.\"\"\""), List::<Str>::nil())))), List::cons(Str::lit("\u{a}"), List::cons(f9(List::cons(Str::lit("class "), List::cons(match v1.clone() {
                                    Adt4::C0(_, h113, _, _, _, _, _, _) => {
                                        h113
                                    }
                                }, List::cons(Str::lit(":"), List::<Str>::nil())))), List::<Str>::nil()))))))))));
                                let a124 = {
                                    let a121 = f43(Str::lit("    "), {
                                        let a114 = f44(v1, f9(List::cons(Str::lit("A client for "), List::cons(v3.clone(), List::cons(Str::lit("."), List::<Str>::nil())))));
                                        let a115 = List::cons(Str::lit(""), List::cons(Str::lit("`dispatch` sends request bytes to the guest and returns its response bytes, for"), List::cons(Str::lit("example `lambda request: world.dispatch(fresh_instance(), request)`."), List::<Str>::nil())));
                                        append_list(a114, a115)
                                    })?;
                                    let a122 = {
                                        let a119 = List::cons(Str::lit("\u{a}    def __init__(self, dispatch: Callable[[bytes], bytes]) -> None:\u{a}        \"\"\"A client over this dispatch function.\"\"\"\u{a}        self._dispatch = dispatch"), List::<Str>::nil());
                                        let a120 = {
                                            let a117 = f48(v8.clone(), v7.clone(), match v2 {
                                                Adt6::C0(_, _, _, _, h116, _) => {
                                                    h116
                                                }
                                            })?;
                                            let a118 = List::cons(Str::lit(""), List::cons(f9(List::cons(Str::lit("    def "), List::cons(v8, List::cons(Str::lit("(self, operation: "), List::cons(v7.clone(), List::cons(Str::lit(", left: int, right: int) -> int:"), List::<Str>::nil())))))), List::cons(Str::lit("        \"\"\"Apply `operation` to two signed 64-bit integers.\u{a}"), List::cons(f9(List::cons(Str::lit("        A refusal by "), List::cons(v3, List::cons(Str::lit(" raises `"), List::cons(v5.clone(), List::cons(Str::lit("`; an operand outside"), List::<Str>::nil())))))), List::cons(Str::lit("        the signed 64-bit range raises ValueError before anything is sent.\u{a}        \"\"\""), List::cons(f9(List::cons(Str::lit("        operation = "), List::cons(v7, List::cons(Str::lit("(operation)"), List::<Str>::nil())))), List::cons(Str::lit("        for operand in (left, right):\u{a}            if isinstance(operand, bool) or not isinstance(operand, int):\u{a}                raise TypeError(f\"{operand!r} is not an int\")\u{a}            if not MINIMUM <= operand <= MAXIMUM:\u{a}                raise ValueError(f\"{operand} is outside the signed 64-bit range\")\u{a}        request = f\"{VERSION}\\t{operation.value}\\t{left}\\t{right}\"\u{a}        return _read(bytes(self._dispatch(request.encode(\"utf-8\"))))\u{a}\u{a}\u{a}def _read(response: bytes) -> int:\u{a}    \"\"\"Read a response: the request version, then `ok` and a value or `error` and a code.\"\"\"\u{a}    try:\u{a}        text = response.decode(\"utf-8\")\u{a}    except UnicodeDecodeError:\u{a}        raise ResponseError(\"it is not UTF-8\") from None\u{a}    fields = text.split(\"\\t\")\u{a}    if len(fields) > 3:\u{a}        raise ResponseError(f\"{text!r} has more than three fields\")\u{a}    if fields[0] != VERSION:"), List::cons(f9(List::cons(Str::lit("        refused = "), List::cons(v6.clone(), List::cons(Str::lit(".UNSUPPORTED_VERSION"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("        raise "), List::cons(v5.clone(), List::cons(Str::lit("(refused.value, refused)"), List::<Str>::nil())))), List::cons(Str::lit("    if len(fields) == 3 and fields[1] == \"ok\":\u{a}        value = fields[2]\u{a}        if not value.lstrip(\"-\").isdigit() or not MINIMUM <= int(value) <= MAXIMUM:\u{a}            raise ResponseError(f\"{value!r} is not a signed 64-bit integer\")\u{a}        return int(value)\u{a}    if len(fields) == 3 and fields[1] == \"error\":"), List::cons(f9(List::cons(Str::lit("        known = next((code for code in "), List::cons(v6, List::cons(Str::lit(" if code.value == fields[2]), None)"), List::<Str>::nil())))), List::cons(f9(List::cons(Str::lit("        raise "), List::cons(v5, List::cons(Str::lit("(fields[2], known)"), List::<Str>::nil())))), List::cons(Str::lit("    raise ResponseError(f\"{text!r} is neither ok nor error\")"), List::<Str>::nil())))))))))))));
                                            append_list(a117, a118)
                                        };
                                        append_list(a119, a120)
                                    };
                                    append_list(a121, a122)
                                };
                                append_list(a123, a124)
                            };
                            append_list(a125, a126)
                        };
                        append_list(a127, a128)
                    };
                    append_list(a129, a130)
                };
                append_list(a131, a132)
            };
            append_list(a133, a134)
        };
        append_list(a135, a136)
    }))
}

pub fn f20(v0: Str) -> Str {
    if f8(v0.clone(), f49()) {
        f9(List::cons(v0, List::cons(Str::lit("_"), List::<Str>::nil())))
    } else {
        v0
    }
}

pub fn f21(v0: Str) -> R<bool> {
    let v1: Bytes = {
        let a137 = v0;
        utf8_encode(a137)
    };
    let a141 = {
        let a139 = {
            let a138 = v1.clone();
            length_bytes(a138)
        };
        let a140 = 0u64;
        nat_eq(a139, a140)
    };
    if bool_not(a141) {
        let a142 = f50(f51(v1.clone(), 0u64, 1u64), f52(), 10u64);
        if bool_not(a142) {
            f53(v1.clone(), 0u64, {
                let a143 = v1;
                length_bytes(a143)
            })
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub fn f22(v0: Str) -> R<Str> {
    f54(v0, Bytes::lit(&[0x22]))
}

pub fn f23(v0: Bytes, v1: Str) -> Str {
    let m145 = {
        let a144 = v0;
        utf8_decode(a144)
    };
    match m145 {
        None => {
            v1
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f24(v0: Bytes, v1: u64, v2: bool, v3: u64) -> R<Bytes> {
    let m146 = v3;
    if m146 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m146 - 1;
        let v5: Bytes = f51(v0.clone(), v1, 1u64);
        if f55(v5.clone(), Bytes::lit(&[0x2d])) {
            let a149 = Bytes::lit(&[0x5f]);
            let a150 = f24(v0, {
                let a147 = v1;
                let a148 = 1u64;
                nat_add(a147, a148)?
            }, false, v4)?;
            Ok(append_bytes(a149, a150))
        } else {
            if f56(v5.clone()) {
                let a155 = if v2 {
                    let a151 = Bytes::lit(&[0x5f]);
                    let a152 = f57(v5);
                    append_bytes(a151, a152)
                } else {
                    f57(v5)
                };
                let a156 = f24(v0, {
                    let a153 = v1;
                    let a154 = 1u64;
                    nat_add(a153, a154)?
                }, false, v4)?;
                Ok(append_bytes(a155, a156))
            } else {
                let a159 = v5.clone();
                let a160 = f24(v0, {
                    let a157 = v1;
                    let a158 = 1u64;
                    nat_add(a157, a158)?
                }, f58(v5), v4)?;
                Ok(append_bytes(a159, a160))
            }
        }
    }
}

pub fn f25(v0: Str, v1: List<Adt7>) -> R<Option<(Str, Str)>> {
    let m161 = v1;
    match m161.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v2, v3)) => {
            Ok(f5(f6(v0.clone(), f7(match v2 {
                Adt7::C0(h162, _, _) => {
                    h162
                }
            })?)?, f25(v0, v3)?))
        }
    }
}

pub fn f26(v0: Str, p168: List<Adt7>) -> R<Option<(Str, Str)>> {
    let m167 = p168;
    {
        let mut m167 = m167;
        loop {
            let step = {
                let v1 = m167;
                let m163 = v1;
                match m163.uncons() {
                    None => {
                        Err::<List<Adt7>, R<Option<(Str, Str)>>>(Ok(None::<(Str, Str)>))
                    }
                    Some((v2, v3)) => {
                        let v4: Str = f4(match v2.clone() {
                            Adt7::C0(h164, _, _) => {
                                h164
                            }
                        })?;
                        if if f8(v4.clone(), List::cons(v0.clone(), List::cons(Str::lit("dispatch"), List::<Str>::nil()))) {
                            true
                        } else {
                            f55(f51({
                                let a165 = v4.clone();
                                utf8_encode(a165)
                            }, 0u64, 1u64), Bytes::lit(&[0x5f]))
                        } {
                            Err::<List<Adt7>, R<Option<(Str, Str)>>>(Ok(f59(match v2 {
                                Adt7::C0(h166, _, _) => {
                                    h166
                                }
                            }, v4)))
                        } else {
                            Ok::<List<Adt7>, R<Option<(Str, Str)>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m167 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f27(v0: Adt4, v1: Adt6) -> Option<(Str, Str)> {
    if f60(Str::lit("unsupported-version"), match v1 {
        Adt6::C0(_, _, _, _, _, h169) => {
            h169
        }
    }) {
        None::<(Str, Str)>
    } else {
        Some((Str::lit("generate.model"), f9(List::cons(match v0 {
            Adt4::C0(h170, _, _, _, _, _, _, _) => {
                h170
            }
        }, List::cons(Str::lit("'s model has no unsupported-version error"), List::<Str>::nil())))))
    }
}

pub fn f28(v0: Str, v1: bool, v2: List<(Str, Str)>, v3: List<Adt3>) -> R<Option<(Str, Str)>> {
    let m171 = v3;
    match m171.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v4, v5)) => {
            Ok(f5(if v1 {
                f6(v0.clone(), f7(match v4.clone() {
                    Adt3::C0(h172, _) => {
                        h172
                    }
                })?)?
            } else {
                None::<(Str, Str)>
            }, f5(f29(v0.clone(), match v4 {
                Adt3::C0(_, h173) => {
                    h173
                }
            }, v2.clone()), f28(v0, v1, v2, v5)?)))
        }
    }
}

pub fn f29(v0: Str, v1: Str, v2: List<(Str, Str)>) -> Option<(Str, Str)> {
    let m174 = f61(v1.clone(), v2);
    match m174 {
        None => {
            Some((Str::lit("generate.world"), f9(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer has no type for WIT "), List::cons(v1, List::<Str>::nil())))))))
        }
        Some(_) => {
            None::<(Str, Str)>
        }
    }
}

pub fn f30(v0: List<Str>) -> R<List<Str>> {
    let a175 = f35(f62(true, v0.clone())?);
    let a176 = f35(f62(false, v0)?);
    Ok(append_list(a175, a176))
}

pub fn f31(v0: Adt4) -> List<Str> {
    let m178 = match v0.clone() {
        Adt4::C0(_, _, _, _, _, _, _, h177) => {
            h177
        }
    };
    match m178 {
        Adt5::C0 => {
            List::cons(match v0 {
                Adt4::C0(_, h179, _, _, _, _, _, _) => {
                    h179
                }
            }, List::cons(Str::lit("MODEL"), List::<Str>::nil()))
        }
        Adt5::C1(v1) => {
            List::cons(match v0 {
                Adt4::C0(_, h180, _, _, _, _, _, _) => {
                    h180
                }
            }, List::cons(match v1.clone() {
                Adt6::C0(_, _, h181, _, _, _) => {
                    h181
                }
            }, List::cons(f9(List::cons(match v1.clone() {
                Adt6::C0(_, _, _, h182, _, _) => {
                    h182
                }
            }, List::cons(Str::lit("Code"), List::<Str>::nil()))), List::cons(match v1 {
                Adt6::C0(_, _, _, h183, _, _) => {
                    h183
                }
            }, List::cons(Str::lit("ResponseError"), List::cons(Str::lit("MODEL"), List::<Str>::nil()))))))
        }
    }
}

pub fn f32(v0: Str, v1: List<Str>, v2: Str, v3: Str) -> List<Str> {
    let v4: Str = {
        let a184 = v2.clone();
        let a185 = Str::lit("(");
        if equal(a184, a185) {
            f9(List::cons(v0.clone(), List::cons({
                let a186 = v1.clone();
                let a187 = Str::lit(", ");
                join(a186, a187)
            }, List::<Str>::nil())))
        } else {
            f9(List::cons(v0.clone(), List::cons(v2.clone(), List::cons({
                let a188 = v1.clone();
                let a189 = Str::lit(", ");
                join(a188, a189)
            }, List::cons(v3.clone(), List::<Str>::nil())))))
        }
    };
    if f63(v4.clone()) {
        List::cons(v4, List::<Str>::nil())
    } else {
        let a190 = List::cons(f9(List::cons(v0, List::cons(v2, List::<Str>::nil()))), f64(Str::lit("    "), v1, Str::lit(",")));
        let a191 = List::cons(v3, List::<Str>::nil());
        append_list(a190, a191)
    }
}

pub fn f33(v0: List<Str>) -> Str {
    let a194 = {
        let a192 = v0;
        let a193 = List::cons(Str::lit(""), List::<Str>::nil());
        append_list(a192, a193)
    };
    let a195 = Str::lit("\u{a}");
    join(a194, a195)
}

pub fn f34(v0: Str, v1: Str, v2: Str) -> Str {
    f9(List::cons(v0.clone(), List::cons(Str::lit(" Generated by `"), List::cons(v1.clone(), List::cons(Str::lit("` from "), List::cons(v2, List::cons(Str::lit(".\u{a}"), List::cons(v0, List::cons(Str::lit(" Do not edit: run `"), List::cons(v1, List::cons(Str::lit("` again.\u{a}"), List::<Str>::nil())))))))))))
}

pub fn f35(v0: List<Str>) -> List<Str> {
    let m196 = v0;
    match m196.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            f65(v1, f35(v2))
        }
    }
}

pub fn f36(v0: Str, v1: Str) -> bool {
    let a203 = {
        let a199 = {
            let a197 = v0;
            utf8_encode(a197)
        };
        let a200 = {
            let a198 = v1;
            utf8_encode(a198)
        };
        compare_bytes(a199, a200)
    };
    let a204 = {
        let a201 = Bytes::lit(&[0x00]);
        let a202 = Bytes::lit(&[0x01]);
        compare_bytes(a201, a202)
    };
    equal(a203, a204)
}

pub fn f37(v0: List<Str>) -> R<List<Str>> {
    let m205 = v0;
    match m205.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f22(v1)?, f37(v2)?))
        }
    }
}

pub fn f38(v0: Adt1) -> Str {
    f9(List::cons(Str::lit("the WIT world "), List::cons(match v0.clone() {
        Adt1::C0(h206, _, _, _, _, _, _, _, _) => {
            h206
        }
    }, List::cons(Str::lit(" ("), List::cons(match v0 {
        Adt1::C0(_, _, _, h207, _, _, _, _, _) => {
            h207
        }
    }, List::cons(Str::lit(")"), List::<Str>::nil()))))))
}

pub fn f39(v0: List<Adt2>) -> List<Str> {
    let m208 = v0;
    match m208.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt2::C0(h209, _, _, _) => {
                    h209
                }
            }, f39(v2))
        }
    }
}

pub fn f40(v0: List<Adt2>) -> R<List<Str>> {
    let m210 = v0;
    match m210.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a221 = List::cons(Str::lit(""), List::cons(f9(List::cons(Str::lit("    def "), List::cons(f20(match v1.clone() {
                Adt2::C0(h211, _, _, _) => {
                    h211
                }
            }), List::cons(Str::lit("(self"), List::cons(f66(match v1.clone() {
                Adt2::C0(_, h212, _, _) => {
                    h212
                }
            })?, List::cons(Str::lit(") -> "), List::cons({
                let m214 = match v1.clone() {
                    Adt2::C0(_, _, h213, _) => {
                        h213
                    }
                };
                match m214 {
                    None => {
                        Str::lit("None")
                    }
                    Some(v3) => {
                        f67(v3)
                    }
                }
            }, List::cons(Str::lit(":"), List::<Str>::nil())))))))), List::<Str>::nil()));
            let a222 = {
                let a219 = f43(Str::lit("        "), {
                    let m216 = match v1.clone() {
                        Adt2::C0(_, _, _, h215) => {
                            h215
                        }
                    };
                    match m216.uncons() {
                        None => {
                            List::cons(f9(List::cons(Str::lit("The export "), List::cons(match v1 {
                                Adt2::C0(h217, _, _, _) => {
                                    h217
                                }
                            }, List::cons(Str::lit("."), List::<Str>::nil())))), List::<Str>::nil())
                        }
                        Some((_, _)) => {
                            match v1 {
                                Adt2::C0(_, _, _, h218) => {
                                    h218
                                }
                            }
                        }
                    }
                })?;
                let a220 = List::cons(Str::lit("        ..."), f40(v2)?);
                append_list(a219, a220)
            };
            Ok(append_list(a221, a222))
        }
    }
}

pub fn f41() -> Str {
    Str::lit("class DispatchError(Exception):\u{a}    \"\"\"A request did not come back from the guest.\"\"\"\u{a}\u{a}\u{a}def dispatch(guest: Guest, request: bytes) -> bytes:\u{a}    \"\"\"Send one request to a fresh instance of the guest and return a copy of its response.\u{a}\u{a}    The guest allocator never frees, so give every request its own instance.\u{a}    \"\"\"\u{a}    if len(request) > 0xFFFF_FFFF:\u{a}        raise DispatchError(f\"a request of {len(request)} bytes is too long\")\u{a}    offset = guest.holo_alloc(len(request))\u{a}    guest.write_memory(offset, bytes(request))\u{a}    packed = guest.holo_run(offset, len(request))\u{a}    # The response location is packed as `offset << 32 | length`.\u{a}    return bytes(guest.read_memory(packed >> 32, packed & 0xFFFF_FFFF))")
}

pub fn f42(v0: Adt4) -> Str {
    f9(List::cons(match v0.clone() {
        Adt4::C0(h223, _, _, _, _, _, _, _) => {
            h223
        }
    }, List::cons(Str::lit("'s model ("), List::cons(match v0.clone() {
        Adt4::C0(_, _, _, _, h224, _, _, _) => {
            h224
        }
    }, List::cons(Str::lit(", blake3:"), List::cons(match v0 {
        Adt4::C0(_, _, _, _, _, h225, _, _) => {
            h225
        }
    }, List::cons(Str::lit(")"), List::<Str>::nil())))))))
}

pub fn f43(v0: Str, v1: List<Str>) -> R<List<Str>> {
    let m226 = v1;
    match m226.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let m227 = v3.clone();
            match m227.uncons() {
                None => {
                    Ok(List::cons(f9(List::cons(v0, List::cons(Str::lit("\"\"\""), List::cons(f68(v2)?, List::cons(Str::lit("\"\"\""), List::<Str>::nil()))))), List::<Str>::nil()))
                }
                Some((_, _)) => {
                    Ok(List::cons(f9(List::cons(v0.clone(), List::cons(Str::lit("\"\"\""), List::cons(f68(v2)?, List::<Str>::nil())))), {
                        let a228 = f69(v0.clone(), v3)?;
                        let a229 = List::cons(f9(List::cons(v0, List::cons(Str::lit("\"\"\""), List::<Str>::nil()))), List::<Str>::nil());
                        append_list(a228, a229)
                    }))
                }
            }
        }
    }
}

pub fn f44(v0: Adt4, v1: Str) -> List<Str> {
    let m231 = match v0 {
        Adt4::C0(_, _, _, h230, _, _, _, _) => {
            h230
        }
    };
    match m231 {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some(v2) => {
            List::cons(v1, List::cons(Str::lit(""), List::cons(v2, List::<Str>::nil())))
        }
    }
}

pub fn f45(v0: List<Adt7>) -> R<List<Str>> {
    let m232 = v0;
    match m232.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f9(List::cons(Str::lit("    "), List::cons(f70(match v1.clone() {
                Adt7::C0(h233, _, _) => {
                    h233
                }
            })?, List::cons(Str::lit(" = "), List::cons(f22(match v1.clone() {
                Adt7::C0(h234, _, _) => {
                    h234
                }
            })?, List::cons(Str::lit("\u{a}    \"\"\""), List::cons(match v1 {
                Adt7::C0(_, h235, _) => {
                    h235
                }
            }, List::cons(Str::lit(".\"\"\""), List::<Str>::nil())))))))), f45(v2)?))
        }
    }
}

pub fn f46(v0: Adt4, v1: Str, v2: List<Adt8>) -> R<List<Str>> {
    let m236 = v2;
    match m236.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v3, v4)) => {
            Ok(List::cons(f9(List::cons(Str::lit("    "), List::cons(f70(match v3.clone() {
                Adt8::C0(h237, _) => {
                    h237
                }
            })?, List::cons(Str::lit(" = "), List::cons(f22(match v3.clone() {
                Adt8::C0(h238, _) => {
                    h238
                }
            })?, List::cons(Str::lit("\u{a}    \"\"\""), List::cons(f71(f72(v0.clone(), v1.clone(), v3))?, List::cons(Str::lit("\"\"\""), List::<Str>::nil())))))))), f46(v0, v1, v4)?))
        }
    }
}

pub fn f47(v0: Adt4, v1: Str, v2: Str, v3: List<Adt8>) -> R<List<Str>> {
    let m239 = v3;
    match m239.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v4, v5)) => {
            Ok(List::cons(f9(List::cons(Str::lit("    "), List::cons(v2.clone(), List::cons(Str::lit("."), List::cons(f70(match v4.clone() {
                Adt8::C0(h240, _) => {
                    h240
                }
            })?, List::cons(Str::lit(": "), List::cons(f22(f72(v0.clone(), v1.clone(), v4))?, List::cons(Str::lit(","), List::<Str>::nil())))))))), f47(v0, v1, v2, v5)?))
        }
    }
}

pub fn f48(v0: Str, v1: Str, v2: List<Adt7>) -> R<List<Str>> {
    let m241 = v2;
    match m241.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v3, v4)) => {
            let v5: Str = f70(match v3.clone() {
                Adt7::C0(h242, _, _) => {
                    h242
                }
            })?;
            Ok(List::cons(f9(List::cons(Str::lit("\u{a}    def "), List::cons(f4(match v3.clone() {
                Adt7::C0(h243, _, _) => {
                    h243
                }
            })?, List::cons(Str::lit("(self, left: int, right: int) -> int:\u{a}        \"\"\""), List::cons(match v3 {
                Adt7::C0(_, h244, _) => {
                    h244
                }
            }, List::cons(Str::lit(": `"), List::cons(v0.clone(), List::cons(Str::lit("("), List::cons(v1.clone(), List::cons(Str::lit("."), List::cons(v5.clone(), List::cons(Str::lit(", left, right)`.\"\"\"\u{a}        return self."), List::cons(v0.clone(), List::cons(Str::lit("("), List::cons(v1.clone(), List::cons(Str::lit("."), List::cons(v5, List::cons(Str::lit(", left, right)"), List::<Str>::nil())))))))))))))))))), f48(v0, v1, v4)?))
        }
    }
}

pub fn f49() -> List<Str> {
    List::cons(Str::lit("False"), List::cons(Str::lit("None"), List::cons(Str::lit("True"), List::cons(Str::lit("and"), List::cons(Str::lit("as"), List::cons(Str::lit("assert"), List::cons(Str::lit("async"), List::cons(Str::lit("await"), List::cons(Str::lit("break"), List::cons(Str::lit("class"), List::cons(Str::lit("continue"), List::cons(Str::lit("def"), List::cons(Str::lit("del"), List::cons(Str::lit("elif"), List::cons(Str::lit("else"), List::cons(Str::lit("except"), List::cons(Str::lit("finally"), List::cons(Str::lit("for"), List::cons(Str::lit("from"), List::cons(Str::lit("global"), List::cons(Str::lit("if"), List::cons(Str::lit("import"), List::cons(Str::lit("in"), List::cons(Str::lit("is"), List::cons(Str::lit("lambda"), List::cons(Str::lit("nonlocal"), List::cons(Str::lit("not"), List::cons(Str::lit("or"), List::cons(Str::lit("pass"), List::cons(Str::lit("raise"), List::cons(Str::lit("return"), List::cons(Str::lit("try"), List::cons(Str::lit("while"), List::cons(Str::lit("with"), List::cons(Str::lit("yield"), List::cons(Str::lit("match"), List::cons(Str::lit("case"), List::cons(Str::lit("type"), List::<Str>::nil()))))))))))))))))))))))))))))))))))))))
}

pub fn f50(v0: Bytes, v1: Bytes, p247: u64) -> bool {
    let m246 = p247;
    {
        let mut m246 = m246;
        loop {
            let step = {
                let v2 = m246;
                let m245 = v2;
                if m245 == 0 {
                    Err::<u64, bool>(false)
                } else {
                    let v3 = m245 - 1;
                    if f55(f51(v1.clone(), v3, 1u64), v0.clone()) {
                        Err::<u64, bool>(true)
                    } else {
                        Ok::<u64, bool>(v3)
                    }
                }
            };
            match step {
                Ok(next) => { m246 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f51(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m251 = {
        let a248 = v0;
        let a249 = v1;
        let a250 = v2;
        slice_bytes(a248, a249, a250)
    };
    match m251 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f52() -> Bytes {
    Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39])
}

pub fn f53(v0: Bytes, p256: u64, p257: u64) -> R<bool> {
    let m255 = (p256, p257);
    {
        let mut m255 = m255;
        loop {
            let step = {
                let (v1, v2) = m255;
                let m252 = v2;
                if m252 == 0 {
                    Err::<(u64, u64), R<bool>>(Ok(true))
                } else {
                    let v3 = m252 - 1;
                    if if f58(f51(v0.clone(), v1, 1u64)) {
                        true
                    } else {
                        f55(f51(v0.clone(), v1, 1u64), Bytes::lit(&[0x5f]))
                    } {
                        Ok::<(u64, u64), R<bool>>(({
                            let a253 = v1;
                            let a254 = 1u64;
                            nat_add(a253, a254)?
                        }, v3))
                    } else {
                        Err::<(u64, u64), R<bool>>(Ok(false))
                    }
                }
            };
            match step {
                Ok(next) => { m255 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f54(v0: Str, v1: Bytes) -> R<Str> {
    Ok(f23({
        let a263 = {
            let a261 = v1.clone();
            let a262 = f73({
                let a258 = v0.clone();
                utf8_encode(a258)
            }, 0u64, v1.clone(), {
                let a260 = {
                    let a259 = v0.clone();
                    utf8_encode(a259)
                };
                length_bytes(a260)
            })?;
            append_bytes(a261, a262)
        };
        let a264 = v1;
        append_bytes(a263, a264)
    }, v0))
}

pub fn f55(v0: Bytes, v1: Bytes) -> bool {
    let a269 = {
        let a265 = v0;
        let a266 = v1;
        compare_bytes(a265, a266)
    };
    let a270 = {
        let a267 = Bytes::lit(&[0x00]);
        let a268 = Bytes::lit(&[0x00]);
        compare_bytes(a267, a268)
    };
    equal(a269, a270)
}

pub fn f56(v0: Bytes) -> bool {
    let a271 = f55(f57(v0.clone()), v0);
    bool_not(a271)
}

pub fn f57(v0: Bytes) -> Bytes {
    f74(v0, f75(), f76(), 26u64)
}

pub fn f58(v0: Bytes) -> bool {
    if f50(v0.clone(), f76(), 26u64) {
        true
    } else {
        f50(v0, f52(), 10u64)
    }
}

pub fn f59(v0: Str, v1: Str) -> Option<(Str, Str)> {
    Some((Str::lit("generate.model"), f9(List::cons(Str::lit("operation "), List::cons(v0, List::cons(Str::lit(" collides with the method "), List::cons(v1, List::<Str>::nil())))))))
}

pub fn f60(v0: Str, p277: List<Adt8>) -> bool {
    let m276 = p277;
    {
        let mut m276 = m276;
        loop {
            let step = {
                let v1 = m276;
                let m272 = v1;
                match m272.uncons() {
                    None => {
                        Err::<List<Adt8>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a274 = match v2 {
                            Adt8::C0(h273, _) => {
                                h273
                            }
                        };
                        let a275 = v0.clone();
                        if equal(a274, a275) {
                            Err::<List<Adt8>, bool>(true)
                        } else {
                            Ok::<List<Adt8>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m276 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f61(v0: Str, p284: List<(Str, Str)>) -> Option<Str> {
    let m283 = p284;
    {
        let mut m283 = m283;
        loop {
            let step = {
                let v1 = m283;
                let m278 = v1;
                match m278.uncons() {
                    None => {
                        Err::<List<(Str, Str)>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a280 = {
                            let (h279, _) = v2;
                            h279
                        };
                        let a281 = v0.clone();
                        if equal(a280, a281) {
                            Err::<List<(Str, Str)>, Option<Str>>(Some({
                                let (_, h282) = v2;
                                h282
                            }))
                        } else {
                            Ok::<List<(Str, Str)>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m283 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f62(v0: bool, v1: List<Str>) -> R<List<Str>> {
    let m285 = v1;
    match m285.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            let a286 = f77(v2.clone())?;
            let a287 = v0;
            if equal(a286, a287) {
                Ok(List::cons(v2, f62(v0, v3)?))
            } else {
                f62(v0, v3)
            }
        }
    }
}

pub fn f63(v0: Str) -> bool {
    let a289 = {
        let a288 = v0;
        length_string(a288)
    };
    let a290 = 100u64;
    nat_le(a289, a290)
}

pub fn f64(v0: Str, v1: List<Str>, v2: Str) -> List<Str> {
    let m291 = v1;
    match m291.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v3, v4)) => {
            List::cons(f9(List::cons(v0.clone(), List::cons(v3, List::cons(v2.clone(), List::<Str>::nil())))), f64(v0, v4, v2))
        }
    }
}

pub fn f65(v0: Str, v1: List<Str>) -> List<Str> {
    let m292 = v1;
    match m292.uncons() {
        None => {
            List::cons(v0, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            if f36(v0.clone(), v2.clone()) {
                List::cons(v0, List::cons(v2, v3))
            } else {
                List::cons(v2, f65(v0, v3))
            }
        }
    }
}

pub fn f66(v0: List<Adt3>) -> R<Str> {
    let m293 = v0;
    match m293.uncons() {
        None => {
            Ok(Str::lit(""))
        }
        Some((v1, v2)) => {
            Ok(f9(List::cons(Str::lit(", "), List::cons(f4(match v1.clone() {
                Adt3::C0(h294, _) => {
                    h294
                }
            })?, List::cons(Str::lit(": "), List::cons(f67(match v1 {
                Adt3::C0(_, h295) => {
                    h295
                }
            }), List::cons(f66(v2)?, List::<Str>::nil())))))))
        }
    }
}

pub fn f67(v0: Str) -> Str {
    let m296 = f61(v0.clone(), f13());
    match m296 {
        None => {
            v0
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f68(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a297 = v0.clone();
        utf8_encode(a297)
    };
    Ok(f23(f78(v1.clone(), 0u64, {
        let a298 = v1;
        length_bytes(a298)
    })?, v0))
}

pub fn f69(v0: Str, v1: List<Str>) -> R<List<Str>> {
    let m299 = v1;
    match m299.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v2, v3)) => {
            Ok(List::cons({
                let a300 = v2.clone();
                let a301 = Str::lit("");
                if equal(a300, a301) {
                    Str::lit("")
                } else {
                    f9(List::cons(v0.clone(), List::cons(f68(v2)?, List::<Str>::nil())))
                }
            }, f69(v0, v3)?))
        }
    }
}

pub fn f70(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a302 = f7(v0.clone())?;
        utf8_encode(a302)
    };
    Ok(f23(f79(v1.clone(), 0u64, {
        let a303 = v1;
        length_bytes(a303)
    })?, v0))
}

pub fn f71(v0: Str) -> R<Str> {
    let v1: Bytes = {
        let a304 = v0.clone();
        utf8_encode(a304)
    };
    Ok(f23(f80(v1.clone(), 0u64, {
        let a305 = v1;
        length_bytes(a305)
    })?, v0))
}

pub fn f72(v0: Adt4, v1: Str, v2: Adt8) -> Str {
    let m307 = match v2.clone() {
        Adt8::C0(_, h306) => {
            h306
        }
    };
    match m307 {
        None => {
            let a309 = match v2.clone() {
                Adt8::C0(h308, _) => {
                    h308
                }
            };
            let a310 = Str::lit("unsupported-version");
            if equal(a309, a310) {
                f9(List::cons(match v0 {
                    Adt4::C0(h311, _, _, _, _, _, _, _) => {
                        h311
                    }
                }, List::cons(Str::lit(" does not answer request version "), List::cons(v1, List::cons(Str::lit("."), List::<Str>::nil())))))
            } else {
                let a313 = match v2.clone() {
                    Adt8::C0(h312, _) => {
                        h312
                    }
                };
                let a314 = Str::lit("unknown-operation");
                if equal(a313, a314) {
                    f9(List::cons(match v0 {
                        Adt4::C0(h315, _, _, _, _, _, _, _) => {
                            h315
                        }
                    }, List::cons(Str::lit(" does not know this operation."), List::<Str>::nil())))
                } else {
                    f9(List::cons(match v0 {
                        Adt4::C0(h316, _, _, _, _, _, _, _) => {
                            h316
                        }
                    }, List::cons(Str::lit(" refused the request: "), List::cons(match v2 {
                        Adt8::C0(h317, _) => {
                            h317
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

pub fn f73(v0: Bytes, v1: u64, v2: Bytes, v3: u64) -> R<Bytes> {
    let m318 = v3;
    if m318 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m318 - 1;
        let a323 = if f55(f51(v0.clone(), v1, 1u64), Bytes::lit(&[0x5c])) {
            Bytes::lit(&[0x5c, 0x5c])
        } else {
            if f55(f51(v0.clone(), v1, 1u64), v2.clone()) {
                let a319 = Bytes::lit(&[0x5c]);
                let a320 = v2.clone();
                append_bytes(a319, a320)
            } else {
                if f55(f51(v0.clone(), v1, 1u64), Bytes::lit(&[0x0a])) {
                    Bytes::lit(&[0x5c, 0x6e])
                } else {
                    f51(v0.clone(), v1, 1u64)
                }
            }
        };
        let a324 = f73(v0, {
            let a321 = v1;
            let a322 = 1u64;
            nat_add(a321, a322)?
        }, v2, v4)?;
        Ok(append_bytes(a323, a324))
    }
}

pub fn f74(v0: Bytes, v1: Bytes, v2: Bytes, p327: u64) -> Bytes {
    let m326 = p327;
    {
        let mut m326 = m326;
        loop {
            let step = {
                let v3 = m326;
                let m325 = v3;
                if m325 == 0 {
                    Err::<u64, Bytes>(v0.clone())
                } else {
                    let v4 = m325 - 1;
                    if f55(f51(v1.clone(), v4, 1u64), v0.clone()) {
                        Err::<u64, Bytes>(f51(v2.clone(), v4, 1u64))
                    } else {
                        Ok::<u64, Bytes>(v4)
                    }
                }
            };
            match step {
                Ok(next) => { m326 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f75() -> Bytes {
    Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a])
}

pub fn f76() -> Bytes {
    Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a])
}

pub fn f77(v0: Str) -> R<bool> {
    let v1: Bytes = {
        let a328 = v0;
        utf8_encode(a328)
    };
    let a332 = {
        let a330 = {
            let a329 = v1.clone();
            length_bytes(a329)
        };
        let a331 = 0u64;
        nat_eq(a330, a331)
    };
    if bool_not(a332) {
        f81(v1.clone(), 0u64, {
            let a333 = v1;
            length_bytes(a333)
        })
    } else {
        Ok(false)
    }
}

pub fn f78(v0: Bytes, v1: u64, v2: u64) -> R<Bytes> {
    let m334 = v2;
    if m334 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v3 = m334 - 1;
        let a336 = {
            let a335 = v0.clone();
            length_bytes(a335)
        };
        let a337 = v1;
        if nat_le(a336, a337) {
            Ok(Bytes::lit(&[]))
        } else {
            if f55(f51(v0.clone(), v1, 3u64), Bytes::lit(&[0x22, 0x22, 0x22])) {
                let a340 = Bytes::lit(&[0x5c, 0x22, 0x5c, 0x22, 0x5c, 0x22]);
                let a341 = f78(v0, {
                    let a338 = v1;
                    let a339 = 3u64;
                    nat_add(a338, a339)?
                }, v3)?;
                Ok(append_bytes(a340, a341))
            } else {
                if f55(f51(v0.clone(), v1, 1u64), Bytes::lit(&[0x5c])) {
                    let a344 = Bytes::lit(&[0x5c, 0x5c]);
                    let a345 = f78(v0, {
                        let a342 = v1;
                        let a343 = 1u64;
                        nat_add(a342, a343)?
                    }, v3)?;
                    Ok(append_bytes(a344, a345))
                } else {
                    let a348 = f51(v0.clone(), v1, 1u64);
                    let a349 = f78(v0, {
                        let a346 = v1;
                        let a347 = 1u64;
                        nat_add(a346, a347)?
                    }, v3)?;
                    Ok(append_bytes(a348, a349))
                }
            }
        }
    }
}

pub fn f79(v0: Bytes, v1: u64, v2: u64) -> R<Bytes> {
    let m350 = v2;
    if m350 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v3 = m350 - 1;
        let a353 = f82(f51(v0.clone(), v1, 1u64));
        let a354 = f79(v0, {
            let a351 = v1;
            let a352 = 1u64;
            nat_add(a351, a352)?
        }, v3)?;
        Ok(append_bytes(a353, a354))
    }
}

pub fn f80(v0: Bytes, v1: u64, v2: u64) -> R<Bytes> {
    let m355 = v2;
    if m355 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v3 = m355 - 1;
        let a357 = {
            let a356 = v0.clone();
            length_bytes(a356)
        };
        let a358 = v1;
        if nat_le(a357, a358) {
            Ok(Bytes::lit(&[]))
        } else {
            if f55(f51(v0.clone(), v1, 3u64), Bytes::lit(&[0x22, 0x22, 0x22])) {
                f80(v0, {
                    let a359 = v1;
                    let a360 = 3u64;
                    nat_add(a359, a360)?
                }, v3)
            } else {
                let a363 = f51(v0.clone(), v1, 1u64);
                let a364 = f80(v0, {
                    let a361 = v1;
                    let a362 = 1u64;
                    nat_add(a361, a362)?
                }, v3)?;
                Ok(append_bytes(a363, a364))
            }
        }
    }
}

pub fn f81(v0: Bytes, p369: u64, p370: u64) -> R<bool> {
    let m368 = (p369, p370);
    {
        let mut m368 = m368;
        loop {
            let step = {
                let (v1, v2) = m368;
                let m365 = v2;
                if m365 == 0 {
                    Err::<(u64, u64), R<bool>>(Ok(true))
                } else {
                    let v3 = m365 - 1;
                    if if f50(f51(v0.clone(), v1, 1u64), f75(), 26u64) {
                        true
                    } else {
                        if f50(f51(v0.clone(), v1, 1u64), f52(), 10u64) {
                            true
                        } else {
                            f55(f51(v0.clone(), v1, 1u64), Bytes::lit(&[0x5f]))
                        }
                    } {
                        Ok::<(u64, u64), R<bool>>(({
                            let a366 = v1;
                            let a367 = 1u64;
                            nat_add(a366, a367)?
                        }, v3))
                    } else {
                        Err::<(u64, u64), R<bool>>(Ok(false))
                    }
                }
            };
            match step {
                Ok(next) => { m368 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f82(v0: Bytes) -> Bytes {
    f74(v0, f76(), f75(), 26u64)
}

pub fn run(p0: Adt0) -> R<Adt9> {
    f0(p0)
}
