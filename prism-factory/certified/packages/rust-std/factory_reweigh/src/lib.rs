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
    C0(Str, Adt2, Str, Str, List<Str>, List<Str>),
}

#[derive(Clone)]
pub enum Adt2 {
    C0,
    C1,
}

#[derive(Clone)]
pub enum Adt3 {
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
pub enum Fn0 {
    F3(Adt0),
}

impl Fn0 {
    pub fn apply(&self, p0: Bytes, p1: Adt1) -> Bytes {
        match self {
            Fn0::F3(k0) => f3(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn1 {
    F6,
}

impl Fn1 {
    pub fn apply(&self, p0: Adt0, p1: Bytes) -> R<Adt0> {
        match self {
            Fn1::F6 => f6(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn2 {
    F21(Bytes),
}

impl Fn2 {
    pub fn apply(&self, p0: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
        match self {
            Fn2::F21(k0) => f21(k0.clone(), p0),
        }
    }
}

#[derive(Clone)]
pub enum Fn3 {
    F25,
}

impl Fn3 {
    pub fn apply(&self, p0: List<(Bytes, Bytes)>, p1: (Bytes, Bytes)) -> List<(Bytes, Bytes)> {
        match self {
            Fn3::F25 => f25(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn4 {
    F27,
}

impl Fn4 {
    pub fn apply(&self, p0: List<(Str, Str)>, p1: (Str, Str)) -> List<(Str, Str)> {
        match self {
            Fn4::F27 => f27(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn5 {
    F30(Str),
}

impl Fn5 {
    pub fn apply(&self, p0: Str, p1: (Str, Str)) -> Str {
        match self {
            Fn5::F30(k0) => f30(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn6 {
    F32(Str),
}

impl Fn6 {
    pub fn apply(&self, p0: Option<bool>, p1: (Str, bool)) -> Option<bool> {
        match self {
            Fn6::F32(k0) => f32(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn7 {
    F37,
}

impl Fn7 {
    pub fn apply(&self, p0: List<Bytes>, p1: Str) -> List<Bytes> {
        match self {
            Fn7::F37 => f37(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn8 {
    F47,
}

impl Fn8 {
    pub fn apply(&self, p0: List<Bytes>, p1: Bytes) -> List<Bytes> {
        match self {
            Fn8::F47 => f47(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn9 {
    F53(Bytes),
}

impl Fn9 {
    pub fn apply(&self, p0: Option<Bytes>, p1: Bytes) -> Option<Bytes> {
        match self {
            Fn9::F53(k0) => f53(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn10 {
    F55(Bytes, u64, Bytes, u64),
}

impl Fn10 {
    pub fn apply(&self, p0: u64) -> R<Option<u64>> {
        match self {
            Fn10::F55(k0, k1, k2, k3) => f55(k0.clone(), *k1, k2.clone(), *k3, p0),
        }
    }
}

pub fn f0(v0: Bytes) -> R<Bytes> {
    let v1: Adt0 = f1(v0)?;
    Ok(f2(Fn0::F3(v1), Bytes::lit(&[]), f4()))
}

pub fn f1(v0: Bytes) -> R<Adt0> {
    let v1: Adt0 = f5(Fn1::F6, Adt0::C0(List::<(Bytes, Bytes)>::nil(), List::<(Str, bool)>::nil(), List::<(Str, Str)>::nil(), List::<(Str, bool)>::nil()), f7(v0)?)?;
    Ok(Adt0::C0(f8(match v1.clone() {
        Adt0::C0(h1, _, _, _) => {
            h1
        }
    }), match v1.clone() {
        Adt0::C0(_, h2, _, _) => {
            h2
        }
    }, f9(match v1.clone() {
        Adt0::C0(_, _, h3, _) => {
            h3
        }
    }), match v1 {
        Adt0::C0(_, _, _, h4) => {
            h4
        }
    }))
}

pub fn f2(v0: Fn0, p8: Bytes, p9: List<Adt1>) -> Bytes {
    let m7 = (p8, p9);
    {
        let mut m7 = m7;
        loop {
            let step = {
                let (v1, v2) = m7;
                let m5 = v2;
                match m5.uncons() {
                    None => {
                        Err::<(Bytes, List<Adt1>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<Adt1>), Bytes>(({
                            let c6: Fn0 = v0.clone();
                            c6.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m7 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f3(v0: Adt0, v1: Bytes, v2: Adt1) -> Bytes {
    let v3: Adt3 = f10(match v2.clone() {
        Adt1::C0(h10, _, _, _, _, _) => {
            h10
        }
    }, match v0.clone() {
        Adt0::C0(_, _, h11, _) => {
            h11
        }
    });
    let v5: Adt3 = {
        let m14 = f11(match v2.clone() {
            Adt1::C0(h12, _, _, _, _, _) => {
                h12
            }
        }, match v0 {
            Adt0::C0(_, _, _, h13) => {
                h13
            }
        });
        match m14 {
            None => {
                v3
            }
            Some(v4) => {
                f12(v3, v4)
            }
        }
    };
    let a16 = v1;
    let a17 = f13(List::cons(Str::lit("weight"), List::cons(match v2 {
        Adt1::C0(h15, _, _, _, _, _) => {
            h15
        }
    }, List::cons(f14(v5), List::<Str>::nil()))));
    append_bytes(a16, a17)
}

pub fn f4() -> List<Adt1> {
    let a22 = {
        let a20 = {
            let a18 = f15();
            let a19 = f16();
            append_list(a18, a19)
        };
        let a21 = f17();
        append_list(a20, a21)
    };
    let a23 = f18();
    append_list(a22, a23)
}

pub fn f5(v0: Fn1, p27: Adt0, p28: List<Bytes>) -> R<Adt0> {
    let m26 = (p27, p28);
    {
        let mut m26 = m26;
        loop {
            let step = {
                let (v1, v2) = m26;
                let m24 = v2;
                match m24.uncons() {
                    None => {
                        Err::<(Adt0, List<Bytes>), R<Adt0>>(Ok(v1))
                    }
                    Some((v3, v4)) => {
                        Ok::<(Adt0, List<Bytes>), R<Adt0>>(({
                            let c25: Fn1 = v0.clone();
                            c25.apply(v1, v3)?
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m26 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f6(v0: Adt0, v1: Bytes) -> R<Adt0> {
    f19(v0, v1)
}

pub fn f7(v0: Bytes) -> R<List<Bytes>> {
    let v1: u64 = {
        let a29 = v0.clone();
        length_bytes(a29)
    };
    let v2: (u64, (u64, List<Bytes>)) = {
        let (h30, _) = f20(Fn2::F21(v0.clone()), v1, (0u64, (0u64, List::<Bytes>::nil())))?;
        h30
    };
    let v3: u64 = {
        let (h31, _) = {
            let (_, h32) = v2.clone();
            h32
        };
        h31
    };
    let v4: List<Bytes> = {
        let (_, h33) = {
            let (_, h34) = v2;
            h34
        };
        h33
    };
    let a35 = v3;
    let a36 = v1;
    if nat_lt(a35, a36) {
        Ok(f22(List::cons(f23(v0, v3, {
            let a37 = v1;
            let a38 = v3;
            nat_sub(a37, a38)
        }), v4)))
    } else {
        Ok(f22(v4))
    }
}

pub fn f8(v0: List<(Bytes, Bytes)>) -> List<(Bytes, Bytes)> {
    f24(Fn3::F25, List::<(Bytes, Bytes)>::nil(), v0)
}

pub fn f9(v0: List<(Str, Str)>) -> List<(Str, Str)> {
    f26(Fn4::F27, List::<(Str, Str)>::nil(), v0)
}

pub fn f10(v0: Str, v1: List<(Str, Str)>) -> Adt3 {
    f28(f29(Fn5::F30(v0), Str::lit("4"), v1))
}

pub fn f11(v0: Str, v1: List<(Str, bool)>) -> Option<bool> {
    f31(Fn6::F32(v0), None::<bool>, v1)
}

pub fn f12(v0: Adt3, v1: bool) -> Adt3 {
    if v1 {
        f33(v0)
    } else {
        f34(v0)
    }
}

pub fn f13(v0: List<Str>) -> Bytes {
    f35(f36(Fn7::F37, List::<Bytes>::nil(), v0))
}

pub fn f14(v0: Adt3) -> Str {
    let m39 = v0;
    match m39 {
        Adt3::C0 => {
            Str::lit("1")
        }
        Adt3::C1 => {
            Str::lit("2")
        }
        Adt3::C2 => {
            Str::lit("3")
        }
        Adt3::C3 => {
            Str::lit("4")
        }
        Adt3::C4 => {
            Str::lit("5")
        }
        Adt3::C5 => {
            Str::lit("6")
        }
        Adt3::C6 => {
            Str::lit("7")
        }
        Adt3::C7 => {
            Str::lit("8")
        }
        Adt3::C8 => {
            Str::lit("9")
        }
        Adt3::C9 => {
            Str::lit("10")
        }
        Adt3::C10 => {
            Str::lit("11")
        }
        Adt3::C11 => {
            Str::lit("12")
        }
        Adt3::C12 => {
            Str::lit("13")
        }
        Adt3::C13 => {
            Str::lit("14")
        }
        Adt3::C14 => {
            Str::lit("15")
        }
        Adt3::C15 => {
            Str::lit("16")
        }
    }
}

pub fn f15() -> List<Adt1> {
    List::cons(f38(Str::lit("cargo-fetch"), Adt2::C1, Str::lit("host"), Str::lit("cargo fetch --locked"), List::cons(Str::lit("Cargo.lock"), List::cons(Str::lit("Cargo.toml"), List::cons(Str::lit("crates/"), List::cons(Str::lit("stdlib/"), List::cons(Str::lit("vendor/"), List::cons(Str::lit("xtask/"), List::<Str>::nil())))))), List::<Str>::nil()), List::cons(f38(Str::lit("factory-model"), Adt2::C0, Str::lit("host"), Str::lit("bash prism-factory/check.sh"), List::cons(Str::lit(".github/workflows/factory.yml"), List::cons(Str::lit("prism-factory/"), List::cons(Str::lit("vendor/lexlean/"), List::<Str>::nil()))), List::<Str>::nil()), List::cons(f38(Str::lit("devcontainer-image"), Adt2::C1, Str::lit("actions"), Str::lit("release.yml/devcontainer-image"), List::cons(Str::lit(".devcontainer/"), List::cons(Str::lit("lean-toolchain"), List::cons(Str::lit("rust-toolchain.toml"), List::cons(Str::lit("tools.lock"), List::<Str>::nil())))), List::<Str>::nil()), List::<Adt1>::nil())))
}

pub fn f16() -> List<Adt1> {
    List::cons(f38(Str::lit("source-vv-dk-3"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/dk-3"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-ho-oc"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/ho-oc"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-dk-2"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/dk-2"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-gates"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/gates"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-st-sy"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/st-sy"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-dk-0-1"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/dk-0-1"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-conformance"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/conformance"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::cons(f38(Str::lit("source-vv-source"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/source-vv/source"), f39(), List::cons(Str::lit("cargo-fetch"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil()))), List::<Adt1>::nil()))))))))
}

pub fn f17() -> List<Adt1> {
    List::cons(f38(Str::lit("gate"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/gate"), f39(), f40()), List::cons(f38(Str::lit("sdk-image-amd64"), Adt2::C1, Str::lit("actions"), Str::lit("release.yml/image-platforms/amd64"), f39(), List::<Str>::nil()), List::cons(f38(Str::lit("sdk-image-arm64"), Adt2::C1, Str::lit("actions"), Str::lit("release.yml/image-platforms/arm64"), f39(), List::<Str>::nil()), List::cons(f38(Str::lit("images"), Adt2::C1, Str::lit("actions"), Str::lit("release.yml/images"), f39(), List::cons(Str::lit("sdk-image-amd64"), List::cons(Str::lit("sdk-image-arm64"), List::<Str>::nil()))), List::cons(f38(Str::lit("reproducibility"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/reproducibility"), f39(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f38(Str::lit("library-closure"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/reproducibility/library-sdk-check"), f39(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f38(Str::lit("browser-closure"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/reproducibility/browser-api-sdk-check"), f39(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f38(Str::lit("installed-sdk"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/installed-sdk"), f39(), List::cons(Str::lit("images"), List::<Str>::nil())), List::cons(f38(Str::lit("native"), Adt2::C0, Str::lit("actions"), Str::lit("release.yml/native"), f39(), List::cons(Str::lit("images"), List::<Str>::nil())), List::<Adt1>::nil())))))))))
}

pub fn f18() -> List<Adt1> {
    List::cons(f38(Str::lit("release"), Adt2::C1, Str::lit("actions"), Str::lit("release.yml/release"), f39(), List::cons(Str::lit("factory-model"), List::cons(Str::lit("gate"), List::cons(Str::lit("reproducibility"), List::cons(Str::lit("library-closure"), List::cons(Str::lit("browser-closure"), List::cons(Str::lit("installed-sdk"), List::cons(Str::lit("native"), List::<Str>::nil())))))))), List::<Adt1>::nil())
}

pub fn f19(v0: Adt0, v1: Bytes) -> R<Adt0> {
    let v2: (Bytes, Bytes) = f41(v1)?;
    let v3: Bytes = {
        let (h40, _) = v2;
        h40
    };
    let v4: (Bytes, Bytes) = f41({
        let (_, h41) = v2;
        h41
    })?;
    let a42 = v3.clone();
    let a43 = f42(Str::lit("tree"));
    if equal(a42, a43) {
        Ok(Adt0::C0(List::cons(({
            let (h44, _) = v4;
            h44
        }, {
            let (_, h45) = v4;
            h45
        }), match v0.clone() {
            Adt0::C0(h46, _, _, _) => {
                h46
            }
        }), match v0.clone() {
            Adt0::C0(_, h47, _, _) => {
                h47
            }
        }, match v0.clone() {
            Adt0::C0(_, _, h48, _) => {
                h48
            }
        }, match v0 {
            Adt0::C0(_, _, _, h49) => {
                h49
            }
        }))
    } else {
        let a50 = v3.clone();
        let a51 = f42(Str::lit("receipt"));
        if equal(a50, a51) {
            Ok(Adt0::C0(match v0.clone() {
                Adt0::C0(h52, _, _, _) => {
                    h52
                }
            }, List::cons((f43({
                let (h53, _) = v4;
                h53
            }), f44({
                let (_, h54) = v4;
                h54
            })), match v0.clone() {
                Adt0::C0(_, h55, _, _) => {
                    h55
                }
            }), match v0.clone() {
                Adt0::C0(_, _, h56, _) => {
                    h56
                }
            }, match v0 {
                Adt0::C0(_, _, _, h57) => {
                    h57
                }
            }))
        } else {
            let a58 = v3.clone();
            let a59 = f42(Str::lit("weight"));
            if equal(a58, a59) {
                Ok(Adt0::C0(match v0.clone() {
                    Adt0::C0(h60, _, _, _) => {
                        h60
                    }
                }, match v0.clone() {
                    Adt0::C0(_, h61, _, _) => {
                        h61
                    }
                }, List::cons((f43({
                    let (h62, _) = v4;
                    h62
                }), f43({
                    let (_, h63) = v4;
                    h63
                })), match v0.clone() {
                    Adt0::C0(_, _, h64, _) => {
                        h64
                    }
                }), match v0 {
                    Adt0::C0(_, _, _, h65) => {
                        h65
                    }
                }))
            } else {
                let a66 = v3;
                let a67 = f42(Str::lit("outcome"));
                if equal(a66, a67) {
                    Ok(Adt0::C0(match v0.clone() {
                        Adt0::C0(h68, _, _, _) => {
                            h68
                        }
                    }, match v0.clone() {
                        Adt0::C0(_, h69, _, _) => {
                            h69
                        }
                    }, match v0.clone() {
                        Adt0::C0(_, _, h70, _) => {
                            h70
                        }
                    }, List::cons((f43({
                        let (h71, _) = v4;
                        h71
                    }), f44({
                        let (_, h72) = v4;
                        h72
                    })), match v0 {
                        Adt0::C0(_, _, _, h73) => {
                            h73
                        }
                    })))
                } else {
                    Ok(v0)
                }
            }
        }
    }
}

pub fn f20(v0: Fn2, p78: u64, p79: (u64, (u64, List<Bytes>))) -> R<((u64, (u64, List<Bytes>)), bool)> {
    let m77 = (p78, p79);
    {
        let mut m77 = m77;
        loop {
            let step = {
                let (v1, v2) = m77;
                let m74 = v1;
                if m74 == 0 {
                    Err::<(u64, (u64, (u64, List<Bytes>))), R<((u64, (u64, List<Bytes>)), bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m74 - 1;
                    let m76 = {
                        let c75: Fn2 = v0.clone();
                        c75.apply(v2.clone())?
                    };
                    match m76 {
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
                Ok(next) => { m77 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f21(v0: Bytes, v1: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
    f45(v0, v1)
}

pub fn f22(v0: List<Bytes>) -> List<Bytes> {
    f46(Fn8::F47, List::<Bytes>::nil(), v0)
}

pub fn f23(v0: Bytes, v1: u64, v2: u64) -> Bytes {
    let m83 = {
        let a80 = v0;
        let a81 = v1;
        let a82 = v2;
        slice_bytes(a80, a81, a82)
    };
    match m83 {
        None => {
            Bytes::lit(&[])
        }
        Some(v3) => {
            v3
        }
    }
}

pub fn f24(v0: Fn3, p87: List<(Bytes, Bytes)>, p88: List<(Bytes, Bytes)>) -> List<(Bytes, Bytes)> {
    let m86 = (p87, p88);
    {
        let mut m86 = m86;
        loop {
            let step = {
                let (v1, v2) = m86;
                let m84 = v2;
                match m84.uncons() {
                    None => {
                        Err::<(List<(Bytes, Bytes)>, List<(Bytes, Bytes)>), List<(Bytes, Bytes)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Bytes, Bytes)>, List<(Bytes, Bytes)>), List<(Bytes, Bytes)>>(({
                            let c85: Fn3 = v0.clone();
                            c85.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m86 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f25(v0: List<(Bytes, Bytes)>, v1: (Bytes, Bytes)) -> List<(Bytes, Bytes)> {
    List::cons(v1, v0)
}

pub fn f26(v0: Fn4, p92: List<(Str, Str)>, p93: List<(Str, Str)>) -> List<(Str, Str)> {
    let m91 = (p92, p93);
    {
        let mut m91 = m91;
        loop {
            let step = {
                let (v1, v2) = m91;
                let m89 = v2;
                match m89.uncons() {
                    None => {
                        Err::<(List<(Str, Str)>, List<(Str, Str)>), List<(Str, Str)>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<(Str, Str)>, List<(Str, Str)>), List<(Str, Str)>>(({
                            let c90: Fn4 = v0.clone();
                            c90.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m91 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f27(v0: List<(Str, Str)>, v1: (Str, Str)) -> List<(Str, Str)> {
    List::cons(v1, v0)
}

pub fn f28(v0: Str) -> Adt3 {
    let a94 = v0.clone();
    let a95 = Str::lit("1");
    if equal(a94, a95) {
        Adt3::C0
    } else {
        let a96 = v0.clone();
        let a97 = Str::lit("2");
        if equal(a96, a97) {
            Adt3::C1
        } else {
            let a98 = v0.clone();
            let a99 = Str::lit("3");
            if equal(a98, a99) {
                Adt3::C2
            } else {
                let a100 = v0.clone();
                let a101 = Str::lit("4");
                if equal(a100, a101) {
                    Adt3::C3
                } else {
                    let a102 = v0.clone();
                    let a103 = Str::lit("5");
                    if equal(a102, a103) {
                        Adt3::C4
                    } else {
                        let a104 = v0.clone();
                        let a105 = Str::lit("6");
                        if equal(a104, a105) {
                            Adt3::C5
                        } else {
                            let a106 = v0.clone();
                            let a107 = Str::lit("7");
                            if equal(a106, a107) {
                                Adt3::C6
                            } else {
                                let a108 = v0.clone();
                                let a109 = Str::lit("8");
                                if equal(a108, a109) {
                                    Adt3::C7
                                } else {
                                    let a110 = v0.clone();
                                    let a111 = Str::lit("9");
                                    if equal(a110, a111) {
                                        Adt3::C8
                                    } else {
                                        let a112 = v0.clone();
                                        let a113 = Str::lit("10");
                                        if equal(a112, a113) {
                                            Adt3::C9
                                        } else {
                                            let a114 = v0.clone();
                                            let a115 = Str::lit("11");
                                            if equal(a114, a115) {
                                                Adt3::C10
                                            } else {
                                                let a116 = v0.clone();
                                                let a117 = Str::lit("12");
                                                if equal(a116, a117) {
                                                    Adt3::C11
                                                } else {
                                                    let a118 = v0.clone();
                                                    let a119 = Str::lit("13");
                                                    if equal(a118, a119) {
                                                        Adt3::C12
                                                    } else {
                                                        let a120 = v0.clone();
                                                        let a121 = Str::lit("14");
                                                        if equal(a120, a121) {
                                                            Adt3::C13
                                                        } else {
                                                            let a122 = v0.clone();
                                                            let a123 = Str::lit("15");
                                                            if equal(a122, a123) {
                                                                Adt3::C14
                                                            } else {
                                                                let a124 = v0;
                                                                let a125 = Str::lit("16");
                                                                if equal(a124, a125) {
                                                                    Adt3::C15
                                                                } else {
                                                                    Adt3::C3
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

pub fn f29(v0: Fn5, p129: Str, p130: List<(Str, Str)>) -> Str {
    let m128 = (p129, p130);
    {
        let mut m128 = m128;
        loop {
            let step = {
                let (v1, v2) = m128;
                let m126 = v2;
                match m126.uncons() {
                    None => {
                        Err::<(Str, List<(Str, Str)>), Str>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Str, List<(Str, Str)>), Str>(({
                            let c127: Fn5 = v0.clone();
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

pub fn f30(v0: Str, v1: Str, v2: (Str, Str)) -> Str {
    let a132 = {
        let (h131, _) = v2;
        h131
    };
    let a133 = v0;
    if equal(a132, a133) {
        let (_, h134) = v2;
        h134
    } else {
        v1
    }
}

pub fn f31(v0: Fn6, p138: Option<bool>, p139: List<(Str, bool)>) -> Option<bool> {
    let m137 = (p138, p139);
    {
        let mut m137 = m137;
        loop {
            let step = {
                let (v1, v2) = m137;
                let m135 = v2;
                match m135.uncons() {
                    None => {
                        Err::<(Option<bool>, List<(Str, bool)>), Option<bool>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<bool>, List<(Str, bool)>), Option<bool>>(({
                            let c136: Fn6 = v0.clone();
                            c136.apply(v1, v3)
                        }, v4))
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

pub fn f32(v0: Str, v1: Option<bool>, v2: (Str, bool)) -> Option<bool> {
    let a141 = {
        let (h140, _) = v2;
        h140
    };
    let a142 = v0;
    if equal(a141, a142) {
        Some({
            let (_, h143) = v2;
            h143
        })
    } else {
        v1
    }
}

pub fn f33(v0: Adt3) -> Adt3 {
    let m144 = v0;
    match m144 {
        Adt3::C0 => {
            Adt3::C0
        }
        Adt3::C1 => {
            Adt3::C0
        }
        Adt3::C2 => {
            Adt3::C1
        }
        Adt3::C3 => {
            Adt3::C2
        }
        Adt3::C4 => {
            Adt3::C3
        }
        Adt3::C5 => {
            Adt3::C4
        }
        Adt3::C6 => {
            Adt3::C5
        }
        Adt3::C7 => {
            Adt3::C6
        }
        Adt3::C8 => {
            Adt3::C7
        }
        Adt3::C9 => {
            Adt3::C8
        }
        Adt3::C10 => {
            Adt3::C9
        }
        Adt3::C11 => {
            Adt3::C10
        }
        Adt3::C12 => {
            Adt3::C11
        }
        Adt3::C13 => {
            Adt3::C12
        }
        Adt3::C14 => {
            Adt3::C13
        }
        Adt3::C15 => {
            Adt3::C14
        }
    }
}

pub fn f34(v0: Adt3) -> Adt3 {
    let m145 = v0;
    match m145 {
        Adt3::C0 => {
            Adt3::C2
        }
        Adt3::C1 => {
            Adt3::C3
        }
        Adt3::C2 => {
            Adt3::C4
        }
        Adt3::C3 => {
            Adt3::C5
        }
        Adt3::C4 => {
            Adt3::C6
        }
        Adt3::C5 => {
            Adt3::C7
        }
        Adt3::C6 => {
            Adt3::C8
        }
        Adt3::C7 => {
            Adt3::C9
        }
        Adt3::C8 => {
            Adt3::C10
        }
        Adt3::C9 => {
            Adt3::C11
        }
        Adt3::C10 => {
            Adt3::C12
        }
        Adt3::C11 => {
            Adt3::C13
        }
        Adt3::C12 => {
            Adt3::C14
        }
        Adt3::C13 => {
            Adt3::C15
        }
        Adt3::C14 => {
            Adt3::C15
        }
        Adt3::C15 => {
            Adt3::C15
        }
    }
}

pub fn f35(v0: List<Bytes>) -> Bytes {
    let a146 = f48(v0, Bytes::lit(&[0x20]));
    let a147 = Bytes::lit(&[0x0a]);
    append_bytes(a146, a147)
}

pub fn f36(v0: Fn7, p151: List<Bytes>, p152: List<Str>) -> List<Bytes> {
    let m150 = (p151, p152);
    {
        let mut m150 = m150;
        loop {
            let step = {
                let (v1, v2) = m150;
                let m148 = v2;
                match m148.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Str>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Str>), List<Bytes>>(({
                            let c149: Fn7 = v0.clone();
                            c149.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m150 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f37(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a153 = v0;
    let a154 = List::cons(f42(v1), List::<Bytes>::nil());
    append_list(a153, a154)
}

pub fn f38(v0: Str, v1: Adt2, v2: Str, v3: Str, v4: List<Str>, v5: List<Str>) -> Adt1 {
    Adt1::C0(v0, v1, v2, v3, v4, v5)
}

pub fn f39() -> List<Str> {
    List::cons(Str::lit(""), List::<Str>::nil())
}

pub fn f40() -> List<Str> {
    List::cons(Str::lit("source-vv-dk-3"), List::cons(Str::lit("source-vv-ho-oc"), List::cons(Str::lit("source-vv-dk-2"), List::cons(Str::lit("source-vv-gates"), List::cons(Str::lit("source-vv-st-sy"), List::cons(Str::lit("source-vv-dk-0-1"), List::cons(Str::lit("source-vv-conformance"), List::cons(Str::lit("source-vv-source"), List::<Str>::nil()))))))))
}

pub fn f41(v0: Bytes) -> R<(Bytes, Bytes)> {
    let m155 = f49(v0.clone(), Bytes::lit(&[0x20]))?;
    match m155 {
        None => {
            Ok((v0, Bytes::lit(&[])))
        }
        Some(v1) => {
            Ok((f23(v0.clone(), 0u64, v1), f23(v0.clone(), {
                let a156 = v1;
                let a157 = 1u64;
                nat_add(a156, a157)?
            }, {
                let a161 = {
                    let a158 = v0;
                    length_bytes(a158)
                };
                let a162 = {
                    let a159 = v1;
                    let a160 = 1u64;
                    nat_add(a159, a160)?
                };
                nat_sub(a161, a162)
            })))
        }
    }
}

pub fn f42(v0: Str) -> Bytes {
    let a163 = v0;
    utf8_encode(a163)
}

pub fn f43(v0: Bytes) -> Str {
    let m165 = {
        let a164 = v0;
        utf8_decode(a164)
    };
    match m165 {
        None => {
            Str::lit("")
        }
        Some(v1) => {
            v1
        }
    }
}

pub fn f44(v0: Bytes) -> bool {
    let a166 = v0;
    let a167 = f42(Str::lit("pass"));
    equal(a166, a167)
}

pub fn f45(v0: Bytes, v1: (u64, (u64, List<Bytes>))) -> R<Option<(u64, (u64, List<Bytes>))>> {
    let v2: u64 = {
        let (h168, _) = v1.clone();
        h168
    };
    let v3: u64 = {
        let (h169, _) = {
            let (_, h170) = v1.clone();
            h170
        };
        h169
    };
    let v4: List<Bytes> = {
        let (_, h171) = {
            let (_, h172) = v1;
            h172
        };
        h171
    };
    let m173 = f50(v0.clone(), v2);
    match m173 {
        None => {
            Ok(None::<(u64, (u64, List<Bytes>))>)
        }
        Some(v5) => {
            let a174 = v5;
            let a175 = 10u8;
            if equal(a174, a175) {
                Ok(Some(({
                    let a176 = v2;
                    let a177 = 1u64;
                    nat_add(a176, a177)?
                }, ({
                    let a178 = v2;
                    let a179 = 1u64;
                    nat_add(a178, a179)?
                }, List::cons(f51(f23(v0, v3, {
                    let a180 = v2;
                    let a181 = v3;
                    nat_sub(a180, a181)
                })), v4)))))
            } else {
                Ok(Some(({
                    let a182 = v2;
                    let a183 = 1u64;
                    nat_add(a182, a183)?
                }, (v3, v4))))
            }
        }
    }
}

pub fn f46(v0: Fn8, p187: List<Bytes>, p188: List<Bytes>) -> List<Bytes> {
    let m186 = (p187, p188);
    {
        let mut m186 = m186;
        loop {
            let step = {
                let (v1, v2) = m186;
                let m184 = v2;
                match m184.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Bytes>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Bytes>), List<Bytes>>(({
                            let c185: Fn8 = v0.clone();
                            c185.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m186 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f47(v0: List<Bytes>, v1: Bytes) -> List<Bytes> {
    List::cons(v1, v0)
}

pub fn f48(v0: List<Bytes>, v1: Bytes) -> Bytes {
    let m189 = f52(Fn9::F53(v1), None::<Bytes>, v0);
    match m189 {
        None => {
            Bytes::lit(&[])
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f49(v0: Bytes, v1: Bytes) -> R<Option<u64>> {
    let v2: u64 = {
        let a190 = v0.clone();
        length_bytes(a190)
    };
    let v3: u64 = {
        let a191 = v1.clone();
        length_bytes(a191)
    };
    let v4: u64 = {
        let (h192, _) = f54(Fn10::F55(v1.clone(), v3, v0.clone(), v2), {
            let a193 = v2;
            let a194 = 1u64;
            nat_add(a193, a194)?
        }, 0u64)?;
        h192
    };
    if f56(v0, v1, v4) {
        Ok(Some(v4))
    } else {
        Ok(None::<u64>)
    }
}

pub fn f50(v0: Bytes, v1: u64) -> Option<u8> {
    let a195 = v0;
    let a196 = v1;
    index_bytes(a195, a196)
}

pub fn f51(v0: Bytes) -> Bytes {
    if f57(v0.clone(), Bytes::lit(&[0x0d])) {
        f23(v0.clone(), 0u64, {
            let a198 = {
                let a197 = v0;
                length_bytes(a197)
            };
            let a199 = 1u64;
            nat_sub(a198, a199)
        })
    } else {
        v0
    }
}

pub fn f52(v0: Fn9, p203: Option<Bytes>, p204: List<Bytes>) -> Option<Bytes> {
    let m202 = (p203, p204);
    {
        let mut m202 = m202;
        loop {
            let step = {
                let (v1, v2) = m202;
                let m200 = v2;
                match m200.uncons() {
                    None => {
                        Err::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(({
                            let c201: Fn9 = v0.clone();
                            c201.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m202 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f53(v0: Bytes, v1: Option<Bytes>, v2: Bytes) -> Option<Bytes> {
    let m205 = v1;
    match m205 {
        None => {
            Some(v2)
        }
        Some(v3) => {
            Some({
                let a208 = {
                    let a206 = v3;
                    let a207 = v0;
                    append_bytes(a206, a207)
                };
                let a209 = v2;
                append_bytes(a208, a209)
            })
        }
    }
}

pub fn f54(v0: Fn10, p214: u64, p215: u64) -> R<(u64, bool)> {
    let m213 = (p214, p215);
    {
        let mut m213 = m213;
        loop {
            let step = {
                let (v1, v2) = m213;
                let m210 = v1;
                if m210 == 0 {
                    Err::<(u64, u64), R<(u64, bool)>>(Ok((v2, false)))
                } else {
                    let v3 = m210 - 1;
                    let m212 = {
                        let c211: Fn10 = v0.clone();
                        c211.apply(v2)?
                    };
                    match m212 {
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
                Ok(next) => { m213 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f55(v0: Bytes, v1: u64, v2: Bytes, v3: u64, v4: u64) -> R<Option<u64>> {
    if f56(v2, v0, v4) {
        Ok(None::<u64>)
    } else {
        let a218 = {
            let a216 = v4;
            let a217 = v1;
            nat_add(a216, a217)?
        };
        let a219 = v3;
        if nat_lt(a218, a219) {
            Ok(Some({
                let a220 = v4;
                let a221 = 1u64;
                nat_add(a220, a221)?
            }))
        } else {
            Ok(None::<u64>)
        }
    }
}

pub fn f56(v0: Bytes, v1: Bytes, v2: u64) -> bool {
    let m226 = {
        let a223 = v0;
        let a224 = v2;
        let a225 = {
            let a222 = v1.clone();
            length_bytes(a222)
        };
        slice_bytes(a223, a224, a225)
    };
    match m226 {
        None => {
            false
        }
        Some(v3) => {
            let a227 = v3;
            let a228 = v1;
            equal(a227, a228)
        }
    }
}

pub fn f57(v0: Bytes, v1: Bytes) -> bool {
    let v2: u64 = {
        let a229 = v0.clone();
        length_bytes(a229)
    };
    let v3: u64 = {
        let a230 = v1.clone();
        length_bytes(a230)
    };
    let a231 = v3;
    let a232 = v2;
    if nat_le(a231, a232) {
        let m238 = {
            let a235 = v0;
            let a236 = {
                let a233 = v2;
                let a234 = v3;
                nat_sub(a233, a234)
            };
            let a237 = v3;
            slice_bytes(a235, a236, a237)
        };
        match m238 {
            None => {
                false
            }
            Some(v4) => {
                let a239 = v4;
                let a240 = v1;
                equal(a239, a240)
            }
        }
    } else {
        false
    }
}

pub fn run(p0: Bytes) -> R<Bytes> {
    f0(p0)
}
