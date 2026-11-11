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
    Ok(f4(f5(Str::lit("rust"), f6(match v1.clone() {
        Adt4::C0(_, _, h5, _, _, _, _, _) => {
            h5
        }
    })?)?, f4({
        let a7 = f7(match v1.clone() {
            Adt4::C0(_, _, h6, _, _, _, _, _) => {
                h6
            }
        })?;
        let a8 = Str::lit("world");
        if equal(a7, a8) {
            Some((Str::lit("generate.model"), Str::lit("an application named world collides with the world module")))
        } else {
            None::<(Str, Str)>
        }
    }, f4(f8(v1)?, f4(f9(Str::lit("rust"), match v0.clone() {
        Adt0::C0(_, h9, _) => {
            h9
        }
    }), f10(Str::lit("rust"), true, f11(), {
        let m12 = match v0 {
            Adt0::C0(_, h11, _) => {
                h11
            }
        };
        match m12 {
            Adt1::C0(_, _, _, _, _, _, _, _, h10) => {
                h10
            }
        }
    })?)))))
}

pub fn f3(v0: Adt0) -> R<List<Adt10>> {
    let v1: Str = match v0.clone() {
        Adt0::C0(h13, _, _) => {
            h13
        }
    };
    let v2: Adt1 = match v0.clone() {
        Adt0::C0(_, h14, _) => {
            h14
        }
    };
    let v3: Adt4 = match v0 {
        Adt0::C0(_, _, h15) => {
            h15
        }
    };
    let v4: Str = f7(match v3.clone() {
        Adt4::C0(_, _, h16, _, _, _, _, _) => {
            h16
        }
    })?;
    Ok(List::cons(f12(Str::lit("Cargo.toml"), f13(v1.clone(), v2.clone(), v3.clone())?), List::cons(f12(Str::lit("lib.rs"), f14(v1.clone(), v2.clone(), v3.clone(), v4.clone())), List::cons(f12(Str::lit("world.rs"), f15(v1.clone(), v2)?), List::cons(f12(f16(List::cons(v4, List::cons(Str::lit(".rs"), List::<Str>::nil()))), {
        let m18 = match v3.clone() {
            Adt4::C0(_, _, _, _, _, _, _, h17) => {
                h17
            }
        };
        match m18 {
            Adt5::C0 => {
                f17(v1, v3)?
            }
            Adt5::C1(v5) => {
                f18(v1, v3, v5)?
            }
        }
    }), List::<Adt10>::nil())))))
}

pub fn f4(v0: Option<(Str, Str)>, v1: Option<(Str, Str)>) -> Option<(Str, Str)> {
    let m19 = v0.clone();
    match m19 {
        None => {
            v1
        }
        Some(_) => {
            v0
        }
    }
}

pub fn f5(v0: Str, v1: Str) -> R<Option<(Str, Str)>> {
    if f19(v1.clone())? {
        Ok(None::<(Str, Str)>)
    } else {
        Ok(Some((Str::lit("generate.model"), f16(List::cons(f20(v1)?, List::cons(Str::lit(" is not a name the "), List::cons(v0, List::cons(Str::lit(" writer can bind"), List::<Str>::nil()))))))))
    }
}

pub fn f6(v0: Str) -> R<Str> {
    Ok(f21(f22({
        let a20 = v0.clone();
        utf8_encode(a20)
    }, 0u64, false, {
        let a22 = {
            let a21 = v0.clone();
            utf8_encode(a21)
        };
        length_bytes(a22)
    })?, v0))
}

pub fn f7(v0: Str) -> R<Str> {
    Ok(f23(f6(v0)?))
}

pub fn f8(v0: Adt4) -> R<Option<(Str, Str)>> {
    let m24 = match v0.clone() {
        Adt4::C0(_, _, _, _, _, _, _, h23) => {
            h23
        }
    };
    match m24 {
        Adt5::C0 => {
            Ok(None::<(Str, Str)>)
        }
        Adt5::C1(v1) => {
            Ok(f4(f24(Str::lit("rust"), match v1.clone() {
                Adt6::C0(_, _, _, _, h25, _) => {
                    h25
                }
            })?, f4(f5(Str::lit("rust"), f6(match v1.clone() {
                Adt6::C0(_, h26, _, _, _, _) => {
                    h26
                }
            })?)?, f4(f25(f7(match v1.clone() {
                Adt6::C0(_, h27, _, _, _, _) => {
                    h27
                }
            })?, match v1.clone() {
                Adt6::C0(_, _, _, _, h28, _) => {
                    h28
                }
            })?, f26(v0, v1)))))
        }
    }
}

pub fn f9(v0: Str, v1: Adt1) -> Option<(Str, Str)> {
    let a32 = {
        let a30 = match v1.clone() {
            Adt1::C0(h29, _, _, _, _, _, _, _, _) => {
                h29
            }
        };
        let a31 = Str::lit("hologram:guest/core-wasm@1");
        equal(a30, a31)
    };
    if bool_not(a32) {
        Some((Str::lit("generate.world"), f16(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer has no host protocol for "), List::cons(match v1 {
            Adt1::C0(h33, _, _, _, _, _, _, _, _) => {
                h33
            }
        }, List::<Str>::nil())))))))
    } else {
        let m35 = match v1.clone() {
            Adt1::C0(_, _, _, _, _, _, _, h34, _) => {
                h34
            }
        };
        match m35.uncons() {
            None => {
                None::<(Str, Str)>
            }
            Some((_, _)) => {
                Some((Str::lit("generate.world"), f16(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer binds import-free worlds; "), List::cons(match v1 {
                    Adt1::C0(h36, _, _, _, _, _, _, _, _) => {
                        h36
                    }
                }, List::cons(Str::lit(" has imports"), List::<Str>::nil()))))))))
            }
        }
    }
}

pub fn f10(v0: Str, v1: bool, v2: List<(Str, Str)>, v3: List<Adt2>) -> R<Option<(Str, Str)>> {
    let m37 = v3;
    match m37.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v4, v5)) => {
            Ok(f4(if v1 {
                f5(v0.clone(), match v4.clone() {
                    Adt2::C0(h38, _, _, _) => {
                        h38
                    }
                })?
            } else {
                None::<(Str, Str)>
            }, f4(f27(v0.clone(), v1, v2.clone(), match v4.clone() {
                Adt2::C0(_, h39, _, _) => {
                    h39
                }
            })?, f4({
                let m41 = match v4 {
                    Adt2::C0(_, _, h40, _) => {
                        h40
                    }
                };
                match m41 {
                    None => {
                        None::<(Str, Str)>
                    }
                    Some(v6) => {
                        f28(v0.clone(), v6, v2.clone())
                    }
                }
            }, f10(v0, v1, v2, v5)?))))
        }
    }
}

pub fn f11() -> List<(Str, Str)> {
    List::cons((Str::lit("bool"), Str::lit("bool")), List::cons((Str::lit("u8"), Str::lit("u8")), List::cons((Str::lit("u16"), Str::lit("u16")), List::cons((Str::lit("u32"), Str::lit("u32")), List::cons((Str::lit("u64"), Str::lit("u64")), List::cons((Str::lit("s8"), Str::lit("i8")), List::cons((Str::lit("s16"), Str::lit("i16")), List::cons((Str::lit("s32"), Str::lit("i32")), List::cons((Str::lit("s64"), Str::lit("i64")), List::cons((Str::lit("f32"), Str::lit("f32")), List::cons((Str::lit("f64"), Str::lit("f64")), List::cons((Str::lit("char"), Str::lit("char")), List::<(Str, Str)>::nil()))))))))))))
}

pub fn f12(v0: Str, v1: Str) -> Adt10 {
    Adt10::C0(v0, v1)
}

pub fn f13(v0: Str, v1: Adt1, v2: Adt4) -> R<Str> {
    Ok(f29(List::cons(f16(List::cons(Str::lit("# Generated by `"), List::cons(v0.clone(), List::cons(Str::lit("` from "), List::cons(match v2.clone() {
        Adt4::C0(h42, _, _, _, _, _, _, _) => {
            h42
        }
    }, List::cons(Str::lit("'s model and the WIT world "), List::cons(match v1 {
        Adt1::C0(h43, _, _, _, _, _, _, _, _) => {
            h43
        }
    }, List::cons(Str::lit("."), List::<Str>::nil())))))))), List::cons(f16(List::cons(Str::lit("# Do not edit: run `"), List::cons(v0.clone(), List::cons(Str::lit("` again."), List::<Str>::nil())))), List::cons(Str::lit("[package]"), List::cons(f16(List::cons(Str::lit("name = "), List::cons(f20(f16(List::cons(match v2.clone() {
        Adt4::C0(_, _, h44, _, _, _, _, _) => {
            h44
        }
    }, List::cons(Str::lit("-client"), List::<Str>::nil()))))?, List::cons(Str::lit(""), List::<Str>::nil())))), List::cons(Str::lit("version = \"0.1.0\"\u{a}edition = \"2021\""), List::cons(f16(List::cons(Str::lit("description = "), List::cons(f20(f16(List::cons(Str::lit("A client for "), List::cons(match v2 {
        Adt4::C0(h45, _, _, _, _, _, _, _) => {
            h45
        }
    }, List::cons(Str::lit(", generated by "), List::cons(v0, List::cons(Str::lit("."), List::<Str>::nil())))))))?, List::<Str>::nil()))), List::cons(Str::lit("publish = false\u{a}\u{a}[lib]\u{a}path = \"lib.rs\"\u{a}\u{a}[dependencies]"), List::<Str>::nil())))))))))
}

