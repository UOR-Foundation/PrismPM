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
    C0(Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, Str, List<Adt1>, List<Adt2>, List<u64>, Str, Str, Str),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt2 {
    C0(Str, Str, Str, Str, List<Str>, List<Str>),
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Str),
    C1(Str, Str),
}

pub fn f0(v0: Adt0) -> R<Adt3> {
    let m1 = f1(v0.clone())?;
    match m1 {
        None => {
            Ok(Adt3::C0(f2(v0)?))
        }
        Some(v1) => {
            Ok(Adt3::C1({
                let (h2, _) = v1;
                h2
            }, {
                let (_, h3) = v1;
                h3
            }))
        }
    }
}

pub fn f1(v0: Adt0) -> R<Option<(Str, Str)>> {
    Ok(f3(f4(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h4, _, _, _, _) => {
            h4
        }
    }), f3(f5(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h5, _, _, _, _, _) => {
            h5
        }
    }), f3(f6(List::cons(match v0.clone() {
        Adt0::C0(h6, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h6
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, h7, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h7
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, h8, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h8
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, h9, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h9
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, h10, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h10
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, h11, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h11
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, h12, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h12
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, h13, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h13
        }
    }, List::cons(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h14) => {
            h14
        }
    }, {
        let a17 = f7(match v0.clone() {
            Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h15, _, _, _, _, _) => {
                h15
            }
        });
        let a18 = f8(match v0.clone() {
            Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h16, _, _, _, _) => {
                h16
            }
        });
        append_list(a17, a18)
    }))))))))))?, f3(f9(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, h19, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h19
        }
    })?, f3(f10(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h20, _, _, _, _, _) => {
            h20
        }
    })?, f3(f11(Str::lit("subject.blake3"), match v0.clone() {
        Adt0::C0(_, h21, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h21
        }
    }, 64u64)?, f3(f11(Str::lit("subject.sha256"), match v0.clone() {
        Adt0::C0(_, _, h22, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h22
        }
    }, 64u64)?, f3(f11(Str::lit("verifier.lean.commit"), match v0.clone() {
        Adt0::C0(_, _, _, _, h23, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h23
        }
    }, 40u64)?, f3(f11(Str::lit("verifier.lexlean.commit"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, h24, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h24
        }
    }, 40u64)?, f3(f11(Str::lit("verifier.lexlean.crateSha256"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, h25, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h25
        }
    }, 64u64)?, f3(f11(Str::lit("the lean4-prod commit"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, h26, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h26
        }
    }, 40u64)?, f3(f11(Str::lit("model.snapshotId"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h27, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h27
        }
    }, 64u64)?, f3(f11(Str::lit("model.document.sha256"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h28, _, _, _, _, _, _, _, _, _, _, _) => {
            h28
        }
    }, 64u64)?, f3(f11(Str::lit("model.document.blake3"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h29, _, _, _, _, _, _, _, _, _, _) => {
            h29
        }
    }, 64u64)?, f3(f11(Str::lit("model.sourceId"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h30, _, _, _, _, _, _, _, _, _) => {
            h30
        }
    }, 64u64)?, f3(f11(Str::lit("model.semanticId"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h31, _, _, _, _, _, _, _, _) => {
            h31
        }
    }, 64u64)?, f3(f11(Str::lit("model.compilerSemanticsId"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h32, _, _, _, _, _, _, _) => {
            h32
        }
    }, 64u64)?, f3(f11(Str::lit("model.leanManifestSha256"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h33, _, _, _, _, _, _) => {
            h33
        }
    }, 64u64)?, f3(f11(Str::lit("evidence.lexleanAttestation.sha256"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h34, _, _) => {
            h34
        }
    }, 64u64)?, f11(Str::lit("evidence.verificationManifest.sha256"), match v0 {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h35, _) => {
            h35
        }
    }, 64u64)?))))))))))))))))))))
}

pub fn f2(v0: Adt0) -> R<Str> {
    Ok(f12(List::cons(f13(Str::lit("_type"), f14(Str::lit("https://in-toto.io/Statement/v1"))?)?, List::cons(f13(Str::lit("predicate"), f15(v0.clone())?)?, List::cons(f13(Str::lit("predicateType"), f14(Str::lit("https://uor.foundation/prism/proof/v1"))?)?, List::cons(f13(Str::lit("subject"), f16(List::cons(f12(List::cons(f13(Str::lit("digest"), f17(match v0.clone() {
        Adt0::C0(_, h36, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h36
        }
    }, match v0.clone() {
        Adt0::C0(_, _, h37, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h37
        }
    })?)?, List::cons(f13(Str::lit("name"), f14(match v0 {
        Adt0::C0(h38, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h38
        }
    })?)?, List::<Str>::nil()))), List::<Str>::nil())))?, List::<Str>::nil()))))))
}

