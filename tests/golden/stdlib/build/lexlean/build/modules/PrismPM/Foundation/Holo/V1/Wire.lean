module
public import Init
set_option autoImplicit false
set_option maxRecDepth 100000
set_option maxHeartbeats 1000000000
namespace PrismPM.Foundation.Holo.V1.Wire

namespace LexLeanRuntime

public class ToMathInt (α : Type) where
  toInt : α -> Int

public class Fixed (α : Type) extends ToMathInt α where
  fromInt : Int -> α
  minimum : Int
  maximum : Int
  bitAnd : α -> α -> α
  bitOr : α -> α -> α
  bitXor : α -> α -> α
  bitNot : α -> α
  shiftLeft : α -> UInt32 -> Option α
  shiftRight : α -> UInt32 -> Option α

public instance : ToMathInt Int where toInt := fun value => value

public instance : Fixed Int8 where
  toInt := Int8.toInt
  fromInt := Int8.ofInt
  minimum := -128
  maximum := 127
  bitAnd := Int8.land
  bitOr := Int8.lor
  bitXor := Int8.xor
  bitNot := Int8.complement
  shiftLeft := fun value amount => if amount.toNat < 8 then some (Int8.shiftLeft value (Int8.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 8 then some (Int8.shiftRight value (Int8.ofNat amount.toNat)) else none

public instance : Fixed Int16 where
  toInt := Int16.toInt
  fromInt := Int16.ofInt
  minimum := -32768
  maximum := 32767
  bitAnd := Int16.land
  bitOr := Int16.lor
  bitXor := Int16.xor
  bitNot := Int16.complement
  shiftLeft := fun value amount => if amount.toNat < 16 then some (Int16.shiftLeft value (Int16.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 16 then some (Int16.shiftRight value (Int16.ofNat amount.toNat)) else none

public instance : Fixed Int32 where
  toInt := Int32.toInt
  fromInt := Int32.ofInt
  minimum := -2147483648
  maximum := 2147483647
  bitAnd := Int32.land
  bitOr := Int32.lor
  bitXor := Int32.xor
  bitNot := Int32.complement
  shiftLeft := fun value amount => if amount.toNat < 32 then some (Int32.shiftLeft value (Int32.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 32 then some (Int32.shiftRight value (Int32.ofNat amount.toNat)) else none

public instance : Fixed Int64 where
  toInt := Int64.toInt
  fromInt := Int64.ofInt
  minimum := -9223372036854775808
  maximum := 9223372036854775807
  bitAnd := Int64.land
  bitOr := Int64.lor
  bitXor := Int64.xor
  bitNot := Int64.complement
  shiftLeft := fun value amount => if amount.toNat < 64 then some (Int64.shiftLeft value (Int64.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 64 then some (Int64.shiftRight value (Int64.ofNat amount.toNat)) else none

public instance : Fixed UInt8 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt8.ofInt
  minimum := 0
  maximum := 255
  bitAnd := UInt8.land
  bitOr := UInt8.lor
  bitXor := UInt8.xor
  bitNot := UInt8.complement
  shiftLeft := fun value amount => if amount.toNat < 8 then some (UInt8.shiftLeft value (UInt8.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 8 then some (UInt8.shiftRight value (UInt8.ofNat amount.toNat)) else none

public instance : Fixed UInt16 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt16.ofInt
  minimum := 0
  maximum := 65535
  bitAnd := UInt16.land
  bitOr := UInt16.lor
  bitXor := UInt16.xor
  bitNot := UInt16.complement
  shiftLeft := fun value amount => if amount.toNat < 16 then some (UInt16.shiftLeft value (UInt16.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 16 then some (UInt16.shiftRight value (UInt16.ofNat amount.toNat)) else none

public instance : Fixed UInt32 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt32.ofInt
  minimum := 0
  maximum := 4294967295
  bitAnd := UInt32.land
  bitOr := UInt32.lor
  bitXor := UInt32.xor
  bitNot := UInt32.complement
  shiftLeft := fun value amount => if amount.toNat < 32 then some (UInt32.shiftLeft value amount) else none
  shiftRight := fun value amount => if amount.toNat < 32 then some (UInt32.shiftRight value amount) else none

public instance : Fixed UInt64 where
  toInt := fun value => Int.ofNat value.toNat
  fromInt := UInt64.ofInt
  minimum := 0
  maximum := 18446744073709551615
  bitAnd := UInt64.land
  bitOr := UInt64.lor
  bitXor := UInt64.xor
  bitNot := UInt64.complement
  shiftLeft := fun value amount => if amount.toNat < 64 then some (UInt64.shiftLeft value (UInt64.ofNat amount.toNat)) else none
  shiftRight := fun value amount => if amount.toNat < 64 then some (UInt64.shiftRight value (UInt64.ofNat amount.toNat)) else none

@[expose] public def checkedFromInt {α : Type} [Fixed α] (value : Int) : Option α :=
  if value < Fixed.minimum (α := α) then none else if Fixed.maximum (α := α) < value then none else some (Fixed.fromInt value)

@[expose] public def checkedConvert {α β : Type} [ToMathInt α] [Fixed β] (value : α) : Option β :=
  checkedFromInt (ToMathInt.toInt value)

@[expose] public def checkedAdd {α : Type} [Fixed α] (left right : α) : Option α :=
  checkedFromInt (ToMathInt.toInt left + ToMathInt.toInt right)

@[expose] public def checkedSubtract {α : Type} [Fixed α] (left right : α) : Option α :=
  checkedFromInt (ToMathInt.toInt left - ToMathInt.toInt right)

@[expose] public def checkedMultiply {α : Type} [Fixed α] (left right : α) : Option α :=
  checkedFromInt (ToMathInt.toInt left * ToMathInt.toInt right)

@[expose] public def checkedNegate {α : Type} [Fixed α] (value : α) : Option α :=
  checkedFromInt (-ToMathInt.toInt value)

@[expose] public def checkedQuotient {α : Type} [Fixed α] (left right : α) : Option α :=
  if ToMathInt.toInt right = 0 then none else checkedFromInt (Int.tdiv (ToMathInt.toInt left) (ToMathInt.toInt right))

@[expose] public def checkedAddInt64 (left right : Int64) : Option Int64 :=
  let value := left + right
  if (0 < right && value < left) || (right < 0 && left < value) then none else some value

@[expose] public def checkedSubtractInt64 (left right : Int64) : Option Int64 :=
  let value := left - right
  if (0 < right && left < value) || (right < 0 && value < left) then none else some value

@[expose] public def checkedNegateInt64 (value : Int64) : Option Int64 :=
  if value == (-9223372036854775808 : Int64) then none else some (-value)

public def magnitudeInt64 (value : Int64) : UInt64 :=
  let bits := value.toUInt64
  if value < 0 then 0 - bits else bits

public def signedMagnitudeInt64 (negative : Bool) (value : UInt64) : Int64 :=
  (if negative then 0 - value else value).toInt64

public def divideMagnitudeInt64 : Nat -> UInt64 -> UInt64 -> UInt64 -> UInt64 -> UInt64
  | 0, _, _, _, quotient => quotient
  | Nat.succ fuel, source, divisor, remainder, quotient =>
      let high := 9223372036854775808 <= source
      let source := source + source
      let remainder := remainder + remainder + if high then 1 else 0
      let quotient := quotient + quotient
      if divisor <= remainder then
        divideMagnitudeInt64 fuel source divisor (remainder - divisor) (quotient + 1)
      else
        divideMagnitudeInt64 fuel source divisor remainder quotient

@[expose] public def checkedQuotientInt64 (left right : Int64) : Option Int64 :=
  if right == 0 then none
  else if left == (-9223372036854775808 : Int64) && right == (-1 : Int64) then none
  else
    let negative := (left < 0) != (right < 0)
    some (signedMagnitudeInt64 negative
      (divideMagnitudeInt64 64 (magnitudeInt64 left) (magnitudeInt64 right) 0 0))

public def multiplyMagnitudeInt64 : Nat -> UInt64 -> UInt64 -> UInt64 -> Bool -> Option UInt64
  | 0, _, _, accumulator, _ => some accumulator
  | Nat.succ fuel, source, multiplicand, accumulator, negative =>
      let high := 9223372036854775808 <= source
      let limit := if negative then 9223372036854775808 else 9223372036854775807
      let halfLimit := if negative then 4611686018427387904 else 4611686018427387903
      if halfLimit < accumulator then none
      else
        let doubled := accumulator + accumulator
        if high then
          if limit < multiplicand || limit - multiplicand < doubled then none
          else multiplyMagnitudeInt64 fuel (source + source) multiplicand
            (doubled + multiplicand) negative
        else
          multiplyMagnitudeInt64 fuel (source + source) multiplicand doubled negative

@[expose] public def checkedMultiplyInt64 (left right : Int64) : Option Int64 :=
  let negative := (left < 0) != (right < 0)
  match multiplyMagnitudeInt64 64 (magnitudeInt64 right) (magnitudeInt64 left) 0 negative with
  | none => none
  | some value => some (signedMagnitudeInt64 negative value)

@[noinline] public def subtract {α : Type} [Sub α] (left right : α) : α := left - right
@[noinline] public def multiply {α : Type} [Mul α] (left right : α) : α := left * right
@[noinline] public def negate {α : Type} [Neg α] (value : α) : α := -value

public class Quotient (α : Type) where
  quotient : α -> α -> α
  remainder : α -> α -> α
  isZero : α -> Bool

public instance : Quotient Nat where
  quotient := Nat.div
  remainder := Nat.mod
  isZero := fun value => value == 0

public instance : Quotient Int where
  quotient := Int.tdiv
  remainder := Int.tmod
  isZero := fun value => value == 0

@[noinline] public def quotient {α : Type} [Quotient α] (left right zeroCase : α) : α :=
  if Quotient.isZero right then zeroCase else Quotient.quotient left right

@[noinline] public def remainder {α : Type} [Quotient α] (left right zeroCase : α) : α :=
  if Quotient.isZero right then zeroCase else Quotient.remainder left right

@[expose] public def bitAnd {α : Type} [Fixed α] (left right : α) : α := Fixed.bitAnd left right
@[expose] public def bitOr {α : Type} [Fixed α] (left right : α) : α := Fixed.bitOr left right
@[expose] public def bitXor {α : Type} [Fixed α] (left right : α) : α := Fixed.bitXor left right
@[expose] public def bitNot {α : Type} [Fixed α] (value : α) : α := Fixed.bitNot value
@[expose] public def shiftLeft {α : Type} [Fixed α] (value : α) (amount : UInt32) : Option α := Fixed.shiftLeft value amount
@[expose] public def shiftRight {α : Type} [Fixed α] (value : α) (amount : UInt32) : Option α := Fixed.shiftRight value amount

public class Appendable (α : Type) where append : α -> α -> α
public instance {α : Type} : Appendable (List α) where append := List.append
public instance : Appendable ByteArray where append := ByteArray.append
@[expose] public def append {α : Type} [Appendable α] (left right : α) : α := Appendable.append left right

public class Lengthable (α : Type) where length : α -> Nat
public instance {α : Type} : Lengthable (List α) where length := List.length
public instance : Lengthable ByteArray where length := ByteArray.size
public instance : Lengthable String where length := String.length
@[expose] public def length {α : Type} [Lengthable α] (value : α) : Nat := Lengthable.length value

@[expose] public def listIndex {α : Type} : List α -> Nat -> Option α
  | [], _ => none
  | head :: _, 0 => some head
  | _ :: tail, index + 1 => listIndex tail index

public class Indexable (α β : Type) where index : α -> Nat -> Option β
public instance {α : Type} : Indexable (List α) α where index := listIndex
public instance : Indexable ByteArray UInt8 where index := fun value offset => value.data[offset]?
@[noinline] public def index {α β : Type} [Indexable α β] (value : α) (offset : Nat) : Option β := Indexable.index value offset

public class Sliceable (α : Type) where slice : α -> Nat -> Nat -> Option α
public instance {α : Type} : Sliceable (List α) where
  slice := fun value start count => if start + count <= value.length then some ((value.drop start).take count) else none
public instance : Sliceable ByteArray where
  slice := fun value start count => if start + count <= value.size then some (value.extract start (start + count)) else none
@[noinline] public def slice {α : Type} [Sliceable α] (value : α) (start count : Nat) : Option α := Sliceable.slice value start count

@[noinline] public def utf8Encode (value : String) : ByteArray := value.toUTF8
@[noinline] public def utf8Decode (value : ByteArray) : Option String := String.fromUTF8? value
@[noinline] public def compareBytes (left right : ByteArray) : Ordering := compare left.toList right.toList
@[expose] public def equal {α : Type} [BEq α] (left right : α) : Bool := left == right

@[noinline] public def splitExact (value delimiter : String) (maximum : UInt32) : Option (List String) :=
  let fields := value.splitOn delimiter
  if delimiter.isEmpty || maximum.toNat < fields.length then none else some fields

@[noinline] public def join (values : List String) (delimiter : String) : String := delimiter.intercalate values

public class Decimal (α : Type) where
  parse : String -> Option α
  format : α -> String

public instance : Decimal Int where
  parse := fun value => match value.toInt? with | some parsed => if toString parsed = value then some parsed else none | none => none
  format := toString

public instance {α : Type} [Fixed α] [ToString α] : Decimal α where
  parse := fun value => match value.toInt? with | some parsed => if toString parsed = value then checkedFromInt parsed else none | none => none
  format := toString

@[noinline] public def parseDecimal {α : Type} [Decimal α] (value : String) : Option α := Decimal.parse value
@[noinline] public def formatDecimal {α : Type} [Decimal α] (value : α) : String := Decimal.format value

end LexLeanRuntime

@[expose] public def wireSlice (value : ByteArray) (start : Nat) (count : Nat) : ByteArray := (match (LexLeanRuntime.slice (value) (start) (count) : Option (ByteArray)) with | Option.none => ByteArray.mk #[] | Option.some part => part)

@[expose] public def wireBytesEqual (left : ByteArray) (right : ByteArray) : Bool := (LexLeanRuntime.equal ((LexLeanRuntime.compareBytes (left) (right) : Ordering)) ((LexLeanRuntime.compareBytes (ByteArray.mk #[UInt8.ofNat (nat_lit 0)]) (ByteArray.mk #[UInt8.ofNat (nat_lit 0)]) : Ordering)) : Bool)

@[expose] public def wireBytesLess (left : ByteArray) (right : ByteArray) : Bool := (LexLeanRuntime.equal ((LexLeanRuntime.compareBytes (left) (right) : Ordering)) ((LexLeanRuntime.compareBytes (ByteArray.mk #[UInt8.ofNat (nat_lit 0)]) (ByteArray.mk #[UInt8.ofNat (nat_lit 1)]) : Ordering)) : Bool)

@[expose] public def wireWindowEquals (value : ByteArray) (start : Nat) (count : Nat) (expected : ByteArray) : Bool := (match (LexLeanRuntime.slice (value) (start) (count) : Option (ByteArray)) with | Option.none => false | Option.some part => wireBytesEqual (part) (expected))

@[expose] public def wireEncodeOctet (value : Nat) : ByteArray := (if (Nat.blt (value) (256)) then (if (Nat.blt (value) (128)) then (if (Nat.blt (value) (64)) then (if (Nat.blt (value) (32)) then (if (Nat.blt (value) (16)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 1), UInt8.ofNat (nat_lit 2), UInt8.ofNat (nat_lit 3), UInt8.ofNat (nat_lit 4), UInt8.ofNat (nat_lit 5), UInt8.ofNat (nat_lit 6), UInt8.ofNat (nat_lit 7), UInt8.ofNat (nat_lit 8), UInt8.ofNat (nat_lit 9), UInt8.ofNat (nat_lit 10), UInt8.ofNat (nat_lit 11), UInt8.ofNat (nat_lit 12), UInt8.ofNat (nat_lit 13), UInt8.ofNat (nat_lit 14), UInt8.ofNat (nat_lit 15)]) ((LexLeanRuntime.subtract (value) (0) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 16), UInt8.ofNat (nat_lit 17), UInt8.ofNat (nat_lit 18), UInt8.ofNat (nat_lit 19), UInt8.ofNat (nat_lit 20), UInt8.ofNat (nat_lit 21), UInt8.ofNat (nat_lit 22), UInt8.ofNat (nat_lit 23), UInt8.ofNat (nat_lit 24), UInt8.ofNat (nat_lit 25), UInt8.ofNat (nat_lit 26), UInt8.ofNat (nat_lit 27), UInt8.ofNat (nat_lit 28), UInt8.ofNat (nat_lit 29), UInt8.ofNat (nat_lit 30), UInt8.ofNat (nat_lit 31)]) ((LexLeanRuntime.subtract (value) (16) : Nat)) (1)) else (if (Nat.blt (value) (48)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 32), UInt8.ofNat (nat_lit 33), UInt8.ofNat (nat_lit 34), UInt8.ofNat (nat_lit 35), UInt8.ofNat (nat_lit 36), UInt8.ofNat (nat_lit 37), UInt8.ofNat (nat_lit 38), UInt8.ofNat (nat_lit 39), UInt8.ofNat (nat_lit 40), UInt8.ofNat (nat_lit 41), UInt8.ofNat (nat_lit 42), UInt8.ofNat (nat_lit 43), UInt8.ofNat (nat_lit 44), UInt8.ofNat (nat_lit 45), UInt8.ofNat (nat_lit 46), UInt8.ofNat (nat_lit 47)]) ((LexLeanRuntime.subtract (value) (32) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 48), UInt8.ofNat (nat_lit 49), UInt8.ofNat (nat_lit 50), UInt8.ofNat (nat_lit 51), UInt8.ofNat (nat_lit 52), UInt8.ofNat (nat_lit 53), UInt8.ofNat (nat_lit 54), UInt8.ofNat (nat_lit 55), UInt8.ofNat (nat_lit 56), UInt8.ofNat (nat_lit 57), UInt8.ofNat (nat_lit 58), UInt8.ofNat (nat_lit 59), UInt8.ofNat (nat_lit 60), UInt8.ofNat (nat_lit 61), UInt8.ofNat (nat_lit 62), UInt8.ofNat (nat_lit 63)]) ((LexLeanRuntime.subtract (value) (48) : Nat)) (1))) else (if (Nat.blt (value) (96)) then (if (Nat.blt (value) (80)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 64), UInt8.ofNat (nat_lit 65), UInt8.ofNat (nat_lit 66), UInt8.ofNat (nat_lit 67), UInt8.ofNat (nat_lit 68), UInt8.ofNat (nat_lit 69), UInt8.ofNat (nat_lit 70), UInt8.ofNat (nat_lit 71), UInt8.ofNat (nat_lit 72), UInt8.ofNat (nat_lit 73), UInt8.ofNat (nat_lit 74), UInt8.ofNat (nat_lit 75), UInt8.ofNat (nat_lit 76), UInt8.ofNat (nat_lit 77), UInt8.ofNat (nat_lit 78), UInt8.ofNat (nat_lit 79)]) ((LexLeanRuntime.subtract (value) (64) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 80), UInt8.ofNat (nat_lit 81), UInt8.ofNat (nat_lit 82), UInt8.ofNat (nat_lit 83), UInt8.ofNat (nat_lit 84), UInt8.ofNat (nat_lit 85), UInt8.ofNat (nat_lit 86), UInt8.ofNat (nat_lit 87), UInt8.ofNat (nat_lit 88), UInt8.ofNat (nat_lit 89), UInt8.ofNat (nat_lit 90), UInt8.ofNat (nat_lit 91), UInt8.ofNat (nat_lit 92), UInt8.ofNat (nat_lit 93), UInt8.ofNat (nat_lit 94), UInt8.ofNat (nat_lit 95)]) ((LexLeanRuntime.subtract (value) (80) : Nat)) (1)) else (if (Nat.blt (value) (112)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 96), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 98), UInt8.ofNat (nat_lit 99), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 102), UInt8.ofNat (nat_lit 103), UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 106), UInt8.ofNat (nat_lit 107), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 111)]) ((LexLeanRuntime.subtract (value) (96) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 113), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 118), UInt8.ofNat (nat_lit 119), UInt8.ofNat (nat_lit 120), UInt8.ofNat (nat_lit 121), UInt8.ofNat (nat_lit 122), UInt8.ofNat (nat_lit 123), UInt8.ofNat (nat_lit 124), UInt8.ofNat (nat_lit 125), UInt8.ofNat (nat_lit 126), UInt8.ofNat (nat_lit 127)]) ((LexLeanRuntime.subtract (value) (112) : Nat)) (1)))) else (if (Nat.blt (value) (192)) then (if (Nat.blt (value) (160)) then (if (Nat.blt (value) (144)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 128), UInt8.ofNat (nat_lit 129), UInt8.ofNat (nat_lit 130), UInt8.ofNat (nat_lit 131), UInt8.ofNat (nat_lit 132), UInt8.ofNat (nat_lit 133), UInt8.ofNat (nat_lit 134), UInt8.ofNat (nat_lit 135), UInt8.ofNat (nat_lit 136), UInt8.ofNat (nat_lit 137), UInt8.ofNat (nat_lit 138), UInt8.ofNat (nat_lit 139), UInt8.ofNat (nat_lit 140), UInt8.ofNat (nat_lit 141), UInt8.ofNat (nat_lit 142), UInt8.ofNat (nat_lit 143)]) ((LexLeanRuntime.subtract (value) (128) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 144), UInt8.ofNat (nat_lit 145), UInt8.ofNat (nat_lit 146), UInt8.ofNat (nat_lit 147), UInt8.ofNat (nat_lit 148), UInt8.ofNat (nat_lit 149), UInt8.ofNat (nat_lit 150), UInt8.ofNat (nat_lit 151), UInt8.ofNat (nat_lit 152), UInt8.ofNat (nat_lit 153), UInt8.ofNat (nat_lit 154), UInt8.ofNat (nat_lit 155), UInt8.ofNat (nat_lit 156), UInt8.ofNat (nat_lit 157), UInt8.ofNat (nat_lit 158), UInt8.ofNat (nat_lit 159)]) ((LexLeanRuntime.subtract (value) (144) : Nat)) (1)) else (if (Nat.blt (value) (176)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 160), UInt8.ofNat (nat_lit 161), UInt8.ofNat (nat_lit 162), UInt8.ofNat (nat_lit 163), UInt8.ofNat (nat_lit 164), UInt8.ofNat (nat_lit 165), UInt8.ofNat (nat_lit 166), UInt8.ofNat (nat_lit 167), UInt8.ofNat (nat_lit 168), UInt8.ofNat (nat_lit 169), UInt8.ofNat (nat_lit 170), UInt8.ofNat (nat_lit 171), UInt8.ofNat (nat_lit 172), UInt8.ofNat (nat_lit 173), UInt8.ofNat (nat_lit 174), UInt8.ofNat (nat_lit 175)]) ((LexLeanRuntime.subtract (value) (160) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 176), UInt8.ofNat (nat_lit 177), UInt8.ofNat (nat_lit 178), UInt8.ofNat (nat_lit 179), UInt8.ofNat (nat_lit 180), UInt8.ofNat (nat_lit 181), UInt8.ofNat (nat_lit 182), UInt8.ofNat (nat_lit 183), UInt8.ofNat (nat_lit 184), UInt8.ofNat (nat_lit 185), UInt8.ofNat (nat_lit 186), UInt8.ofNat (nat_lit 187), UInt8.ofNat (nat_lit 188), UInt8.ofNat (nat_lit 189), UInt8.ofNat (nat_lit 190), UInt8.ofNat (nat_lit 191)]) ((LexLeanRuntime.subtract (value) (176) : Nat)) (1))) else (if (Nat.blt (value) (224)) then (if (Nat.blt (value) (208)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 192), UInt8.ofNat (nat_lit 193), UInt8.ofNat (nat_lit 194), UInt8.ofNat (nat_lit 195), UInt8.ofNat (nat_lit 196), UInt8.ofNat (nat_lit 197), UInt8.ofNat (nat_lit 198), UInt8.ofNat (nat_lit 199), UInt8.ofNat (nat_lit 200), UInt8.ofNat (nat_lit 201), UInt8.ofNat (nat_lit 202), UInt8.ofNat (nat_lit 203), UInt8.ofNat (nat_lit 204), UInt8.ofNat (nat_lit 205), UInt8.ofNat (nat_lit 206), UInt8.ofNat (nat_lit 207)]) ((LexLeanRuntime.subtract (value) (192) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 208), UInt8.ofNat (nat_lit 209), UInt8.ofNat (nat_lit 210), UInt8.ofNat (nat_lit 211), UInt8.ofNat (nat_lit 212), UInt8.ofNat (nat_lit 213), UInt8.ofNat (nat_lit 214), UInt8.ofNat (nat_lit 215), UInt8.ofNat (nat_lit 216), UInt8.ofNat (nat_lit 217), UInt8.ofNat (nat_lit 218), UInt8.ofNat (nat_lit 219), UInt8.ofNat (nat_lit 220), UInt8.ofNat (nat_lit 221), UInt8.ofNat (nat_lit 222), UInt8.ofNat (nat_lit 223)]) ((LexLeanRuntime.subtract (value) (208) : Nat)) (1)) else (if (Nat.blt (value) (240)) then wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 224), UInt8.ofNat (nat_lit 225), UInt8.ofNat (nat_lit 226), UInt8.ofNat (nat_lit 227), UInt8.ofNat (nat_lit 228), UInt8.ofNat (nat_lit 229), UInt8.ofNat (nat_lit 230), UInt8.ofNat (nat_lit 231), UInt8.ofNat (nat_lit 232), UInt8.ofNat (nat_lit 233), UInt8.ofNat (nat_lit 234), UInt8.ofNat (nat_lit 235), UInt8.ofNat (nat_lit 236), UInt8.ofNat (nat_lit 237), UInt8.ofNat (nat_lit 238), UInt8.ofNat (nat_lit 239)]) ((LexLeanRuntime.subtract (value) (224) : Nat)) (1) else wireSlice (ByteArray.mk #[UInt8.ofNat (nat_lit 240), UInt8.ofNat (nat_lit 241), UInt8.ofNat (nat_lit 242), UInt8.ofNat (nat_lit 243), UInt8.ofNat (nat_lit 244), UInt8.ofNat (nat_lit 245), UInt8.ofNat (nat_lit 246), UInt8.ofNat (nat_lit 247), UInt8.ofNat (nat_lit 248), UInt8.ofNat (nat_lit 249), UInt8.ofNat (nat_lit 250), UInt8.ofNat (nat_lit 251), UInt8.ofNat (nat_lit 252), UInt8.ofNat (nat_lit 253), UInt8.ofNat (nat_lit 254), UInt8.ofNat (nat_lit 255)]) ((LexLeanRuntime.subtract (value) (240) : Nat)) (1))))) else ByteArray.mk #[])

@[expose] public def wireDecodeOctetSearch : (value : ByteArray) -> (count : Nat) -> Nat
  | _value, Nat.zero => 0
  | value, Nat.succ rest => (if wireBytesEqual (value) (wireEncodeOctet (rest)) then rest else wireDecodeOctetSearch (value) (rest))

@[expose] public def wireDecodeOctet (value : ByteArray) : Nat := wireDecodeOctetSearch (value) (256)

@[expose] public def wireEncodeLe16 (value : Nat) : ByteArray := (if (Nat.ble (value) (65535)) then (LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (1) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (256) (0) : Nat)) (256) (0) : Nat))) : ByteArray) else ByteArray.mk #[])

