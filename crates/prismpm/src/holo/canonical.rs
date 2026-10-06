//! Canonical JSON encode, strict decode, validation, and content identity.

use super::model_document::ModelDocument;
use crate::error::PrismError;
use serde::de::{Deserialize, DeserializeSeed, Deserializer, Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fmt;

struct UniqueValue<const FLOATS: bool = false>(Value);

impl<'de, const FLOATS: bool> Deserialize<'de> for UniqueValue<FLOATS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        UniqueSeed::<FLOATS> { depth: 0 }
            .deserialize(deserializer)
            .map(Self)
    }
}

struct UniqueSeed<const FLOATS: bool> {
    depth: usize,
}

impl<'de, const FLOATS: bool> DeserializeSeed<'de> for UniqueSeed<FLOATS> {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        if FLOATS && self.depth > 64 {
            return Err(D::Error::custom("OCI JSON nesting exceeds bound"));
        }
        deserializer.deserialize_any(UniqueVisitor::<FLOATS> { depth: self.depth })
    }
}

struct UniqueVisitor<const FLOATS: bool> {
    depth: usize,
}

impl<'de, const FLOATS: bool> Visitor<'de> for UniqueVisitor<FLOATS> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(if FLOATS {
            "JSON without duplicate keys"
        } else {
            "canonical JSON without duplicate keys or floats"
        })
    }

    fn visit_bool<E: Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<Self::Value, E> {
        if FLOATS {
            Number::from_f64(value)
                .map(Value::Number)
                .ok_or_else(|| E::custom("nonfinite JSON number"))
        } else {
            Err(E::custom("floating-point values are forbidden"))
        }
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E: Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E: Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(UniqueSeed::<FLOATS> {
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(A::Error::custom(format!("duplicate object key {key}")));
            }
            values.insert(
                key,
                map.next_value_seed(UniqueSeed::<FLOATS> {
                    depth: self.depth + 1,
                })?,
            );
        }
        Ok(Value::Object(values))
    }
}

fn write_value(value: &Value, out: &mut Vec<u8>) -> Result<(), PrismError> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Number(number) if number.is_i64() || number.is_u64() => {
            out.extend_from_slice(number.to_string().as_bytes());
        }
        Value::Number(_) => return Err(PrismError::new("PP4004", "floats are forbidden")),
        Value::String(text) => out.extend_from_slice(
            serde_json::to_string(text)
                .map_err(|error| PrismError::new("PP9001", error.to_string()))?
                .as_bytes(),
        ),
        Value::Array(values) => {
            out.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                write_value(value, out)?;
            }
            out.push(b']');
        }
        Value::Object(values) => {
            out.push(b'{');
            let mut rows: Vec<_> = values.iter().collect();
            rows.sort_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
            for (index, (key, value)) in rows.into_iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                out.extend_from_slice(
                    serde_json::to_string(key)
                        .map_err(|error| PrismError::new("PP9001", error.to_string()))?
                        .as_bytes(),
                );
                out.push(b':');
                write_value(value, out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

/// Encode an already-validated JSON value with Prism's canonical JSON rules.
pub fn encode_value(value: &Value) -> Result<Vec<u8>, PrismError> {
    let mut out = Vec::new();
    write_value(value, &mut out)?;
    Ok(out)
}

/// Encode a validated model document with sorted ASCII object keys and no final LF.
pub fn encode_canonical(doc: &ModelDocument) -> Result<Vec<u8>, PrismError> {
    super::validate::validate(doc)?;
    let value =
        serde_json::to_value(doc).map_err(|error| PrismError::new("PP9001", error.to_string()))?;
    encode_value(&value)
}

/// Strictly decode, validate, and require the input bytes to be canonical.
pub fn decode_canonical(bytes: &[u8]) -> Result<ModelDocument, PrismError> {
    let value = decode_value(bytes, "model-document")?;
    let doc: ModelDocument = serde_json::from_value(value).map_err(|error| {
        PrismError::new("PP4004", format!("invalid model-document shape: {error}"))
    })?;
    super::validate::validate(&doc)?;
    if encode_canonical(&doc)? != bytes {
        return Err(PrismError::new(
            "PP4004",
            "model-document fields do not have their canonical typed shape",
        ));
    }
    Ok(doc)
}

// OCI hashes original bytes and permits insignificant JSON whitespace. Share
// duplicate detection without imposing Prism's canonical encoding on that data.
pub(crate) fn decode_json_unique(bytes: &[u8]) -> Result<Value, PrismError> {
    let fail = |error| PrismError::new("PP4004", format!("malformed OCI JSON: {error}"));
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let value = UniqueValue::<true>::deserialize(&mut decoder)
        .map_err(fail)?
        .0;
    decoder.end().map_err(fail)?;
    Ok(value)
}

/// Strictly decode one canonical JSON value, rejecting duplicates, floats,
/// trailing bytes, noncanonical member order, and nonminimal serialization.
pub fn decode_value(bytes: &[u8], description: &str) -> Result<Value, PrismError> {
    let mut stream = serde_json::Deserializer::from_slice(bytes).into_iter::<UniqueValue>();
    let unique = stream
        .next()
        .ok_or_else(|| PrismError::new("PP4004", format!("{description} JSON is empty")))?
        .map_err(|error| {
            let message = error.to_string();
            if message.contains("floating-point values are forbidden") {
                PrismError::new("PP4004", message)
            } else {
                PrismError::new("PP4004", format!("malformed {description} JSON: {message}"))
            }
        })?;
    if stream.byte_offset() != bytes.len() {
        return Err(PrismError::new(
            "PP4004",
            format!("bytes follow the canonical {description} JSON value"),
        ));
    }
    let canonical = encode_value(&unique.0)?;
    if canonical != bytes {
        return Err(
            PrismError::new("PP4004", format!("{description} bytes are not canonical"))
                .with_note(String::from_utf8(canonical).expect("canonical JSON is UTF-8")),
        );
    }
    Ok(unique.0)
}

/// Compute the SHA-256 identity of canonical model-document bytes.
#[must_use]
pub fn content_id(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