pub fn f3(v0: Option<(Str, Str)>, v1: Option<(Str, Str)>) -> Option<(Str, Str)> {
    let m39 = v0.clone();
    match m39 {
        None => {
            v1
        }
        Some(_) => {
            v0
        }
    }
}

pub fn f4(v0: List<Adt2>) -> Option<(Str, Str)> {
    let m40 = v0;
    match m40.uncons() {
        None => {
            Some((Str::lit("proof.theorems"), Str::lit("the model declares no theorem, and a proof statement that lists none claims nothing")))
        }
        Some((_, _)) => {
            None::<(Str, Str)>
        }
    }
}

pub fn f5(v0: List<Adt1>) -> Option<(Str, Str)> {
    let m41 = v0;
    match m41.uncons() {
        None => {
            Some((Str::lit("proof.input"), Str::lit("the build lists no generated Lean module")))
        }
        Some((_, _)) => {
            None::<(Str, Str)>
        }
    }
}

pub fn f6(p44: List<Str>) -> R<Option<(Str, Str)>> {
    let m43 = p44;
    {
        let mut m43 = m43;
        loop {
            let step = {
                let v0 = m43;
                let m42 = v0;
                match m42.uncons() {
                    None => {
                        Err::<List<Str>, R<Option<(Str, Str)>>>(Ok(None::<(Str, Str)>))
                    }
                    Some((v1, v2)) => {
                        if f18(v1.clone())? {
                            Ok::<List<Str>, R<Option<(Str, Str)>>>(v2)
                        } else {
                            Err::<List<Str>, R<Option<(Str, Str)>>>(Ok(Some((Str::lit("proof.input"), f19(List::cons(Str::lit("the text "), List::cons(f20(v1)?, List::cons(Str::lit(" is empty or holds a control character"), List::<Str>::nil()))))))))
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m43 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f7(v0: List<Adt1>) -> List<Str> {
    let m45 = v0;
    match m45.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1 {
                Adt1::C0(h46, _) => {
                    h46
                }
            }, f7(v2))
        }
    }
}

pub fn f8(v0: List<Adt2>) -> List<Str> {
    let m47 = v0;
    match m47.uncons() {
        None => {
            List::<Str>::nil()
        }
        Some((v1, v2)) => {
            List::cons(match v1.clone() {
                Adt2::C0(h48, _, _, _, _, _) => {
                    h48
                }
            }, List::cons(match v1.clone() {
                Adt2::C0(_, h49, _, _, _, _) => {
                    h49
                }
            }, List::cons(match v1.clone() {
                Adt2::C0(_, _, h50, _, _, _) => {
                    h50
                }
            }, List::cons(match v1.clone() {
                Adt2::C0(_, _, _, h51, _, _) => {
                    h51
                }
            }, {
                let a56 = match v1.clone() {
                    Adt2::C0(_, _, _, _, h52, _) => {
                        h52
                    }
                };
                let a57 = {
                    let a54 = match v1 {
                        Adt2::C0(_, _, _, _, _, h53) => {
                            h53
                        }
                    };
                    let a55 = f8(v2);
                    append_list(a54, a55)
                };
                append_list(a56, a57)
            }))))
        }
    }
}

pub fn f9(v0: Str) -> R<Option<(Str, Str)>> {
    let m58 = f21(v0.clone())?;
    match m58 {
        None => {
            Ok(Some((Str::lit("proof.input"), f19(List::cons(Str::lit("the SDK image "), List::cons(f20(v0)?, List::cons(Str::lit(" is not pinned as <repository>@sha256:<64 hex>"), List::<Str>::nil())))))))
        }
        Some(_) => {
            Ok(None::<(Str, Str)>)
        }
    }
}

pub fn f10(v0: List<Adt1>) -> R<Option<(Str, Str)>> {
    let m59 = v0;
    match m59.uncons() {
        None => {
            Ok(None::<(Str, Str)>)
        }
        Some((v1, v2)) => {
            let m61 = f22(match v1.clone() {
                Adt1::C0(h60, _) => {
                    h60
                }
            })?;
            match m61 {
                None => {
                    Ok(Some((Str::lit("proof.input"), f19(List::cons(f20(match v1 {
                        Adt1::C0(h62, _) => {
                            h62
                        }
                    })?, List::cons(Str::lit(" is not a generated Lean module path, lexlean/build/modules/<module>.lean"), List::<Str>::nil()))))))
                }
                Some(v3) => {
                    Ok(f3(f11(f19(List::cons(Str::lit("the SHA-256 of "), List::cons(v3, List::<Str>::nil()))), match v1 {
                        Adt1::C0(_, h63) => {
                            h63
                        }
                    }, 64u64)?, f10(v2)?))
                }
            }
        }
    }
}

pub fn f11(v0: Str, v1: Str, v2: u64) -> R<Option<(Str, Str)>> {
    if f23(v1, v2)? {
        Ok(None::<(Str, Str)>)
    } else {
        Ok(Some((Str::lit("proof.input"), f19(List::cons(v0, List::cons(Str::lit(" is not "), List::cons(f24(v2)?, List::cons(Str::lit(" lowercase hex characters"), List::<Str>::nil()))))))))
    }
}

pub fn f12(v0: List<Str>) -> Str {
    f19(List::cons(Str::lit("{"), List::cons({
        let a64 = v0;
        let a65 = Str::lit(",");
        join(a64, a65)
    }, List::cons(Str::lit("}"), List::<Str>::nil()))))
}

pub fn f13(v0: Str, v1: Str) -> R<Str> {
    Ok(f19(List::cons(f14(v0)?, List::cons(Str::lit(":"), List::cons(v1, List::<Str>::nil())))))
}

pub fn f14(v0: Str) -> R<Str> {
    f20(v0)
}

pub fn f15(v0: Adt0) -> R<Str> {
    Ok(f12(List::cons(f13(Str::lit("evidence"), f12(List::cons(f13(Str::lit("lexleanAttestation"), f25(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h66, _, _) => {
            h66
        }
    })?)?, List::cons(f13(Str::lit("verificationManifest"), f25(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h67, _) => {
            h67
        }
    })?)?, List::<Str>::nil()))))?, List::cons(f13(Str::lit("model"), f12(List::cons(f13(Str::lit("compilerSemanticsId"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h68, _, _, _, _, _, _, _) => {
            h68
        }
    })?)?, List::cons(f13(Str::lit("document"), f17(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h69, _, _, _, _, _, _, _, _, _, _) => {
            h69
        }
    }, match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h70, _, _, _, _, _, _, _, _, _, _, _) => {
            h70
        }
    })?)?, List::cons(f13(Str::lit("leanManifestSha256"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h71, _, _, _, _, _, _) => {
            h71
        }
    })?)?, List::cons(f13(Str::lit("modules"), f16(f26(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h72, _, _, _, _, _) => {
            h72
        }
    })?))?, List::cons(f13(Str::lit("semanticId"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h73, _, _, _, _, _, _, _, _) => {
            h73
        }
    })?)?, List::cons(f13(Str::lit("snapshotId"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h74, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h74
        }
    })?)?, List::cons(f13(Str::lit("sourceId"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h75, _, _, _, _, _, _, _, _, _) => {
            h75
        }
    })?)?, List::<Str>::nil())))))))))?, List::cons(f13(Str::lit("policy"), f12(List::cons(f13(Str::lit("digest"), f25(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h76, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h76
        }
    })?)?, List::cons(f13(Str::lit("uri"), f14(Str::lit("https://uor.foundation/prism/proof/v1/policy/axiom-audit"))?)?, List::<Str>::nil()))))?, List::cons(f13(Str::lit("tcb"), f27(v0.clone())?)?, List::cons(f13(Str::lit("theorems"), f16(f28(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h77, _, _, _, _) => {
            h77
        }
    })?))?, List::cons(f13(Str::lit("timeVerified"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h78) => {
            h78
        }
    })?)?, List::cons(f13(Str::lit("verificationResult"), f14(f29(if f30(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h79, _, _, _, _) => {
            h79
        }
    }) {
        f31(match v0.clone() {
            Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h80, _, _, _) => {
                h80
            }
        })
    } else {
        false
    }))?)?, List::cons(f13(Str::lit("verifiedLevels"), f32(List::cons(Str::lit("P2"), List::<Str>::nil()))?)?, List::cons(f13(Str::lit("verifier"), f12(List::cons(f13(Str::lit("id"), f14(Str::lit("https://uor.foundation/prism/verifier/lean4"))?)?, List::cons(f13(Str::lit("lean"), f12(List::cons(f13(Str::lit("commit"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, h81, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h81
        }
    })?)?, List::cons(f13(Str::lit("version"), f14(f19(List::cons(Str::lit("v"), List::cons(match v0.clone() {
        Adt0::C0(_, _, _, h82, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h82
        }
    }, List::<Str>::nil()))))?)?, List::<Str>::nil()))))?, List::cons(f13(Str::lit("lexlean"), f12(List::cons(f13(Str::lit("commit"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, h83, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h83
        }
    })?)?, List::cons(f13(Str::lit("crateSha256"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, h84, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h84
        }
    })?)?, List::cons(f13(Str::lit("version"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, h85, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h85
        }
    })?)?, List::<Str>::nil())))))?, List::cons(f13(Str::lit("prismpm"), f12(List::cons(f13(Str::lit("version"), f14(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, h86, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h86
        }
    })?)?, List::<Str>::nil())))?, List::cons(f13(Str::lit("sdkImage"), f14(match v0 {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, h87, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h87
        }
    })?)?, List::<Str>::nil())))))))?, List::<Str>::nil())))))))))))
}