@[expose] public def wireDecodeLe16 (value : ByteArray) : Nat := ((0 + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (0) (1))) (1) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (1) (1))) (256) : Nat))

@[expose] public def wireEncodeLe32 (value : Nat) : ByteArray := (if (Nat.ble (value) (4294967295)) then (LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (1) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (256) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (65536) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (16777216) (0) : Nat)) (256) (0) : Nat))) : ByteArray) else ByteArray.mk #[])

@[expose] public def wireDecodeLe32 (value : ByteArray) : Nat := ((((0 + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (0) (1))) (1) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (1) (1))) (256) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (2) (1))) (65536) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (3) (1))) (16777216) : Nat))

@[expose] public def wireEncodeLe64 (value : Nat) : ByteArray := (if (Nat.ble (value) (18446744073709551615)) then (LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (1) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (256) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (65536) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (16777216) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (4294967296) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (1099511627776) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (281474976710656) (0) : Nat)) (256) (0) : Nat))) : ByteArray)) (wireEncodeOctet ((LexLeanRuntime.remainder ((LexLeanRuntime.quotient (value) (72057594037927936) (0) : Nat)) (256) (0) : Nat))) : ByteArray) else ByteArray.mk #[])

@[expose] public def wireDecodeLe64 (value : ByteArray) : Nat := ((((((((0 + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (0) (1))) (1) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (1) (1))) (256) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (2) (1))) (65536) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (3) (1))) (16777216) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (4) (1))) (4294967296) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (5) (1))) (1099511627776) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (6) (1))) (281474976710656) : Nat)) + (LexLeanRuntime.multiply (wireDecodeOctet (wireSlice (value) (7) (1))) (72057594037927936) : Nat))