pub fn f14(v0: Str, v1: Adt1, v2: Adt4, v3: Str) -> Str {
    f29({
        let a59 = List::cons(f30(Str::lit("//"), v0.clone(), f16(List::cons(match v2.clone() {
            Adt4::C0(h46, _, _, _, _, _, _, _) => {
                h46
            }
        }, List::cons(Str::lit("'s model and the WIT world "), List::cons(match v1.clone() {
            Adt1::C0(h47, _, _, _, _, _, _, _, _) => {
                h47
            }
        }, List::<Str>::nil()))))), List::cons(f16(List::cons(Str::lit("//! A client for "), List::cons(match v2.clone() {
            Adt4::C0(h48, _, _, _, _, _, _, _) => {
                h48
            }
        }, List::cons(Str::lit(", generated by "), List::cons(v0, List::cons(Str::lit("."), List::<Str>::nil())))))), List::<Str>::nil()));
        let a60 = {
            let a57 = {
                let m50 = match v2.clone() {
                    Adt4::C0(_, _, _, h49, _, _, _, _) => {
                        h49
                    }
                };
                match m50 {
                    None => {
                        List::<Str>::nil()
                    }
                    Some(v4) => {
                        List::cons(Str::lit("//!"), f31(List::cons(v4, List::<Str>::nil())))
                    }
                }
            };
            let a58 = {
                let a55 = List::cons(Str::lit("//!"), List::cons(f16(List::cons(Str::lit("//! [`world`] is the world "), List::cons(match v1 {
                    Adt1::C0(h51, _, _, _, _, _, _, _, _) => {
                        h51
                    }
                }, List::cons(Str::lit(": implement [`world::Guest`] over your WebAssembly engine,"), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("//! and [`world::dispatch`] sends it a request. [`"), List::cons(v3.clone(), List::cons(Str::lit("`] is the client of "), List::cons(match v2 {
                    Adt4::C0(h52, _, _, _, _, _, _, _) => {
                        h52
                    }
                }, List::cons(Str::lit("."), List::<Str>::nil())))))), List::cons(Str::lit("\u{a}#![forbid(unsafe_code)]\u{a}"), List::<Str>::nil()))));
                let a56 = {
                    let a53 = f32(List::cons(f16(List::cons(Str::lit("pub mod "), List::cons(v3.clone(), List::cons(Str::lit(";"), List::<Str>::nil())))), List::cons(Str::lit("pub mod world;"), List::<Str>::nil())));
                    let a54 = List::cons(Str::lit(""), List::cons(f16(List::cons(Str::lit("pub use "), List::cons(v3, List::cons(Str::lit("::*;"), List::<Str>::nil())))), List::cons(Str::lit("\u{a}/// What a set of bindings was generated from: a name and both digests, as prismpm.lock records them.\u{a}#[derive(Clone, Copy, Debug, PartialEq, Eq)]\u{a}pub struct Identity {\u{a}    /// The world's contract, or the application's name.\u{a}    pub name: &'static str,\u{a}    /// The BLAKE3 digest, lowercase hex.\u{a}    pub blake3: &'static str,\u{a}    /// The SHA-256 digest, lowercase hex.\u{a}    pub sha256: &'static str,\u{a}}"), List::<Str>::nil())));
                    append_list(a53, a54)
                };
                append_list(a55, a56)
            };
            append_list(a57, a58)
        };
        append_list(a59, a60)
    })
}

pub fn f15(v0: Str, v1: Adt1) -> R<Str> {
    Ok(f29({
        let a77 = List::cons(f30(Str::lit("//"), v0, f33(v1.clone())), List::cons(f16(List::cons(Str::lit("//! The world "), List::cons(match v1.clone() {
            Adt1::C0(h61, _, _, _, _, _, _, _, _) => {
                h61
            }
        }, List::cons(Str::lit(": its identity, its exports, and how a host sends it a request."), List::<Str>::nil())))), List::cons(Str::lit("\u{a}use core::fmt;\u{a}\u{a}pub use crate::Identity;\u{a}\u{a}/// The world these bindings were generated from. It equals `inputs.world` in prismpm.lock.\u{a}pub const WORLD: Identity = Identity {"), List::cons(f16(List::cons(Str::lit("    name: "), List::cons(f20(match v1.clone() {
            Adt1::C0(h62, _, _, _, _, _, _, _, _) => {
                h62
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    blake3: "), List::cons(f20(match v1.clone() {
            Adt1::C0(_, _, _, _, h63, _, _, _, _) => {
                h63
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    sha256: "), List::cons(f20(match v1.clone() {
            Adt1::C0(_, _, _, _, _, h64, _, _, _) => {
                h64
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit("};\u{a}\u{a}/// Every export the world requires, sorted. A guest that offers any other is not of this world."), List::cons({
            let v2: List<Str> = f32(List::cons(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, h65, _, _) => {
                    h65
                }
            }, f34(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, _, _, h66) => {
                    h66
                }
            })));
            f16(List::cons(Str::lit("pub const EXPORTS: [&str; "), List::cons(f35({
                let a67 = v2.clone();
                length_list(a67)
            })?, List::cons(Str::lit("] = ["), List::cons({
                let a68 = f36(v2)?;
                let a69 = Str::lit(", ");
                join(a68, a69)
            }, List::cons(Str::lit("];"), List::<Str>::nil()))))))
        }, List::cons(Str::lit(""), List::cons(f16(List::cons(Str::lit("/// A guest of "), List::cons(match v1.clone() {
            Adt1::C0(h70, _, _, _, _, _, _, _, _) => {
                h70
            }
        }, List::cons(Str::lit(", instantiated by the host's WebAssembly engine."), List::<Str>::nil())))), List::cons(Str::lit("///"), List::cons(f16(List::cons(Str::lit("/// The guest imports nothing. Implement this over the engine's instance: each export, and `"), List::cons(match v1.clone() {
            Adt1::C0(_, _, _, _, _, _, h71, _, _) => {
                h71
            }
        }, List::cons(Str::lit("` as a read and a write."), List::<Str>::nil())))), List::cons(Str::lit("pub trait Guest {\u{a}    /// What the engine reports when a call into the guest fails, such as a trap.\u{a}    type Error;\u{a}"), List::<Str>::nil())))))))))))));
        let a78 = {
            let a75 = f37(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, _, _, h72) => {
                    h72
                }
            })?;
            let a76 = List::cons(f16(List::cons(Str::lit("    /// Copy `bytes` into the guest's `"), List::cons(match v1.clone() {
                Adt1::C0(_, _, _, _, _, _, h73, _, _) => {
                    h73
                }
            }, List::cons(Str::lit("` at `offset`."), List::<Str>::nil())))), List::cons(Str::lit("    fn write_memory(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error>;\u{a}"), List::cons(f16(List::cons(Str::lit("    /// Copy `length` bytes out of the guest's `"), List::cons(match v1 {
                Adt1::C0(_, _, _, _, _, _, h74, _, _) => {
                    h74
                }
            }, List::cons(Str::lit("` at `offset`. Fails when they are outside it."), List::<Str>::nil())))), List::cons(Str::lit("    fn read_memory(&mut self, offset: u32, length: u32) -> Result<Vec<u8>, Self::Error>;\u{a}}\u{a}"), List::cons(f38(), List::<Str>::nil())))));
            append_list(a75, a76)
        };
        append_list(a77, a78)
    }))
}

pub fn f16(v0: List<Str>) -> Str {
    let a79 = v0;
    let a80 = Str::lit("");
    join(a79, a80)
}