pub fn f16(v0: List<Str>) -> Str {
    f19(List::cons(Str::lit("["), List::cons({
        let a88 = v0;
        let a89 = Str::lit(",");
        join(a88, a89)
    }, List::cons(Str::lit("]"), List::<Str>::nil()))))
}

pub fn f17(v0: Str, v1: Str) -> R<Str> {
    Ok(f12(List::cons(f13(Str::lit("blake3"), f14(v0)?)?, List::cons(f13(Str::lit("sha256"), f14(v1)?)?, List::<Str>::nil()))))
}

pub fn f18(v0: Str) -> R<bool> {
    let a92 = {
        let a91 = {
            let a90 = v0.clone();
            utf8_encode(a90)
        };
        length_bytes(a91)
    };
    let a93 = 0u64;
    if nat_eq(a92, a93) {
        Ok(false)
    } else {
        f33({
            let a94 = v0.clone();
            utf8_encode(a94)
        }, 0u64, {
            let a96 = {
                let a95 = v0;
                utf8_encode(a95)
            };
            length_bytes(a96)
        })
    }
}

pub fn f19(v0: List<Str>) -> Str {
    let a97 = v0;
    let a98 = Str::lit("");
    join(a97, a98)
}

pub fn f20(v0: Str) -> R<Str> {
    f34(v0, Bytes::lit(&[0x22]))
}