@[expose] public def wireHexOctetValid (value : ByteArray) : Bool := (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 48)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 49)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 50)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 51)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 52)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 53)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 54)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 55)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 56)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 57)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 97)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 98)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 99)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 100)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 101)]) || (wireBytesEqual (value) (ByteArray.mk #[UInt8.ofNat (nat_lit 102)]) || false))))))))))))))))

@[expose] public def wireHexBytesValid : (value : ByteArray) -> (count : Nat) -> Bool
  | _value, Nat.zero => true
  | value, Nat.succ rest => (wireHexOctetValid (wireSlice (value) (rest) (1)) && (wireHexBytesValid (value) (rest) && true))

@[expose] public def wireKappaLabelValid (value : ByteArray) : Bool := ((Nat.beq ((LexLeanRuntime.length (value) : Nat)) (71)) && (wireWindowEquals (value) (0) (7) (ByteArray.mk #[UInt8.ofNat (nat_lit 98), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 107), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 51), UInt8.ofNat (nat_lit 58)]) && (wireHexBytesValid (wireSlice (value) (7) (64)) (64) && true)))

@[expose] public def emptyCapabilities : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (ByteArray.mk #[UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 58), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 103), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 46), UInt8.ofNat (nat_lit 102), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 122), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 99), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 98), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 121), UInt8.ofNat (nat_lit 45), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 116)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 41), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)

@[expose] public def wireManifestPrefix : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (ByteArray.mk #[UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 58), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 103), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 46), UInt8.ofNat (nat_lit 102), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 122), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 45), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 102), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 116)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 3), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)

@[expose] public def wireManifestSuffix : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (ByteArray.mk #[UInt8.ofNat (nat_lit 82), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 2), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 8), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 95), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 110)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 26), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 103), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 58), UInt8.ofNat (nat_lit 103), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 99), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 45), UInt8.ofNat (nat_lit 119), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 64), UInt8.ofNat (nat_lit 49)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 3), UInt8.ofNat (nat_lit 10), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 120), UInt8.ofNat (nat_lit 46), UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 108)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 8), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 98), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 101)]) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) : ByteArray)

@[expose] public def appManifest (requires : ByteArray) (guest : ByteArray) (view : ByteArray) : Option (ByteArray) := (if (wireKappaLabelValid (requires) && (wireKappaLabelValid (guest) && (wireKappaLabelValid (view) && true))) then Option.some ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireManifestPrefix) : ByteArray)) (requires) : ByteArray)) (guest) : ByteArray)) (view) : ByteArray)) (wireManifestSuffix) : ByteArray)) else Option.none)

