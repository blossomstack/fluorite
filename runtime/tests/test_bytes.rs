//! The wire contract for `Bytes`: a base64 **string**, not an array of numbers.
//!
//! This is the test the type never had, which is how it shipped disagreeing
//! with every other generator. TypeScript and Go declare a `Bytes` field as
//! `string`, Swift as `Data` (base64 by Codable default) — so a Rust peer that
//! writes `[137,80,78,71]` does not interoperate with any of them.

use std::collections::HashMap;

use fluorite::Bytes;
use serde::{Deserialize, Serialize};

const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
/// `PNG_MAGIC` under standard base64, with padding.
const PNG_MAGIC_B64: &str = "iVBORw0KGgo=";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Holder {
    data: Bytes,
}

#[test]
fn serializes_as_a_base64_string_not_a_number_array() -> anyhow::Result<()> {
    let json = serde_json::to_value(Bytes::from(PNG_MAGIC.to_vec()))?;

    assert!(
        json.is_string(),
        "a Bytes field must cross the wire as a string — TypeScript and Go \
         both declare it `string`. Got: {json}"
    );
    assert_eq!(json.as_str(), Some(PNG_MAGIC_B64));
    Ok(())
}

#[test]
fn round_trips_through_json() -> anyhow::Result<()> {
    let original = Holder {
        data: Bytes::from(PNG_MAGIC.to_vec()),
    };
    let encoded = serde_json::to_string(&original)?;
    assert_eq!(encoded, format!(r#"{{"data":"{PNG_MAGIC_B64}"}}"#));
    assert_eq!(serde_json::from_str::<Holder>(&encoded)?, original);
    Ok(())
}

#[test]
fn deserializes_base64_written_by_another_language() -> anyhow::Result<()> {
    // Exactly what a TypeScript or Go peer emits for these bytes.
    let from_peer: Holder = serde_json::from_str(&format!(r#"{{"data":"{PNG_MAGIC_B64}"}}"#))?;
    assert_eq!(from_peer.data.as_slice(), PNG_MAGIC);
    Ok(())
}

/// The reason `Bytes` is a newtype rather than a `#[serde(with = ...)]`
/// helper: a helper needs one module per shape, and silently misses any shape
/// nobody wrote one for.
#[test]
fn composes_through_option_vec_and_map() -> anyhow::Result<()> {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Nested {
        one: Option<Bytes>,
        none: Option<Bytes>,
        many: Vec<Bytes>,
        keyed: HashMap<String, Bytes>,
    }

    let value = Nested {
        one: Some(Bytes::from(PNG_MAGIC.to_vec())),
        none: None,
        many: vec![Bytes::from(vec![0x00]), Bytes::from(vec![0xFF])],
        keyed: HashMap::from([("k".to_owned(), Bytes::from(vec![0x01, 0x02]))]),
    };

    let json = serde_json::to_value(&value)?;
    assert_eq!(json["one"].as_str(), Some(PNG_MAGIC_B64));
    assert!(json["none"].is_null());
    assert_eq!(json["many"][0].as_str(), Some("AA=="));
    assert_eq!(json["many"][1].as_str(), Some("/w=="));
    assert_eq!(json["keyed"]["k"].as_str(), Some("AQI="));

    assert_eq!(serde_json::from_value::<Nested>(json)?, value);
    Ok(())
}

#[test]
fn empty_bytes_are_an_empty_string() -> anyhow::Result<()> {
    let json = serde_json::to_value(Bytes::default())?;
    assert_eq!(json.as_str(), Some(""));
    assert_eq!(serde_json::from_value::<Bytes>(json)?, Bytes::default());
    Ok(())
}

#[test]
fn invalid_base64_is_an_error_not_a_panic() {
    let err = serde_json::from_str::<Bytes>(r#""not valid base64!""#)
        .expect_err("invalid base64 must be rejected");
    assert!(
        err.to_string().contains("invalid base64"),
        "the error should say what was wrong; got: {err}"
    );
}

#[test]
fn a_number_array_is_rejected() {
    // The old (broken) Rust encoding. Accepting it would let the defect live on
    // in one direction and stay invisible.
    assert!(
        serde_json::from_str::<Bytes>("[137,80,78,71]").is_err(),
        "the pre-fix array encoding must not deserialize"
    );
}

/// horsie derives `schemars::JsonSchema` on every generated type, so `Bytes`
/// needs one — and it must describe the base64 string, not a byte array.
#[cfg(feature = "schemars")]
#[test]
fn json_schema_describes_a_base64_string() -> anyhow::Result<()> {
    let schema = serde_json::to_value(schemars::schema_for!(Bytes))?;
    assert_eq!(schema["type"].as_str(), Some("string"));
    assert_eq!(schema["contentEncoding"].as_str(), Some("base64"));
    Ok(())
}