pub fn f21(v0: Str) -> R<Option<Str>> {
    let a101 = {
        let a100 = {
            let a99 = v0.clone();
            utf8_encode(a99)
        };
        length_bytes(a100)
    };
    let a102 = 73u64;
    if nat_lt(a101, a102) {
        Ok(None::<Str>)
    } else {
        f35({
            let a103 = v0.clone();
            utf8_encode(a103)
        }, 1u64, {
            let a105 = {
                let a104 = v0;
                utf8_encode(a104)
            };
            length_bytes(a105)
        })
    }
}

pub fn f22(v0: Str) -> R<Option<Str>> {
    let v1: Bytes = {
        let a106 = v0;
        utf8_encode(a106)
    };
    if f36(f37(v1.clone(), 0u64, 22u64), f38()) {
        let m108 = f39(v1.clone(), 22u64, {
            let a107 = v1;
            length_bytes(a107)
        })?;
        match m108 {
            None => {
                Ok(None::<Str>)
            }
            Some(v2) => {
                let a110 = {
                    let a109 = v2.clone();
                    length_bytes(a109)
                };
                let a111 = 0u64;
                if nat_eq(a110, a111) {
                    Ok(None::<Str>)
                } else {
                    let a112 = v2;
                    Ok(utf8_decode(a112))
                }
            }
        }
    } else {
        Ok(None::<Str>)
    }
}

pub fn f23(v0: Str, v1: u64) -> R<bool> {
    let a115 = {
        let a114 = {
            let a113 = v0.clone();
            utf8_encode(a113)
        };
        length_bytes(a114)
    };
    let a116 = v1;
    if nat_eq(a115, a116) {
        f40({
            let a117 = v0;
            utf8_encode(a117)
        }, 0u64, v1)
    } else {
        Ok(false)
    }
}

pub fn f24(v0: u64) -> R<Str> {
    Ok(f41(v0, {
        let a118 = v0;
        let a119 = 1u64;
        nat_add(a118, a119)?
    }))
}

pub fn f25(v0: Str) -> R<Str> {
    Ok(f12(List::cons(f13(Str::lit("sha256"), f14(v0)?)?, List::<Str>::nil())))
}