@[expose] public def validAppManifest (value : ByteArray) : Bool := ((Nat.beq ((LexLeanRuntime.length (value) : Nat)) (356)) && (wireWindowEquals (value) (0) (57) (wireManifestPrefix) && (wireWindowEquals (value) (270) (86) (wireManifestSuffix) && (wireKappaLabelValid (wireSlice (value) (57) (71)) && (wireKappaLabelValid (wireSlice (value) (128) (71)) && (wireKappaLabelValid (wireSlice (value) (199) (71)) && true))))))

@[expose] public def manifestReference (value : ByteArray) (index : Nat) : Option (ByteArray) := (if (validAppManifest (value) && ((Nat.blt (index) (3)) && true)) then Option.some (wireSlice (value) ((57 + (LexLeanRuntime.multiply (index) (71) : Nat))) (71)) else Option.none)

@[expose] public def wireDirectoryPrefix : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireEncodeLe16 (62)) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 58), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 103), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 46), UInt8.ofNat (nat_lit 102), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 120), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 99), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 45), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 99), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 121), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 118), UInt8.ofNat (nat_lit 49)]) : ByteArray)

@[expose] public def wireProvenancePrefix : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireEncodeLe16 (49)) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 104), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 58), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 46), UInt8.ofNat (nat_lit 102), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 117), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 97), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 120), UInt8.ofNat (nat_lit 116), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 110), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 114), UInt8.ofNat (nat_lit 105), UInt8.ofNat (nat_lit 115), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 112), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 45), UInt8.ofNat (nat_lit 109), UInt8.ofNat (nat_lit 111), UInt8.ofNat (nat_lit 100), UInt8.ofNat (nat_lit 101), UInt8.ofNat (nat_lit 108), UInt8.ofNat (nat_lit 47), UInt8.ofNat (nat_lit 118), UInt8.ofNat (nat_lit 49)]) : ByteArray)