pub fn f17(v0: Str, v1: Adt4) -> R<Str> {
    let v2: Str = match v1.clone() {
        Adt4::C0(h81, _, _, _, _, _, _, _) => {
            h81
        }
    };
    Ok(f29({
        let a90 = List::cons(f30(Str::lit("//"), v0, f39(v1.clone())), List::cons(f16(List::cons(Str::lit("//! The client of "), List::cons(v2.clone(), List::cons(Str::lit(", over request and response bytes."), List::<Str>::nil())))), List::cons(Str::lit("\u{a}pub use crate::Identity;\u{a}\u{a}/// The model this client was generated from. It equals `inputs.model` in prismpm.lock.\u{a}pub const MODEL: Identity = Identity {"), List::cons(f16(List::cons(Str::lit("    name: "), List::cons(f20(match v1.clone() {
            Adt4::C0(h82, _, _, _, _, _, _, _) => {
                h82
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    blake3: "), List::cons(f20(match v1.clone() {
            Adt4::C0(_, _, _, _, _, h83, _, _) => {
                h83
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    sha256: "), List::cons(f20(match v1.clone() {
            Adt4::C0(_, _, _, _, _, _, h84, _) => {
                h84
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit("};\u{a}"), List::<Str>::nil())))))));
        let a91 = {
            let a88 = f40(Str::lit(""), f41(v1.clone(), f16(List::cons(Str::lit("A client for "), List::cons(v2.clone(), List::cons(Str::lit(", over request and response bytes."), List::<Str>::nil()))))));
            let a89 = List::cons(Str::lit("///"), List::cons(f16(List::cons(Str::lit("/// "), List::cons(v2, List::cons(Str::lit("'s model describes no typed operations, so the client sends bytes."), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("pub struct "), List::cons(match v1.clone() {
                Adt4::C0(_, h85, _, _, _, _, _, _) => {
                    h85
                }
            }, List::cons(Str::lit("<D> {"), List::<Str>::nil())))), List::cons(Str::lit("    dispatch: D,\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("impl<D, E> "), List::cons(match v1.clone() {
                Adt4::C0(_, h86, _, _, _, _, _, _) => {
                    h86
                }
            }, List::cons(Str::lit("<D>"), List::<Str>::nil())))), List::cons(Str::lit("where\u{a}    D: FnMut(&[u8]) -> Result<Vec<u8>, E>,\u{a}{\u{a}    /// A client over this dispatch function.\u{a}    pub fn new(dispatch: D) -> Self {"), List::cons(f16(List::cons(Str::lit("        "), List::cons(match v1 {
                Adt4::C0(_, h87, _, _, _, _, _, _) => {
                    h87
                }
            }, List::cons(Str::lit(" { dispatch }"), List::<Str>::nil())))), List::cons(Str::lit("    }\u{a}\u{a}    /// Send one request and return the response bytes.\u{a}    pub fn invoke(&mut self, request: &[u8]) -> Result<Vec<u8>, E> {\u{a}        (self.dispatch)(request)\u{a}    }\u{a}}"), List::<Str>::nil()))))))));
            append_list(a88, a89)
        };
        append_list(a90, a91)
    }))
}

pub fn f18(v0: Str, v1: Adt4, v2: Adt6) -> R<Str> {
    let v3: Str = match v1.clone() {
        Adt4::C0(h92, _, _, _, _, _, _, _) => {
            h92
        }
    };
    let v4: Str = match v2.clone() {
        Adt6::C0(h93, _, _, _, _, _) => {
            h93
        }
    };
    let v5: Str = match v2.clone() {
        Adt6::C0(_, _, _, h94, _, _) => {
            h94
        }
    };
    let v6: Str = f16(List::cons(v5.clone(), List::cons(Str::lit("Code"), List::<Str>::nil())));
    let v7: Str = match v2.clone() {
        Adt6::C0(_, _, h95, _, _, _) => {
            h95
        }
    };
    let v8: Str = f7(match v2.clone() {
        Adt6::C0(_, h96, _, _, _, _) => {
            h96
        }
    })?;
    let v9: Str = f16(List::cons(Str::lit("Result<i64, "), List::cons(v5.clone(), List::cons(Str::lit("<E>>"), List::<Str>::nil()))));
    Ok(f29({
        let a157 = List::cons(f30(Str::lit("//"), v0, f39(v1.clone())), List::cons(f16(List::cons(Str::lit("//! The client of "), List::cons(v3.clone(), List::cons(Str::lit(": one method per operation, over any dispatch function."), List::<Str>::nil())))), List::cons(Str::lit("\u{a}use core::fmt;\u{a}\u{a}pub use crate::Identity;\u{a}\u{a}/// The model this client was generated from. It equals `inputs.model` in prismpm.lock.\u{a}pub const MODEL: Identity = Identity {"), List::cons(f16(List::cons(Str::lit("    name: "), List::cons(f20(match v1.clone() {
            Adt4::C0(h97, _, _, _, _, _, _, _) => {
                h97
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    blake3: "), List::cons(f20(match v1.clone() {
            Adt4::C0(_, _, _, _, _, h98, _, _) => {
                h98
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    sha256: "), List::cons(f20(match v1.clone() {
            Adt4::C0(_, _, _, _, _, _, h99, _) => {
                h99
            }
        })?, List::cons(Str::lit(","), List::<Str>::nil())))), List::cons(Str::lit("};\u{a}"), List::cons(f16(List::cons(Str::lit("const APPLICATION: &str = "), List::cons(f20(v3.clone())?, List::cons(Str::lit(";"), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("const VERSION: &str = "), List::cons(f20(v4.clone())?, List::cons(Str::lit(";"), List::<Str>::nil())))), List::cons(Str::lit(""), List::cons(f16(List::cons(Str::lit("/// An operation "), List::cons(v3.clone(), List::cons(Str::lit(" accepts."), List::<Str>::nil())))), List::cons(Str::lit("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]"), List::cons(f16(List::cons(Str::lit("pub enum "), List::cons(v7.clone(), List::cons(Str::lit(" {"), List::<Str>::nil())))), List::<Str>::nil())))))))))))));
        let a158 = {
            let a155 = f42(match v2.clone() {
                Adt6::C0(_, _, _, _, h100, _) => {
                    h100
                }
            })?;
            let a156 = {
                let a153 = List::cons(Str::lit("}\u{a}"), List::cons(f16(List::cons(Str::lit("impl "), List::cons(v7.clone(), List::cons(Str::lit(" {"), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    /// Every operation "), List::cons(v3.clone(), List::cons(Str::lit(" accepts."), List::<Str>::nil())))), List::<Str>::nil())));
                let a154 = {
                    let a151 = f43(Str::lit("    "), f16(List::cons(Str::lit("pub const ALL: [Self; "), List::cons(f35({
                        let a102 = match v2.clone() {
                            Adt6::C0(_, _, _, _, h101, _) => {
                                h101
                            }
                        };
                        length_list(a102)
                    })?, List::cons(Str::lit("]"), List::<Str>::nil())))), f44(match v2.clone() {
                        Adt6::C0(_, _, _, _, h103, _) => {
                            h103
                        }
                    })?);
                    let a152 = {
                        let a149 = List::cons(Str::lit("\u{a}    /// The name the guest reads in a request.\u{a}    pub const fn name(self) -> &'static str {\u{a}        match self {"), List::<Str>::nil());
                        let a150 = {
                            let a147 = f45(match v2.clone() {
                                Adt6::C0(_, _, _, _, h104, _) => {
                                    h104
                                }
                            })?;
                            let a148 = {
                                let a145 = List::cons(Str::lit("        }\u{a}    }\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("/// An answer with which "), List::cons(v3.clone(), List::cons(Str::lit(" refuses a request."), List::<Str>::nil())))), List::cons(Str::lit("#[derive(Clone, Debug, PartialEq, Eq, Hash)]"), List::cons(f16(List::cons(Str::lit("pub enum "), List::cons(v6.clone(), List::cons(Str::lit(" {"), List::<Str>::nil())))), List::<Str>::nil()))));
                                let a146 = {
                                    let a143 = f46(v1.clone(), v4.clone(), match v2.clone() {
                                        Adt6::C0(_, _, _, _, _, h105) => {
                                            h105
                                        }
                                    })?;
                                    let a144 = {
                                        let a141 = List::cons(Str::lit("    /// An answer this client was not generated with.\u{a}    Other(String),\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("impl "), List::cons(v6.clone(), List::cons(Str::lit(" {"), List::<Str>::nil())))), List::cons(Str::lit("    /// The code as the guest wrote it.\u{a}    pub fn code(&self) -> &str {\u{a}        match self {"), List::<Str>::nil())));
                                        let a142 = {
                                            let a139 = f47(match v2.clone() {
                                                Adt6::C0(_, _, _, _, _, h106) => {
                                                    h106
                                                }
                                            })?;
                                            let a140 = {
                                                let a137 = List::cons(Str::lit("            Self::Other(code) => code,\u{a}        }\u{a}    }\u{a}\u{a}    /// What the model says this answer means, for a person.\u{a}    pub fn message(&self) -> &str {\u{a}        match self {"), List::<Str>::nil());
                                                let a138 = {
                                                    let a135 = f48(v1.clone(), v4, match v2.clone() {
                                                        Adt6::C0(_, _, _, _, _, h107) => {
                                                            h107
                                                        }
                                                    })?;
                                                    let a136 = {
                                                        let a133 = f49(Str::lit("            "), Str::lit("Self::Other(_)"), f20(f16(List::cons(v3.clone(), List::cons(Str::lit(" refused the request."), List::<Str>::nil()))))?);
                                                        let a134 = {
                                                            let a131 = List::cons(Str::lit("        }\u{a}    }\u{a}\u{a}    fn from_code(code: &str) -> Self {\u{a}        match code {"), List::<Str>::nil());
                                                            let a132 = {
                                                                let a129 = f50(match v2.clone() {
                                                                    Adt6::C0(_, _, _, _, _, h108) => {
                                                                        h108
                                                                    }
                                                                })?;
                                                                let a130 = {
                                                                    let a127 = List::cons(Str::lit("            other => Self::Other(other.to_string()),\u{a}        }\u{a}    }\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("/// Why a call to "), List::cons(v3.clone(), List::cons(Str::lit(" gave no result."), List::<Str>::nil())))), List::cons(Str::lit("#[derive(Debug)]"), List::cons(f16(List::cons(Str::lit("pub enum "), List::cons(v5.clone(), List::cons(Str::lit("<E> {"), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    /// "), List::cons(v3.clone(), List::cons(Str::lit(" refused the request, for example on overflow."), List::<Str>::nil())))), List::cons(f16(List::cons(Str::lit("    Refused("), List::cons(v6.clone(), List::cons(Str::lit("),"), List::<Str>::nil())))), List::cons(Str::lit("    /// The dispatch function failed before an answer came back.\u{a}    Dispatch(E),\u{a}    /// The answer is not one this client can read.\u{a}    Response(String),\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("impl<E: fmt::Display> fmt::Display for "), List::cons(v5.clone(), List::cons(Str::lit("<E> {"), List::<Str>::nil())))), List::cons(Str::lit("    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\u{a}        match self {\u{a}            Self::Refused(code) => write!(f, \"{} ({})\", code.message(), code.code()),\u{a}            Self::Dispatch(error) => write!(f, \"the request did not reach {APPLICATION}: {error}\"),\u{a}            Self::Response(why) => write!(f, \"{APPLICATION} sent an unreadable answer: {why}\"),\u{a}        }\u{a}    }\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("impl<E: fmt::Debug + fmt::Display> std::error::Error for "), List::cons(v5.clone(), List::cons(Str::lit("<E> {}"), List::<Str>::nil())))), List::cons(Str::lit(""), List::<Str>::nil())))))))))));
                                                                    let a128 = {
                                                                        let a125 = f40(Str::lit(""), f41(v1.clone(), f16(List::cons(Str::lit("A client for "), List::cons(v3.clone(), List::cons(Str::lit("."), List::<Str>::nil()))))));
                                                                        let a126 = {
                                                                            let a123 = List::cons(Str::lit("///\u{a}/// `dispatch` sends request bytes to the guest and returns its response bytes, for\u{a}/// example `|request| crate::world::dispatch(&mut fresh_instance(), request)`."), List::cons(f16(List::cons(Str::lit("pub struct "), List::cons(match v1.clone() {
                                                                                Adt4::C0(_, h109, _, _, _, _, _, _) => {
                                                                                    h109
                                                                                }
                                                                            }, List::cons(Str::lit("<D> {"), List::<Str>::nil())))), List::cons(Str::lit("    dispatch: D,\u{a}}\u{a}"), List::cons(f16(List::cons(Str::lit("impl<D, E> "), List::cons(match v1.clone() {
                                                                                Adt4::C0(_, h110, _, _, _, _, _, _) => {
                                                                                    h110
                                                                                }
                                                                            }, List::cons(Str::lit("<D>"), List::<Str>::nil())))), List::cons(Str::lit("where\u{a}    D: FnMut(&[u8]) -> Result<Vec<u8>, E>,\u{a}{\u{a}    /// A client over this dispatch function.\u{a}    pub fn new(dispatch: D) -> Self {"), List::cons(f16(List::cons(Str::lit("        "), List::cons(match v1 {
                                                                                Adt4::C0(_, h111, _, _, _, _, _, _) => {
                                                                                    h111
                                                                                }
                                                                            }, List::cons(Str::lit(" { dispatch }"), List::<Str>::nil())))), List::cons(Str::lit("    }"), List::<Str>::nil())))))));
                                                                            let a124 = {
                                                                                let a121 = f51(v8.clone(), v7.clone(), v9.clone(), match v2 {
                                                                                    Adt6::C0(_, _, _, _, h112, _) => {
                                                                                        h112
                                                                                    }
                                                                                })?;
                                                                                let a122 = {
                                                                                    let a119 = List::cons(Str::lit("\u{a}    /// Apply `operation` to two signed 64-bit integers.\u{a}    ///"), List::cons(f16(List::cons(Str::lit("    /// A refusal by "), List::cons(v3, List::cons(Str::lit(" is `"), List::cons(v5.clone(), List::cons(Str::lit("::Refused`."), List::<Str>::nil())))))), List::<Str>::nil()));
                                                                                    let a120 = {
                                                                                        let a117 = f52(Str::lit("    "), f16(List::cons(Str::lit("pub fn "), List::cons(v8, List::<Str>::nil()))), List::cons(Str::lit("&mut self"), List::cons(f16(List::cons(Str::lit("operation: "), List::cons(v7, List::<Str>::nil()))), List::cons(Str::lit("left: i64"), List::cons(Str::lit("right: i64"), List::<Str>::nil())))), v9.clone());
                                                                                        let a118 = {
                                                                                            let a115 = List::cons(Str::lit("        let request = format!(\"{VERSION}\\t{}\\t{left}\\t{right}\", operation.name());"), List::cons(f16(List::cons(Str::lit("        let response = (self.dispatch)(request.as_bytes()).map_err("), List::cons(v5.clone(), List::cons(Str::lit("::Dispatch)?;"), List::<Str>::nil())))), List::cons(Str::lit("        read(&response)\u{a}    }\u{a}}\u{a}\u{a}/// Read a response: the request version, then `ok` and a value or `error` and a code."), List::<Str>::nil())));
                                                                                            let a116 = {
                                                                                                let a113 = f52(Str::lit(""), Str::lit("fn read<E>"), List::cons(Str::lit("response: &[u8]"), List::<Str>::nil()), v9);
                                                                                                let a114 = List::cons(if f53(v6.clone(), v5.clone()) {
                                                                                                    f16(List::cons(Str::lit("    use "), List::cons(v6, List::cons(Str::lit(" as Code;\u{a}    use "), List::cons(v5, List::cons(Str::lit("::{Refused, Response};"), List::<Str>::nil()))))))
                                                                                                } else {
                                                                                                    f16(List::cons(Str::lit("    use "), List::cons(v5, List::cons(Str::lit("::{Refused, Response};\u{a}    use "), List::cons(v6, List::cons(Str::lit(" as Code;"), List::<Str>::nil()))))))
                                                                                                }, List::cons(Str::lit("    let Ok(text) = core::str::from_utf8(response) else {\u{a}        return Err(Response(\"it is not UTF-8\".to_string()));\u{a}    };\u{a}    let mut fields = text.split('\\t');\u{a}    let (version, status, value) = (fields.next(), fields.next(), fields.next());\u{a}    if fields.next().is_some() {\u{a}        return Err(Response(format!(\"{text:?} has more than three fields\")));\u{a}    }\u{a}    if version != Some(VERSION) {\u{a}        return Err(Refused(Code::UnsupportedVersion));\u{a}    }\u{a}    match (status, value) {\u{a}        (Some(\"ok\"), Some(value)) => value\u{a}            .parse()\u{a}            .map_err(|_| Response(format!(\"{value:?} is not a signed 64-bit integer\"))),\u{a}        (Some(\"error\"), Some(code)) => Err(Refused(Code::from_code(code))),\u{a}        _ => Err(Response(format!(\"{text:?} is neither ok nor error\"))),\u{a}    }\u{a}}"), List::<Str>::nil()));
                                                                                                append_list(a113, a114)
                                                                                            };
                                                                                            append_list(a115, a116)
                                                                                        };
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
                                                };
                                                append_list(a137, a138)
                                            };
                                            append_list(a139, a140)
                                        };
                                        append_list(a141, a142)
                                    };
                                    append_list(a143, a144)
                                };
                                append_list(a145, a146)
                            };
                            append_list(a147, a148)
                        };
                        append_list(a149, a150)
                    };
                    append_list(a151, a152)
                };
                append_list(a153, a154)
            };
            append_list(a155, a156)
        };
        append_list(a157, a158)
    }))
}

pub fn f19(v0: Str) -> R<bool> {
    let v1: Bytes = {
        let a159 = v0;
        utf8_encode(a159)
    };
    let a163 = {
        let a161 = {
            let a160 = v1.clone();
            length_bytes(a160)
        };
        let a162 = 0u64;
        nat_eq(a161, a162)
    };
    if bool_not(a163) {
        let a164 = f54(f55(v1.clone(), 0u64, 1u64), f56(), 10u64);
        if bool_not(a164) {
            f57(v1.clone(), 0u64, {
                let a165 = v1;
                length_bytes(a165)
            })
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub fn f20(v0: Str) -> R<Str> {
    f58(v0, Bytes::lit(&[0x22]))
}

pub fn f21(v0: Bytes, v1: Str) -> Str {
    let m167 = {
        let a166 = v0;
        utf8_decode(a166)
    };
    match m167 {
        None => {
            v1
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f22(v0: Bytes, v1: u64, v2: bool, v3: u64) -> R<Bytes> {
    let m168 = v3;
    if m168 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m168 - 1;
        let v5: Bytes = f55(v0.clone(), v1, 1u64);
        if f59(v5.clone(), Bytes::lit(&[0x2d])) {
            let a171 = Bytes::lit(&[0x5f]);
            let a172 = f22(v0, {
                let a169 = v1;
                let a170 = 1u64;
                nat_add(a169, a170)?
            }, false, v4)?;
            Ok(append_bytes(a171, a172))
        } else {
            if f60(v5.clone()) {
                let a177 = if v2 {
                    let a173 = Bytes::lit(&[0x5f]);
                    let a174 = f61(v5);
                    append_bytes(a173, a174)
                } else {
                    f61(v5)
                };
                let a178 = f22(v0, {
                    let a175 = v1;
                    let a176 = 1u64;
                    nat_add(a175, a176)?
                }, false, v4)?;
                Ok(append_bytes(a177, a178))
            } else {
                let a181 = v5.clone();
                let a182 = f22(v0, {
                    let a179 = v1;
                    let a180 = 1u64;
                    nat_add(a179, a180)?
                }, f62(v5), v4)?;
                Ok(append_bytes(a181, a182))
            }
        }
    }
}

pub fn f23(v0: Str) -> Str {
    if f63(v0.clone(), f64()) {
        f16(List::cons(Str::lit("r#"), List::cons(v0, List::<Str>::nil())))
    } else {
        v0
    }
}

pub fn f24(v0: Str, v1: List<Adt7>) -> R<Option<(Str, Str)>> {
    let m183 = v1;
    match m183.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v2, v3)) => {
            Ok(f4(f5(v0.clone(), f6(match v2 {
                Adt7::C0(h184, _, _) => {
                    h184
                }
            })?)?, f24(v0, v3)?))
        }
    }
}

pub fn f25(v0: Str, p189: List<Adt7>) -> R<Option<(Str, Str)>> {
    let m188 = p189;
    {
        let mut m188 = m188;
        loop {
            let step = {
                let v1 = m188;
                let m185 = v1;
                match m185.uncons() {
                    None => {
                        Err::<List<Adt7>, R<Option<(Str, Str)>>>(Ok(None::<(Str, Str)>))
                    }
                    Some((v2, v3)) => {
                        let v4: Str = f7(match v2.clone() {
                            Adt7::C0(h186, _, _) => {
                                h186
                            }
                        })?;
                        if f63(v4.clone(), List::cons(Str::lit("new"), List::cons(v0.clone(), List::cons(Str::lit("dispatch"), List::cons(Str::lit("read"), List::<Str>::nil()))))) {
                            Err::<List<Adt7>, R<Option<(Str, Str)>>>(Ok(f65(match v2 {
                                Adt7::C0(h187, _, _) => {
                                    h187
                                }
                            }, v4)))
                        } else {
                            Ok::<List<Adt7>, R<Option<(Str, Str)>>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m188 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f26(v0: Adt4, v1: Adt6) -> Option<(Str, Str)> {
    if f66(Str::lit("unsupported-version"), match v1 {
        Adt6::C0(_, _, _, _, _, h190) => {
            h190
        }
    }) {
        None::<(Str, Str)>
    } else {
        Some((Str::lit("generate.model"), f16(List::cons(match v0 {
            Adt4::C0(h191, _, _, _, _, _, _, _) => {
                h191
            }
        }, List::cons(Str::lit("'s model has no unsupported-version error"), List::<Str>::nil())))))
    }
}

pub fn f27(v0: Str, v1: bool, v2: List<(Str, Str)>, v3: List<Adt3>) -> R<Option<(Str, Str)>> {
    let m192 = v3;
    match m192.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v4, v5)) => {
            Ok(f4(if v1 {
                f5(v0.clone(), f6(match v4.clone() {
                    Adt3::C0(h193, _) => {
                        h193
                    }
                })?)?
            } else {
                None::<(Str, Str)>
            }, f4(f28(v0.clone(), match v4 {
                Adt3::C0(_, h194) => {
                    h194
                }
            }, v2.clone()), f27(v0, v1, v2, v5)?)))
        }
    }
}

pub fn f28(v0: Str, v1: Str, v2: List<(Str, Str)>) -> Option<(Str, Str)> {
    let m195 = f67(v1.clone(), v2);
    match m195 {
        None => {
            Some((Str::lit("generate.world"), f16(List::cons(Str::lit("the "), List::cons(v0, List::cons(Str::lit(" writer has no type for WIT "), List::cons(v1, List::<Str>::nil())))))))
        }
        Some(_) => {
            None::<(Str, Str)>
        }
    }
}

pub fn f29(v0: List<Str>) -> Str {
    let a198 = {
        let a196 = v0;
        let a197 = List::cons(Str::lit(""), List::<Str>::nil());
        append_list(a196, a197)
    };
    let a199 = Str::lit("\u{a}");
    join(a198, a199)
}

pub fn f30(v0: Str, v1: Str, v2: Str) -> Str {
    f16(List::cons(v0.clone(), List::cons(Str::lit(" Generated by `"), List::cons(v1.clone(), List::cons(Str::lit("` from "), List::cons(v2, List::cons(Str::lit(".\u{a}"), List::cons(v0, List::cons(Str::lit(" Do not edit: run `"), List::cons(v1, List::cons(Str::lit("` again.\u{a}"), List::<Str>::nil())))))))))))
}

pub fn f31(v0: List<Str>) -> List<Str> {
    let m200 = v0;
    match m200.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(f68(f16(List::cons(Str::lit("//! "), List::cons(v1, List::<Str>::nil())))), f31(v2))
        }
    }
}

pub fn f32(v0: List<Str>) -> List<Str> {
    let m201 = v0;
    match m201.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            f69(v1, f32(v2))
        }
    }
}

pub fn f33(v0: Adt1) -> Str {
    f16(List::cons(Str::lit("the WIT world "), List::cons(match v0.clone() {
        Adt1::C0(h202, _, _, _, _, _, _, _, _) => {
            h202
        }
    }, List::cons(Str::lit(" ("), List::cons(match v0 {
        Adt1::C0(_, _, _, h203, _, _, _, _, _) => {
            h203
        }
    }, List::cons(Str::lit(")"), List::<Str>::nil()))))))
}

pub fn f34(v0: List<Adt2>) -> List<Str> {
    let m204 = v0;
    match m204.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt2::C0(h205, _, _, _) => {
                    h205
                }
            }, f34(v2))
        }
    }
}

pub fn f35(v0: u64) -> R<Str> {
    Ok(f70(v0, {
        let a206 = v0;
        let a207 = 1u64;
        nat_add(a206, a207)?
    }))
}

pub fn f36(v0: List<Str>) -> R<List<Str>> {
    let m208 = v0;
    match m208.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f20(v1)?, f36(v2)?))
        }
    }
}

pub fn f37(v0: List<Adt2>) -> R<List<Str>> {
    let m209 = v0;
    match m209.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a215 = f40(Str::lit("    "), match v1.clone() {
                Adt2::C0(_, _, _, h210) => {
                    h210
                }
            });
            let a216 = List::cons(f16(List::cons(Str::lit("    fn "), List::cons(f23(match v1.clone() {
                Adt2::C0(h211, _, _, _) => {
                    h211
                }
            }), List::cons(Str::lit("(&mut self"), List::cons(f71(match v1.clone() {
                Adt2::C0(_, h212, _, _) => {
                    h212
                }
            })?, List::cons(Str::lit(") -> Result<"), List::cons({
                let m214 = match v1 {
                    Adt2::C0(_, _, h213, _) => {
                        h213
                    }
                };
                match m214 {
                    None => {
                        Str::lit("()")
                    }
                    Some(v3) => {
                        f72(v3)
                    }
                }
            }, List::cons(Str::lit(", Self::Error>;"), List::<Str>::nil())))))))), List::cons(Str::lit(""), f37(v2)?));
            Ok(append_list(a215, a216))
        }
    }
}

pub fn f38() -> Str {
    Str::lit("/// Why a request did not come back from the guest.\u{a}#[derive(Debug)]\u{a}pub enum DispatchError<E> {\u{a}    /// The request is longer than a guest can address.\u{a}    TooLong(usize),\u{a}    /// A call into the guest failed, as the engine reported it.\u{a}    Guest(E),\u{a}}\u{a}\u{a}impl<E: fmt::Display> fmt::Display for DispatchError<E> {\u{a}    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {\u{a}        match self {\u{a}            Self::TooLong(length) => write!(f, \"a request of {length} bytes is too long\"),\u{a}            Self::Guest(error) => write!(f, \"the guest failed: {error}\"),\u{a}        }\u{a}    }\u{a}}\u{a}\u{a}impl<E: fmt::Debug + fmt::Display> std::error::Error for DispatchError<E> {}\u{a}\u{a}/// Why [`dispatch`] failed for a guest `G`.\u{a}pub type Failure<G> = DispatchError<<G as Guest>::Error>;\u{a}\u{a}/// Send one request to a fresh instance of the guest and return a copy of its response.\u{a}/// The guest allocator never frees, so give every request its own instance.\u{a}pub fn dispatch<G: Guest>(guest: &mut G, request: &[u8]) -> Result<Vec<u8>, Failure<G>> {\u{a}    let length = u32::try_from(request.len()).map_err(|_| DispatchError::TooLong(request.len()))?;\u{a}    let failed = DispatchError::Guest;\u{a}    let offset = guest.holo_alloc(length).map_err(failed)?;\u{a}    guest.write_memory(offset, request).map_err(failed)?;\u{a}    let packed = guest.holo_run(offset, length).map_err(failed)?;\u{a}    // The response location is packed as `offset << 32 | length`.\u{a}    let at = (packed >> 32) as u32;\u{a}    let size = (packed & 0xffff_ffff) as u32;\u{a}    guest.read_memory(at, size).map_err(failed)\u{a}}")
}

pub fn f39(v0: Adt4) -> Str {
    f16(List::cons(match v0.clone() {
        Adt4::C0(h217, _, _, _, _, _, _, _) => {
            h217
        }
    }, List::cons(Str::lit("'s model ("), List::cons(match v0.clone() {
        Adt4::C0(_, _, _, _, h218, _, _, _) => {
            h218
        }
    }, List::cons(Str::lit(", blake3:"), List::cons(match v0 {
        Adt4::C0(_, _, _, _, _, h219, _, _) => {
            h219
        }
    }, List::cons(Str::lit(")"), List::<Str>::nil())))))))
}

pub fn f40(v0: Str, v1: List<Str>) -> List<Str> {
    let m220 = v1;
    match m220.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v2, v3)) => {
            List::cons(f68(f16(List::cons(v0.clone(), List::cons(Str::lit("/// "), List::cons(v2, List::<Str>::nil()))))), f40(v0, v3))
        }
    }
}

pub fn f41(v0: Adt4, v1: Str) -> List<Str> {
    let m222 = match v0 {
        Adt4::C0(_, _, _, h221, _, _, _, _) => {
            h221
        }
    };
    match m222 {
        None => {
            List::cons(v1, List::<Str>::nil())
        }
        Some(v2) => {
            List::cons(v1, List::cons(Str::lit(""), List::cons(v2, List::<Str>::nil())))
        }
    }
}

pub fn f42(v0: List<Adt7>) -> R<List<Str>> {
    let m223 = v0;
    match m223.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f16(List::cons(Str::lit("    /// "), List::cons(match v1.clone() {
                Adt7::C0(_, h224, _) => {
                    h224
                }
            }, List::cons(Str::lit(".\u{a}    "), List::cons(f73(match v1 {
                Adt7::C0(h225, _, _) => {
                    h225
                }
            })?, List::cons(Str::lit(","), List::<Str>::nil())))))), f42(v2)?))
        }
    }
}

pub fn f43(v0: Str, v1: Str, v2: List<Str>) -> List<Str> {
    let v3: Str = {
        let a226 = v2.clone();
        let a227 = Str::lit(", ");
        join(a226, a227)
    };
    let v4: Str = f16(List::cons(v0.clone(), List::cons(v1.clone(), List::cons(Str::lit(" = ["), List::cons(v3.clone(), List::cons(Str::lit("];"), List::<Str>::nil()))))));
    if if f74(v4.clone()) {
        let a229 = {
            let a228 = v3;
            length_string(a228)
        };
        let a230 = 60u64;
        nat_le(a229, a230)
    } else {
        false
    } {
        List::cons(v4, List::<Str>::nil())
    } else {
        let a231 = List::cons(f16(List::cons(v0.clone(), List::cons(v1, List::cons(Str::lit(" = ["), List::<Str>::nil())))), f75(f16(List::cons(v0.clone(), List::cons(Str::lit("    "), List::<Str>::nil()))), v2, Str::lit(",")));
        let a232 = List::cons(f16(List::cons(v0, List::cons(Str::lit("];"), List::<Str>::nil()))), List::<Str>::nil());
        append_list(a231, a232)
    }
}

pub fn f44(v0: List<Adt7>) -> R<List<Str>> {
    let m233 = v0;
    match m233.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f16(List::cons(Str::lit("Self::"), List::cons(f73(match v1 {
                Adt7::C0(h234, _, _) => {
                    h234
                }
            })?, List::<Str>::nil()))), f44(v2)?))
        }
    }
}

pub fn f45(v0: List<Adt7>) -> R<List<Str>> {
    let m235 = v0;
    match m235.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a238 = f49(Str::lit("            "), f16(List::cons(Str::lit("Self::"), List::cons(f73(match v1.clone() {
                Adt7::C0(h236, _, _) => {
                    h236
                }
            })?, List::<Str>::nil()))), f20(match v1 {
                Adt7::C0(h237, _, _) => {
                    h237
                }
            })?);
            let a239 = f45(v2)?;
            Ok(append_list(a238, a239))
        }
    }
}

pub fn f46(v0: Adt4, v1: Str, v2: List<Adt8>) -> R<List<Str>> {
    let m240 = v2;
    match m240.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v3, v4)) => {
            Ok(List::cons(f16(List::cons(Str::lit("    /// `"), List::cons(match v3.clone() {
                Adt8::C0(h241, _) => {
                    h241
                }
            }, List::cons(Str::lit("`: "), List::cons(f76(v0.clone(), v1.clone(), v3.clone()), List::cons(Str::lit("\u{a}    "), List::cons(f73(match v3 {
                Adt8::C0(h242, _) => {
                    h242
                }
            })?, List::cons(Str::lit(","), List::<Str>::nil())))))))), f46(v0, v1, v4)?))
        }
    }
}

pub fn f47(v0: List<Adt8>) -> R<List<Str>> {
    let m243 = v0;
    match m243.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a246 = f49(Str::lit("            "), f16(List::cons(Str::lit("Self::"), List::cons(f73(match v1.clone() {
                Adt8::C0(h244, _) => {
                    h244
                }
            })?, List::<Str>::nil()))), f20(match v1 {
                Adt8::C0(h245, _) => {
                    h245
                }
            })?);
            let a247 = f47(v2)?;
            Ok(append_list(a246, a247))
        }
    }
}

