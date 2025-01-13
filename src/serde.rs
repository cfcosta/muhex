use serde::{de, Deserializer, Serializer};
use std::fmt;

use crate::{decode, encode};

pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let hex_string = encode(value);
    serializer.serialize_str(&hex_string)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: From<Vec<u8>>,
{
    struct HexVisitor;

    impl de::Visitor<'_> for HexVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a hex-encoded string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            decode(value).map_err(de::Error::custom)
        }
    }

    let bytes = deserializer.deserialize_str(HexVisitor)?;
    Ok(T::from(bytes))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        #[serde(with = "super")]
        data: Vec<u8>,
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = TestStruct {
            data: vec![0x01, 0x02, 0x03, 0xff],
        };

        let serialized = serde_json::to_string(&original).unwrap();
        assert_eq!(serialized, r#"{"data":"010203ff"}"#);

        let deserialized: TestStruct =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }
}