@[expose] public def wireSectionKind (index : Nat) : ByteArray := (if (Nat.blt (index) (8)) then (if (Nat.beq (index) (0)) then ByteArray.mk #[UInt8.ofNat (nat_lit 15)] else (if (Nat.beq (index) (1)) then ByteArray.mk #[UInt8.ofNat (nat_lit 8)] else (if (Nat.blt (index) (4)) then ByteArray.mk #[UInt8.ofNat (nat_lit 14)] else ByteArray.mk #[UInt8.ofNat (nat_lit 16)]))) else ByteArray.mk #[])

@[expose] public def wireTableRow (index : Nat) (offset : Nat) (length : Nat) : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireSectionKind (index)) : ByteArray)) (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (wireEncodeLe64 (offset)) : ByteArray)) (wireEncodeLe64 (length)) : ByteArray)

@[expose] public def wireRowOffset (body : ByteArray) (index : Nat) : Nat := (match (LexLeanRuntime.slice (body) ((18 + (LexLeanRuntime.multiply (index) (24) : Nat))) (8) : Option (ByteArray)) with | Option.none => 0 | Option.some part => wireDecodeLe64 (part))

@[expose] public def wireRowLength (body : ByteArray) (index : Nat) : Nat := (match (LexLeanRuntime.slice (body) ((26 + (LexLeanRuntime.multiply (index) (24) : Nat))) (8) : Option (ByteArray)) with | Option.none => 0 | Option.some part => wireDecodeLe64 (part))