pub fn f48(v0: Adt4, v1: Str, v2: List<Adt8>) -> R<List<Str>> {
    let m248 = v2;
    match m248.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v3, v4)) => {
            let a250 = f49(Str::lit("            "), f16(List::cons(Str::lit("Self::"), List::cons(f73(match v3.clone() {
                Adt8::C0(h249, _) => {
                    h249
                }
            })?, List::<Str>::nil()))), f20(f76(v0.clone(), v1.clone(), v3))?);
            let a251 = f48(v0, v1, v4)?;
            Ok(append_list(a250, a251))
        }
    }
}

pub fn f49(v0: Str, v1: Str, v2: Str) -> List<Str> {
    let v3: Str = f16(List::cons(v0.clone(), List::cons(v1.clone(), List::cons(Str::lit(" => "), List::cons(v2.clone(), List::cons(Str::lit(","), List::<Str>::nil()))))));
    if f74(v3.clone()) {
        List::cons(v3, List::<Str>::nil())
    } else {
        List::cons(f16(List::cons(v0.clone(), List::cons(v1, List::cons(Str::lit(" => {"), List::<Str>::nil())))), List::cons(f16(List::cons(v0.clone(), List::cons(Str::lit("    "), List::cons(v2, List::<Str>::nil())))), List::cons(f16(List::cons(v0, List::cons(Str::lit("}"), List::<Str>::nil()))), List::<Str>::nil())))
    }
}

