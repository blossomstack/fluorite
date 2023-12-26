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