@[expose] public def wireSectionBytesUnchecked (body : ByteArray) (index : Nat) : ByteArray := wireSlice (body) (wireRowOffset (body) (index)) (wireRowLength (body) (index))

@[expose] public def wireRowsValid : (body : ByteArray) -> (index : Nat) -> (offset : Nat) -> (remaining : Nat) -> Bool
  | body, _index, offset, Nat.zero => (Nat.beq (offset) ((LexLeanRuntime.length (body) : Nat)))
  | body, index, offset, Nat.succ rest => (if ((Nat.blt (index) (8)) && ((Nat.ble (offset) ((LexLeanRuntime.length (body) : Nat))) && true)) then (if (wireWindowEquals (body) ((10 + (LexLeanRuntime.multiply (index) (24) : Nat))) (1) (wireSectionKind (index)) && (wireWindowEquals (body) ((11 + (LexLeanRuntime.multiply (index) (24) : Nat))) (7) (ByteArray.mk #[UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0)]) && ((Nat.beq (wireRowOffset (body) (index)) (offset)) && ((Nat.ble (wireRowLength (body) (index)) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (body) : Nat)) (offset) : Nat))) && true)))) then wireRowsValid (body) ((index + 1)) ((offset + wireRowLength (body) (index))) (rest) else false) else false)

@[expose] public def wireBlobLabel (value : ByteArray) : ByteArray := wireSlice (value) (0) (71)

@[expose] public def wireBlobContent (value : ByteArray) : ByteArray := wireSlice (value) (71) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (value) : Nat)) (71) : Nat))

@[expose] public def wireBlobValid (value : ByteArray) : Bool := ((Nat.ble (71) ((LexLeanRuntime.length (value) : Nat))) && (wireKappaLabelValid (wireBlobLabel (value)) && true))

@[expose] public def wireBlobsValid (blob0 : ByteArray) (blob1 : ByteArray) (blob2 : ByteArray) (blob3 : ByteArray) : Bool := (wireBlobValid (blob0) && (wireBlobValid (blob1) && (wireBlobValid (blob2) && (wireBlobValid (blob3) && (wireBytesLess (wireBlobLabel (blob0)) (wireBlobLabel (blob1)) && (wireBytesLess (wireBlobLabel (blob1)) (wireBlobLabel (blob2)) && (wireBytesLess (wireBlobLabel (blob2)) (wireBlobLabel (blob3)) && true)))))))

@[expose] public def wireReferencePresent (label : ByteArray) (blob0 : ByteArray) (blob1 : ByteArray) (blob2 : ByteArray) (blob3 : ByteArray) : Bool := (wireBytesEqual (label) (wireBlobLabel (blob0)) || (wireBytesEqual (label) (wireBlobLabel (blob1)) || (wireBytesEqual (label) (wireBlobLabel (blob2)) || (wireBytesEqual (label) (wireBlobLabel (blob3)) || false))))

@[expose] public def wireCapabilitiesPresent (label : ByteArray) (blob0 : ByteArray) (blob1 : ByteArray) (blob2 : ByteArray) (blob3 : ByteArray) : Bool := ((wireBytesEqual (label) (wireBlobLabel (blob0)) && (wireBytesEqual (wireBlobContent (blob0)) (emptyCapabilities) && true)) || ((wireBytesEqual (label) (wireBlobLabel (blob1)) && (wireBytesEqual (wireBlobContent (blob1)) (emptyCapabilities) && true)) || ((wireBytesEqual (label) (wireBlobLabel (blob2)) && (wireBytesEqual (wireBlobContent (blob2)) (emptyCapabilities) && true)) || ((wireBytesEqual (label) (wireBlobLabel (blob3)) && (wireBytesEqual (wireBlobContent (blob3)) (emptyCapabilities) && true)) || false))))

@[expose] public def wirePayloadsValid (manifest : ByteArray) (blob0 : ByteArray) (blob1 : ByteArray) (blob2 : ByteArray) (blob3 : ByteArray) : Bool := (validAppManifest (manifest) && (wireBlobsValid (blob0) (blob1) (blob2) (blob3) && (wireCapabilitiesPresent (wireSlice (manifest) (57) (71)) (blob0) (blob1) (blob2) (blob3) && (wireReferencePresent (wireSlice (manifest) (128) (71)) (blob0) (blob1) (blob2) (blob3) && (wireReferencePresent (wireSlice (manifest) (199) (71)) (blob0) (blob1) (blob2) (blob3) && true)))))

@[expose] public def wireBodyPayloadsValid (body : ByteArray) : Bool := (wireWindowEquals (wireSectionBytesUnchecked (body) (2)) (0) ((LexLeanRuntime.length (wireDirectoryPrefix) : Nat)) (wireDirectoryPrefix) && (wireWindowEquals (wireSectionBytesUnchecked (body) (3)) (0) ((LexLeanRuntime.length (wireProvenancePrefix) : Nat)) (wireProvenancePrefix) && (wirePayloadsValid (wireSectionBytesUnchecked (body) (0)) (wireSectionBytesUnchecked (body) (4)) (wireSectionBytesUnchecked (body) (5)) (wireSectionBytesUnchecked (body) (6)) (wireSectionBytesUnchecked (body) (7)) && true)))

@[expose] public def validArchiveBody (body : ByteArray) : Bool := (if ((Nat.ble (202) ((LexLeanRuntime.length (body) : Nat))) && (wireWindowEquals (body) (0) (10) (ByteArray.mk #[UInt8.ofNat (nat_lit 72), UInt8.ofNat (nat_lit 79), UInt8.ofNat (nat_lit 76), UInt8.ofNat (nat_lit 79), UInt8.ofNat (nat_lit 4), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 8), UInt8.ofNat (nat_lit 0)]) && true)) then (if wireRowsValid (body) (0) (202) (8) then wireBodyPayloadsValid (body) else false) else false)

@[expose] public def wireComposeBodyUnchecked (manifest : ByteArray) (metadata : ByteArray) (directory : ByteArray) (provenance : ByteArray) (blob0 : ByteArray) (blob1 : ByteArray) (blob2 : ByteArray) (blob3 : ByteArray) : ByteArray := (LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (ByteArray.mk #[UInt8.ofNat (nat_lit 72), UInt8.ofNat (nat_lit 79), UInt8.ofNat (nat_lit 76), UInt8.ofNat (nat_lit 79), UInt8.ofNat (nat_lit 4), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 0), UInt8.ofNat (nat_lit 8), UInt8.ofNat (nat_lit 0)]) : ByteArray)) (wireTableRow (0) (202) ((LexLeanRuntime.length (manifest) : Nat))) : ByteArray)) (wireTableRow (1) ((202 + (LexLeanRuntime.length (manifest) : Nat))) ((LexLeanRuntime.length (metadata) : Nat))) : ByteArray)) (wireTableRow (2) (((202 + (LexLeanRuntime.length (manifest) : Nat)) + (LexLeanRuntime.length (metadata) : Nat))) ((LexLeanRuntime.length (directory) : Nat))) : ByteArray)) (wireTableRow (3) ((((202 + (LexLeanRuntime.length (manifest) : Nat)) + (LexLeanRuntime.length (metadata) : Nat)) + (LexLeanRuntime.length (directory) : Nat))) ((LexLeanRuntime.length (provenance) : Nat))) : ByteArray)) (wireTableRow (4) (((((202 + (LexLeanRuntime.length (manifest) : Nat)) + (LexLeanRuntime.length (metadata) : Nat)) + (LexLeanRuntime.length (directory) : Nat)) + (LexLeanRuntime.length (provenance) : Nat))) ((LexLeanRuntime.length (blob0) : Nat))) : ByteArray)) (wireTableRow (5) ((((((202 + (LexLeanRuntime.length (manifest) : Nat)) + (LexLeanRuntime.length (metadata) : Nat)) + (LexLeanRuntime.length (directory) : Nat)) + (LexLeanRuntime.length (provenance) : Nat)) + (LexLeanRuntime.length (blob0) : Nat))) ((LexLeanRuntime.length (blob1) : Nat))) : ByteArray)) (wireTableRow (6) (((((((202 + (LexLeanRuntime.length (manifest) : Nat)) + (LexLeanRuntime.length (metadata) : Nat)) + (LexLeanRuntime.length (directory) : Nat)) + (LexLeanRuntime.length (provenance) : Nat)) + (LexLeanRuntime.length (blob0) : Nat)) + (LexLeanRuntime.length (blob1) : Nat))) ((LexLeanRuntime.length (blob2) : Nat))) : ByteArray)) (wireTableRow (7) ((((((((202 + (LexLeanRuntime.length (manifest) : Nat)) + (LexLeanRuntime.length (metadata) : Nat)) + (LexLeanRuntime.length (directory) : Nat)) + (LexLeanRuntime.length (provenance) : Nat)) + (LexLeanRuntime.length (blob0) : Nat)) + (LexLeanRuntime.length (blob1) : Nat)) + (LexLeanRuntime.length (blob2) : Nat))) ((LexLeanRuntime.length (blob3) : Nat))) : ByteArray)) (manifest) : ByteArray)) (metadata) : ByteArray)) (directory) : ByteArray)) (provenance) : ByteArray)) (blob0) : ByteArray)) (blob1) : ByteArray)) (blob2) : ByteArray)) (blob3) : ByteArray)

@[expose] public def archiveBody (manifest : ByteArray) (metadata : ByteArray) (directory : ByteArray) (provenance : ByteArray) (blob0 : ByteArray) (blob1 : ByteArray) (blob2 : ByteArray) (blob3 : ByteArray) : Option (ByteArray) := (if wirePayloadsValid (manifest) (blob0) (blob1) (blob2) (blob3) then Option.some (wireComposeBodyUnchecked (manifest) (metadata) ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireDirectoryPrefix) : ByteArray)) (directory) : ByteArray)) ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (wireProvenancePrefix) : ByteArray)) (provenance) : ByteArray)) (blob0) (blob1) (blob2) (blob3)) else Option.none)

