#![allow(dead_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![deny(clippy::wildcard_enum_match_arm)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        deprecated
    )
)]

use std::collections::HashMap;

use serde::{
    de::{Error, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Any {
    String(String),
    Bool(bool),
    UInt32(u32),
    UInt64(u64),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    List(Vec<Any>),
    Map(HashMap<String, Any>),
}

impl Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Any::String(s) => serializer.serialize_str(s),
            Any::Bool(b) => serializer.serialize_bool(*b),
            Any::UInt32(u) => serializer.serialize_u32(*u),
            Any::UInt64(u) => serializer.serialize_u64(*u),
            Any::Int32(i) => serializer.serialize_i32(*i),
            Any::Int64(i) => serializer.serialize_i64(*i),
            Any::Float32(f) => serializer.serialize_f32(*f),
            Any::Float64(f) => serializer.serialize_f64(*f),
            Any::List(l) => {
                let mut s = serializer.serialize_seq(Some(l.len()))?;
                for ele in l.iter() {
                    s.serialize_element(ele)?;
                }
                s.end()
            }
            Any::Map(m) => {
                let mut s = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m.iter() {
                    s.serialize_entry(k, v)?;
                }
                s.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Any {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(AnyVisitor {})
    }
}

struct AnyVisitor {}

impl<'de> Visitor<'de> for AnyVisitor {
    type Value = Any;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Expecting a valid Any type.")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_i32(v as i32)
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_i32(v as i32)
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::Int32(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::Int64(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_u32(v as u32)
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_u32(v as u32)
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::UInt32(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::UInt64(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::Float32(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::Float64(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(v.encode_utf8(&mut [0u8; 4]))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::String(v.to_string()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(v)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Any::String(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Some(e) = seq.next_element()? {
            vec.push(e);
        }
        Ok(Any::List(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut m = HashMap::new();

        while let Some((k, v)) = map.next_entry()? {
            m.insert(k, v);
        }
        Ok(Any::Map(m))
    }
}

/// A byte string that crosses the wire as base64.
///
/// A newtype rather than a bare `Vec<u8>` because serde writes `Vec<u8>` as a
/// JSON array of numbers, and every other fluorite generator says a `Bytes`
/// field is a base64 string: TypeScript and Go emit `string`, Swift emits
/// `Data`. A Rust peer emitting `[137,80,78,71]` where a TypeScript peer reads
/// `"iVBORw=="` does not interoperate, and the array form is about three times
/// the size.
///
/// The newtype also composes: `Option<Bytes>`, `Vec<Bytes>` and
/// `HashMap<String, Bytes>` all encode correctly with no serde attribute at
/// the field, which a `#[serde(with = ...)]` helper could only manage with one
/// module per shape. It follows the same pattern as the other primitives that
/// map to a dedicated type — `uuid::Uuid`, `rust_decimal::Decimal`,
/// `url::Url`.
///
/// Standard base64 *with* padding, matching Go's `encoding/json` for `[]byte`
/// and Swift's `Data.base64EncodedString()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<&[u8]> for Bytes {
    fn from(v: &[u8]) -> Self {
        Self(v.to_vec())
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(b: Bytes) -> Self {
        b.0
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use base64::Engine as _;
        serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(&self.0))
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use base64::Engine as _;
        let s = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(s.as_bytes())
            .map(Bytes)
            .map_err(|e| D::Error::custom(format!("invalid base64: {e}")))
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for Bytes {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Bytes".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "contentEncoding": "base64",
        })
    }
}