pub fn f26(v0: List<Adt1>) -> R<List<Str>> {
    let m120 = v0;
    match m120.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f12(List::cons(f13(Str::lit("leanSha256"), f14(match v1.clone() {
                Adt1::C0(_, h121) => {
                    h121
                }
            })?)?, List::cons(f13(Str::lit("name"), f14({
                let m123 = f22(match v1.clone() {
                    Adt1::C0(h122, _) => {
                        h122
                    }
                })?;
                match m123 {
                    None => {
                        match v1 {
                            Adt1::C0(h124, _) => {
                                h124
                            }
                        }
                    }
                    Some(v3) => {
                        v3
                    }
                }
            })?)?, List::<Str>::nil()))), f26(v2)?))
        }
    }
}

pub fn f27(v0: Adt0) -> R<Str> {
    let v1: Str = f19(List::cons(Str::lit("leanprover/lean4:v"), List::cons(match v0.clone() {
        Adt0::C0(_, _, _, h125, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h125
        }
    }, List::<Str>::nil())));
    Ok(f16(List::cons(f42(Str::lit("lean4-kernel"), Some(match v0.clone() {
        Adt0::C0(_, _, _, _, h126, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h126
        }
    }), Str::lit("type-checks every listed theorem; the verdict is its acceptance"), v1.clone())?, List::cons(f42(Str::lit("leanchecker"), None::<Str>, Str::lit("replays each module environment through the same kernel; not independent of it"), v1)?, List::cons(f42(Str::lit("lexlean"), Some(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, h127, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h127
        }
    }), Str::lit("translates .lex.tex into the Lean that was checked"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, h128, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h128
        }
    })?, List::cons(f42(Str::lit("lean4-prod"), None::<Str>, Str::lit("exports LCNF from the checked Lean and renders the Rust core"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, h129, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h129
        }
    })?, List::cons(f42(Str::lit("rustc"), None::<Str>, Str::lit("compiles the Rust core to the Core-Wasm guest, the subject"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, h130, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h130
        }
    })?, List::cons(f42(Str::lit("wasm-bindgen-cli"), None::<Str>, Str::lit("generates the browser adapter around the guest; outside the subject"), match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, h131, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h131
        }
    })?, List::cons(f42(Str::lit("sdk-image"), f21(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, h132, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h132
        }
    })?, Str::lit("hosts every step above; pinned in prismpm.lock"), match v0 {
        Adt0::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, h133, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h133
        }
    })?, List::<Str>::nil())))))))))
}

pub fn f28(v0: List<Adt2>) -> R<List<Str>> {
    let m134 = v0;
    match m134.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f43(v1)?, f28(v2)?))
        }
    }
}

pub fn f29(v0: bool) -> Str {
    if v0 {
        Str::lit("PASSED")
    } else {
        Str::lit("FAILED")
    }
}