pub fn f50(v0: List<Adt8>) -> R<List<Str>> {
    let m252 = v0;
    match m252.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            let a255 = f49(Str::lit("            "), f20(match v1.clone() {
                Adt8::C0(h253, _) => {
                    h253
                }
            })?, f16(List::cons(Str::lit("Self::"), List::cons(f73(match v1 {
                Adt8::C0(h254, _) => {
                    h254
                }
            })?, List::<Str>::nil()))));
            let a256 = f50(v2)?;
            Ok(append_list(a255, a256))
        }
    }
}

pub fn f51(v0: Str, v1: Str, v2: Str, v3: List<Adt7>) -> R<List<Str>> {
    let m257 = v3;
    match m257.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v4, v5)) => {
            let v6: Str = f73(match v4.clone() {
                Adt7::C0(h258, _, _) => {
                    h258
                }
            })?;
            let a265 = List::cons(Str::lit(""), List::cons(f16(List::cons(Str::lit("    /// "), List::cons(match v4.clone() {
                Adt7::C0(_, h259, _) => {
                    h259
                }
            }, List::cons(Str::lit(": `"), List::cons(v0.clone(), List::cons(Str::lit("("), List::cons(v1.clone(), List::cons(Str::lit("::"), List::cons(v6.clone(), List::cons(Str::lit(", left, right)`."), List::<Str>::nil())))))))))), List::<Str>::nil()));
            let a266 = {
                let a263 = f52(Str::lit("    "), f16(List::cons(Str::lit("pub fn "), List::cons(f7(match v4 {
                    Adt7::C0(h260, _, _) => {
                        h260
                    }
                })?, List::<Str>::nil()))), List::cons(Str::lit("&mut self"), List::cons(Str::lit("left: i64"), List::cons(Str::lit("right: i64"), List::<Str>::nil()))), v2.clone());
                let a264 = {
                    let a261 = List::cons(f16(List::cons(Str::lit("        self."), List::cons(v0.clone(), List::cons(Str::lit("("), List::cons(v1.clone(), List::cons(Str::lit("::"), List::cons(v6, List::cons(Str::lit(", left, right)"), List::<Str>::nil())))))))), List::cons(Str::lit("    }"), List::<Str>::nil()));
                    let a262 = f51(v0, v1, v2, v5)?;
                    append_list(a261, a262)
                };
                append_list(a263, a264)
            };
            Ok(append_list(a265, a266))
        }
    }
}

