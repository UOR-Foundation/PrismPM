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
    C0(Str, Str, u64, List<Adt1>, List<Str>, List<Adt2>, Str, List<Adt3>),
}

#[derive(Clone)]
pub enum Adt1 {
    C0(Str, Str),
}

#[derive(Clone)]
pub enum Adt2 {
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
}

#[derive(Clone)]
pub enum Adt3 {
    C0(Str, Str, List<(Str, Str)>),
}

#[derive(Clone)]
pub enum Fn0 {
    F18,
    F21,
    F25,
}

impl Fn0 {
    pub fn apply(&self, p0: Bytes, p1: Str) -> Bytes {
        match self {
            Fn0::F18 => f18(p0, p1),
            Fn0::F21 => f21(p0, p1),
            Fn0::F25 => f25(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn1 {
    F20,
}

impl Fn1 {
    pub fn apply(&self, p0: Bytes, p1: Adt0) -> R<Bytes> {
        match self {
            Fn1::F20 => f20(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn2 {
    F28,
    F36,
}

impl Fn2 {
    pub fn apply(&self, p0: List<Bytes>, p1: Str) -> List<Bytes> {
        match self {
            Fn2::F28 => f28(p0, p1),
            Fn2::F36 => f36(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn3 {
    F30,
}

impl Fn3 {
    pub fn apply(&self, p0: Bytes, p1: (Str, List<Str>)) -> Bytes {
        match self {
            Fn3::F30 => f30(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn4 {
    F35(Bytes),
}

impl Fn4 {
    pub fn apply(&self, p0: Option<Bytes>, p1: Bytes) -> Option<Bytes> {
        match self {
            Fn4::F35(k0) => f35(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn5 {
    F47,
}

impl Fn5 {
    pub fn apply(&self, p0: List<Bytes>) -> Option<List<Bytes>> {
        match self {
            Fn5::F47 => f47(p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn6 {
    F50(Bytes),
}

impl Fn6 {
    pub fn apply(&self, p0: u64) -> R<Option<u64>> {
        match self {
            Fn6::F50(k0) => f50(k0.clone(), p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn7 {
    F53,
}

impl Fn7 {
    pub fn apply(&self, p0: Bytes, p1: Adt1) -> Bytes {
        match self {
            Fn7::F53 => f53(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn8 {
    F55,
}

impl Fn8 {
    pub fn apply(&self, p0: Bytes, p1: Adt3) -> Bytes {
        match self {
            Fn8::F55 => f55(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn9 {
    F60,
}

impl Fn9 {
    pub fn apply(&self, p0: (Option<Bytes>, List<Bytes>), p1: Bytes) -> (Option<Bytes>, List<Bytes>) {
        match self {
            Fn9::F60 => f60(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn10 {
    F64,
}

impl Fn10 {
    pub fn apply(&self, p0: Bytes, p1: (Str, Str)) -> Bytes {
        match self {
            Fn10::F64 => f64(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn11 {
    F67,
}

impl Fn11 {
    pub fn apply(&self, p0: List<Bytes>, p1: Bytes) -> List<Bytes> {
        match self {
            Fn11::F67 => f67(p0, p1),
        }
    }
}

pub fn f0(v0: Bytes) -> R<Bytes> {
    let a1 = v0.clone();
    let a2 = f1(Str::lit("release"));
    if equal(a1, a2) {
        Ok(f2())
    } else {
        let a3 = v0.clone();
        let a4 = f1(Str::lit("cut"));
        if equal(a3, a4) {
            Ok(f3(Bytes::lit(&[]), f4()))
        } else {
            let a5 = v0;
            let a6 = f1(Str::lit("delegate"));
            if equal(a5, a6) {
                f5()
            } else {
                Ok(f3(Bytes::lit(&[]), f6()))
            }
        }
    }
}

pub fn f1(v0: Str) -> Bytes {
    let a7 = v0;
    utf8_encode(a7)
}

pub fn f2() -> Bytes {
    let v0: Bytes = {
        let a8 = f3(Bytes::lit(&[]), f7());
        let a9 = f8(Str::lit("header"));
        append_bytes(a8, a9)
    };
    let v1: Bytes = {
        let a10 = v0;
        let a11 = f9(f10());
        append_bytes(a10, a11)
    };
    let v2: Bytes = {
        let a12 = f3(v1, f11());
        let a13 = f12(List::cons(Str::lit("factory"), f10()));
        append_bytes(a12, a13)
    };
    f3({
        let a14 = f3(v2, f13());
        let a15 = f14(f15());
        append_bytes(a14, a15)
    }, f16())
}

pub fn f3(v0: Bytes, v1: List<Str>) -> Bytes {
    f17(Fn0::F18, v0, v1)
}

pub fn f4() -> List<Str> {
    List::cons(Str::lit("# Generated by the Prism factory from prism-factory/model/src/Factory/Workflow.lex.tex."), List::cons(Str::lit("# Do not edit: `prism-factory/factory.sh workflow --name cut --write` rewrites it and `check` refuses drift."), List::cons(Str::lit("# A release is cut by the certified plan, not by a person. When a rehearsal of"), List::cons(Str::lit("# the workspace version passes on main or a release branch, the plan at that"), List::cons(Str::lit("# commit says whether every obligation that does not depend on the run or the"), List::cons(Str::lit("# release mode is proved (`cuttable`). If it is, and the version's milestone has"), List::cons(Str::lit("# no open issue, this tags vVERSION at the rehearsed commit, dispatches the"), List::cons(Str::lit("# publication run at the tag (which proves the build from the rehearsal's"), List::cons(Str::lit("# receipts), and closes the milestone."), List::cons(Str::lit("name: Release cut"), List::cons(Str::lit("on:"), List::cons(Str::lit("  workflow_run:"), List::cons(Str::lit("    workflows: [PrismPM OCI/native and Cargo publication]"), List::cons(Str::lit("    types: [completed]"), List::cons(Str::lit("    branches: [main, release/v*]"), List::cons(Str::lit("  workflow_dispatch:"), List::cons(Str::lit("permissions:"), List::cons(Str::lit("  contents: read"), List::cons(Str::lit("concurrency:"), List::cons(Str::lit("  group: release-cut"), List::cons(Str::lit("  cancel-in-progress: false"), List::cons(Str::lit("jobs:"), List::cons(Str::lit("  cut:"), List::cons(Str::lit("    if: ${{ github.event_name == 'workflow_dispatch' || (github.event.workflow_run.conclusion == 'success' && github.event.workflow_run.head_repository.full_name == github.repository) }}"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 60"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      actions: write"), List::cons(Str::lit("      contents: write"), List::cons(Str::lit("      issues: write"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          ref: ${{ github.event.workflow_run.head_sha || github.sha }}"), List::cons(Str::lit("          fetch-depth: 0"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Cut the release the certified plan allows"), List::cons(Str::lit("        env:"), List::cons(Str::lit("          GH_TOKEN: ${{ github.token }}"), List::cons(Str::lit("          CUT_REVISION: ${{ github.event.workflow_run.head_sha || github.sha }}"), List::cons(Str::lit("        run: bash prism-factory/factory.sh actions-cut"), List::<Str>::nil())))))))))))))))))))))))))))))))))))))))))
}

pub fn f5() -> R<Bytes> {
    f19(Fn1::F20, f17(Fn0::F21, Bytes::lit(&[]), f22()), f23())
}

pub fn f6() -> List<Str> {
    List::cons(Str::lit("# Generated by the Prism factory from prism-factory/model/src/Factory/Workflow.lex.tex."), List::cons(Str::lit("# Do not edit: `prism-factory/factory.sh workflow --write` rewrites it and `check` refuses drift."), List::cons(Str::lit("# Every decision (identity, route, order, releasability) is made by the certified"), List::cons(Str::lit("# factory core; this file only gives it a runner and a checkout. Release runs"), List::cons(Str::lit("# record receipts (release.yml, job `receipts`); this one only reads them."), List::cons(Str::lit("name: Factory"), List::cons(Str::lit("on:"), List::cons(Str::lit("  pull_request:"), List::cons(Str::lit("  workflow_dispatch:"), List::cons(Str::lit("permissions:"), List::cons(Str::lit("  contents: read"), List::cons(Str::lit("concurrency:"), List::cons(Str::lit("  group: factory-${{ github.ref }}"), List::cons(Str::lit("  cancel-in-progress: true"), List::cons(Str::lit("jobs:"), List::cons(Str::lit("  factory:"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 60"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: read"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Run the factory"), List::cons(Str::lit("        run: bash prism-factory/factory.sh ci"), List::cons(Str::lit("  pages:"), List::cons(Str::lit("    # Publishes the factory page on a manual run once Pages is set to deploy"), List::cons(Str::lit("    # from Actions and the repository variable FACTORY_PAGES is true."), List::cons(Str::lit("    if: ${{ github.event_name == 'workflow_dispatch' && vars.FACTORY_PAGES == 'true' }}"), List::cons(Str::lit("    needs: factory"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 30"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: read"), List::cons(Str::lit("      pages: write"), List::cons(Str::lit("      id-token: write"), List::cons(Str::lit("    environment:"), List::cons(Str::lit("      name: github-pages"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Build the factory page"), List::cons(Str::lit("        run: bash prism-factory/factory.sh site target/factory-site"), List::cons(Str::lit("      - uses: actions/configure-pages@45bfe0192ca1faeb007ade9deae92b16b8254a0d"), List::cons(Str::lit("      - uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          path: target/factory-site"), List::cons(Str::lit("      - uses: actions/deploy-pages@368f82528645a54fb793d4d04e342629a3f51346"), List::<Str>::nil()))))))))))))))))))))))))))))))))))))))))))))))))))
}

pub fn f7() -> List<Str> {
    List::cons(Str::lit("# Generated by the Prism factory from prism-factory/model and prism-factory/jobs/."), List::cons(Str::lit("# Do not edit: `prism-factory/factory.sh workflow --write` rewrites it and the"), List::cons(Str::lit("# factory-model obligation refuses drift. The factory job decides every route"), List::cons(Str::lit("# with the certified core (prism-factory/certified); each job below runs only"), List::cons(Str::lit("# what that route establishes, and the receipts job records what it proved."), List::<Str>::nil())))))
}

pub fn f8(v0: Str) -> Bytes {
    f24(List::cons(f1(Str::lit("#fragment")), List::cons(f1(v0), List::<Bytes>::nil())))
}

pub fn f9(v0: List<Str>) -> Bytes {
    f17(Fn0::F25, Bytes::lit(&[]), v0)
}

pub fn f10() -> List<Str> {
    List::cons(Str::lit("policy"), List::cons(Str::lit("devcontainer-image"), List::cons(Str::lit("vv-sdk"), List::cons(Str::lit("source-vv"), List::cons(Str::lit("gate"), List::cons(Str::lit("image-platforms"), List::cons(Str::lit("images"), List::cons(Str::lit("reproducibility"), List::cons(Str::lit("installed-sdk"), List::cons(Str::lit("native"), List::cons(Str::lit("oci-native"), List::cons(Str::lit("crates"), List::cons(Str::lit("release"), List::<Str>::nil())))))))))))))
}

pub fn f11() -> List<Str> {
    List::cons(Str::lit(""), List::cons(Str::lit("  # Records a receipt for every obligation this run established, on the"), List::cons(Str::lit("  # factory-receipts branch, and reweighs the schedule by what passed and failed."), List::cons(Str::lit("  receipts:"), List::<Str>::nil()))))
}

pub fn f12(v0: List<Str>) -> Bytes {
    let a18 = {
        let a16 = f1(Str::lit("    needs: ["));
        let a17 = f26(f27(Fn2::F28, List::<Bytes>::nil(), v0), Bytes::lit(&[0x2c, 0x20]));
        append_bytes(a16, a17)
    };
    let a19 = f1(Str::lit("]\u{a}"));
    append_bytes(a18, a19)
}

pub fn f13() -> List<Str> {
    List::cons(Str::lit("    if: ${{ always() && needs.factory.result == 'success' }}"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 15"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: write"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Record receipts"), List::cons(Str::lit("        env:"), List::cons(Str::lit("          FACTORY_PLAN: ${{ needs.factory.outputs.plan }}"), List::cons(Str::lit("          FACTORY_RESULTS: |"), List::<Str>::nil())))))))))))))
}

pub fn f14(v0: List<(Str, List<Str>)>) -> Bytes {
    f29(Fn3::F30, Bytes::lit(&[]), v0)
}

pub fn f15() -> List<(Str, List<Str>)> {
    List::cons((Str::lit("policy"), List::cons(Str::lit("policy"), List::<Str>::nil())), List::cons((Str::lit("factory-model"), List::cons(Str::lit("factory"), List::<Str>::nil())), List::cons((Str::lit("plumbing"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil())), List::cons((Str::lit("vv-sdk"), List::cons(Str::lit("vv-sdk"), List::<Str>::nil())), List::cons((Str::lit("source-vv"), List::cons(Str::lit("source-vv"), List::cons(Str::lit("gate"), List::<Str>::nil()))), List::cons((Str::lit("build"), List::cons(Str::lit("image-platforms"), List::cons(Str::lit("images"), List::cons(Str::lit("reproducibility"), List::cons(Str::lit("installed-sdk"), List::cons(Str::lit("native"), List::<Str>::nil())))))), List::cons((Str::lit("gate"), List::cons(Str::lit("gate"), List::<Str>::nil())), List::cons((Str::lit("oci-native"), List::cons(Str::lit("oci-native"), List::<Str>::nil())), List::cons((Str::lit("crates"), List::cons(Str::lit("crates"), List::<Str>::nil())), List::cons((Str::lit("release"), List::cons(Str::lit("release"), List::<Str>::nil())), List::<(Str, List<Str>)>::nil()))))))))))
}

pub fn f16() -> List<Str> {
    List::cons(Str::lit("        run: bash prism-factory/factory.sh actions-receipts"), List::<Str>::nil())
}

pub fn f17(v0: Fn0, p23: Bytes, p24: List<Str>) -> Bytes {
    let m22 = (p23, p24);
    {
        let mut m22 = m22;
        loop {
            let step = {
                let (v1, v2) = m22;
                let m20 = v2;
                match m20.uncons() {
                    None => {
                        Err::<(Bytes, List<Str>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<Str>), Bytes>(({
                            let c21: Fn0 = v0.clone();
                            c21.apply(v1, v3)
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

pub fn f18(v0: Bytes, v1: Str) -> Bytes {
    let a28 = v0;
    let a29 = {
        let a26 = {
            let a25 = v1;
            utf8_encode(a25)
        };
        let a27 = Bytes::lit(&[0x0a]);
        append_bytes(a26, a27)
    };
    append_bytes(a28, a29)
}

pub fn f19(v0: Fn1, p33: Bytes, p34: List<Adt0>) -> R<Bytes> {
    let m32 = (p33, p34);
    {
        let mut m32 = m32;
        loop {
            let step = {
                let (v1, v2) = m32;
                let m30 = v2;
                match m30.uncons() {
                    None => {
                        Err::<(Bytes, List<Adt0>), R<Bytes>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<Adt0>), R<Bytes>>(({
                            let c31: Fn1 = v0.clone();
                            c31.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m32 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f20(v0: Bytes, v1: Adt0) -> R<Bytes> {
    let a35 = v0;
    let a36 = f31(v1)?;
    Ok(append_bytes(a35, a36))
}

pub fn f21(v0: Bytes, v1: Str) -> Bytes {
    let a37 = v0;
    let a38 = f32(v1);
    append_bytes(a37, a38)
}

pub fn f22() -> List<Str> {
    List::cons(Str::lit("# Generated by the Prism factory from prism-factory/model/src/Factory/Delegation.lex.tex."), List::cons(Str::lit("# Do not edit: `prism-factory/factory.sh workflow --name delegate --write` rewrites it and `check` refuses drift."), List::cons(Str::lit("# GitHub is the forge. It triggers, names the revision, computes each obligation's"), List::cons(Str::lit("# identity with the certified core, and verifies the node's signed receipts. It"), List::cons(Str::lit("# runs no obligation. A Prism node establishes every obligation whose identity has"), List::cons(Str::lit("# no passing receipt, in a declared sandbox, and dispatches prism-verdict back here."), List::cons(Str::lit("name: Prism delegation"), List::cons(Str::lit("on:"), List::cons(Str::lit("  pull_request:"), List::cons(Str::lit("  push:"), List::cons(Str::lit("    branches: [main, 'release/v*']"), List::cons(Str::lit("    tags: ['v*']"), List::cons(Str::lit("  workflow_dispatch:"), List::cons(Str::lit("  repository_dispatch:"), List::cons(Str::lit("    types: [prism-verdict]"), List::cons(Str::lit("permissions:"), List::cons(Str::lit("  contents: read"), List::cons(Str::lit("concurrency:"), List::cons(Str::lit("  group: prism-delegation-${{ github.event_name }}-${{ github.event.client_payload.revision || github.ref }}"), List::cons(Str::lit("  cancel-in-progress: true"), List::cons(Str::lit("jobs:"), List::<Str>::nil())))))))))))))))))))))
}

pub fn f23() -> List<Adt0> {
    List::cons(Adt0::C0(Str::lit("submit"), Str::lit("${{ github.event_name != 'repository_dispatch' }}"), 10u64, List::cons(Adt1::C0(Str::lit("contents"), Str::lit("read")), List::cons(Adt1::C0(Str::lit("statuses"), Str::lit("write")), List::cons(Adt1::C0(Str::lit("id-token"), Str::lit("write")), List::<Adt1>::nil()))), List::<Str>::nil(), List::cons(Adt2::C0, List::cons(Adt2::C1, List::cons(Adt2::C2, List::cons(Adt2::C3, List::cons(Adt2::C5, List::<Adt2>::nil()))))), Str::lit(""), List::cons(Adt3::C0(Str::lit("Submit the workload to the Prism node"), Str::lit("bash prism-factory/factory.sh delegate-submit"), List::<(Str, Str)>::nil()), List::<Adt3>::nil())), List::cons(Adt0::C0(Str::lit("verify"), Str::lit("${{ github.event_name == 'repository_dispatch' }}"), 10u64, List::cons(Adt1::C0(Str::lit("contents"), Str::lit("read")), List::cons(Adt1::C0(Str::lit("statuses"), Str::lit("write")), List::<Adt1>::nil())), List::<Str>::nil(), List::cons(Adt2::C1, List::cons(Adt2::C2, List::cons(Adt2::C4, List::cons(Adt2::C5, List::<Adt2>::nil())))), Str::lit("${{ github.event.client_payload.revision }}"), List::cons(Adt3::C0(Str::lit("Verify the node's receipts and report them"), Str::lit("bash prism-factory/factory.sh delegate-verify"), List::cons((Str::lit("PRISM_VERDICT"), Str::lit("${{ toJSON(github.event.client_payload) }}")), List::<(Str, Str)>::nil())), List::<Adt3>::nil())), List::<Adt0>::nil()))
}

pub fn f24(v0: List<Bytes>) -> Bytes {
    let a39 = f26(v0, Bytes::lit(&[0x20]));
    let a40 = Bytes::lit(&[0x0a]);
    append_bytes(a39, a40)
}

pub fn f25(v0: Bytes, v1: Str) -> Bytes {
    let v2: Bytes = {
        let a41 = v0;
        let a42 = f8(v1.clone());
        append_bytes(a41, a42)
    };
    let a43 = v1;
    let a44 = Str::lit("policy");
    if equal(a43, a44) {
        f3(v2, f33())
    } else {
        v2
    }
}

pub fn f26(v0: List<Bytes>, v1: Bytes) -> Bytes {
    let m45 = f34(Fn4::F35(v1), None::<Bytes>, v0);
    match m45 {
        None => {
            Bytes::lit(&[])
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f27(v0: Fn2, p49: List<Bytes>, p50: List<Str>) -> List<Bytes> {
    let m48 = (p49, p50);
    {
        let mut m48 = m48;
        loop {
            let step = {
                let (v1, v2) = m48;
                let m46 = v2;
                match m46.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Str>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Str>), List<Bytes>>(({
                            let c47: Fn2 = v0.clone();
                            c47.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m48 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f28(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a51 = v0;
    let a52 = List::cons(f1(v1), List::<Bytes>::nil());
    append_list(a51, a52)
}

pub fn f29(v0: Fn3, p56: Bytes, p57: List<(Str, List<Str>)>) -> Bytes {
    let m55 = (p56, p57);
    {
        let mut m55 = m55;
        loop {
            let step = {
                let (v1, v2) = m55;
                let m53 = v2;
                match m53.uncons() {
                    None => {
                        Err::<(Bytes, List<(Str, List<Str>)>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<(Str, List<Str>)>), Bytes>(({
                            let c54: Fn3 = v0.clone();
                            c54.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m55 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f30(v0: Bytes, v1: (Str, List<Str>)) -> Bytes {
    let a62 = v0;
    let a63 = {
        let a60 = f1(Str::lit("            "));
        let a61 = f24(List::cons(f1({
            let (h58, _) = v1;
            h58
        }), f27(Fn2::F36, List::<Bytes>::nil(), {
            let (_, h59) = v1;
            h59
        })));
        append_bytes(a60, a61)
    };
    append_bytes(a62, a63)
}

pub fn f31(v0: Adt0) -> R<Bytes> {
    Ok(f37(List::cons(f38(List::cons(f39(Str::lit("  ")), List::cons(f39(match v0.clone() {
        Adt0::C0(h64, _, _, _, _, _, _, _) => {
            h64
        }
    }), List::cons(f39(Str::lit(":")), List::<Bytes>::nil())))), List::cons(f38(List::cons(f39(Str::lit("    if: ")), List::cons(f39(match v0.clone() {
        Adt0::C0(_, h65, _, _, _, _, _, _) => {
            h65
        }
    }), List::<Bytes>::nil()))), List::cons(f32(Str::lit("    runs-on: ubuntu-24.04")), List::cons(f38(List::cons(f39(Str::lit("    timeout-minutes: ")), List::cons(f40(match v0.clone() {
        Adt0::C0(_, _, h66, _, _, _, _, _) => {
            h66
        }
    })?, List::<Bytes>::nil()))), List::cons(f32(Str::lit("    permissions:")), List::cons(f41(match v0.clone() {
        Adt0::C0(_, _, _, h67, _, _, _, _) => {
            h67
        }
    }), List::cons(f32(Str::lit("    steps:")), List::cons(f32(f42()), List::cons(f32(Str::lit("        with:")), List::cons(f43(match v0.clone() {
        Adt0::C0(_, _, _, _, _, _, h68, _) => {
            h68
        }
    }), List::cons(f32(Str::lit("          submodules: false")), List::cons(f44(match v0 {
        Adt0::C0(_, _, _, _, _, _, _, h69) => {
            h69
        }
    }), List::<Bytes>::nil()))))))))))))))
}

pub fn f32(v0: Str) -> Bytes {
    f38(List::cons(f39(v0), List::<Bytes>::nil()))
}

pub fn f33() -> List<Str> {
    List::cons(Str::lit(""), List::cons(Str::lit("  # The factory route. Every obligation is keyed by its own declared inputs"), List::cons(Str::lit("  # (prism-factory/model/src/Factory/PrismPM.lex.tex). An obligation with a"), List::cons(Str::lit("  # passing receipt for its identity on the factory-receipts branch is proved:"), List::cons(Str::lit("  # its jobs are skipped and its retained evidence is that run's."), List::cons(Str::lit("  factory:"), List::cons(Str::lit("    needs: policy"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 60"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: read"), List::cons(Str::lit("    outputs:"), List::cons(Str::lit("      kappa: ${{ steps.route.outputs.kappa }}"), List::cons(Str::lit("      route: ${{ steps.route.outputs.route }}"), List::cons(Str::lit("      source-revision: ${{ steps.route.outputs.source-revision }}"), List::cons(Str::lit("      run-id: ${{ steps.route.outputs.run-id }}"), List::cons(Str::lit("      run-attempt: ${{ steps.route.outputs.run-attempt }}"), List::cons(Str::lit("      build-route: ${{ steps.route.outputs.build-route }}"), List::cons(Str::lit("      build-identity: ${{ steps.route.outputs.build-identity }}"), List::cons(Str::lit("      build-revision: ${{ steps.route.outputs.build-revision }}"), List::cons(Str::lit("      build-run-id: ${{ steps.route.outputs.build-run-id }}"), List::cons(Str::lit("      build-run-attempt: ${{ steps.route.outputs.build-run-attempt }}"), List::cons(Str::lit("      vv-sdk: ${{ steps.route.outputs.vv-sdk }}"), List::cons(Str::lit("      vv-sdk-run-id: ${{ steps.route.outputs.vv-sdk-run-id }}"), List::cons(Str::lit("      plan: ${{ steps.route.outputs.plan }}"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          fetch-depth: 0"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - id: route"), List::cons(Str::lit("        env:"), List::cons(Str::lit("          FACTORY_MODE: ${{ needs.policy.outputs.rehearsal == 'true' && 'rehearsal' || 'publish' }}"), List::cons(Str::lit("        run: bash prism-factory/factory.sh actions-route"), List::<Str>::nil()))))))))))))))))))))))))))))))))))
}

pub fn f34(v0: Fn4, p73: Option<Bytes>, p74: List<Bytes>) -> Option<Bytes> {
    let m72 = (p73, p74);
    {
        let mut m72 = m72;
        loop {
            let step = {
                let (v1, v2) = m72;
                let m70 = v2;
                match m70.uncons() {
                    None => {
                        Err::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(({
                            let c71: Fn4 = v0.clone();
                            c71.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m72 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f35(v0: Bytes, v1: Option<Bytes>, v2: Bytes) -> Option<Bytes> {
    let m75 = v1;
    match m75 {
        None => {
            Some(v2)
        }
        Some(v3) => {
            Some({
                let a78 = {
                    let a76 = v3;
                    let a77 = v0;
                    append_bytes(a76, a77)
                };
                let a79 = v2;
                append_bytes(a78, a79)
            })
        }
    }
}

pub fn f36(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a80 = v0;
    let a81 = List::cons(f45(v1), List::<Bytes>::nil());
    append_list(a80, a81)
}

pub fn f37(v0: List<Bytes>) -> Bytes {
    let m84 = {
        let (h82, _) = f46(Fn5::F47, {
            let a83 = v0.clone();
            length_list(a83)
        }, v0);
        h82
    };
    match m84.uncons() {
        None => {
            Bytes::lit(&[])
        }
        Some((v1, _)) => {
            v1
        }
    }
}

pub fn f38(v0: List<Bytes>) -> Bytes {
    let a85 = f37(v0);
    let a86 = Bytes::lit(&[0x0a]);
    append_bytes(a85, a86)
}

pub fn f39(v0: Str) -> Bytes {
    let a87 = v0;
    utf8_encode(a87)
}

pub fn f40(v0: u64) -> R<Bytes> {
    let v1: Bytes = {
        let a124 = f48(v0, 10000000000000000000u64);
        let a125 = {
            let a122 = f48(v0, 1000000000000000000u64);
            let a123 = {
                let a120 = f48(v0, 100000000000000000u64);
                let a121 = {
                    let a118 = f48(v0, 10000000000000000u64);
                    let a119 = {
                        let a116 = f48(v0, 1000000000000000u64);
                        let a117 = {
                            let a114 = f48(v0, 100000000000000u64);
                            let a115 = {
                                let a112 = f48(v0, 10000000000000u64);
                                let a113 = {
                                    let a110 = f48(v0, 1000000000000u64);
                                    let a111 = {
                                        let a108 = f48(v0, 100000000000u64);
                                        let a109 = {
                                            let a106 = f48(v0, 10000000000u64);
                                            let a107 = {
                                                let a104 = f48(v0, 1000000000u64);
                                                let a105 = {
                                                    let a102 = f48(v0, 100000000u64);
                                                    let a103 = {
                                                        let a100 = f48(v0, 10000000u64);
                                                        let a101 = {
                                                            let a98 = f48(v0, 1000000u64);
                                                            let a99 = {
                                                                let a96 = f48(v0, 100000u64);
                                                                let a97 = {
                                                                    let a94 = f48(v0, 10000u64);
                                                                    let a95 = {
                                                                        let a92 = f48(v0, 1000u64);
                                                                        let a93 = {
                                                                            let a90 = f48(v0, 100u64);
                                                                            let a91 = {
                                                                                let a88 = f48(v0, 10u64);
                                                                                let a89 = f48(v0, 1u64);
                                                                                append_bytes(a88, a89)
                                                                            };
                                                                            append_bytes(a90, a91)
                                                                        };
                                                                        append_bytes(a92, a93)
                                                                    };
                                                                    append_bytes(a94, a95)
                                                                };
                                                                append_bytes(a96, a97)
                                                            };
                                                            append_bytes(a98, a99)
                                                        };
                                                        append_bytes(a100, a101)
                                                    };
                                                    append_bytes(a102, a103)
                                                };
                                                append_bytes(a104, a105)
                                            };
                                            append_bytes(a106, a107)
                                        };
                                        append_bytes(a108, a109)
                                    };
                                    append_bytes(a110, a111)
                                };
                                append_bytes(a112, a113)
                            };
                            append_bytes(a114, a115)
                        };
                        append_bytes(a116, a117)
                    };
                    append_bytes(a118, a119)
                };
                append_bytes(a120, a121)
            };
            append_bytes(a122, a123)
        };
        append_bytes(a124, a125)
    };
    let v2: u64 = {
        let (h126, _) = f49(Fn6::F50(v1.clone()), {
            let a127 = v1.clone();
            length_bytes(a127)
        }, 0u64)?;
        h126
    };
    let a128 = v0;
    let a129 = 0u64;
    if nat_eq(a128, a129) {
        let a130 = Str::lit("0");
        Ok(utf8_encode(a130))
    } else {
        Ok(f51(v1.clone(), v2, {
            let a132 = {
                let a131 = v1;
                length_bytes(a131)
            };
            let a133 = v2;
            nat_sub(a132, a133)
        }))
    }
}

pub fn f41(v0: List<Adt1>) -> Bytes {
    f52(Fn7::F53, Bytes::lit(&[]), v0)
}

pub fn f42() -> Str {
    Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262")
}

pub fn f43(v0: Str) -> Bytes {
    let a134 = v0.clone();
    let a135 = Str::lit("");
    if equal(a134, a135) {
        Bytes::lit(&[])
    } else {
        f38(List::cons(f39(Str::lit("          ref: ")), List::cons(f39(v0), List::<Bytes>::nil())))
    }
}

pub fn f44(v0: List<Adt3>) -> Bytes {
    f54(Fn8::F55, Bytes::lit(&[]), v0)
}

pub fn f45(v0: Str) -> Bytes {
    let a138 = {
        let a136 = f1(Str::lit("${{ needs."));
        let a137 = f1(v0);
        append_bytes(a136, a137)
    };
    let a139 = f1(Str::lit(".result }}"));
    append_bytes(a138, a139)
}

pub fn f46(v0: Fn5, p144: u64, p145: List<Bytes>) -> (List<Bytes>, bool) {
    let m143 = (p144, p145);
    {
        let mut m143 = m143;
        loop {
            let step = {
                let (v1, v2) = m143;
                let m140 = v1;
                if m140 == 0 {
                    Err::<(u64, List<Bytes>), (List<Bytes>, bool)>((v2, false))
                } else {
                    let v3 = m140 - 1;
                    let m142 = {
                        let c141: Fn5 = v0.clone();
                        c141.apply(v2.clone())
                    };
                    match m142 {
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
                Ok(next) => { m143 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f47(v0: List<Bytes>) -> Option<List<Bytes>> {
    f56(v0)
}

pub fn f48(v0: u64, v1: u64) -> Bytes {
    f51(Bytes::lit(&[0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39]), {
        let a149 = {
            let a146 = v0;
            let a147 = v1;
            let a148 = 0u64;
            nat_quot(a146, a147, a148)
        };
        let a150 = 10u64;
        let a151 = 0u64;
        nat_rem(a149, a150, a151)
    }, 1u64)
}

pub fn f49(v0: Fn6, p156: u64, p157: u64) -> R<(u64, bool)> {
    let m155 = (p156, p157);
    {
        let mut m155 = m155;
        loop {
            let step = {
                let (v1, v2) = m155;
                let m152 = v1;
                if m152 == 0 {
                    Err::<(u64, u64), R<(u64, bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m152 - 1;
                    let m154 = {
                        let c153: Fn6 = v0.clone();
                        c153.apply(v2)?
                    };
                    match m154 {
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
                Ok(next) => { m155 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f50(v0: Bytes, v1: u64) -> R<Option<u64>> {
    f57(v0, v1)
}

pub fn f51(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m161 = {
        let a158 = v0;
        let a159 = v1;
        let a160 = v2;
        slice_bytes(a158, a159, a160)
    };
    match m161 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f52(v0: Fn7, p165: Bytes, p166: List<Adt1>) -> Bytes {
    let m164 = (p165, p166);
    {
        let mut m164 = m164;
        loop {
            let step = {
                let (v1, v2) = m164;
                let m162 = v2;
                match m162.uncons() {
                    None => {
                        Err::<(Bytes, List<Adt1>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<Adt1>), Bytes>(({
                            let c163: Fn7 = v0.clone();
                            c163.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m164 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f53(v0: Bytes, v1: Adt1) -> Bytes {
    let a169 = v0;
    let a170 = f38(List::cons(f39(Str::lit("      ")), List::cons(f39(match v1.clone() {
        Adt1::C0(h167, _) => {
            h167
        }
    }), List::cons(f39(Str::lit(": ")), List::cons(f39(match v1 {
        Adt1::C0(_, h168) => {
            h168
        }
    }), List::<Bytes>::nil())))));
    append_bytes(a169, a170)
}

pub fn f54(v0: Fn8, p174: Bytes, p175: List<Adt3>) -> Bytes {
    let m173 = (p174, p175);
    {
        let mut m173 = m173;
        loop {
            let step = {
                let (v1, v2) = m173;
                let m171 = v2;
                match m171.uncons() {
                    None => {
                        Err::<(Bytes, List<Adt3>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<Adt3>), Bytes>(({
                            let c172: Fn8 = v0.clone();
                            c172.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m173 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f55(v0: Bytes, v1: Adt3) -> Bytes {
    let a183 = {
        let a180 = {
            let a177 = v0;
            let a178 = f38(List::cons(f39(Str::lit("      - name: ")), List::cons(f39(match v1.clone() {
                Adt3::C0(h176, _, _) => {
                    h176
                }
            }), List::<Bytes>::nil())));
            append_bytes(a177, a178)
        };
        let a181 = f58(match v1.clone() {
            Adt3::C0(_, _, h179) => {
                h179
            }
        });
        append_bytes(a180, a181)
    };
    let a184 = f38(List::cons(f39(Str::lit("        run: ")), List::cons(f39(match v1 {
        Adt3::C0(_, h182, _) => {
            h182
        }
    }), List::<Bytes>::nil())));
    append_bytes(a183, a184)
}

pub fn f56(v0: List<Bytes>) -> Option<List<Bytes>> {
    let m185 = v0.clone();
    match m185.uncons() {
        None => {
            None::<List<Bytes>>
        }
        Some((_, v2)) => {
            let m186 = v2;
            match m186.uncons() {
                None => {
                    None::<List<Bytes>>
                }
                Some((_, _)) => {
                    let v5: (Option<Bytes>, List<Bytes>) = f59(Fn9::F60, (None::<Bytes>, List::<Bytes>::nil()), v0);
                    let m188 = {
                        let (h187, _) = v5;
                        h187
                    };
                    match m188 {
                        None => {
                            Some(f61({
                                let (_, h190) = v5;
                                h190
                            }))
                        }
                        Some(v6) => {
                            Some(f61(List::cons(v6, {
                                let (_, h189) = v5;
                                h189
                            })))
                        }
                    }
                }
            }
        }
    }
}

pub fn f57(v0: Bytes, v1: u64) -> R<Option<u64>> {
    let m191 = f62(v0, v1);
    match m191 {
        None => {
            Ok(None::<u64>)
        }
        Some(v2) => {
            let a192 = v2;
            let a193 = 48u8;
            if equal(a192, a193) {
                Ok(Some({
                    let a194 = v1;
                    let a195 = 1u64;
                    nat_add(a194, a195)?
                }))
            } else {
                Ok(None::<u64>)
            }
        }
    }
}

pub fn f58(v0: List<(Str, Str)>) -> Bytes {
    let m196 = v0.clone();
    match m196.uncons() {
        None => {
            Bytes::lit(&[])
        }
        Some((_, _)) => {
            f63(Fn10::F64, f32(Str::lit("        env:")), v0)
        }
    }
}

pub fn f59(v0: Fn9, p200: (Option<Bytes>, List<Bytes>), p201: List<Bytes>) -> (Option<Bytes>, List<Bytes>) {
    let m199 = (p200, p201);
    {
        let mut m199 = m199;
        loop {
            let step = {
                let (v1, v2) = m199;
                let m197 = v2;
                match m197.uncons() {
                    None => {
                        Err::<((Option<Bytes>, List<Bytes>), List<Bytes>), (Option<Bytes>, List<Bytes>)>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<((Option<Bytes>, List<Bytes>), List<Bytes>), (Option<Bytes>, List<Bytes>)>(({
                            let c198: Fn9 = v0.clone();
                            c198.apply(v1, v3)
                        }, v4))
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

pub fn f60(v0: (Option<Bytes>, List<Bytes>), v1: Bytes) -> (Option<Bytes>, List<Bytes>) {
    f65(v0, v1)
}

pub fn f61(v0: List<Bytes>) -> List<Bytes> {
    f66(Fn11::F67, List::<Bytes>::nil(), v0)
}

pub fn f62(v0: Bytes, v1: u64) -> Option<u8> {
    let a202 = v0;
    let a203 = v1;
    index_bytes(a202, a203)
}

pub fn f63(v0: Fn10, p207: Bytes, p208: List<(Str, Str)>) -> Bytes {
    let m206 = (p207, p208);
    {
        let mut m206 = m206;
        loop {
            let step = {
                let (v1, v2) = m206;
                let m204 = v2;
                match m204.uncons() {
                    None => {
                        Err::<(Bytes, List<(Str, Str)>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<(Str, Str)>), Bytes>(({
                            let c205: Fn10 = v0.clone();
                            c205.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m206 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f64(v0: Bytes, v1: (Str, Str)) -> Bytes {
    let a211 = v0;
    let a212 = f38(List::cons(f39(Str::lit("          ")), List::cons(f39({
        let (h209, _) = v1;
        h209
    }), List::cons(f39(Str::lit(": ")), List::cons(f39({
        let (_, h210) = v1;
        h210
    }), List::<Bytes>::nil())))));
    append_bytes(a211, a212)
}

pub fn f65(v0: (Option<Bytes>, List<Bytes>), v1: Bytes) -> (Option<Bytes>, List<Bytes>) {
    let m214 = {
        let (h213, _) = v0;
        h213
    };
    match m214 {
        None => {
            (Some(v1), {
                let (_, h215) = v0;
                h215
            })
        }
        Some(v2) => {
            (None::<Bytes>, List::cons({
                let a216 = v2;
                let a217 = v1;
                append_bytes(a216, a217)
            }, {
                let (_, h218) = v0;
                h218
            }))
        }
    }
}

pub fn f66(v0: Fn11, p222: List<Bytes>, p223: List<Bytes>) -> List<Bytes> {
    let m221 = (p222, p223);
    {
        let mut m221 = m221;
        loop {
            let step = {
                let (v1, v2) = m221;
                let m219 = v2;
                match m219.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Bytes>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Bytes>), List<Bytes>>(({
                            let c220: Fn11 = v0.clone();
                            c220.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m221 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f67(v0: List<Bytes>, v1: Bytes) -> List<Bytes> {
    List::cons(v1, v0)
}