pub fn f30(p137: List<Adt2>) -> bool {
    let m136 = p137;
    {
        let mut m136 = m136;
        loop {
            let step = {
                let v0 = m136;
                let m135 = v0;
                match m135.uncons() {
                    None => {
                        Err::<List<Adt2>, bool>(true)
                    }
                    Some((v1, v2)) => {
                        if f44(v1) {
                            Ok::<List<Adt2>, bool>(v2)
                        } else {
                            Err::<List<Adt2>, bool>(false)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m136 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f31(p142: List<u64>) -> bool {
    let m141 = p142;
    {
        let mut m141 = m141;
        loop {
            let step = {
                let v0 = m141;
                let m138 = v0;
                match m138.uncons() {
                    None => {
                        Err::<List<u64>, bool>(true)
                    }
                    Some((v1, v2)) => {
                        let a139 = v1;
                        let a140 = 0u64;
                        if nat_eq(a139, a140) {
                            Ok::<List<u64>, bool>(v2)
                        } else {
                            Err::<List<u64>, bool>(false)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m141 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32(v0: List<Str>) -> R<Str> {
    Ok(f16(f45(v0)?))
}

pub fn f33(v0: Bytes, p147: u64, p148: u64) -> R<bool> {
    let m146 = (p147, p148);
    {
        let mut m146 = m146;
        loop {
            let step = {
                let (v1, v2) = m146;
                let m143 = v2;
                if m143 == 0 {
                    Err::<(u64, u64), R<bool>>(Ok(true))
                } else {
                    let v3 = m143 - 1;
                    if f46(f37(v0.clone(), v1, 1u64), Bytes::lit(&[0x20])) {
                        Err::<(u64, u64), R<bool>>(Ok(false))
                    } else {
                        Ok::<(u64, u64), R<bool>>(({
                            let a144 = v1;
                            let a145 = 1u64;
                            nat_add(a144, a145)?
                        }, v3))
                    }
                }
            };
            match step {
                Ok(next) => { m146 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f34(v0: Str, v1: Bytes) -> R<Str> {
    Ok(f47({
        let a154 = {
            let a152 = v1.clone();
            let a153 = f48({
                let a149 = v0.clone();
                utf8_encode(a149)
            }, 0u64, v1.clone(), {
                let a151 = {
                    let a150 = v0.clone();
                    utf8_encode(a150)
                };
                length_bytes(a151)
            })?;
            append_bytes(a152, a153)
        };
        let a155 = v1;
        append_bytes(a154, a155)
    }, v0))
}

pub fn f35(v0: Bytes, p167: u64, p168: u64) -> R<Option<Str>> {
    let m166 = (p167, p168);
    {
        let mut m166 = m166;
        loop {
            let step = {
                let (v1, v2) = m166;
                let m156 = v2;
                if m156 == 0 {
                    Err::<(u64, u64), R<Option<Str>>>(Ok(None::<Str>))
                } else {
                    let v3 = m156 - 1;
                    if if f36(f37(v0.clone(), v1, 8u64), Bytes::lit(&[0x40, 0x73, 0x68, 0x61, 0x32, 0x35, 0x36, 0x3a])) {
                        if f36(f37(v0.clone(), {
                            let a157 = v1;
                            let a158 = 72u64;
                            nat_add(a157, a158)?
                        }, 1u64), Bytes::lit(&[])) {
                            f40(f37(v0.clone(), {
                                let a159 = v1;
                                let a160 = 8u64;
                                nat_add(a159, a160)?
                            }, 64u64), 0u64, 64u64)?
                        } else {
                            false
                        }
                    } else {
                        false
                    } {
                        let a163 = f37(v0.clone(), {
                            let a161 = v1;
                            let a162 = 8u64;
                            nat_add(a161, a162)?
                        }, 64u64);
                        Err::<(u64, u64), R<Option<Str>>>(Ok(utf8_decode(a163)))
                    } else {
                        Ok::<(u64, u64), R<Option<Str>>>(({
                            let a164 = v1;
                            let a165 = 1u64;
                            nat_add(a164, a165)?
                        }, v3))
                    }
                }
            };
            match step {
                Ok(next) => { m166 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f36(v0: Bytes, v1: Bytes) -> bool {
    let a173 = {
        let a169 = v0;
        let a170 = v1;
        compare_bytes(a169, a170)
    };
    let a174 = {
        let a171 = Bytes::lit(&[0x00]);
        let a172 = Bytes::lit(&[0x00]);
        compare_bytes(a171, a172)
    };
    equal(a173, a174)
}

pub fn f37(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m178 = {
        let a175 = v0;
        let a176 = v1;
        let a177 = v2;
        slice_bytes(a175, a176, a177)
    };
    match m178 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f38() -> Bytes {
    Bytes::lit(&[0x6c, 0x65, 0x78, 0x6c, 0x65, 0x61, 0x6e, 0x2f, 0x62, 0x75, 0x69, 0x6c, 0x64, 0x2f, 0x6d, 0x6f, 0x64, 0x75, 0x6c, 0x65, 0x73, 0x2f])
}

pub fn f39(v0: Bytes, v1: u64, v2: u64) -> R<Option<Bytes>> {
    let m179 = v2;
    if m179 == 0 {
        Ok(None::<Bytes>)
    } else {
        let v3 = m179 - 1;
        if if f36(f37(v0.clone(), v1, 5u64), Bytes::lit(&[0x2e, 0x6c, 0x65, 0x61, 0x6e])) {
            f36(f37(v0.clone(), {
                let a180 = v1;
                let a181 = 5u64;
                nat_add(a180, a181)?
            }, 1u64), Bytes::lit(&[]))
        } else {
            false
        } {
            Ok(Some(Bytes::lit(&[])))
        } else {
            if f36(f37(v0.clone(), v1, 1u64), Bytes::lit(&[])) {
                Ok(None::<Bytes>)
            } else {
                let m184 = f39(v0.clone(), {
                    let a182 = v1;
                    let a183 = 1u64;
                    nat_add(a182, a183)?
                }, v3)?;
                match m184 {
                    None => {
                        Ok(None::<Bytes>)
                    }
                    Some(v4) => {
                        Ok(Some({
                            let a185 = if f36(f37(v0.clone(), v1, 1u64), Bytes::lit(&[0x2f])) {
                                Bytes::lit(&[0x2e])
                            } else {
                                f37(v0, v1, 1u64)
                            };
                            let a186 = v4;
                            append_bytes(a185, a186)
                        }))
                    }
                }
            }
        }
    }
}

pub fn f40(v0: Bytes, p191: u64, p192: u64) -> R<bool> {
    let m190 = (p191, p192);
    {
        let mut m190 = m190;
        loop {
            let step = {
                let (v1, v2) = m190;
                let m187 = v2;
                if m187 == 0 {
                    Err::<(u64, u64), R<bool>>(Ok(true))
                } else {
                    let v3 = m187 - 1;
                    if f49(f37(v0.clone(), v1, 1u64), f50(), 16u64) {
                        Ok::<(u64, u64), R<bool>>(({
                            let a188 = v1;
                            let a189 = 1u64;
                            nat_add(a188, a189)?
                        }, v3))
                    } else {
                        Err::<(u64, u64), R<bool>>(Ok(false))
                    }
                }
            };
            match step {
                Ok(next) => { m190 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f41(v0: u64, v1: u64) -> Str {
    let m193 = v1;
    if m193 == 0 {
        Str::lit("")
    } else {
        let v2 = m193 - 1;
        let a194 = v0;
        let a195 = 10u64;
        if nat_lt(a194, a195) {
            f51(v0)
        } else {
            f19(List::cons(f41({
                let a196 = v0;
                let a197 = 10u64;
                let a198 = 0u64;
                nat_quot(a196, a197, a198)
            }, v2), List::cons(f51({
                let a199 = v0;
                let a200 = 10u64;
                let a201 = 0u64;
                nat_rem(a199, a200, a201)
            }), List::<Str>::nil())))
        }
    }
}

pub fn f42(v0: Str, v1: Option<Str>, v2: Str, v3: Str) -> R<Str> {
    Ok(f12(List::cons(f13(Str::lit("component"), f14(v0)?)?, {
        let a203 = {
            let m202 = v1;
            match m202 {
                None => {
                    List::<Str>::nil()
                }
                Some(v4) => {
                    List::cons(f13(Str::lit("digest"), f14(v4)?)?, List::<Str>::nil())
                }
            }
        };
        let a204 = List::cons(f13(Str::lit("role"), f14(v2)?)?, List::cons(f13(Str::lit("trust"), f14(Str::lit("trusted"))?)?, List::cons(f13(Str::lit("version"), f14(v3)?)?, List::<Str>::nil())));
        append_list(a203, a204)
    })))
}

pub fn f43(v0: Adt2) -> R<Str> {
    Ok(f12(List::cons(f13(Str::lit("axioms"), f12(List::cons(f13(Str::lit("observed"), f32(match v0.clone() {
        Adt2::C0(_, _, _, _, _, h205) => {
            h205
        }
    })?)?, List::cons(f13(Str::lit("policy"), f12(List::cons(f13(Str::lit("axioms"), f32(match v0.clone() {
        Adt2::C0(_, _, _, _, h206, _) => {
            h206
        }
    })?)?, List::cons(f13(Str::lit("kind"), f14(match v0.clone() {
        Adt2::C0(_, _, _, h207, _, _) => {
            h207
        }
    })?)?, List::<Str>::nil()))))?, List::<Str>::nil()))))?, List::cons(f13(Str::lit("module"), f14(match v0.clone() {
        Adt2::C0(_, h208, _, _, _, _) => {
            h208
        }
    })?)?, List::cons(f13(Str::lit("name"), f14(match v0.clone() {
        Adt2::C0(h209, _, _, _, _, _) => {
            h209
        }
    })?)?, List::cons(f13(Str::lit("verdict"), f14(f29(f44(v0)))?)?, List::<Str>::nil()))))))
}

pub fn f44(v0: Adt2) -> bool {
    let a211 = match v0.clone() {
        Adt2::C0(_, _, h210, _, _, _) => {
            h210
        }
    };
    let a212 = Str::lit("ok");
    if equal(a211, a212) {
        f52(match v0.clone() {
            Adt2::C0(_, _, _, h213, _, _) => {
                h213
            }
        }, match v0.clone() {
            Adt2::C0(_, _, _, _, h214, _) => {
                h214
            }
        }, match v0 {
            Adt2::C0(_, _, _, _, _, h215) => {
                h215
            }
        })
    } else {
        false
    }
}

pub fn f45(v0: List<Str>) -> R<List<Str>> {
    let m216 = v0;
    match m216.uncons() {
        None => {
            Ok(List::<Str>::nil())
        }
        Some((v1, v2)) => {
            Ok(List::cons(f20(v1)?, f45(v2)?))
        }
    }
}

pub fn f46(v0: Bytes, v1: Bytes) -> bool {
    let a221 = {
        let a217 = v0;
        let a218 = v1;
        compare_bytes(a217, a218)
    };
    let a222 = {
        let a219 = Bytes::lit(&[0x00]);
        let a220 = Bytes::lit(&[0x01]);
        compare_bytes(a219, a220)
    };
    equal(a221, a222)
}

pub fn f47(v0: Bytes, v1: Str) -> Str {
    let m224 = {
        let a223 = v0;
        utf8_decode(a223)
    };
    match m224 {
        None => {
            v1
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f48(v0: Bytes, v1: u64, v2: Bytes, v3: u64) -> R<Bytes> {
    let m225 = v3;
    if m225 == 0 {
        Ok(Bytes::lit(&[]))
    } else {
        let v4 = m225 - 1;
        let a230 = if f36(f37(v0.clone(), v1, 1u64), Bytes::lit(&[0x5c])) {
            Bytes::lit(&[0x5c, 0x5c])
        } else {
            if f36(f37(v0.clone(), v1, 1u64), v2.clone()) {
                let a226 = Bytes::lit(&[0x5c]);
                let a227 = v2.clone();
                append_bytes(a226, a227)
            } else {
                if f36(f37(v0.clone(), v1, 1u64), Bytes::lit(&[0x0a])) {
                    Bytes::lit(&[0x5c, 0x6e])
                } else {
                    f37(v0.clone(), v1, 1u64)
                }
            }
        };
        let a231 = f48(v0, {
            let a228 = v1;
            let a229 = 1u64;
            nat_add(a228, a229)?
        }, v2, v4)?;
        Ok(append_bytes(a230, a231))
    }
}

pub fn f49(v0: Bytes, v1: Bytes, p234: u64) -> bool {
    let m233 = p234;
    {
        let mut m233 = m233;
        loop {
            let step = {
                let v2 = m233;
                let m232 = v2;
                if m232 == 0 {
                    Err::<u64, bool>(false)
                } else {
                    let v3 = m232 - 1;
                    if f36(f37(v1.clone(), v3, 1u64), v0.clone()) {
                        Err::<u64, bool>(true)
                    } else {
                        Ok::<u64, bool>(v3)
                    }
                }
            };
            match step {
                Ok(next) => { m233 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f50() -> Bytes {
    Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66])
}

pub fn f51(v0: u64) -> Str {
    f47(f37(f53(), v0, 1u64), Str::lit("0"))
}

pub fn f52(v0: Str, v1: List<Str>, v2: List<Str>) -> bool {
    let a235 = v0.clone();
    let a236 = Str::lit("none");
    if equal(a235, a236) {
        f54(v2)
    } else {
        let a237 = v0.clone();
        let a238 = Str::lit("allow");
        if equal(a237, a238) {
            f55(v2, v1)
        } else {
            let a239 = v0;
            let a240 = Str::lit("exact");
            if equal(a239, a240) {
                if f55(v2.clone(), v1.clone()) {
                    f55(v1, v2)
                } else {
                    false
                }
            } else {
                false
            }
        }
    }
}

pub fn f53() -> Bytes {
    Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39])
}

pub fn f54(v0: List<Str>) -> bool {
    let m241 = v0;
    match m241.uncons() {
        None => {
            true
        }
        Some((_, _)) => {
            false
        }
    }
}

pub fn f55(p244: List<Str>, v1: List<Str>) -> bool {
    let m243 = p244;
    {
        let mut m243 = m243;
        loop {
            let step = {
                let v0 = m243;
                let m242 = v0;
                match m242.uncons() {
                    None => {
                        Err::<List<Str>, bool>(true)
                    }
                    Some((v2, v3)) => {
                        if f56(v2, v1.clone()) {
                            Ok::<List<Str>, bool>(v3)
                        } else {
                            Err::<List<Str>, bool>(false)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m243 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f56(v0: Str, p249: List<Str>) -> bool {
    let m248 = p249;
    {
        let mut m248 = m248;
        loop {
            let step = {
                let v1 = m248;
                let m245 = v1;
                match m245.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        let a246 = v2;
                        let a247 = v0.clone();
                        if equal(a246, a247) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
                        }
                    }
                }
            };
            match step {
                Ok(next) => { m248 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn run(p0: Adt0) -> R<Adt3> {
    f0(p0)
}