@[expose] public def frameArchive (body : ByteArray) (footer : ByteArray) : Option (ByteArray) := (if (validArchiveBody (body) && ((Nat.beq ((LexLeanRuntime.length (footer) : Nat)) (32)) && true)) then Option.some ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (body) : ByteArray)) (footer) : ByteArray)) else Option.none)

@[expose] public def validArchiveFrame (value : ByteArray) : Bool := (if (Nat.ble (234) ((LexLeanRuntime.length (value) : Nat))) then validArchiveBody (wireSlice (value) (0) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (value) : Nat)) (32) : Nat))) else false)

@[expose] public def archiveBodyBytes (value : ByteArray) : Option (ByteArray) := (if validArchiveFrame (value) then Option.some (wireSlice (value) (0) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (value) : Nat)) (32) : Nat))) else Option.none)

@[expose] public def archiveFooter (value : ByteArray) : Option (ByteArray) := (if validArchiveFrame (value) then Option.some (wireSlice (value) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (value) : Nat)) (32) : Nat)) (32)) else Option.none)

@[expose] public def archiveSection (value : ByteArray) (index : Nat) : Option (ByteArray) := (if (validArchiveFrame (value) && ((Nat.blt (index) (8)) && true)) then Option.some (wireSectionBytesUnchecked (value) (index)) else Option.none)