pub fn f52(v0: Str, v1: Str, v2: List<Str>, v3: Str) -> List<Str> {
    let v4: Str = f16(List::cons(v0.clone(), List::cons(v1.clone(), List::cons(Str::lit("("), List::cons({
        let a267 = v2.clone();
        let a268 = Str::lit(", ");
        join(a267, a268)
    }, List::cons(Str::lit(") -> "), List::cons(v3.clone(), List::cons(Str::lit(" {"), List::<Str>::nil()))))))));
    if f74(v4.clone()) {
        List::cons(v4, List::<Str>::nil())
    } else {
        let a269 = List::cons(f16(List::cons(v0.clone(), List::cons(v1, List::cons(Str::lit("("), List::<Str>::nil())))), f75(f16(List::cons(v0.clone(), List::cons(Str::lit("    "), List::<Str>::nil()))), v2, Str::lit(",")));
        let a270 = List::cons(f16(List::cons(v0, List::cons(Str::lit(") -> "), List::cons(v3, List::cons(Str::lit(" {"), List::<Str>::nil()))))), List::<Str>::nil());
        append_list(a269, a270)
    }
}

pub fn f53(v0: Str, v1: Str) -> bool {
    let a277 = {
        let a273 = {
            let a271 = v0;
            utf8_encode(a271)
        };
        let a274 = {
            let a272 = v1;
            utf8_encode(a272)
        };
        compare_bytes(a273, a274)
    };
    let a278 = {
        let a275 = Bytes::lit(&[0x00]);
        let a276 = Bytes::lit(&[0x01]);
        compare_bytes(a275, a276)
    };
    equal(a277, a278)
}

