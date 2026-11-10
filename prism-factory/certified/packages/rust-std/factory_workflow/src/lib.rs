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
pub enum Fn0 {
    F16,
    F18,
}

impl Fn0 {
    pub fn apply(&self, p0: Bytes, p1: Str) -> Bytes {
        match self {
            Fn0::F16 => f16(p0, p1),
            Fn0::F18 => f18(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn1 {
    F21,
    F27,
}

impl Fn1 {
    pub fn apply(&self, p0: List<Bytes>, p1: Str) -> List<Bytes> {
        match self {
            Fn1::F21 => f21(p0, p1),
            Fn1::F27 => f27(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn2 {
    F23,
}

impl Fn2 {
    pub fn apply(&self, p0: Bytes, p1: (Str, List<Str>)) -> Bytes {
        match self {
            Fn2::F23 => f23(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn3 {
    F26(Bytes),
}

impl Fn3 {
    pub fn apply(&self, p0: Option<Bytes>, p1: Bytes) -> Option<Bytes> {
        match self {
            Fn3::F26(k0) => f26(k0.clone(), p0, p1),
        }
    }
}

pub fn f0(v0: Bytes) -> Bytes {
    let a1 = v0;
    let a2 = f1(Str::lit("release"));
    if equal(a1, a2) {
        f2()
    } else {
        f3(Bytes::lit(&[]), f4())
    }
}

pub fn f1(v0: Str) -> Bytes {
    let a3 = v0;
    utf8_encode(a3)
}

pub fn f2() -> Bytes {
    let v0: Bytes = {
        let a4 = f3(Bytes::lit(&[]), f5());
        let a5 = f6(Str::lit("header"));
        append_bytes(a4, a5)
    };
    let v1: Bytes = {
        let a6 = v0;
        let a7 = f7(f8());
        append_bytes(a6, a7)
    };
    let v2: Bytes = {
        let a8 = f3(v1, f9());
        let a9 = f10(List::cons(Str::lit("factory"), f8()));
        append_bytes(a8, a9)
    };
    f3({
        let a10 = f3(v2, f11());
        let a11 = f12(f13());
        append_bytes(a10, a11)
    }, f14())
}

pub fn f3(v0: Bytes, v1: List<Str>) -> Bytes {
    f15(Fn0::F16, v0, v1)
}

pub fn f4() -> List<Str> {
    List::cons(Str::lit("# Generated by the Prism factory from prism-factory/model/src/Factory/Workflow.lex.tex."), List::cons(Str::lit("# Do not edit: `prism-factory/factory.sh workflow --write` rewrites it and `check` refuses drift."), List::cons(Str::lit("# Every decision (identity, route, order, releasability) is made by the certified"), List::cons(Str::lit("# factory core; this file only gives it a runner and a checkout. Release runs"), List::cons(Str::lit("# record receipts (release.yml, job `receipts`); this one only reads them."), List::cons(Str::lit("name: Factory"), List::cons(Str::lit("on:"), List::cons(Str::lit("  pull_request:"), List::cons(Str::lit("  workflow_dispatch:"), List::cons(Str::lit("permissions:"), List::cons(Str::lit("  contents: read"), List::cons(Str::lit("concurrency:"), List::cons(Str::lit("  group: factory-${{ github.ref }}"), List::cons(Str::lit("  cancel-in-progress: true"), List::cons(Str::lit("jobs:"), List::cons(Str::lit("  factory:"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 60"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: read"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Run the factory"), List::cons(Str::lit("        run: bash prism-factory/factory.sh ci"), List::cons(Str::lit("  pages:"), List::cons(Str::lit("    # Publishes the factory page on a manual run once Pages is set to deploy"), List::cons(Str::lit("    # from Actions and the repository variable FACTORY_PAGES is true."), List::cons(Str::lit("    if: ${{ github.event_name == 'workflow_dispatch' && vars.FACTORY_PAGES == 'true' }}"), List::cons(Str::lit("    needs: factory"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 30"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: read"), List::cons(Str::lit("      pages: write"), List::cons(Str::lit("      id-token: write"), List::cons(Str::lit("    environment:"), List::cons(Str::lit("      name: github-pages"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Build the factory page"), List::cons(Str::lit("        run: bash prism-factory/factory.sh site target/factory-site"), List::cons(Str::lit("      - uses: actions/configure-pages@45bfe0192ca1faeb007ade9deae92b16b8254a0d"), List::cons(Str::lit("      - uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          path: target/factory-site"), List::cons(Str::lit("      - uses: actions/deploy-pages@368f82528645a54fb793d4d04e342629a3f51346"), List::<Str>::nil()))))))))))))))))))))))))))))))))))))))))))))))))))
}

pub fn f5() -> List<Str> {
    List::cons(Str::lit("# Generated by the Prism factory from prism-factory/model and prism-factory/jobs/."), List::cons(Str::lit("# Do not edit: `prism-factory/factory.sh workflow --write` rewrites it and the"), List::cons(Str::lit("# factory-model obligation refuses drift. The factory job decides every route"), List::cons(Str::lit("# with the certified core (prism-factory/certified); each job below runs only"), List::cons(Str::lit("# what that route establishes, and the receipts job records what it proved."), List::<Str>::nil())))))
}

pub fn f6(v0: Str) -> Bytes {
    f17(List::cons(f1(Str::lit("#fragment")), List::cons(f1(v0), List::<Bytes>::nil())))
}

pub fn f7(v0: List<Str>) -> Bytes {
    f15(Fn0::F18, Bytes::lit(&[]), v0)
}

pub fn f8() -> List<Str> {
    List::cons(Str::lit("policy"), List::cons(Str::lit("devcontainer-image"), List::cons(Str::lit("vv-sdk"), List::cons(Str::lit("source-vv"), List::cons(Str::lit("gate"), List::cons(Str::lit("image-platforms"), List::cons(Str::lit("images"), List::cons(Str::lit("reproducibility"), List::cons(Str::lit("installed-sdk"), List::cons(Str::lit("native"), List::cons(Str::lit("oci-native"), List::cons(Str::lit("crates"), List::cons(Str::lit("release"), List::<Str>::nil())))))))))))))
}

pub fn f9() -> List<Str> {
    List::cons(Str::lit(""), List::cons(Str::lit("  # Records a receipt for every obligation this run established, on the"), List::cons(Str::lit("  # factory-receipts branch, and reweighs the schedule by what passed and failed."), List::cons(Str::lit("  receipts:"), List::<Str>::nil()))))
}

pub fn f10(v0: List<Str>) -> Bytes {
    let a14 = {
        let a12 = f1(Str::lit("    needs: ["));
        let a13 = f19(f20(Fn1::F21, List::<Bytes>::nil(), v0), Bytes::lit(&[0x2c, 0x20]));
        append_bytes(a12, a13)
    };
    let a15 = f1(Str::lit("]\u{a}"));
    append_bytes(a14, a15)
}

pub fn f11() -> List<Str> {
    List::cons(Str::lit("    if: ${{ always() && needs.factory.result == 'success' }}"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 15"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: write"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - name: Record receipts"), List::cons(Str::lit("        env:"), List::cons(Str::lit("          FACTORY_PLAN: ${{ needs.factory.outputs.plan }}"), List::cons(Str::lit("          FACTORY_RESULTS: |"), List::<Str>::nil())))))))))))))
}

pub fn f12(v0: List<(Str, List<Str>)>) -> Bytes {
    f22(Fn2::F23, Bytes::lit(&[]), v0)
}

pub fn f13() -> List<(Str, List<Str>)> {
    List::cons((Str::lit("policy"), List::cons(Str::lit("policy"), List::<Str>::nil())), List::cons((Str::lit("factory-model"), List::cons(Str::lit("factory"), List::<Str>::nil())), List::cons((Str::lit("plumbing"), List::cons(Str::lit("devcontainer-image"), List::<Str>::nil())), List::cons((Str::lit("vv-sdk"), List::cons(Str::lit("vv-sdk"), List::<Str>::nil())), List::cons((Str::lit("source-vv"), List::cons(Str::lit("source-vv"), List::cons(Str::lit("gate"), List::<Str>::nil()))), List::cons((Str::lit("build"), List::cons(Str::lit("image-platforms"), List::cons(Str::lit("images"), List::cons(Str::lit("reproducibility"), List::cons(Str::lit("installed-sdk"), List::cons(Str::lit("native"), List::<Str>::nil())))))), List::cons((Str::lit("gate"), List::cons(Str::lit("gate"), List::<Str>::nil())), List::cons((Str::lit("oci-native"), List::cons(Str::lit("oci-native"), List::<Str>::nil())), List::cons((Str::lit("crates"), List::cons(Str::lit("crates"), List::<Str>::nil())), List::cons((Str::lit("release"), List::cons(Str::lit("release"), List::<Str>::nil())), List::<(Str, List<Str>)>::nil()))))))))))
}

pub fn f14() -> List<Str> {
    List::cons(Str::lit("        run: bash prism-factory/factory.sh actions-receipts"), List::<Str>::nil())
}

pub fn f15(v0: Fn0, p19: Bytes, p20: List<Str>) -> Bytes {
    let m18 = (p19, p20);
    {
        let mut m18 = m18;
        loop {
            let step = {
                let (v1, v2) = m18;
                let m16 = v2;
                match m16.uncons() {
                    None => {
                        Err::<(Bytes, List<Str>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<Str>), Bytes>(({
                            let c17: Fn0 = v0.clone();
                            c17.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m18 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f16(v0: Bytes, v1: Str) -> Bytes {
    let a24 = v0;
    let a25 = {
        let a22 = {
            let a21 = v1;
            utf8_encode(a21)
        };
        let a23 = Bytes::lit(&[0x0a]);
        append_bytes(a22, a23)
    };
    append_bytes(a24, a25)
}

pub fn f17(v0: List<Bytes>) -> Bytes {
    let a26 = f19(v0, Bytes::lit(&[0x20]));
    let a27 = Bytes::lit(&[0x0a]);
    append_bytes(a26, a27)
}

pub fn f18(v0: Bytes, v1: Str) -> Bytes {
    let v2: Bytes = {
        let a28 = v0;
        let a29 = f6(v1.clone());
        append_bytes(a28, a29)
    };
    let a30 = v1;
    let a31 = Str::lit("policy");
    if equal(a30, a31) {
        f3(v2, f24())
    } else {
        v2
    }
}

pub fn f19(v0: List<Bytes>, v1: Bytes) -> Bytes {
    let m32 = f25(Fn3::F26(v1), None::<Bytes>, v0);
    match m32 {
        None => {
            Bytes::lit(&[])
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f20(v0: Fn1, p36: List<Bytes>, p37: List<Str>) -> List<Bytes> {
    let m35 = (p36, p37);
    {
        let mut m35 = m35;
        loop {
            let step = {
                let (v1, v2) = m35;
                let m33 = v2;
                match m33.uncons() {
                    None => {
                        Err::<(List<Bytes>, List<Str>), List<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(List<Bytes>, List<Str>), List<Bytes>>(({
                            let c34: Fn1 = v0.clone();
                            c34.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m35 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f21(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a38 = v0;
    let a39 = List::cons(f1(v1), List::<Bytes>::nil());
    append_list(a38, a39)
}

pub fn f22(v0: Fn2, p43: Bytes, p44: List<(Str, List<Str>)>) -> Bytes {
    let m42 = (p43, p44);
    {
        let mut m42 = m42;
        loop {
            let step = {
                let (v1, v2) = m42;
                let m40 = v2;
                match m40.uncons() {
                    None => {
                        Err::<(Bytes, List<(Str, List<Str>)>), Bytes>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Bytes, List<(Str, List<Str>)>), Bytes>(({
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

pub fn f23(v0: Bytes, v1: (Str, List<Str>)) -> Bytes {
    let a49 = v0;
    let a50 = {
        let a47 = f1(Str::lit("            "));
        let a48 = f17(List::cons(f1({
            let (h45, _) = v1;
            h45
        }), f20(Fn1::F27, List::<Bytes>::nil(), {
            let (_, h46) = v1;
            h46
        })));
        append_bytes(a47, a48)
    };
    append_bytes(a49, a50)
}

pub fn f24() -> List<Str> {
    List::cons(Str::lit(""), List::cons(Str::lit("  # The factory route. Every obligation is keyed by its own declared inputs"), List::cons(Str::lit("  # (prism-factory/model/src/Factory/PrismPM.lex.tex). An obligation with a"), List::cons(Str::lit("  # passing receipt for its identity on the factory-receipts branch is proved:"), List::cons(Str::lit("  # its jobs are skipped and its retained evidence is that run's."), List::cons(Str::lit("  factory:"), List::cons(Str::lit("    needs: policy"), List::cons(Str::lit("    runs-on: ubuntu-24.04"), List::cons(Str::lit("    timeout-minutes: 60"), List::cons(Str::lit("    permissions:"), List::cons(Str::lit("      contents: read"), List::cons(Str::lit("    outputs:"), List::cons(Str::lit("      kappa: ${{ steps.route.outputs.kappa }}"), List::cons(Str::lit("      route: ${{ steps.route.outputs.route }}"), List::cons(Str::lit("      source-revision: ${{ steps.route.outputs.source-revision }}"), List::cons(Str::lit("      run-id: ${{ steps.route.outputs.run-id }}"), List::cons(Str::lit("      run-attempt: ${{ steps.route.outputs.run-attempt }}"), List::cons(Str::lit("      build-route: ${{ steps.route.outputs.build-route }}"), List::cons(Str::lit("      build-identity: ${{ steps.route.outputs.build-identity }}"), List::cons(Str::lit("      build-revision: ${{ steps.route.outputs.build-revision }}"), List::cons(Str::lit("      build-run-id: ${{ steps.route.outputs.build-run-id }}"), List::cons(Str::lit("      build-run-attempt: ${{ steps.route.outputs.build-run-attempt }}"), List::cons(Str::lit("      vv-sdk: ${{ steps.route.outputs.vv-sdk }}"), List::cons(Str::lit("      vv-sdk-run-id: ${{ steps.route.outputs.vv-sdk-run-id }}"), List::cons(Str::lit("      plan: ${{ steps.route.outputs.plan }}"), List::cons(Str::lit("    steps:"), List::cons(Str::lit("      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262"), List::cons(Str::lit("        with:"), List::cons(Str::lit("          fetch-depth: 0"), List::cons(Str::lit("          submodules: false"), List::cons(Str::lit("      - id: route"), List::cons(Str::lit("        env:"), List::cons(Str::lit("          FACTORY_MODE: ${{ needs.policy.outputs.rehearsal == 'true' && 'rehearsal' || 'publish' }}"), List::cons(Str::lit("        run: bash prism-factory/factory.sh actions-route"), List::<Str>::nil()))))))))))))))))))))))))))))))))))
}

pub fn f25(v0: Fn3, p54: Option<Bytes>, p55: List<Bytes>) -> Option<Bytes> {
    let m53 = (p54, p55);
    {
        let mut m53 = m53;
        loop {
            let step = {
                let (v1, v2) = m53;
                let m51 = v2;
                match m51.uncons() {
                    None => {
                        Err::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(v1)
                    }
                    Some((v3, v4)) => {
                        Ok::<(Option<Bytes>, List<Bytes>), Option<Bytes>>(({
                            let c52: Fn3 = v0.clone();
                            c52.apply(v1, v3)
                        }, v4))
                    }
                }
            };
            match step {
                Ok(next) => { m53 = next; }
                Err(done) => break done,
            }
        }
    }
}

pub fn f26(v0: Bytes, v1: Option<Bytes>, v2: Bytes) -> Option<Bytes> {
    let m56 = v1;
    match m56 {
        None => {
            Some(v2)
        }
        Some(v3) => {
            Some({
                let a59 = {
                    let a57 = v3;
                    let a58 = v0;
                    append_bytes(a57, a58)
                };
                let a60 = v2;
                append_bytes(a59, a60)
            })
        }
    }
}

pub fn f27(v0: List<Bytes>, v1: Str) -> List<Bytes> {
    let a61 = v0;
    let a62 = List::cons(f28(v1), List::<Bytes>::nil());
    append_list(a61, a62)
}

pub fn f28(v0: Str) -> Bytes {
    let a65 = {
        let a63 = f1(Str::lit("${{ needs."));
        let a64 = f1(v0);
        append_bytes(a63, a64)
    };
    let a66 = f1(Str::lit(".result }}"));
    append_bytes(a65, a66)
}

pub fn run(p0: Bytes) -> Bytes {
    f0(p0)
}
