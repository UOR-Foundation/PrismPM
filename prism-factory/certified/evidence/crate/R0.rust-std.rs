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
    F123,
}

impl Fn1 {
    pub fn apply(&self, p0: List<Bytes>, p1: Adt1) -> List<Bytes> {
        match self {
            Fn1::F12(k0) => f12(k0.clone(), p0, p1),
            Fn1::F123 => f123(p0, p1),
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
    F26(Bytes),
}

impl Fn4 {
    pub fn apply(&self, p0: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
        match self {
            Fn4::F26(k0) => f26(k0.clone(), p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn5 {
    F30,
}

impl Fn5 {
    pub fn apply(&self, p0: List<(Bytes, Bytes)>, p1: (Bytes, Bytes)) -> List<(Bytes, Bytes)> {
        match self {
            Fn5::F30 => f30(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn6 {
    F32,
}

impl Fn6 {
    pub fn apply(&self, p0: List<(Str, Str)>, p1: (Str, Str)) -> List<(Str, Str)> {
        match self {
            Fn6::F32 => f32(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn7 {
    F36(List<(Str, Str)>),
}

impl Fn7 {
    pub fn apply(&self, p0: List<(Str, u64)>, p1: Adt1) -> List<(Str, u64)> {
        match self {
            Fn7::F36(k0) => f36(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn8 {
    F39,
    F81(Str, Str),
    F122(Str),
}

impl Fn8 {
    pub fn apply(&self, p0: List<Bytes>, p1: Str) -> List<Bytes> {
        match self {
            Fn8::F39 => f39(p0, p1),
            Fn8::F81(k0, k1) => f81(k0.clone(), k1.clone(), p0, p1),
            Fn8::F122(k0) => f122(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn9 {
    F52,
}

impl Fn9 {
    pub fn apply(&self, p0: List<Bytes>, p1: Bytes) -> List<Bytes> {
        match self {
            Fn9::F52 => f52(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn10 {
    F54,
}

impl Fn10 {
    pub fn apply(&self, p0: List<Bytes>) -> Option<List<Bytes>> {
        match self {
            Fn10::F54 => f54(p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn11 {
    F66,
}

impl Fn11 {
    pub fn apply(&self, p0: bool, p1: Adt4) -> bool {
        match self {
            Fn11::F66 => f66(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn12 {
    F68,
}

impl Fn12 {
    pub fn apply(&self, p0: List<Adt4>, p1: Adt1) -> List<Adt4> {
        match self {
            Fn12::F68 => f68(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn13 {
    F80(List<Str>, Str),
    F108(List<Str>),
}

impl Fn13 {
    pub fn apply(&self, p0: List<Bytes>, p1: (Bytes, Bytes)) -> List<Bytes> {
        match self {
            Fn13::F80(k0, k1) => f80(k0.clone(), k1.clone(), p0, p1),
            Fn13::F108(k0) => f108(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn14 {
    F84(Str),
}

impl Fn14 {
    pub fn apply(&self, p0: Str, p1: (Str, Str)) -> Str {
        match self {
            Fn14::F84(k0) => f84(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn15 {
    F86(Bytes),
}

impl Fn15 {
    pub fn apply(&self, p0: Option<Bytes>, p1: Bytes) -> Option<Bytes> {
        match self {
            Fn15::F86(k0) => f86(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn16 {
    F89(List<Str>),
}

impl Fn16 {
    pub fn apply(&self, p0: Option<Str>, p1: (List<Str>, Str)) -> Option<Str> {
        match self {
            Fn16::F89(k0) => f89(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn17 {
    F93(List<Adt1>),
}

impl Fn17 {
    pub fn apply(&self, p0: List<Adt1>, p1: Str) -> List<Adt1> {
        match self {
            Fn17::F93(k0) => f93(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn18 {
    F96(List<Adt1>),
}

impl Fn18 {
    pub fn apply(&self, p0: bool, p1: Str) -> bool {
        match self {
            Fn18::F96(k0) => f96(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn19 {
    F99(Bytes, u64, Bytes, u64),
}

impl Fn19 {
    pub fn apply(&self, p0: u64) -> R<Option<u64>> {
        match self {
            Fn19::F99(k0, k1, k2, k3) => f99(k0.clone(), *k1, k2.clone(), *k3, p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn20 {
    F103,
}

impl Fn20 {
    pub fn apply(&self, p0: (Option<Bytes>, List<Bytes>), p1: Bytes) -> (Option<Bytes>, List<Bytes>) {
        match self {
            Fn20::F103 => f103(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn21 {
    F113(Str),
}

impl Fn21 {
    pub fn apply(&self, p0: Option<bool>, p1: (Str, bool)) -> Option<bool> {
        match self {
            Fn21::F113(k0) => f113(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn22 {
    F118(Bytes, u64, u64),
}

impl Fn22 {
    pub fn apply(&self, p0: (u64, Adt6)) -> R<(u64, Adt6)> {
        match self {
            Fn22::F118(k0, k1, k2) => f118(k0.clone(), *k1, *k2, p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn23 {
    F121(Str),
}

impl Fn23 {
    pub fn apply(&self, p0: Option<Adt1>, p1: Adt1) -> Option<Adt1> {
        match self {
            Fn23::F121(k0) => f121(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn24 {
    F129,
}

impl Fn24 {
    pub fn apply(&self, p0: Adt8) -> R<Adt8> {
        match self {
            Fn24::F129 => f129(p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn25 {
    F145,
}

impl Fn25 {
    pub fn apply(&self, p0: (u64, (u32, u32))) -> (u64, (u32, u32)) {
        match self {
            Fn25::F145 => f145(p0),
        }
    }
}

pub fn f0(v0: Bytes) -> R<Bytes> {
    let v1: Adt0 = f1(v0)?;
    Ok(f2(f3(f4(), match v1.clone() {
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
    }))
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

pub fn f2(v0: List<Adt1>, v1: List<(Str, Str)>, v2: List<(Bytes, Bytes)>) -> Bytes {
    let v3: Bytes = f10(f11(Fn1::F12(v2), List::<Bytes>::nil(), v0.clone()));
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
    append_bytes(a11, a12)
}

pub fn f3(v0: List<Adt2>, v1: List<(Bytes, Bytes)>, v2: List<(Str, bool)>) -> R<List<Adt1>> {
    let (h13, _) = f18(Fn3::F19(v2, v1), (List::<Adt1>::nil(), List::<(List<Str>, Str)>::nil()), v0)?;
    Ok(h13)
}

pub fn f4() -> List<Adt2> {
    let a18 = {
        let a16 = {
            let a14 = f20();
            let a15 = f21();
            append_list(a14, a15)
        };
        let a17 = f22();
        append_list(a16, a17)
    };
    let a19 = f23();
    append_list(a18, a19)
}

pub fn f5(v0: Fn0, p23: Adt0, p24: List<Bytes>) -> R<Adt0> {
    let m22 = (p23, p24);
    {
        let mut m22 = m22;
        loop {
            let step = {
                let (v1, v2) = m22;
                let m20 = v2;
                match m20.uncons() {
                    None => {
                        Err::<(Adt0, List<Bytes>), R<Adt0>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(Adt0, List<Bytes>), R<Adt0>>(({
                            let c21: Fn0 = v0.clone();
                            c21.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m22 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: Adt0, v1: Bytes) -> R<Adt0> {
    f24(v0, v1)
}

pub fn f7(v0: Bytes) -> R<List<Bytes>> {
    let v1: u64 = {
        let a25 = v0.clone();
        length_bytes(a25)
    };
    let v2: (u64, (u64, List<Bytes>)) = {
        let (h26, _) = f25(Fn4::F26(v0.clone()), v1, (0u64, (0u64, List::<Bytes>::nil())))?;
        h26
    };
    let v3: u64 = {
        let (h27, _) = {
            let (_, h28) = v2.clone();
            h28
        };
        h27
    };
    let v4: List<Bytes> = {
        let (_, h29) = {
            let (_, h30) = v2;
            h30
        };
        h29
    };
    let a31 = v3;
    let a32 = v1;
    if nat_lt(a31, a32) {
        Ok(f27(List::cons(f28(v0, v3, {
            let a33 = v1;
            let a34 = v3;
            nat_sub(a33, a34)
        }), v4)))
    } else {
        Ok(f27(v4))
    }
}

pub fn f8(v0: List<(Bytes, Bytes)>) -> List<(Bytes, Bytes)> {
    f29(Fn5::F30, List::<(Bytes, Bytes)>::nil(), v0)
}

pub fn f9(v0: List<(Str, Str)>) -> List<(Str, Str)> {
    f31(Fn6::F32, List::<(Str, Str)>::nil(), v0)
}

pub fn f10(v0: List<Bytes>) -> Bytes {
    f33(f27(v0))
}

pub fn f11(v0: Fn1, p38: List<Bytes>, p39: List<Adt1>) -> List<Bytes> {
    let m37 = (p38, p39);
    {
        let mut m37 = m37;
        loop {
            let step = {
                let (v1, v2) = m37;
                let m35 = v2;
                match m35.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Adt1>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Adt1>), List<Bytes>>(({
                            let c36: Fn1 = v0.clone();
                            c36.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m37 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f12(v0: List<(Bytes, Bytes)>, v1: List<Bytes>, v2: Adt1) -> List<Bytes> {
    List::cons(f34(v2, v0), v1)
}

pub fn f13(v0: Fn2, p43: Bytes, p44: List<(Str, u64)>) -> Bytes {
    let m42 = (p43, p44);
    {
        let mut m42 = m42;
        loop {
            let step = {
                let (v1, v2) = m42;
                let m40 = v2;
                match m40.uncons() {
                    None => {
                        Err::<(Bytes, List<(Str, u64)>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<(Str, u64)>), Bytes>(({
                            let c41: Fn2 = v0.clone();
                            c41.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m42 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f14(v0: Bytes, v1: (Str, u64)) -> Bytes {
    let a46 = v0;
    let a47 = f16(List::cons(Str::lit("schedule"), List::cons({
        let (h45, _) = v1;
        h45
    }, List::<Str>::nil())));
    append_bytes(a46, a47)
}

pub fn f15(v0: List<Adt1>, v1: List<(Str, Str)>) -> List<(Str, u64)> {
    f35(Fn7::F36(v1), List::<(Str, u64)>::nil(), v0)
}

pub fn f16(v0: List<Str>) -> Bytes {
    f37(f38(Fn8::F39, List::<Bytes>::nil(), v0))
}

pub fn f17(v0: List<Adt1>) -> bool {
    f40(f41(v0))
}

pub fn f18(v0: Fn3, p51: (List<Adt1>, List<(List<Str>, Str)>), p52: List<Adt2>) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
    let m50 = (p51, p52);
    {
        let mut m50 = m50;
        loop {
            let step = {
                let (v1, v2) = m50;
                let m48 = v2;
                match m48.uncons() {
                    None => {
                        Err::<((List<Adt1>, List<(List<Str>, Str)>), List<Adt2>), R<(List<Adt1>, List<(List<Str>, Str)>)>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<((List<Adt1>, List<(List<Str>, Str)>), List<Adt2>), R<(List<Adt1>, List<(List<Str>, Str)>)>>(({
                            let c49: Fn3 = v0.clone();
                            c49.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m50 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f19(v0: List<(Str, bool)>, v1: List<(Bytes, Bytes)>, v2: (List<Adt1>, List<(List<Str>, Str)>), v3: Adt2) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
    f42(v1, v0, v2, v3)
}

pub fn f20() -> List<Adt2> {
    List::cons(f43(Str::lit("cargo-fetch"), Adt3::C1, Str::lit("host"), Str::lit("cargo fetch --locked"), List::cons(Str::lit("Cargo.lock"), List::cons(Str::lit("Cargo.toml"), List::cons(Str::lit("crates/"), List::cons(Str::lit("stdlib/"), List::cons(Str::lit("vendor/"), List::cons(Str::lit("xtask/"), List::<Str>::nil())))))), List::<Str>::nil()), List::cons(f43(Str::lit("factory-model"), Adt3::C0, Str::lit("host"), Str::lit("bash prism-factory/check.sh"), List::cons(Str::lit(".github/workflows/factory.yml"), List::cons(Str::lit("prism-factory/"), List::cons(Str::lit("vendor/lexlean/"), List::<Str>::nil()))), List::<Str>::nil()), List::cons(f43(Str::lit("devcontainer-image"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/devcontainer-image"), List::cons(Str::lit(".devcontainer/"), List::cons(Str::lit("lean-toolchain"), List::cons(Str::lit("rust-toolchain.toml"), List::cons(Str::lit("tools.lock"), List::<Str>::nil())))), List::<Str>::nil()), List::<Adt2>::nil())))
}

pub fn f21() -> List<Adt2> {
    List::cons(f43(Str::lit("source-vv-dk-3"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/dk-3"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-ho-oc"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/ho-oc"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-dk-2"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/dk-2"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-gates"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/gates"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-st-sy"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/st-sy"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-dk-0-1"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/dk-0-1"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-conformance"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/conformance"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f43(Str::lit("source-vv-source"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/source"), f44(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::<Adt2>::nil()))))))))
}

pub fn f22() -> List<Adt2> {
    List::cons(f43(Str::lit("gate"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/gate"), f44(), f45()), List::cons(f43(Str::lit("sdk-image-amd64"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/image-platforms/amd64"), f44(), List::<Str>::nil()), List::cons(f43(Str::lit("sdk-image-arm64"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/image-platforms/arm64"), f44(), List::<Str>::nil()), List::cons(f43(Str::lit("images"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/images"), f44(), List::cons(Str::lit("sdk-image-amd64"), List::cons(Str::lit("sdk-image-arm64"), List::<Str>::nil()))), List::cons(f43(Str::lit("reproducibility"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/reproducibility"), f44(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f43(Str::lit("library-closure"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/reproducibility/library-sdk-check"), f44(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f43(Str::lit("browser-closure"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/reproducibility/browser-api-sdk-check"), f44(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f43(Str::lit("installed-sdk"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/installed-sdk"), f44(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f43(Str::lit("native"), Adt3::C0, Str::lit("actions"), Str::lit("release.yml/native"), f44(), List::cons(Str::lit("images"), List::<Str>::nil())), List::<Adt2>::nil())))))))))
}

pub fn f23() -> List<Adt2> {
    List::cons(f43(Str::lit("release"), Adt3::C1, Str::lit("actions"), Str::lit("release.yml/release"), f44(), List::cons(Str::lit("factory-model"), List::cons(Str::lit("gate"), List::cons(Str::lit("reproducibility"), List::cons(Str::lit("library-closure"), List::cons(Str::lit("browser-closure"), List::cons(Str::lit("installed-sdk"), List::cons(Str::lit("native"), List::<Str>::nil())))))))), List::<Adt2>::nil())
}

pub fn f24(v0: Adt0, v1: Bytes) -> R<Adt0> {
    let v2: (Bytes, Bytes) = f46(v1)?;
    let v3: Bytes = {
        let (h53, _) = v2;
        h53
    };
    let v4: (Bytes, Bytes) = f46({
        let (_, h54) = v2;
        h54
    })?;
    let a55 = v3.clone();
    let a56 = f47(Str::lit("tree"));
    if equal(a55, a56) {
        Ok(Adt0::C0(List::cons(({
            let (h57, _) = v4;
            h57
        }, {
            let (_, h58) = v4;
            h58
        }), match v0.clone() {
            Adt0::C0(h59, _, _, _) => {
                h59
            }
        }), match v0.clone() {
            Adt0::C0(_, h60, _, _) => {
                h60
            }
        }, match v0.clone() {
            Adt0::C0(_, _, h61, _) => {
                h61
            }
        }, match v0 {
            Adt0::C0(_, _, _, h62) => {
                h62
            }
        }))
    } else {
        let a63 = v3.clone();
        let a64 = f47(Str::lit("receipt"));
        if equal(a63, a64) {
            Ok(Adt0::C0(match v0.clone() {
                Adt0::C0(h65, _, _, _) => {
                    h65
                }
            }, List::cons((f48({
                let (h66, _) = v4;
                h66
            }), f49({
                let (_, h67) = v4;
                h67
            })), match v0.clone() {
                Adt0::C0(_, h68, _, _) => {
                    h68
                }
            }), match v0.clone() {
                Adt0::C0(_, _, h69, _) => {
                    h69
                }
            }, match v0 {
                Adt0::C0(_, _, _, h70) => {
                    h70
                }
            }))
        } else {
            let a71 = v3.clone();
            let a72 = f47(Str::lit("weight"));
            if equal(a71, a72) {
                Ok(Adt0::C0(match v0.clone() {
                    Adt0::C0(h73, _, _, _) => {
                        h73
                    }
                }, match v0.clone() {
                    Adt0::C0(_, h74, _, _) => {
                        h74
                    }
                }, List::cons((f48({
                    let (h75, _) = v4;
                    h75
                }), f48({
                    let (_, h76) = v4;
                    h76
                })), match v0.clone() {
                    Adt0::C0(_, _, h77, _) => {
                        h77
                    }
                }), match v0 {
                    Adt0::C0(_, _, _, h78) => {
                        h78
                    }
                }))
            } else {
                let a79 = v3;
                let a80 = f47(Str::lit("outcome"));
                if equal(a79, a80) {
                    Ok(Adt0::C0(match v0.clone() {
                        Adt0::C0(h81, _, _, _) => {
                            h81
                        }
                    }, match v0.clone() {
                        Adt0::C0(_, h82, _, _) => {
                            h82
                        }
                    }, match v0.clone() {
                        Adt0::C0(_, _, h83, _) => {
                            h83
                        }
                    }, List::cons((f48({
                        let (h84, _) = v4;
                        h84
                    }), f49({
                        let (_, h85) = v4;
                        h85
                    })), match v0 {
                        Adt0::C0(_, _, _, h86) => {
                            h86
                        }
                    })))
                } else {
                    Ok(v0)
                }
            }
        }
    }
}

pub fn f25(v0: Fn4, p91: u64, p92: (u64, (u64, List<Bytes>))) -> R<((u64, (u64, List<Bytes>)), bool)> {
    let m90 = (p91, p92);
    {
        let mut m90 = m90;
        loop {
            let step = {
                let (v1, v2) = m90;
                let m87 = v1;
                if m87 == 0 {
                    Err::<(u64, (u64, (u64, List<Bytes>))), R<((u64, (u64, List<Bytes>)), bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m87 - 1;
                    let m89 = {
                        let c88: Fn4 = v0.clone();
                        c88.apply(v2.clone())?
                    };
                    match m89 {
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
                Ok(next) => { m90 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f26(v0: Bytes, v1: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
    f50(v0, v1)
}

pub fn f27(v0: List<Bytes>) -> List<Bytes> {
    f51(Fn9::F52, List::<Bytes>::nil(), v0)
}

pub fn f28(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m96 = {
        let a93 = v0;
        let a94 = v1;
        let a95 = v2;
        slice_bytes(a93, a94, a95)
    };
    match m96 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f29(v0: Fn5, p100: List<(Bytes, Bytes)>, p101: List<(Bytes, Bytes)>) -> List<(Bytes, Bytes)> {
    let m99 = (p100, p101);
    {
        let mut m99 = m99;
        loop {
            let step = {
                let (v1, v2) = m99;
                let m97 = v2;
                match m97.uncons() {
                    None => {
                        Err::<(List<(Bytes, Bytes)>, List<(Bytes, Bytes)>), List<(Bytes, Bytes)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Bytes, Bytes)>, List<(Bytes, Bytes)>), List<(Bytes, Bytes)>>(({
                            let c98: Fn5 = v0.clone();
                            c98.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m99 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f30(v0: List<(Bytes, Bytes)>, v1: (Bytes, Bytes)) -> List<(Bytes, Bytes)> {
    List::cons(v1, v0)
}

pub fn f31(v0: Fn6, p105: List<(Str, Str)>, p106: List<(Str, Str)>) -> List<(Str, Str)> {
    let m104 = (p105, p106);
    {
        let mut m104 = m104;
        loop {
            let step = {
                let (v1, v2) = m104;
                let m102 = v2;
                match m102.uncons() {
                    None => {
                        Err::<(List<(Str, Str)>, List<(Str, Str)>), List<(Str, Str)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Str, Str)>, List<(Str, Str)>), List<(Str, Str)>>(({
                            let c103: Fn6 = v0.clone();
                            c103.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m104 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f32(v0: List<(Str, Str)>, v1: (Str, Str)) -> List<(Str, Str)> {
    List::cons(v1, v0)
}

pub fn f33(v0: List<Bytes>) -> Bytes {
    let m109 = {
        let (h107, _) = f53(Fn10::F54, {
            let a108 = v0.clone();
            length_list(a108)
        }, v0);
        h107
    };
    match m109.uncons() {
        None => {
            Bytes::lit(&[])
        }
        Some((v1, _)) => {
            v1
        }
    }
}

pub fn f34(v0: Adt1, v1: List<(Bytes, Bytes)>) -> Bytes {
    let v2: Adt2 = match v0.clone() {
        Adt1::C0(h110, _, _) => {
            h110
        }
    };
    let v3: Str = match v2.clone() {
        Adt2::C0(h111, _, _, _, _, _) => {
            h111
        }
    };
    let v4: Bytes = f16(List::cons(Str::lit("obligation"), List::cons(v3.clone(), List::cons(f55(match v0.clone() {
        Adt1::C0(_, _, h112) => {
            h112
        }
    }), List::cons(match v0.clone() {
        Adt1::C0(_, h113, _) => {
            h113
        }
    }, List::<Str>::nil())))));
    let v5: Bytes = {
        let a119 = {
            let a116 = f16(List::cons(Str::lit("kind"), List::cons(v3.clone(), List::cons(f56(match v2.clone() {
                Adt2::C0(_, h114, _, _, _, _) => {
                    h114
                }
            }), List::<Str>::nil()))));
            let a117 = f16(List::cons(Str::lit("environment"), List::cons(v3.clone(), List::cons(match v2.clone() {
                Adt2::C0(_, _, h115, _, _, _) => {
                    h115
                }
            }, List::<Str>::nil()))));
            append_bytes(a116, a117)
        };
        let a120 = f16(List::cons(Str::lit("run"), List::cons(v3.clone(), List::cons(match v2.clone() {
            Adt2::C0(_, _, _, h118, _, _) => {
                h118
            }
        }, List::<Str>::nil()))));
        append_bytes(a119, a120)
    };
    let v6: Bytes = if f57(v0) {
        f58(v3.clone(), match v2.clone() {
            Adt2::C0(_, _, _, _, h121, _) => {
                h121
            }
        }, v1)
    } else {
        Bytes::lit(&[])
    };
    let a130 = {
        let a122 = v4;
        let a123 = v5;
        append_bytes(a122, a123)
    };
    let a131 = {
        let a128 = {
            let a126 = f59(Str::lit("input"), v3.clone(), match v2.clone() {
                Adt2::C0(_, _, _, _, h124, _) => {
                    h124
                }
            });
            let a127 = f59(Str::lit("needs"), v3, match v2 {
                Adt2::C0(_, _, _, _, _, h125) => {
                    h125
                }
            });
            append_bytes(a126, a127)
        };
        let a129 = v6;
        append_bytes(a128, a129)
    };
    append_bytes(a130, a131)
}

pub fn f35(v0: Fn7, p135: List<(Str, u64)>, p136: List<Adt1>) -> List<(Str, u64)> {
    let m134 = (p135, p136);
    {
        let mut m134 = m134;
        loop {
            let step = {
                let (v1, v2) = m134;
                let m132 = v2;
                match m132.uncons() {
                    None => {
                        Err::<(List<(Str, u64)>, List<Adt1>), List<(Str, u64)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Str, u64)>, List<Adt1>), List<(Str, u64)>>(({
                            let c133: Fn7 = v0.clone();
                            c133.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m134 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f36(v0: List<(Str, Str)>, v1: List<(Str, u64)>, v2: Adt1) -> List<(Str, u64)> {
    if f60(match v2.clone() {
        Adt1::C0(_, _, h137) => {
            h137
        }
    }) {
        f61(({
            let m140 = match v2.clone() {
                Adt1::C0(h139, _, _) => {
                    h139
                }
            };
            match m140 {
                Adt2::C0(h138, _, _, _, _, _) => {
                    h138
                }
            }
        }, f62(f63({
            let m143 = match v2 {
                Adt1::C0(h142, _, _) => {
                    h142
                }
            };
            match m143 {
                Adt2::C0(h141, _, _, _, _, _) => {
                    h141
                }
            }
        }, v0))), v1)
    } else {
        v1
    }
}

pub fn f37(v0: List<Bytes>) -> Bytes {
    let a144 = f64(v0, Bytes::lit(&[0x20]));
    let a145 = Bytes::lit(&[0x0a]);
    append_bytes(a144, a145)
}

pub fn f38(v0: Fn8, p149: List<Bytes>, p150: List<Str>) -> List<Bytes> {
    let m148 = (p149, p150);
    {
        let mut m148 = m148;
        loop {
            let step = {
                let (v1, v2) = m148;
                let m146 = v2;
                match m146.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Str>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Str>), List<Bytes>>(({
                            let c147: Fn8 = v0.clone();
                            c147.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m148 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f39(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a151 = v0;
    let a152 = List::cons(f47(v1), List::<Bytes>::nil());
    append_list(a151, a152)
}

pub fn f40(v0: List<Adt4>) -> bool {
    f65(Fn11::F66, true, v0)
}

pub fn f41(v0: List<Adt1>) -> List<Adt4> {
    f67(Fn12::F68, List::<Adt4>::nil(), v0)
}

pub fn f42(v0: List<(Bytes, Bytes)>, v1: List<(Str, bool)>, v2: (List<Adt1>, List<(List<Str>, Str)>), v3: Adt2) -> R<(List<Adt1>, List<(List<Str>, Str)>)> {
    let v4: List<Adt1> = {
        let (h153, _) = v2;
        h153
    };
    let v5: List<(List<Str>, Str)> = {
        let (_, h154) = v2;
        h154
    };
    let v6: List<Str> = match v3.clone() {
        Adt2::C0(_, _, _, _, h155, _) => {
            h155
        }
    };
    let v8: Str = {
        let m156 = f69(v6.clone(), v5.clone());
        match m156 {
            None => {
                f70(v6.clone(), v0)?
            }
            Some(v7) => {
                v7
            }
        }
    };
    let v9: List<Adt1> = f71(match v3.clone() {
        Adt2::C0(_, _, _, _, _, h157) => {
            h157
        }
    }, v4.clone());
    let v10: Str = f72(v3.clone(), v8.clone(), v9.clone())?;
    let v11: List<Adt4> = if f73(match v3.clone() {
        Adt2::C0(_, _, _, _, _, h158) => {
            h158
        }
    }, v4.clone()) {
        f41(v9)
    } else {
        List::cons(Adt4::C3, List::<Adt4>::nil())
    };
    Ok(({
        let a160 = v4;
        let a161 = List::cons(Adt1::C0(v3.clone(), v10.clone(), f74(match v3 {
            Adt2::C0(_, h159, _, _, _, _) => {
                h159
            }
        }, v10, v1, v11)), List::<Adt1>::nil());
        append_list(a160, a161)
    }, List::cons((v6, v8), v5)))
}

pub fn f43(v0: Str, v1: Adt3, v2: Str, v3: Str, v4: List<Str>, v5: List<Str>) -> Adt2 {
    Adt2::C0(v0, v1, v2, v3, v4, v5)
}

pub fn f44() -> List<Str> {
    List::cons(Str::lit(""), List::<Str>::nil())
}

pub fn f45() -> List<Str> {
    List::cons(Str::lit("source-vv-dk-3"), List::cons(Str::lit("source-vv-ho-oc"), List::cons(Str::lit("source-vv-dk-2"), List::cons(Str::lit("source-vv-gates"), List::cons(Str::lit("source-vv-st-sy"), List::cons(Str::lit("source-vv-dk-0-1"), List::cons(Str::lit("source-vv-conformance"), List::cons(Str::lit("source-vv-source"), List::<Str>::nil()))))))))
}

pub fn f46(v0: Bytes) -> R<(Bytes, Bytes)> {
    let m162 = f75(v0.clone(), Bytes::lit(&[0x20]))?;
    match m162 {
        None => {
            Ok((v0, Bytes::lit(&[])))
        }
        Some(v1) => {
            Ok((f28(v0.clone(), 0u64, v1), f28(v0.clone(), {
                let a163 = v1;
                let a164 = 1u64;
                nat_add(a163, a164)?
            }, {
                let a168 = {
                    let a165 = v0;
                    length_bytes(a165)
                };
                let a169 = {
                    let a166 = v1;
                    let a167 = 1u64;
                    nat_add(a166, a167)?
                };
                nat_sub(a168, a169)
            })))
        }
    }
}

pub fn f47(v0: Str) -> Bytes {
    let a170 = v0;
    utf8_encode(a170)
}

pub fn f48(v0: Bytes) -> Str {
    let m172 = {
        let a171 = v0;
        utf8_decode(a171)
    };
    match m172 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f49(v0: Bytes) -> bool {
    let a173 = v0;
    let a174 = f47(Str::lit("pass"));
    equal(a173, a174)
}

pub fn f50(v0: Bytes, v1: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
    let v2: u64 = {
        let (h175, _) = v1.clone();
        h175
    };
    let v3: u64 = {
        let (h176, _) = {
            let (_, h177) = v1.clone();
            h177
        };
        h176
    };
    let v4: List<Bytes> = {
        let (_, h178) = {
            let (_, h179) = v1;
            h179
        };
        h178
    };
    let m180 = f76(v0.clone(), v2);
    match m180 {
        None => {
            Ok(None::<(u64, (u64, List<Bytes>))>)
        }
        Some(v5) => {
            let a181 = v5;
            let a182 = 10u8;
            if equal(a181, a182) {
                Ok(Some(({
                    let a183 = v2;
                    let a184 = 1u64;
                    nat_add(a183, a184)?
                }, ({
                    let a185 = v2;
                    let a186 = 1u64;
                    nat_add(a185, a186)?
                }, List::cons(f77(f28(v0, v3, {
                    let a187 = v2;
                    let a188 = v3;
                    nat_sub(a187, a188)
                })), v4)))))
            } else {
                Ok(Some(({
                    let a189 = v2;
                    let a190 = 1u64;
                    nat_add(a189, a190)?
                }, (v3, v4))))
            }
        }
    }
}

pub fn f51(v0: Fn9, p194: List<Bytes>, p195: List<Bytes>) -> List<Bytes> {
    let m193 = (p194, p195);
    {
        let mut m193 = m193;
        loop {
            let step = {
                let (v1, v2) = m193;
                let m191 = v2;
                match m191.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Bytes>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Bytes>), List<Bytes>>(({
                            let c192: Fn9 = v0.clone();
                            c192.apply(v1, v3)
                        }, v4))
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

pub fn f52(v0: List<Bytes>, v1: Bytes) -> List<Bytes> {
    List::cons(v1, v0)
}

pub fn f53(v0: Fn10, p200: u64, p201: List<Bytes>) -> (List<Bytes>, bool) {
    let m199 = (p200, p201);
    {
        let mut m199 = m199;
        loop {
            let step = {
                let (v1, v2) = m199;
                let m196 = v1;
                if m196 == 0 {
                    Err::<(u64, List<Bytes>), (List<Bytes>, bool)>((v2, false))
                } else {
                    let v3 = m196 - 1;
                    let m198 = {
                        let c197: Fn10 = v0.clone();
                        c197.apply(v2.clone())
                    };
                    match m198 {
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
                Ok(next) => { m199 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f54(v0: List<Bytes>) -> Option<List<Bytes>> {
    f78(v0)
}

pub fn f55(v0: Adt4) -> Str {
    let m202 = v0;
    match m202 {
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

pub fn f56(v0: Adt3) -> Str {
    let m203 = v0;
    match m203 {
        Adt3::C0 => {
            Str::lit("hermetic")
        }
        Adt3::C1 => {
            Str::lit("fetch")
        }
    }
}

pub fn f57(v0: Adt1) -> bool {
    if f60(match v0.clone() {
        Adt1::C0(_, _, h204) => {
            h204
        }
    }) {
        let a208 = {
            let m207 = match v0 {
                Adt1::C0(h206, _, _) => {
                    h206
                }
            };
            match m207 {
                Adt2::C0(_, _, h205, _, _, _) => {
                    h205
                }
            }
        };
        let a209 = Str::lit("host");
        equal(a208, a209)
    } else {
        false
    }
}

pub fn f58(v0: Str, v1: List<Str>, v2: List<(Bytes, Bytes)>) -> Bytes {
    f10(f79(Fn13::F80(v1, v0), List::<Bytes>::nil(), v2))
}

pub fn f59(v0: Str, v1: Str, v2: List<Str>) -> Bytes {
    f10(f38(Fn8::F81(v1, v0), List::<Bytes>::nil(), v2))
}

pub fn f60(v0: Adt4) -> bool {
    let m210 = v0;
    match m210 {
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

pub fn f61(v0: (Str, u64), v1: List<(Str, u64)>) -> List<(Str, u64)> {
    let m211 = v1;
    match m211.uncons() {
        None => {
            List::cons(v0, List::<(Str, u64)>::nil())
        }
        Some((v2, v3)) => {
            let a214 = {
                let (_, h212) = v2.clone();
                h212
            };
            let a215 = {
                let (_, h213) = v0.clone();
                h213
            };
            if nat_lt(a214, a215) {
                List::cons(v0, List::cons(v2, v3))
            } else {
                List::cons(v2, f61(v0, v3))
            }
        }
    }
}

pub fn f62(v0: Adt5) -> u64 {
    let m216 = v0;
    match m216 {
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

pub fn f63(v0: Str, v1: List<(Str, Str)>) -> Adt5 {
    f82(f83(Fn14::F84(v0), Str::lit("4"), v1))
}

pub fn f64(v0: List<Bytes>, v1: Bytes) -> Bytes {
    let m217 = f85(Fn15::F86(v1), None::<Bytes>, v0);
    match m217 {
        None => {
            Bytes::lit(&[])
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f65(v0: Fn11, p221: bool, p222: List<Adt4>) -> bool {
    let m220 = (p221, p222);
    {
        let mut m220 = m220;
        loop {
            let step = {
                let (v1, v2) = m220;
                let m218 = v2;
                match m218.uncons() {
                    None => {
                        Err::<(bool, List<Adt4>), bool>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(bool, List<Adt4>), bool>(({
                            let c219: Fn11 = v0.clone();
                            c219.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m220 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f66(v0: bool, v1: Adt4) -> bool {
    if v0 {
        f87(v1)
    } else {
        false
    }
}

pub fn f67(v0: Fn12, p226: List<Adt4>, p227: List<Adt1>) -> List<Adt4> {
    let m225 = (p226, p227);
    {
        let mut m225 = m225;
        loop {
            let step = {
                let (v1, v2) = m225;
                let m223 = v2;
                match m223.uncons() {
                    None => {
                        Err::<(List<Adt4>, List<Adt1>), List<Adt4>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Adt4>, List<Adt1>), List<Adt4>>(({
                            let c224: Fn12 = v0.clone();
                            c224.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m225 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f68(v0: List<Adt4>, v1: Adt1) -> List<Adt4> {
    let a229 = v0;
    let a230 = List::cons(match v1 {
        Adt1::C0(_, _, h228) => {
            h228
        }
    }, List::<Adt4>::nil());
    append_list(a229, a230)
}

pub fn f69(v0: List<Str>, v1: List<(List<Str>, Str)>) -> Option<Str> {
    f88(Fn16::F89(v0), None::<Str>, v1)
}

pub fn f70(v0: List<Str>, v1: List<(Bytes, Bytes)>) -> R<Str> {
    Ok(f48(f90(f91(v0, v1))?))
}

pub fn f71(v0: List<Str>, v1: List<Adt1>) -> List<Adt1> {
    f92(Fn17::F93(v1), List::<Adt1>::nil(), v0)
}

pub fn f72(v0: Adt2, v1: Str, v2: List<Adt1>) -> R<Str> {
    Ok(f48(f90(f94(v0, v1, v2))?))
}

pub fn f73(v0: List<Str>, v1: List<Adt1>) -> bool {
    f95(Fn18::F96(v1), true, v0)
}

pub fn f74(v0: Adt3, v1: Str, v2: List<(Str, bool)>, v3: List<Adt4>) -> Adt4 {
    if f40(v3) {
        let m231 = f97(v1, v2);
        match m231 {
            None => {
                Adt4::C1
            }
            Some(v4) => {
                if v4 {
                    Adt4::C0
                } else {
                    let m232 = v0;
                    match m232 {
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

pub fn f75(v0: Bytes, v1: Bytes) -> R<Option<u64>> {
    let v2: u64 = {
        let a233 = v0.clone();
        length_bytes(a233)
    };
    let v3: u64 = {
        let a234 = v1.clone();
        length_bytes(a234)
    };
    let v4: u64 = {
        let (h235, _) = f98(Fn19::F99(v1.clone(), v3, v0.clone(), v2), {
            let a236 = v2;
            let a237 = 1u64;
            nat_add(a236, a237)?
        }, 0u64)?;
        h235
    };
    if f100(v0, v1, v4) {
        Ok(Some(v4))
    } else {
        Ok(None::<u64>)
    }
}

pub fn f76(v0: Bytes, v1: u64) -> Option<u8> {
    let a238 = v0;
    let a239 = v1;
    index_bytes(a238, a239)
}

pub fn f77(v0: Bytes) -> Bytes {
    if f101(v0.clone(), Bytes::lit(&[0x0d])) {
        f28(v0.clone(), 0u64, {
            let a241 = {
                let a240 = v0;
                length_bytes(a240)
            };
            let a242 = 1u64;
            nat_sub(a241, a242)
        })
    } else {
        v0
    }
}

pub fn f78(v0: List<Bytes>) -> Option<List<Bytes>> {
    let m243 = v0.clone();
    match m243.uncons() {
        None => {
            None::<List<Bytes>>
        }
        Some((_, v2)) => {
            let m244 = v2;
            match m244.uncons() {
                None => {
                    None::<List<Bytes>>
                }
                Some((_, _)) => {
                    let v5: (Option<Bytes>, List<Bytes>) = f102(Fn20::F103, (None::<Bytes>, List::<Bytes>::nil()), v0);
                    let m246 = {
                        let (h245, _) = v5;
                        h245
                    };
                    match m246 {
                        None => {
                            Some(f27({
                                let (_, h248) = v5;
                                h248
                            }))
                        }
                        Some(v6) => {
                            Some(f27(List::cons(v6, {
                                let (_, h247) = v5;
                                h247
                            })))
                        }
                    }
                }
            }
        }
    }
}

pub fn f79(v0: Fn13, p252: List<Bytes>, p253: List<(Bytes, Bytes)>) -> List<Bytes> {
    let m251 = (p252, p253);
    {
        let mut m251 = m251;
        loop {
            let step = {
                let (v1, v2) = m251;
                let m249 = v2;
                match m249.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<(Bytes, Bytes)>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<(Bytes, Bytes)>), List<Bytes>>(({
                            let c250: Fn13 = v0.clone();
                            c250.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m251 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f80(v0: List<Str>, v1: Str, v2: List<Bytes>, v3: (Bytes, Bytes)) -> List<Bytes> {
    if f104(v0, {
        let (_, h254) = v3.clone();
        h254
    }) {
        List::cons(f37(List::cons(f47(Str::lit("file")), List::cons(f47(v1), List::cons({
            let (_, h255) = v3;
            h255
        }, List::<Bytes>::nil())))), v2)
    } else {
        v2
    }
}

pub fn f81(v0: Str, v1: Str, v2: List<Bytes>, v3: Str) -> List<Bytes> {
    List::cons(f16(List::cons(v1, List::cons(v0, List::cons(v3, List::<Str>::nil())))), v2)
}

pub fn f82(v0: Str) -> Adt5 {
    let a256 = v0.clone();
    let a257 = Str::lit("1");
    if equal(a256, a257) {
        Adt5::C0
    } else {
        let a258 = v0.clone();
        let a259 = Str::lit("2");
        if equal(a258, a259) {
            Adt5::C1
        } else {
            let a260 = v0.clone();
            let a261 = Str::lit("3");
            if equal(a260, a261) {
                Adt5::C2
            } else {
                let a262 = v0.clone();
                let a263 = Str::lit("4");
                if equal(a262, a263) {
                    Adt5::C3
                } else {
                    let a264 = v0.clone();
                    let a265 = Str::lit("5");
                    if equal(a264, a265) {
                        Adt5::C4
                    } else {
                        let a266 = v0.clone();
                        let a267 = Str::lit("6");
                        if equal(a266, a267) {
                            Adt5::C5
                        } else {
                            let a268 = v0.clone();
                            let a269 = Str::lit("7");
                            if equal(a268, a269) {
                                Adt5::C6
                            } else {
                                let a270 = v0.clone();
                                let a271 = Str::lit("8");
                                if equal(a270, a271) {
                                    Adt5::C7
                                } else {
                                    let a272 = v0.clone();
                                    let a273 = Str::lit("9");
                                    if equal(a272, a273) {
                                        Adt5::C8
                                    } else {
                                        let a274 = v0.clone();
                                        let a275 = Str::lit("10");
                                        if equal(a274, a275) {
                                            Adt5::C9
                                        } else {
                                            let a276 = v0.clone();
                                            let a277 = Str::lit("11");
                                            if equal(a276, a277) {
                                                Adt5::C10
                                            } else {
                                                let a278 = v0.clone();
                                                let a279 = Str::lit("12");
                                                if equal(a278, a279) {
                                                    Adt5::C11
                                                } else {
                                                    let a280 = v0.clone();
                                                    let a281 = Str::lit("13");
                                                    if equal(a280, a281) {
                                                        Adt5::C12
                                                    } else {
                                                        let a282 = v0.clone();
                                                        let a283 = Str::lit("14");
                                                        if equal(a282, a283) {
                                                            Adt5::C13
                                                        } else {
                                                            let a284 = v0.clone();
                                                            let a285 = Str::lit("15");
                                                            if equal(a284, a285) {
                                                                Adt5::C14
                                                            } else {
                                                                let a286 = v0;
                                                                let a287 = Str::lit("16");
                                                                if equal(a286, a287) {
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

pub fn f83(v0: Fn14, p291: Str, p292: List<(Str, Str)>) -> Str {
    let m290 = (p291, p292);
    {
        let mut m290 = m290;
        loop {
            let step = {
                let (v1, v2) = m290;
                let m288 = v2;
                match m288.uncons() {
                    None => {
                        Err::<(Str, List<(Str, Str)>), Str>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Str, List<(Str, Str)>), Str>(({
                            let c289: Fn14 = v0.clone();
                            c289.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m290 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f84(v0: Str, v1: Str, v2: (Str, Str)) -> Str {
    let a294 = {
        let (h293, _) = v2;
        h293
    };
    let a295 = v0;
    if equal(a294, a295) {
        let (_, h296) = v2;
        h296
    } else {
        v1
    }
}

pub fn f85(v0: Fn15, p300: Option<Bytes>, p301: List<Bytes>) -> Option<Bytes> {
    let m299 = (p300, p301);
    {
        let mut m299 = m299;
        loop {
            let step = {
                let (v1, v2) = m299;
                let m297 = v2;
                match m297.uncons() {
                    None => {
                        Err::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(({
                            let c298: Fn15 = v0.clone();
                            c298.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m299 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f86(v0: Bytes, v1: Option<Bytes>, v2: Bytes) -> Option<Bytes> {
    let m302 = v1;
    match m302 {
        None => {
            Some(v2)
        }
        Some(v3) => {
            Some({
                let a305 = {
                    let a303 = v3;
                    let a304 = v0;
                    append_bytes(a303, a304)
                };
                let a306 = v2;
                append_bytes(a305, a306)
            })
        }
    }
}

pub fn f87(v0: Adt4) -> bool {
    let m307 = v0;
    match m307 {
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

pub fn f88(v0: Fn16, p311: Option<Str>, p312: List<(List<Str>, Str)>) -> Option<Str> {
    let m310 = (p311, p312);
    {
        let mut m310 = m310;
        loop {
            let step = {
                let (v1, v2) = m310;
                let m308 = v2;
                match m308.uncons() {
                    None => {
                        Err::<(Option<Str>, List<(List<Str>, Str)>), Option<Str>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Str>, List<(List<Str>, Str)>), Option<Str>>(({
                            let c309: Fn16 = v0.clone();
                            c309.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m310 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f89(v0: List<Str>, v1: Option<Str>, v2: (List<Str>, Str)) -> Option<Str> {
    if f105({
        let (h313, _) = v2;
        h313
    }, v0) {
        Some({
            let (_, h314) = v2;
            h314
        })
    } else {
        v1
    }
}

pub fn f90(v0: Bytes) -> R<Bytes> {
    Ok(f106(f107(v0)?))
}

pub fn f91(v0: List<Str>, v1: List<(Bytes, Bytes)>) -> Bytes {
    f10(f79(Fn13::F108(v0), List::<Bytes>::nil(), v1))
}

pub fn f92(v0: Fn17, p318: List<Adt1>, p319: List<Str>) -> List<Adt1> {
    let m317 = (p318, p319);
    {
        let mut m317 = m317;
        loop {
            let step = {
                let (v1, v2) = m317;
                let m315 = v2;
                match m315.uncons() {
                    None => {
                        Err::<(List<Adt1>, List<Str>), List<Adt1>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Adt1>, List<Str>), List<Adt1>>(({
                            let c316: Fn17 = v0.clone();
                            c316.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m317 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f93(v0: List<Adt1>, v1: List<Adt1>, v2: Str) -> List<Adt1> {
    let m320 = f109(v2, v0);
    match m320 {
        None => {
            v1
        }
        Some(v3) => {
            let a321 = v1;
            let a322 = List::cons(v3, List::<Adt1>::nil());
            append_list(a321, a322)
        }
    }
}

pub fn f94(v0: Adt2, v1: Str, v2: List<Adt1>) -> Bytes {
    f33(List::cons(f37(List::cons(f47(Str::lit("prism-factory/obligation/1")), List::<Bytes>::nil())), List::cons(f37(List::cons(f47(Str::lit("name")), List::cons(f47(match v0.clone() {
        Adt2::C0(h323, _, _, _, _, _) => {
            h323
        }
    }), List::<Bytes>::nil()))), List::cons(f37(List::cons(f47(Str::lit("kind")), List::cons(f47(f56(match v0.clone() {
        Adt2::C0(_, h324, _, _, _, _) => {
            h324
        }
    })), List::<Bytes>::nil()))), List::cons(f37(List::cons(f47(Str::lit("environment")), List::cons(f47(match v0.clone() {
        Adt2::C0(_, _, h325, _, _, _) => {
            h325
        }
    }), List::<Bytes>::nil()))), List::cons(f37(List::cons(f47(Str::lit("run")), List::cons(f47(match v0.clone() {
        Adt2::C0(_, _, _, h326, _, _) => {
            h326
        }
    }), List::<Bytes>::nil()))), List::cons(f110(Str::lit("input"), match v0 {
        Adt2::C0(_, _, _, _, h327, _) => {
            h327
        }
    }), List::cons(f111(v2), List::cons(f37(List::cons(f47(Str::lit("closure")), List::cons(f47(v1), List::<Bytes>::nil()))), List::<Bytes>::nil())))))))))
}

pub fn f95(v0: Fn18, p331: bool, p332: List<Str>) -> bool {
    let m330 = (p331, p332);
    {
        let mut m330 = m330;
        loop {
            let step = {
                let (v1, v2) = m330;
                let m328 = v2;
                match m328.uncons() {
                    None => {
                        Err::<(bool, List<Str>), bool>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(bool, List<Str>), bool>(({
                            let c329: Fn18 = v0.clone();
                            c329.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m330 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f96(v0: List<Adt1>, v1: bool, v2: Str) -> bool {
    let m333 = f109(v2, v0);
    match m333 {
        None => {
            false
        }
        Some(_) => {
            v1
        }
    }
}

pub fn f97(v0: Str, v1: List<(Str, bool)>) -> Option<bool> {
    f112(Fn21::F113(v0), None::<bool>, v1)
}

pub fn f98(v0: Fn19, p338: u64, p339: u64) -> R<(u64, bool)> {
    let m337 = (p338, p339);
    {
        let mut m337 = m337;
        loop {
            let step = {
                let (v1, v2) = m337;
                let m334 = v1;
                if m334 == 0 {
                    Err::<(u64, u64), R<(u64, bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m334 - 1;
                    let m336 = {
                        let c335: Fn19 = v0.clone();
                        c335.apply(v2)?
                    };
                    match m336 {
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
                Ok(next) => { m337 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f99(v0: Bytes, v1: u64, v2: Bytes, v3: u64, v4: u64) -> R<Option<u64>> {
    if f100(v2, v0, v4) {
        Ok(None::<u64>)
    } else {
        let a342 = {
            let a340 = v4;
            let a341 = v1;
            nat_add(a340, a341)?
        };
        let a343 = v3;
        if nat_lt(a342, a343) {
            Ok(Some({
                let a344 = v4;
                let a345 = 1u64;
                nat_add(a344, a345)?
            }))
        } else {
            Ok(None::<u64>)
        }
    }
}

pub fn f100(v0: Bytes, v1: Bytes, v2: u64) -> bool {
    let m350 = {
        let a347 = v0;
        let a348 = v2;
        let a349 = {
            let a346 = v1.clone();
            length_bytes(a346)
        };
        slice_bytes(a347, a348, a349)
    };
    match m350 {
        None => {
            false
        }
        Some(v3) => {
            let a351 = v3;
            let a352 = v1;
            equal(a351, a352)
        }
    }
}

pub fn f101(v0: Bytes, v1: Bytes) -> bool {
    let v2: u64 = {
        let a353 = v0.clone();
        length_bytes(a353)
    };
    let v3: u64 = {
        let a354 = v1.clone();
        length_bytes(a354)
    };
    let a355 = v3;
    let a356 = v2;
    if nat_le(a355, a356) {
        let m362 = {
            let a359 = v0;
            let a360 = {
                let a357 = v2;
                let a358 = v3;
                nat_sub(a357, a358)
            };
            let a361 = v3;
            slice_bytes(a359, a360, a361)
        };
        match m362 {
            None => {
                false
            }
            Some(v4) => {
                let a363 = v4;
                let a364 = v1;
                equal(a363, a364)
            }
        }
    } else {
        false
    }
}

pub fn f102(v0: Fn20, p368: (Option<Bytes>, List<Bytes>), p369: List<Bytes>) -> (Option<Bytes>, List<Bytes>) {
    let m367 = (p368, p369);
    {
        let mut m367 = m367;
        loop {
            let step = {
                let (v1, v2) = m367;
                let m365 = v2;
                match m365.uncons() {
                    None => {
                        Err::<((Option<Bytes>, List<Bytes>), List<Bytes>), (Option<Bytes>, List<Bytes>)>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<((Option<Bytes>, List<Bytes>), List<Bytes>), (Option<Bytes>, List<Bytes>)>(({
                            let c366: Fn20 = v0.clone();
                            c366.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m367 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f103(v0: (Option<Bytes>, List<Bytes>), v1: Bytes) -> (Option<Bytes>, List<Bytes>) {
    f114(v0, v1)
}

pub fn f104(p372: List<Str>, v1: Bytes) -> bool {
    let m371 = p372;
    {
        let mut m371 = m371;
        loop {
            let step = {
                let v0 = m371;
                let m370 = v0;
                match m370.uncons() {
                    None => {
                        Err::<List<Str>, bool>(false)
                    }
                    Some((v2, v3)) => {
                        if f115(v1.clone(), f47(v2)) {
                            Err::<List<Str>, bool>(true)
                        } else {
                            Ok::<List<Str>, bool>(v3)
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

pub fn f105(p379: List<Str>, p380: List<Str>) -> bool {
    let m378 = (p379, p380);
    {
        let mut m378 = m378;
        loop {
            let step = {
                let (v0, v1) = m378;
                let m373 = v0;
                match m373.uncons() {
                    None => {
                        let m374 = v1;
                        match m374.uncons() {
                            None => {
                                Err::<(List<Str>, List<Str>), bool>(true)
                            }
                            Some((_, _)) => {
                                Err::<(List<Str>, List<Str>), bool>(false)
                            }
                        }
                    }
                    Some((v4, v5)) => {
                        let m375 = v1;
                        match m375.uncons() {
                            None => {
                                Err::<(List<Str>, List<Str>), bool>(false)
                            }
                            Some((v6, v7)) => {
                                let a376 = v4;
                                let a377 = v6;
                                if equal(a376, a377) {
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
                Ok(next) => { m378 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f106(v0: Adt6) -> Bytes {
    let a401 = f116(match v0.clone() {
        Adt6::C0(h381, _, _, _, _, _, _, _) => {
            h381
        }
    });
    let a402 = {
        let a399 = f116(match v0.clone() {
            Adt6::C0(_, h382, _, _, _, _, _, _) => {
                h382
            }
        });
        let a400 = {
            let a397 = f116(match v0.clone() {
                Adt6::C0(_, _, h383, _, _, _, _, _) => {
                    h383
                }
            });
            let a398 = {
                let a395 = f116(match v0.clone() {
                    Adt6::C0(_, _, _, h384, _, _, _, _) => {
                        h384
                    }
                });
                let a396 = {
                    let a393 = f116(match v0.clone() {
                        Adt6::C0(_, _, _, _, h385, _, _, _) => {
                            h385
                        }
                    });
                    let a394 = {
                        let a391 = f116(match v0.clone() {
                            Adt6::C0(_, _, _, _, _, h386, _, _) => {
                                h386
                            }
                        });
                        let a392 = {
                            let a389 = f116(match v0.clone() {
                                Adt6::C0(_, _, _, _, _, _, h387, _) => {
                                    h387
                                }
                            });
                            let a390 = f116(match v0 {
                                Adt6::C0(_, _, _, _, _, _, _, h388) => {
                                    h388
                                }
                            });
                            append_bytes(a389, a390)
                        };
                        append_bytes(a391, a392)
                    };
                    append_bytes(a393, a394)
                };
                append_bytes(a395, a396)
            };
            append_bytes(a397, a398)
        };
        append_bytes(a399, a400)
    };
    append_bytes(a401, a402)
}

pub fn f107(v0: Bytes) -> R<Adt6> {
    let v1: u64 = {
        let a403 = v0.clone();
        length_bytes(a403)
    };
    let v2: u64 = {
        let a409 = {
            let a406 = {
                let a404 = v1;
                let a405 = 72u64;
                nat_add(a404, a405)?
            };
            let a407 = 64u64;
            let a408 = 0u64;
            nat_quot(a406, a407, a408)
        };
        let a410 = 64u64;
        nat_mul(a409, a410)?
    };
    let (_, h411) = f117(Fn22::F118(v0, v1, v2), {
        let a412 = v2;
        let a413 = 64u64;
        let a414 = 0u64;
        nat_quot(a412, a413, a414)
    }, (0u64, f119()))?;
    Ok(h411)
}

pub fn f108(v0: List<Str>, v1: List<Bytes>, v2: (Bytes, Bytes)) -> List<Bytes> {
    if f104(v0, {
        let (_, h415) = v2.clone();
        h415
    }) {
        List::cons(f37(List::cons(f47(Str::lit("file")), List::cons({
            let (h416, _) = v2;
            h416
        }, List::cons({
            let (_, h417) = v2;
            h417
        }, List::<Bytes>::nil())))), v1)
    } else {
        v1
    }
}

pub fn f109(v0: Str, v1: List<Adt1>) -> Option<Adt1> {
    f120(Fn23::F121(v0), None::<Adt1>, v1)
}

pub fn f110(v0: Str, v1: List<Str>) -> Bytes {
    f10(f38(Fn8::F122(v0), List::<Bytes>::nil(), v1))
}

pub fn f111(v0: List<Adt1>) -> Bytes {
    f10(f11(Fn1::F123, List::<Bytes>::nil(), v0))
}

pub fn f112(v0: Fn21, p421: Option<bool>, p422: List<(Str, bool)>) -> Option<bool> {
    let m420 = (p421, p422);
    {
        let mut m420 = m420;
        loop {
            let step = {
                let (v1, v2) = m420;
                let m418 = v2;
                match m418.uncons() {
                    None => {
                        Err::<(Option<bool>, List<(Str, bool)>), Option<bool>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<bool>, List<(Str, bool)>), Option<bool>>(({
                            let c419: Fn21 = v0.clone();
                            c419.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m420 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f113(v0: Str, v1: Option<bool>, v2: (Str, bool)) -> Option<bool> {
    let a424 = {
        let (h423, _) = v2;
        h423
    };
    let a425 = v0;
    if equal(a424, a425) {
        let m426 = v1;
        match m426 {
            None => {
                Some({
                    let (_, h428) = v2;
                    h428
                })
            }
            Some(v3) => {
                Some(if v3 {
                    true
                } else {
                    let (_, h427) = v2;
                    h427
                })
            }
        }
    } else {
        v1
    }
}

pub fn f114(v0: (Option<Bytes>, List<Bytes>), v1: Bytes) -> (Option<Bytes>, List<Bytes>) {
    let m430 = {
        let (h429, _) = v0;
        h429
    };
    match m430 {
        None => {
            (Some(v1), {
                let (_, h431) = v0;
                h431
            })
        }
        Some(v2) => {
            (None::<Bytes>, List::cons({
                let a432 = v2;
                let a433 = v1;
                append_bytes(a432, a433)
            }, {
                let (_, h434) = v0;
                h434
            }))
        }
    }
}

pub fn f115(v0: Bytes, v1: Bytes) -> bool {
    let m439 = {
        let a436 = v0;
        let a437 = 0u64;
        let a438 = {
            let a435 = v1.clone();
            length_bytes(a435)
        };
        slice_bytes(a436, a437, a438)
    };
    match m439 {
        None => {
            false
        }
        Some(v2) => {
            let a440 = v2;
            let a441 = v1;
            equal(a440, a441)
        }
    }
}

pub fn f116(v0: u32) -> Bytes {
    let a470 = f124({
        let a442 = f125(v0, 28u32);
        let a443 = 15u32;
        fixed_u32::bit_and(a442, a443)
    });
    let a471 = {
        let a468 = f124({
            let a444 = f125(v0, 24u32);
            let a445 = 15u32;
            fixed_u32::bit_and(a444, a445)
        });
        let a469 = {
            let a466 = f124({
                let a446 = f125(v0, 20u32);
                let a447 = 15u32;
                fixed_u32::bit_and(a446, a447)
            });
            let a467 = {
                let a464 = f124({
                    let a448 = f125(v0, 16u32);
                    let a449 = 15u32;
                    fixed_u32::bit_and(a448, a449)
                });
                let a465 = {
                    let a462 = f124({
                        let a450 = f125(v0, 12u32);
                        let a451 = 15u32;
                        fixed_u32::bit_and(a450, a451)
                    });
                    let a463 = {
                        let a460 = f124({
                            let a452 = f125(v0, 8u32);
                            let a453 = 15u32;
                            fixed_u32::bit_and(a452, a453)
                        });
                        let a461 = {
                            let a458 = f124({
                                let a454 = f125(v0, 4u32);
                                let a455 = 15u32;
                                fixed_u32::bit_and(a454, a455)
                            });
                            let a459 = f124({
                                let a456 = v0;
                                let a457 = 15u32;
                                fixed_u32::bit_and(a456, a457)
                            });
                            append_bytes(a458, a459)
                        };
                        append_bytes(a460, a461)
                    };
                    append_bytes(a462, a463)
                };
                append_bytes(a464, a465)
            };
            append_bytes(a466, a467)
        };
        append_bytes(a468, a469)
    };
    append_bytes(a470, a471)
}

pub fn f117(v0: Fn22, p475: u64, p476: (u64, Adt6)) -> R<(u64, Adt6)> {
    let m474 = (p475, p476);
    {
        let mut m474 = m474;
        loop {
            let step = {
                let (v1, v2) = m474;
                let m472 = v1;
                if m472 == 0 {
                    Err::<(u64, (u64, Adt6)), R<(u64, Adt6)>>(Ok(v2))
                } else {
                    let v3 = m472 - 1;
                    Ok::<(u64, (u64, Adt6)), R<(u64, Adt6)>>((v3, {
                        let c473: Fn22 = v0.clone();
                        c473.apply(v2)?
                    }))
                }
            };
            match step {
                Ok(next) => { m474 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f118(v0: Bytes, v1: u64, v2: u64, v3: (u64, Adt6)) -> R<(u64, Adt6)> {
    Ok(({
        let a478 = {
            let (h477, _) = v3.clone();
            h477
        };
        let a479 = 64u64;
        nat_add(a478, a479)?
    }, f126({
        let (_, h480) = v3;
        h480
    }, f127(v0, v1, v2, {
        let (h481, _) = v3;
        h481
    })?)?))
}

pub fn f119() -> Adt6 {
    Adt6::C0(1779033703u32, 3144134277u32, 1013904242u32, 2773480762u32, 1359893119u32, 2600822924u32, 528734635u32, 1541459225u32)
}

pub fn f120(v0: Fn23, p485: Option<Adt1>, p486: List<Adt1>) -> Option<Adt1> {
    let m484 = (p485, p486);
    {
        let mut m484 = m484;
        loop {
            let step = {
                let (v1, v2) = m484;
                let m482 = v2;
                match m482.uncons() {
                    None => {
                        Err::<(Option<Adt1>, List<Adt1>), Option<Adt1>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Adt1>, List<Adt1>), Option<Adt1>>(({
                            let c483: Fn23 = v0.clone();
                            c483.apply(v1, v3)
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

pub fn f121(v0: Str, v1: Option<Adt1>, v2: Adt1) -> Option<Adt1> {
    let a490 = {
        let m489 = match v2.clone() {
            Adt1::C0(h488, _, _) => {
                h488
            }
        };
        match m489 {
            Adt2::C0(h487, _, _, _, _, _) => {
                h487
            }
        }
    };
    let a491 = v0;
    if equal(a490, a491) {
        Some(v2)
    } else {
        v1
    }
}

pub fn f122(v0: Str, v1: List<Bytes>, v2: Str) -> List<Bytes> {
    List::cons(f37(List::cons(f47(v0), List::cons(f47(v2), List::<Bytes>::nil()))), v1)
}

pub fn f123(v0: List<Bytes>, v1: Adt1) -> List<Bytes> {
    List::cons(f37(List::cons(f47(Str::lit("needs")), List::cons(f47({
        let m494 = match v1.clone() {
            Adt1::C0(h493, _, _) => {
                h493
            }
        };
        match m494 {
            Adt2::C0(h492, _, _, _, _, _) => {
                h492
            }
        }
    }), List::cons(f47(match v1 {
        Adt1::C0(_, h495, _) => {
            h495
        }
    }), List::<Bytes>::nil())))), v0)
}

pub fn f124(v0: u32) -> Bytes {
    let a498 = {
        let a496 = v0;
        let a497 = 8u32;
        fixed_u32::bit_and(a496, a497)
    };
    let a499 = 0u32;
    if equal(a498, a499) {
        let a502 = {
            let a500 = v0;
            let a501 = 4u32;
            fixed_u32::bit_and(a500, a501)
        };
        let a503 = 0u32;
        if equal(a502, a503) {
            let a506 = {
                let a504 = v0;
                let a505 = 2u32;
                fixed_u32::bit_and(a504, a505)
            };
            let a507 = 0u32;
            if equal(a506, a507) {
                let a510 = {
                    let a508 = v0;
                    let a509 = 1u32;
                    fixed_u32::bit_and(a508, a509)
                };
                let a511 = 0u32;
                if equal(a510, a511) {
                    Bytes::lit(&[0x30])
                } else {
                    Bytes::lit(&[0x31])
                }
            } else {
                let a514 = {
                    let a512 = v0;
                    let a513 = 1u32;
                    fixed_u32::bit_and(a512, a513)
                };
                let a515 = 0u32;
                if equal(a514, a515) {
                    Bytes::lit(&[0x32])
                } else {
                    Bytes::lit(&[0x33])
                }
            }
        } else {
            let a518 = {
                let a516 = v0;
                let a517 = 2u32;
                fixed_u32::bit_and(a516, a517)
            };
            let a519 = 0u32;
            if equal(a518, a519) {
                let a522 = {
                    let a520 = v0;
                    let a521 = 1u32;
                    fixed_u32::bit_and(a520, a521)
                };
                let a523 = 0u32;
                if equal(a522, a523) {
                    Bytes::lit(&[0x34])
                } else {
                    Bytes::lit(&[0x35])
                }
            } else {
                let a526 = {
                    let a524 = v0;
                    let a525 = 1u32;
                    fixed_u32::bit_and(a524, a525)
                };
                let a527 = 0u32;
                if equal(a526, a527) {
                    Bytes::lit(&[0x36])
                } else {
                    Bytes::lit(&[0x37])
                }
            }
        }
    } else {
        let a530 = {
            let a528 = v0;
            let a529 = 4u32;
            fixed_u32::bit_and(a528, a529)
        };
        let a531 = 0u32;
        if equal(a530, a531) {
            let a534 = {
                let a532 = v0;
                let a533 = 2u32;
                fixed_u32::bit_and(a532, a533)
            };
            let a535 = 0u32;
            if equal(a534, a535) {
                let a538 = {
                    let a536 = v0;
                    let a537 = 1u32;
                    fixed_u32::bit_and(a536, a537)
                };
                let a539 = 0u32;
                if equal(a538, a539) {
                    Bytes::lit(&[0x38])
                } else {
                    Bytes::lit(&[0x39])
                }
            } else {
                let a542 = {
                    let a540 = v0;
                    let a541 = 1u32;
                    fixed_u32::bit_and(a540, a541)
                };
                let a543 = 0u32;
                if equal(a542, a543) {
                    Bytes::lit(&[0x61])
                } else {
                    Bytes::lit(&[0x62])
                }
            }
        } else {
            let a546 = {
                let a544 = v0;
                let a545 = 2u32;
                fixed_u32::bit_and(a544, a545)
            };
            let a547 = 0u32;
            if equal(a546, a547) {
                let a550 = {
                    let a548 = v0;
                    let a549 = 1u32;
                    fixed_u32::bit_and(a548, a549)
                };
                let a551 = 0u32;
                if equal(a550, a551) {
                    Bytes::lit(&[0x63])
                } else {
                    Bytes::lit(&[0x64])
                }
            } else {
                let a554 = {
                    let a552 = v0;
                    let a553 = 1u32;
                    fixed_u32::bit_and(a552, a553)
                };
                let a555 = 0u32;
                if equal(a554, a555) {
                    Bytes::lit(&[0x65])
                } else {
                    Bytes::lit(&[0x66])
                }
            }
        }
    }
}

pub fn f125(v0: u32, v1: u32) -> u32 {
    let m558 = {
        let a556 = v0;
        let a557 = v1;
        fixed_u32::shift_right(a556, a557)
    };
    match m558 {
        None => {
            0u32
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f126(v0: Adt6, v1: Adt7) -> R<Adt6> {
    let v2: Adt8 = f128(Fn24::F129, 64u64, Adt8::C0(match v0.clone() {
        Adt6::C0(h559, _, _, _, _, _, _, _) => {
            h559
        }
    }, match v0.clone() {
        Adt6::C0(_, h560, _, _, _, _, _, _) => {
            h560
        }
    }, match v0.clone() {
        Adt6::C0(_, _, h561, _, _, _, _, _) => {
            h561
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, h562, _, _, _, _) => {
            h562
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, h563, _, _, _) => {
            h563
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, _, h564, _, _) => {
            h564
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, _, _, h565, _) => {
            h565
        }
    }, match v0.clone() {
        Adt6::C0(_, _, _, _, _, _, _, h566) => {
            h566
        }
    }, v1, 0u64))?;
    Ok(Adt6::C0(f130(match v0.clone() {
        Adt6::C0(h567, _, _, _, _, _, _, _) => {
            h567
        }
    }, match v2.clone() {
        Adt8::C0(h568, _, _, _, _, _, _, _, _, _) => {
            h568
        }
    }), f130(match v0.clone() {
        Adt6::C0(_, h569, _, _, _, _, _, _) => {
            h569
        }
    }, match v2.clone() {
        Adt8::C0(_, h570, _, _, _, _, _, _, _, _) => {
            h570
        }
    }), f130(match v0.clone() {
        Adt6::C0(_, _, h571, _, _, _, _, _) => {
            h571
        }
    }, match v2.clone() {
        Adt8::C0(_, _, h572, _, _, _, _, _, _, _) => {
            h572
        }
    }), f130(match v0.clone() {
        Adt6::C0(_, _, _, h573, _, _, _, _) => {
            h573
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, h574, _, _, _, _, _, _) => {
            h574
        }
    }), f130(match v0.clone() {
        Adt6::C0(_, _, _, _, h575, _, _, _) => {
            h575
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, _, h576, _, _, _, _, _) => {
            h576
        }
    }), f130(match v0.clone() {
        Adt6::C0(_, _, _, _, _, h577, _, _) => {
            h577
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, _, _, h578, _, _, _, _) => {
            h578
        }
    }), f130(match v0.clone() {
        Adt6::C0(_, _, _, _, _, _, h579, _) => {
            h579
        }
    }, match v2.clone() {
        Adt8::C0(_, _, _, _, _, _, h580, _, _, _) => {
            h580
        }
    }), f130(match v0 {
        Adt6::C0(_, _, _, _, _, _, _, h581) => {
            h581
        }
    }, match v2 {
        Adt8::C0(_, _, _, _, _, _, _, h582, _, _) => {
            h582
        }
    })))
}

pub fn f127(v0: Bytes, v1: u64, v2: u64, v3: u64) -> R<Adt7> {
    Ok(Adt7::C0(f131(v0.clone(), v1, v2, v3)?, f131(v0.clone(), v1, v2, {
        let a583 = v3;
        let a584 = 4u64;
        nat_add(a583, a584)?
    })?, f131(v0.clone(), v1, v2, {
        let a585 = v3;
        let a586 = 8u64;
        nat_add(a585, a586)?
    })?, f131(v0.clone(), v1, v2, {
        let a587 = v3;
        let a588 = 12u64;
        nat_add(a587, a588)?
    })?, f131(v0.clone(), v1, v2, {
        let a589 = v3;
        let a590 = 16u64;
        nat_add(a589, a590)?
    })?, f131(v0.clone(), v1, v2, {
        let a591 = v3;
        let a592 = 20u64;
        nat_add(a591, a592)?
    })?, f131(v0.clone(), v1, v2, {
        let a593 = v3;
        let a594 = 24u64;
        nat_add(a593, a594)?
    })?, f131(v0.clone(), v1, v2, {
        let a595 = v3;
        let a596 = 28u64;
        nat_add(a595, a596)?
    })?, f131(v0.clone(), v1, v2, {
        let a597 = v3;
        let a598 = 32u64;
        nat_add(a597, a598)?
    })?, f131(v0.clone(), v1, v2, {
        let a599 = v3;
        let a600 = 36u64;
        nat_add(a599, a600)?
    })?, f131(v0.clone(), v1, v2, {
        let a601 = v3;
        let a602 = 40u64;
        nat_add(a601, a602)?
    })?, f131(v0.clone(), v1, v2, {
        let a603 = v3;
        let a604 = 44u64;
        nat_add(a603, a604)?
    })?, f131(v0.clone(), v1, v2, {
        let a605 = v3;
        let a606 = 48u64;
        nat_add(a605, a606)?
    })?, f131(v0.clone(), v1, v2, {
        let a607 = v3;
        let a608 = 52u64;
        nat_add(a607, a608)?
    })?, f131(v0.clone(), v1, v2, {
        let a609 = v3;
        let a610 = 56u64;
        nat_add(a609, a610)?
    })?, f131(v0, v1, v2, {
        let a611 = v3;
        let a612 = 60u64;
        nat_add(a611, a612)?
    })?))
}

pub fn f128(v0: Fn24, p616: u64, p617: Adt8) -> R<Adt8> {
    let m615 = (p616, p617);
    {
        let mut m615 = m615;
        loop {
            let step = {
                let (v1, v2) = m615;
                let m613 = v1;
                if m613 == 0 {
                    Err::<(u64, Adt8), R<Adt8>>(Ok(v2))
                } else {
                    let v3 = m613 - 1;
                    Ok::<(u64, Adt8), R<Adt8>>((v3, {
                        let c614: Fn24 = v0.clone();
                        c614.apply(v2)?
                    }))
                }
            };
            match step {
                Ok(next) => { m615 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f129(v0: Adt8) -> R<Adt8> {
    f132(v0)
}

pub fn f130(v0: u32, v1: u32) -> u32 {
    let m620 = {
        let a618 = f133(v0);
        let a619 = f133(v1);
        fixed_u64::checked_add(a618, a619)
    };
    match m620 {
        None => {
            0u32
        }
        Some(v2) => {
            f134(v2)
        }
    }
}

pub fn f131(v0: Bytes, v1: u64, v2: u64, v3: u64) -> R<u32> {
    let a623 = {
        let a621 = v3;
        let a622 = 8u64;
        nat_add(a621, a622)?
    };
    let a624 = v2;
    if nat_eq(a623, a624) {
        Ok(f135({
            let a627 = {
                let a625 = v1;
                let a626 = 8u64;
                nat_mul(a625, a626)?
            };
            let a628 = 4294967296u64;
            let a629 = 0u64;
            nat_quot(a627, a628, a629)
        }))
    } else {
        let a632 = {
            let a630 = v3;
            let a631 = 4u64;
            nat_add(a630, a631)?
        };
        let a633 = v2;
        if nat_eq(a632, a633) {
            Ok(f135({
                let a636 = {
                    let a634 = v1;
                    let a635 = 8u64;
                    nat_mul(a634, a635)?
                };
                let a637 = 4294967296u64;
                let a638 = 0u64;
                nat_rem(a636, a637, a638)
            }))
        } else {
            Ok(f136(f137(v0.clone(), v1, v3), f137(v0.clone(), v1, {
                let a639 = v3;
                let a640 = 1u64;
                nat_add(a639, a640)?
            }), f137(v0.clone(), v1, {
                let a641 = v3;
                let a642 = 2u64;
                nat_add(a641, a642)?
            }), f137(v0, v1, {
                let a643 = v3;
                let a644 = 3u64;
                nat_add(a643, a644)?
            })))
        }
    }
}

pub fn f132(v0: Adt8) -> R<Adt8> {
    let v1: u32 = f138(match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, _, _, _, h645) => {
            h645
        }
    });
    let v2: u32 = f130(match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, _, h646, _, _) => {
            h646
        }
    }, f130(f139(match v0.clone() {
        Adt8::C0(_, _, _, _, h647, _, _, _, _, _) => {
            h647
        }
    }), f130(f140(match v0.clone() {
        Adt8::C0(_, _, _, _, h648, _, _, _, _, _) => {
            h648
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, h649, _, _, _, _) => {
            h649
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, h650, _, _, _) => {
            h650
        }
    }), f130(v1, {
        let m653 = match v0.clone() {
            Adt8::C0(_, _, _, _, _, _, _, _, h652, _) => {
                h652
            }
        };
        match m653 {
            Adt7::C0(h651, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
                h651
            }
        }
    }))));
    let v3: u32 = f130(f141(match v0.clone() {
        Adt8::C0(h654, _, _, _, _, _, _, _, _, _) => {
            h654
        }
    }), f142(match v0.clone() {
        Adt8::C0(h655, _, _, _, _, _, _, _, _, _) => {
            h655
        }
    }, match v0.clone() {
        Adt8::C0(_, h656, _, _, _, _, _, _, _, _) => {
            h656
        }
    }, match v0.clone() {
        Adt8::C0(_, _, h657, _, _, _, _, _, _, _) => {
            h657
        }
    }));
    Ok(Adt8::C0(f130(v2, v3), match v0.clone() {
        Adt8::C0(h658, _, _, _, _, _, _, _, _, _) => {
            h658
        }
    }, match v0.clone() {
        Adt8::C0(_, h659, _, _, _, _, _, _, _, _) => {
            h659
        }
    }, match v0.clone() {
        Adt8::C0(_, _, h660, _, _, _, _, _, _, _) => {
            h660
        }
    }, f130(match v0.clone() {
        Adt8::C0(_, _, _, h661, _, _, _, _, _, _) => {
            h661
        }
    }, v2), match v0.clone() {
        Adt8::C0(_, _, _, _, h662, _, _, _, _, _) => {
            h662
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, h663, _, _, _, _) => {
            h663
        }
    }, match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, h664, _, _, _) => {
            h664
        }
    }, f143(match v0.clone() {
        Adt8::C0(_, _, _, _, _, _, _, _, h665, _) => {
            h665
        }
    }), {
        let a667 = match v0 {
            Adt8::C0(_, _, _, _, _, _, _, _, _, h666) => {
                h666
            }
        };
        let a668 = 1u64;
        nat_add(a667, a668)?
    }))
}

pub fn f133(v0: u32) -> u64 {
    let m670 = {
        let a669 = v0;
        fixed_u64::convert(i128::from(a669))
    };
    match m670 {
        None => {
            0u64
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f134(v0: u64) -> u32 {
    let m674 = {
        let a673 = {
            let a671 = v0;
            let a672 = 4294967295u64;
            fixed_u64::bit_and(a671, a672)
        };
        fixed_u32::convert(i128::from(a673))
    };
    match m674 {
        None => {
            0u32
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f135(v0: u64) -> u32 {
    let (_, h675) = {
        let (_, h676) = f144(Fn25::F145, 32u64, (v0, (1u32, 0u32)));
        h676
    };
    h675
}

pub fn f136(v0: u32, v1: u32, v2: u32, v3: u32) -> u32 {
    let a681 = f146(v0, 24u32);
    let a682 = {
        let a679 = f146(v1, 16u32);
        let a680 = {
            let a677 = f146(v2, 8u32);
            let a678 = v3;
            fixed_u32::bit_or(a677, a678)
        };
        fixed_u32::bit_or(a679, a680)
    };
    fixed_u32::bit_or(a681, a682)
}

pub fn f137(v0: Bytes, v1: u64, v2: u64) -> u32 {
    let a683 = v2;
    let a684 = v1;
    if nat_lt(a683, a684) {
        let m687 = {
            let a685 = v0;
            let a686 = v2;
            index_bytes(a685, a686)
        };
        match m687 {
            None => {
                0u32
            }
            Some(v3) => {
                let m689 = {
                    let a688 = v3;
                    fixed_u32::convert(i128::from(a688))
                };
                match m689 {
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
        let a690 = v2;
        let a691 = v1;
        if nat_eq(a690, a691) {
            128u32
        } else {
            0u32
        }
    }
}

pub fn f138(v0: u64) -> u32 {
    let a692 = v0;
    let a693 = 32u64;
    if nat_lt(a692, a693) {
        let a694 = v0;
        let a695 = 16u64;
        if nat_lt(a694, a695) {
            let a696 = v0;
            let a697 = 8u64;
            if nat_lt(a696, a697) {
                let a698 = v0;
                let a699 = 4u64;
                if nat_lt(a698, a699) {
                    let a700 = v0;
                    let a701 = 2u64;
                    if nat_lt(a700, a701) {
                        let a702 = v0;
                        let a703 = 1u64;
                        if nat_lt(a702, a703) {
                            1116352408u32
                        } else {
                            1899447441u32
                        }
                    } else {
                        let a704 = v0;
                        let a705 = 3u64;
                        if nat_lt(a704, a705) {
                            3049323471u32
                        } else {
                            3921009573u32
                        }
                    }
                } else {
                    let a706 = v0;
                    let a707 = 6u64;
                    if nat_lt(a706, a707) {
                        let a708 = v0;
                        let a709 = 5u64;
                        if nat_lt(a708, a709) {
                            961987163u32
                        } else {
                            1508970993u32
                        }
                    } else {
                        let a710 = v0;
                        let a711 = 7u64;
                        if nat_lt(a710, a711) {
                            2453635748u32
                        } else {
                            2870763221u32
                        }
                    }
                }
            } else {
                let a712 = v0;
                let a713 = 12u64;
                if nat_lt(a712, a713) {
                    let a714 = v0;
                    let a715 = 10u64;
                    if nat_lt(a714, a715) {
                        let a716 = v0;
                        let a717 = 9u64;
                        if nat_lt(a716, a717) {
                            3624381080u32
                        } else {
                            310598401u32
                        }
                    } else {
                        let a718 = v0;
                        let a719 = 11u64;
                        if nat_lt(a718, a719) {
                            607225278u32
                        } else {
                            1426881987u32
                        }
                    }
                } else {
                    let a720 = v0;
                    let a721 = 14u64;
                    if nat_lt(a720, a721) {
                        let a722 = v0;
                        let a723 = 13u64;
                        if nat_lt(a722, a723) {
                            1925078388u32
                        } else {
                            2162078206u32
                        }
                    } else {
                        let a724 = v0;
                        let a725 = 15u64;
                        if nat_lt(a724, a725) {
                            2614888103u32
                        } else {
                            3248222580u32
                        }
                    }
                }
            }
        } else {
            let a726 = v0;
            let a727 = 24u64;
            if nat_lt(a726, a727) {
                let a728 = v0;
                let a729 = 20u64;
                if nat_lt(a728, a729) {
                    let a730 = v0;
                    let a731 = 18u64;
                    if nat_lt(a730, a731) {
                        let a732 = v0;
                        let a733 = 17u64;
                        if nat_lt(a732, a733) {
                            3835390401u32
                        } else {
                            4022224774u32
                        }
                    } else {
                        let a734 = v0;
                        let a735 = 19u64;
                        if nat_lt(a734, a735) {
                            264347078u32
                        } else {
                            604807628u32
                        }
                    }
                } else {
                    let a736 = v0;
                    let a737 = 22u64;
                    if nat_lt(a736, a737) {
                        let a738 = v0;
                        let a739 = 21u64;
                        if nat_lt(a738, a739) {
                            770255983u32
                        } else {
                            1249150122u32
                        }
                    } else {
                        let a740 = v0;
                        let a741 = 23u64;
                        if nat_lt(a740, a741) {
                            1555081692u32
                        } else {
                            1996064986u32
                        }
                    }
                }
            } else {
                let a742 = v0;
                let a743 = 28u64;
                if nat_lt(a742, a743) {
                    let a744 = v0;
                    let a745 = 26u64;
                    if nat_lt(a744, a745) {
                        let a746 = v0;
                        let a747 = 25u64;
                        if nat_lt(a746, a747) {
                            2554220882u32
                        } else {
                            2821834349u32
                        }
                    } else {
                        let a748 = v0;
                        let a749 = 27u64;
                        if nat_lt(a748, a749) {
                            2952996808u32
                        } else {
                            3210313671u32
                        }
                    }
                } else {
                    let a750 = v0;
                    let a751 = 30u64;
                    if nat_lt(a750, a751) {
                        let a752 = v0;
                        let a753 = 29u64;
                        if nat_lt(a752, a753) {
                            3336571891u32
                        } else {
                            3584528711u32
                        }
                    } else {
                        let a754 = v0;
                        let a755 = 31u64;
                        if nat_lt(a754, a755) {
                            113926993u32
                        } else {
                            338241895u32
                        }
                    }
                }
            }
        }
    } else {
        let a756 = v0;
        let a757 = 48u64;
        if nat_lt(a756, a757) {
            let a758 = v0;
            let a759 = 40u64;
            if nat_lt(a758, a759) {
                let a760 = v0;
                let a761 = 36u64;
                if nat_lt(a760, a761) {
                    let a762 = v0;
                    let a763 = 34u64;
                    if nat_lt(a762, a763) {
                        let a764 = v0;
                        let a765 = 33u64;
                        if nat_lt(a764, a765) {
                            666307205u32
                        } else {
                            773529912u32
                        }
                    } else {
                        let a766 = v0;
                        let a767 = 35u64;
                        if nat_lt(a766, a767) {
                            1294757372u32
                        } else {
                            1396182291u32
                        }
                    }
                } else {
                    let a768 = v0;
                    let a769 = 38u64;
                    if nat_lt(a768, a769) {
                        let a770 = v0;
                        let a771 = 37u64;
                        if nat_lt(a770, a771) {
                            1695183700u32
                        } else {
                            1986661051u32
                        }
                    } else {
                        let a772 = v0;
                        let a773 = 39u64;
                        if nat_lt(a772, a773) {
                            2177026350u32
                        } else {
                            2456956037u32
                        }
                    }
                }
            } else {
                let a774 = v0;
                let a775 = 44u64;
                if nat_lt(a774, a775) {
                    let a776 = v0;
                    let a777 = 42u64;
                    if nat_lt(a776, a777) {
                        let a778 = v0;
                        let a779 = 41u64;
                        if nat_lt(a778, a779) {
                            2730485921u32
                        } else {
                            2820302411u32
                        }
                    } else {
                        let a780 = v0;
                        let a781 = 43u64;
                        if nat_lt(a780, a781) {
                            3259730800u32
                        } else {
                            3345764771u32
                        }
                    }
                } else {
                    let a782 = v0;
                    let a783 = 46u64;
                    if nat_lt(a782, a783) {
                        let a784 = v0;
                        let a785 = 45u64;
                        if nat_lt(a784, a785) {
                            3516065817u32
                        } else {
                            3600352804u32
                        }
                    } else {
                        let a786 = v0;
                        let a787 = 47u64;
                        if nat_lt(a786, a787) {
                            4094571909u32
                        } else {
                            275423344u32
                        }
                    }
                }
            }
        } else {
            let a788 = v0;
            let a789 = 56u64;
            if nat_lt(a788, a789) {
                let a790 = v0;
                let a791 = 52u64;
                if nat_lt(a790, a791) {
                    let a792 = v0;
                    let a793 = 50u64;
                    if nat_lt(a792, a793) {
                        let a794 = v0;
                        let a795 = 49u64;
                        if nat_lt(a794, a795) {
                            430227734u32
                        } else {
                            506948616u32
                        }
                    } else {
                        let a796 = v0;
                        let a797 = 51u64;
                        if nat_lt(a796, a797) {
                            659060556u32
                        } else {
                            883997877u32
                        }
                    }
                } else {
                    let a798 = v0;
                    let a799 = 54u64;
                    if nat_lt(a798, a799) {
                        let a800 = v0;
                        let a801 = 53u64;
                        if nat_lt(a800, a801) {
                            958139571u32
                        } else {
                            1322822218u32
                        }
                    } else {
                        let a802 = v0;
                        let a803 = 55u64;
                        if nat_lt(a802, a803) {
                            1537002063u32
                        } else {
                            1747873779u32
                        }
                    }
                }
            } else {
                let a804 = v0;
                let a805 = 60u64;
                if nat_lt(a804, a805) {
                    let a806 = v0;
                    let a807 = 58u64;
                    if nat_lt(a806, a807) {
                        let a808 = v0;
                        let a809 = 57u64;
                        if nat_lt(a808, a809) {
                            1955562222u32
                        } else {
                            2024104815u32
                        }
                    } else {
                        let a810 = v0;
                        let a811 = 59u64;
                        if nat_lt(a810, a811) {
                            2227730452u32
                        } else {
                            2361852424u32
                        }
                    }
                } else {
                    let a812 = v0;
                    let a813 = 62u64;
                    if nat_lt(a812, a813) {
                        let a814 = v0;
                        let a815 = 61u64;
                        if nat_lt(a814, a815) {
                            2428436474u32
                        } else {
                            2756734187u32
                        }
                    } else {
                        let a816 = v0;
                        let a817 = 63u64;
                        if nat_lt(a816, a817) {
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
    let a820 = {
        let a818 = f147(v0, 6u32, 26u32);
        let a819 = f147(v0, 11u32, 21u32);
        fixed_u32::bit_xor(a818, a819)
    };
    let a821 = f147(v0, 25u32, 7u32);
    fixed_u32::bit_xor(a820, a821)
}

pub fn f140(v0: u32, v1: u32, v2: u32) -> u32 {
    let a827 = {
        let a822 = v0;
        let a823 = v1;
        fixed_u32::bit_and(a822, a823)
    };
    let a828 = {
        let a825 = {
            let a824 = v0;
            fixed_u32::bit_not(a824)
        };
        let a826 = v2;
        fixed_u32::bit_and(a825, a826)
    };
    fixed_u32::bit_xor(a827, a828)
}

pub fn f141(v0: u32) -> u32 {
    let a831 = {
        let a829 = f147(v0, 2u32, 30u32);
        let a830 = f147(v0, 13u32, 19u32);
        fixed_u32::bit_xor(a829, a830)
    };
    let a832 = f147(v0, 22u32, 10u32);
    fixed_u32::bit_xor(a831, a832)
}

pub fn f142(v0: u32, v1: u32, v2: u32) -> u32 {
    let a841 = {
        let a837 = {
            let a833 = v0;
            let a834 = v1;
            fixed_u32::bit_and(a833, a834)
        };
        let a838 = {
            let a835 = v0;
            let a836 = v2;
            fixed_u32::bit_and(a835, a836)
        };
        fixed_u32::bit_xor(a837, a838)
    };
    let a842 = {
        let a839 = v1;
        let a840 = v2;
        fixed_u32::bit_and(a839, a840)
    };
    fixed_u32::bit_xor(a841, a842)
}

pub fn f143(v0: Adt7) -> Adt7 {
    Adt7::C0(match v0.clone() {
        Adt7::C0(_, h843, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h843
        }
    }, match v0.clone() {
        Adt7::C0(_, _, h844, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h844
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, h845, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h845
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, h846, _, _, _, _, _, _, _, _, _, _, _) => {
            h846
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, h847, _, _, _, _, _, _, _, _, _, _) => {
            h847
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, h848, _, _, _, _, _, _, _, _, _) => {
            h848
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, h849, _, _, _, _, _, _, _, _) => {
            h849
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, h850, _, _, _, _, _, _, _) => {
            h850
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, h851, _, _, _, _, _, _) => {
            h851
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, h852, _, _, _, _, _) => {
            h852
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, h853, _, _, _, _) => {
            h853
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, h854, _, _, _) => {
            h854
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, h855, _, _) => {
            h855
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h856, _) => {
            h856
        }
    }, match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, h857) => {
            h857
        }
    }, f130(f148(match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, _, _, _, _, _, h858, _) => {
            h858
        }
    }), f130(match v0.clone() {
        Adt7::C0(_, _, _, _, _, _, _, _, _, h859, _, _, _, _, _, _) => {
            h859
        }
    }, f130(f149(match v0.clone() {
        Adt7::C0(_, h860, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h860
        }
    }), match v0 {
        Adt7::C0(h861, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _) => {
            h861
        }
    }))))
}

pub fn f144(v0: Fn25, p865: u64, p866: (u64, (u32, u32))) -> (u64, (u32, u32)) {
    let m864 = (p865, p866);
    {
        let mut m864 = m864;
        loop {
            let step = {
                let (v1, v2) = m864;
                let m862 = v1;
                if m862 == 0 {
                    Err::<(u64, (u64, (u32, u32))), (u64, (u32, u32))>(v2)
                } else {
                    let v3 = m862 - 1;
                    Ok::<(u64, (u64, (u32, u32))), (u64, (u32, u32))>((v3, {
                        let c863: Fn25 = v0.clone();
                        c863.apply(v2)
                    }))
                }
            };
            match step {
                Ok(next) => { m864 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f145(v0: (u64, (u32, u32))) -> (u64, (u32, u32)) {
    let v1: u64 = {
        let (h867, _) = v0;
        h867
    };
    let v2: u32 = {
        let (h868, _) = {
            let (_, h869) = v0;
            h869
        };
        h868
    };
    let v3: u32 = {
        let (_, h870) = {
            let (_, h871) = v0;
            h871
        };
        h870
    };
    ({
        let a872 = v1;
        let a873 = 2u64;
        let a874 = 0u64;
        nat_quot(a872, a873, a874)
    }, (f146(v2, 1u32), {
        let a878 = {
            let a875 = v1;
            let a876 = 2u64;
            let a877 = 0u64;
            nat_rem(a875, a876, a877)
        };
        let a879 = 1u64;
        if nat_eq(a878, a879) {
            let a880 = v3;
            let a881 = v2;
            fixed_u32::bit_or(a880, a881)
        } else {
            v3
        }
    }))
}

pub fn f146(v0: u32, v1: u32) -> u32 {
    let m884 = {
        let a882 = v0;
        let a883 = v1;
        fixed_u32::shift_left(a882, a883)
    };
    match m884 {
        None => {
            0u32
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f147(v0: u32, v1: u32, v2: u32) -> u32 {
    let a885 = f125(v0, v1);
    let a886 = f146(v0, v2);
    fixed_u32::bit_or(a885, a886)
}

pub fn f148(v0: u32) -> u32 {
    let a889 = {
        let a887 = f147(v0, 17u32, 15u32);
        let a888 = f147(v0, 19u32, 13u32);
        fixed_u32::bit_xor(a887, a888)
    };
    let a890 = f125(v0, 10u32);
    fixed_u32::bit_xor(a889, a890)
}

pub fn f149(v0: u32) -> u32 {
    let a893 = {
        let a891 = f147(v0, 7u32, 25u32);
        let a892 = f147(v0, 18u32, 14u32);
        fixed_u32::bit_xor(a891, a892)
    };
    let a894 = f125(v0, 3u32);
    fixed_u32::bit_xor(a893, a894)
}