@[expose] public def archiveExtension (value : ByteArray) (index : Nat) : Option (ByteArray) := (if (validArchiveFrame (value) && ((Nat.blt (index) (2)) && true)) then (if (Nat.beq (index) (0)) then Option.some (wireSlice (wireSectionBytesUnchecked (value) ((index + 2))) ((LexLeanRuntime.length (wireDirectoryPrefix) : Nat)) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (wireSectionBytesUnchecked (value) ((index + 2))) : Nat)) ((LexLeanRuntime.length (wireDirectoryPrefix) : Nat)) : Nat))) else Option.some (wireSlice (wireSectionBytesUnchecked (value) ((index + 2))) ((LexLeanRuntime.length (wireProvenancePrefix) : Nat)) ((LexLeanRuntime.subtract ((LexLeanRuntime.length (wireSectionBytesUnchecked (value) ((index + 2))) : Nat)) ((LexLeanRuntime.length (wireProvenancePrefix) : Nat)) : Nat)))) else Option.none)

@[expose] public def contentBlob (label : ByteArray) (content : ByteArray) : Option (ByteArray) := (if wireKappaLabelValid (label) then Option.some ((LexLeanRuntime.append ((LexLeanRuntime.append (ByteArray.mk #[]) (label) : ByteArray)) (content) : ByteArray)) else Option.none)

@[expose] public def contentBlobLabel (value : ByteArray) : Option (ByteArray) := (if wireBlobValid (value) then Option.some (wireBlobLabel (value)) else Option.none)

@[expose] public def contentBlobBytes (value : ByteArray) : Option (ByteArray) := (if wireBlobValid (value) then Option.some (wireBlobContent (value)) else Option.none)

end PrismPM.Foundation.Holo.V1.Wire