pub fn f54(v0: Bytes, v1: Bytes, p281: u64) -> bool {
    let m280 = p281;
    {
        let mut m280 = m280;
        loop {
            let step = {
                let v2 = m280;
                let m279 = v2;
                if m279 == 0 {
                    Err::<u64, bool>(false)
                } else {
                    let v3 = m279 - 1;
                    if f59(f55(v1.clone(), v3, 1u64), v0.clone()) {
                        Err::<u64, bool>(true)
                    } else {
                        Ok::<u64, bool>(v3)
                    }
                }
            };
            match step {
                Ok(next) => { m280 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f55(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m285 = {
        let a282 = v0;
        let a283 = v1;
        let a284 = v2;
        slice_bytes(a282, a283, a284)
    };
    match m285 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f56() -> Bytes {
    Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39])
}

pub fn f57(v0: Bytes, p290: u64, p291: u64) -> R<bool> {
    let m289 = (p290, p291);
    {
        let mut m289 = m289;
        loop {
            let step = {
                let (v1, v2) = m289;
                let m286 = v2;
                if m286 == 0 {
                    Err::<(u64, u64), R<bool>>(Ok(true))
                } else {
                    let v3 = m286 - 1;
                    if if f62(f55(v0.clone(), v1, 1u64)) {
                        true
                    } else {
                        f59(f55(v0.clone(), v1, 1u64), Bytes::lit(&[0x5f]))
                    } {
                        Ok::<(u64, u64), R<bool>>(({
                            let a287 = v1;
                            let a288 = 1u64;
                            nat_add(a287, a288)?
                        }, v3))
                    } else {
                        Err::<(u64, u64), R<bool>>(Ok(false))
                    }
                }
            };
            match step {
                Ok(next) => { m289 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f58(v0: Str, v1: Bytes) -> R<Str> {
    Ok(f21({
        let a297 = {
            let a295 = v1.clone();
            let a296 = f77({
                let a292 = v0.clone();
                utf8_encode(a292)
            }, 0u64, v1.clone(), {
                let a294 = {
                    let a293 = v0.clone();
                    utf8_encode(a293)
                };
                length_bytes(a294)
            })?;
            append_bytes(a295, a296)
        };
        let a298 = v1;
        append_bytes(a297, a298)
    }, v0))
}

pub fn f59(v0: Bytes, v1: Bytes) -> bool {
    let a303 = {
        let a299 = v0;
        let a300 = v1;
        compare_bytes(a299, a300)
    };
    let a304 = {
        let a301 = Bytes::lit(&[0x00]);
        let a302 = Bytes::lit(&[0x00]);
        compare_bytes(a301, a302)
    };
    equal(a303, a304)
}

pub fn f60(v0: Bytes) -> bool {
    let a305 = f59(f61(v0.clone()), v0);
    bool_not(a305)
}

pub fn f61(v0: Bytes) -> Bytes {
    f78(v0, f79(), f80(), 26u64)
}

pub fn f62(v0: Bytes) -> bool {
    if f54(v0.clone(), f80(), 26u64) {
        true
    } else {
        f54(v0, f56(), 10u64)
    }
}

pub fn f63(v0: Str, p310: List<Str>) -> bool {
    let m309 = p310;
    {
        let mut m309 = m309;
        loop {
            let step = {
                let v1 = m309;
                let m306 = v1;
                match m306.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a307 = v2;
                        let a308 = v0.clone();
                        if equal(a307, a308) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m309 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f64() -> List<Str> {
    List::cons(Str::lit("as"), List::cons(Str::lit("break"), List::cons(Str::lit("const"), List::cons(Str::lit("continue"), List::cons(Str::lit("crate"), List::cons(Str::lit("else"), List::cons(Str::lit("enum"), List::cons(Str::lit("extern"), List::cons(Str::lit("false"), List::cons(Str::lit("fn"), List::cons(Str::lit("for"), List::cons(Str::lit("if"), List::cons(Str::lit("impl"), List::cons(Str::lit("in"), List::cons(Str::lit("let"), List::cons(Str::lit("loop"), List::cons(Str::lit("match"), List::cons(Str::lit("mod"), List::cons(Str::lit("move"), List::cons(Str::lit("mut"), List::cons(Str::lit("pub"), List::cons(Str::lit("ref"), List::cons(Str::lit("return"), List::cons(Str::lit("self"), List::cons(Str::lit("Self"), List::cons(Str::lit("static"), List::cons(Str::lit("struct"), List::cons(Str::lit("super"), List::cons(Str::lit("trait"), List::cons(Str::lit("true"), List::cons(Str::lit("type"), List::cons(Str::lit("unsafe"), List::cons(Str::lit("use"), List::cons(Str::lit("where"), List::cons(Str::lit("while"), List::cons(Str::lit("async"), List::cons(Str::lit("await"), List::cons(Str::lit("dyn"), List::cons(Str::lit("abstract"), List::cons(Str::lit("become"), List::cons(Str::lit("box"), List::cons(Str::lit("do"), List::cons(Str::lit("final"), List::cons(Str::lit("macro"), List::cons(Str::lit("override"), List::cons(Str::lit("priv"), List::cons(Str::lit("typeof"), List::cons(Str::lit("unsized"), List::cons(Str::lit("virtual"), List::cons(Str::lit("yield"), List::cons(Str::lit("try"), List::cons(Str::lit("gen"), List::<Str>::nil()))))))))))))))))))))))))))))))))))))))))))))))))))))
}

pub fn f65(v0: Str, v1: Str) -> Option<(Str, Str)> {
    Some((Str::lit("generate.model"), f16(List::cons(Str::lit("operation "), List::cons(v0, List::cons(Str::lit(" collides with the method "), List::cons(v1, List::<Str>::nil())))))))
}

pub fn f66(v0: Str, p316: List<Adt8>) -> bool {
    let m315 = p316;
    {
        let mut m315 = m315;
        loop {
            let step = {
                let v1 = m315;
                let m311 = v1;
                match m311.uncons() {
                    None => {
                        Err::<List<Adt8>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a313 = match v2 {
                            Adt8::C0(h312, _) => {
                                h312
                            }
                        };
                        let a314 = v0.clone();
                        if equal(a313, a314) {
                            Err::<List<Adt8>, bool>(true)
                        } else {
                            Ok::<List<Adt8>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m315 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f67(v0: Str, p323: List<(Str, Str)>) -> Option<Str> {
    let m322 = p323;
    {
        let mut m322 = m322;
        loop {
            let step = {
                let v1 = m322;
                let m317 = v1;
                match m317.uncons() {
                    None => {
                        Err::<List<(Str, Str)>, Option<Str>>(None::<Str>)
                    }
                    Some((v2, v3)) => {
                        let a319 = {
                            let (h318, _) = v2;
                            h318
                        };
                        let a320 = v0.clone();
                        if equal(a319, a320) {
                            Err::<List<(Str, Str)>, Option<Str>>(Some({
                                let (_, h321) = v2;
                                h321
                            }))
                        } else {
                            Ok::<List<(Str, Str)>, Option<Str>>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m322 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f68(v0: Str) -> Str {
    f21(f81({
        let a324 = v0.clone();
        utf8_encode(a324)
    }, {
        let a326 = {
            let a325 = v0.clone();
            utf8_encode(a325)
        };
        length_bytes(a326)
    }), v0)
}

pub fn f69(v0: Str, v1: List<Str>) -> List<Str> {
    let m327 = v1;
    match m327.uncons() {
        None => {
            List::cons(v0, List::<Str>::nil())
        }
        Some((v2, v3)) => {
            if f53(v0.clone(), v2.clone()) {
                List::cons(v0, List::cons(v2, v3))
            } else {
                List::cons(v2, f69(v0, v3))
            }
        }
    }
}

pub fn f70(v0: u64, v1: u64) -> Str {
    let m328 = v1;
    if m328 == 0 {
        Str::lit("")
    } else {
        let v2 = m328 - 1;
        let a329 = v0;
        let a330 = 10u64;
        if nat_lt(a329, a330) {
            f82(v0)
        } else {
            f16(List::cons(f70({
                let a331 = v0;
                let a332 = 10u64;
                let a333 = 0u64;
                nat_quot(a331, a332, a333)
            }, v2), List::cons(f82({
                let a334 = v0;
                let a335 = 10u64;
                let a336 = 0u64;
                nat_rem(a334, a335, a336)
            }), List::<Str>::nil())))
        }
    }
}

pub fn f71(v0: List<Adt3>) -> R<Str> {
    let m337 = v0;
    match m337.uncons() {
        None => {
            Ok(Str::lit(""))
        }
        Some((v1, v2)) => {
            Ok(f16(List::cons(Str::lit(", "), List::cons(f7(match v1.clone() {
                Adt3::C0(h338, _) => {
                    h338
                }
            })?, List::cons(Str::lit(": "), List::cons(f72(match v1 {
                Adt3::C0(_, h339) => {
                    h339
                }
            }), List::cons(f71(v2)?, List::<Str>::nil())))))))
        }
    }
}

pub fn f72(v0: Str) -> Str {
    let m340 = f67(v0.clone(), f11());
    match m340 {
        None => {
            v0
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f73(v0: Str) -> R<Str> {
    Ok(f21(f83({
        let a341 = v0.clone();
        utf8_encode(a341)
    }, 0u64, true, {
        let a343 = {
            let a342 = v0.clone();
            utf8_encode(a342)
        };
        length_bytes(a343)
    })?, v0))
}

pub fn f74(v0: Str) -> bool {
    let a345 = {
        let a344 = v0;
        length_string(a344)
    };
    let a346 = 100u64;
    nat_le(a345, a346)
}

pub fn f75(v0: Str, v1: List<Str>, v2: Str) -> List<Str> {
    let m347 = v1;
    match m347.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v3, v4)) => {
            List::cons(f16(List::cons(v0.clone(), List::cons(v3, List::cons(v2.clone(), List::<Str>::nil())))), f75(v0, v4, v2))
        }
    }
}

pub fn f76(v0: Adt4, v1: Str, v2: Adt8) -> Str {
    let m349 = match v2.clone() {
        Adt8::C0(_, h348) => {
            h348
        }
    };
    match m349 {
        None => {
            let a351 = match v2.clone() {
                Adt8::C0(h350, _) => {
                    h350
                }
            };
            let a352 = Str::lit("unsupported-version");
            if equal(a351, a352) {
                f16(List::cons(match v0 {
                    Adt4::C0(h353, _, _, _, _, _, _, _) => {
                        h353
                    }
                }, List::cons(Str::lit(" does not answer request version "), List::cons(v1, List::cons(Str::lit("."), List::<Str>::nil())))))
            } else {
                let a355 = match v2.clone() {
                    Adt8::C0(h354, _) => {
                        h354
                    }
                };
                let a356 = Str::lit("unknown-operation");
                if equal(a355, a356) {
                    f16(List::cons(match v0 {
                        Adt4::C0(h357, _, _, _, _, _, _, _) => {
                            h357
                        }
                    }, List::cons(Str::lit(" does not know this operation."), List::<Str>::nil())))
                } else {
                    f16(List::cons(match v0 {
                        Adt4::C0(h358, _, _, _, _, _, _, _) => {
                            h358
                        }
                    }, List::cons(Str::lit(" refused the request: "), List::cons(match v2 {
                        Adt8::C0(h359, _) => {
                            h359
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

pub fn f77(v0: Bytes, v1: u64, v2: Bytes, v3: u64) -> R<Bytes> {
    let m360 = v3;
    if m360 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m360 - 1;
        let a365 = if f59(f55(v0.clone(), v1, 1u64), Bytes::lit(&[0x5c])) {
            Bytes::lit(&[0x5c, 0x5c])
        } else {
            if f59(f55(v0.clone(), v1, 1u64), v2.clone()) {
                let a361 = Bytes::lit(&[0x5c]);
                let a362 = v2.clone();
                append_bytes(a361, a362)
            } else {
                if f59(f55(v0.clone(), v1, 1u64), Bytes::lit(&[0x0a])) {
                    Bytes::lit(&[0x5c, 0x6e])
                } else {
                    f55(v0.clone(), v1, 1u64)
                }
            }
        };
        let a366 = f77(v0, {
            let a363 = v1;
            let a364 = 1u64;
            nat_add(a363, a364)?
        }, v2, v4)?;
        Ok(append_bytes(a365, a366))
    }
}

pub fn f78(v0: Bytes, v1: Bytes, v2: Bytes, p369: u64) -> Bytes {
    let m368 = p369;
    {
        let mut m368 = m368;
        loop {
            let step = {
                let v3 = m368;
                let m367 = v3;
                if m367 == 0 {
                    Err::<u64, Bytes>(v0.clone())
                } else {
                    let v4 = m367 - 1;
                    if f59(f55(v1.clone(), v4, 1u64), v0.clone()) {
                        Err::<u64, Bytes>(f55(v2.clone(), v4, 1u64))
                    } else {
                        Ok::<u64, Bytes>(v4)
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

pub fn f79() -> Bytes {
    Bytes::lit(&[0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a])
}

pub fn f80() -> Bytes {
    Bytes::lit(&[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a])
}

pub fn f81(p372: Bytes, p373: u64) -> Bytes {
    let m371 = (p372, p373);
    {
        let mut m371 = m371;
        loop {
            let step = {
                let (v0, v1) = m371;
                let m370 = v1;
                if m370 == 0 {
                    Err::<(Bytes, u64), Bytes>(Bytes::lit(&[]))
                } else {
                    let v2 = m370 - 1;
                    if f59(f55(v0.clone(), v2, 1u64), Bytes::lit(&[0x20])) {
                        Ok::<(Bytes, u64), Bytes>((f55(v0, 0u64, v2), v2))
                    } else {
                        Err::<(Bytes, u64), Bytes>(v0)
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

pub fn f82(v0: u64) -> Str {
    f21(f55(f56(), v0, 1u64), Str::lit("0"))
}

pub fn f83(v0: Bytes, v1: u64, v2: bool, v3: u64) -> R<Bytes> {
    let m374 = v3;
    if m374 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m374 - 1;
        if f84(f55(v0.clone(), v1, 1u64)) {
            f83(v0, {
                let a375 = v1;
                let a376 = 1u64;
                nat_add(a375, a376)?
            }, true, v4)
        } else {
            let a379 = if v2 {
                f85(f55(v0.clone(), v1, 1u64))
            } else {
                f55(v0.clone(), v1, 1u64)
            };
            let a380 = f83(v0, {
                let a377 = v1;
                let a378 = 1u64;
                nat_add(a377, a378)?
            }, false, v4)?;
            Ok(append_bytes(a379, a380))
        }
    }
}

pub fn f84(v0: Bytes) -> bool {
    if f59(v0.clone(), Bytes::lit(&[0x2d])) {
        true
    } else {
        f59(v0, Bytes::lit(&[0x5f]))
    }
}

pub fn f85(v0: Bytes) -> Bytes {
    f78(v0, f80(), f79(), 26u64)
}

pub fn run(p0: Adt0) -> R<Adt9> {
    f0(p0)
}
