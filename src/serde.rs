use std::fmt;

use serde::{Deserializer, Serializer, de};

use crate::{decode, encode};

pub trait FromBytes {
    type Error: fmt::Display;
    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl FromBytes for Vec<u8> {
    type Error = std::convert::Infallible;

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(bytes)
    }
}

impl<const N: usize> FromBytes for [u8; N] {
    type Error = String;

    fn from_bytes(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        bytes
            .try_into()
            .map_err(|v: Vec<u8>| format!("expected array of length {}, got length {}", N, v.len()))
    }
}

#[inline(always)]
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let hex_string = encode(value);
    serializer.serialize_str(&hex_string)
}

#[inline(always)]
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromBytes,
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
    T::from_bytes(bytes).map_err(de::Error::custom)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        #[serde(with = "crate::serde")]
        data: Vec<u8>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStructFixedWidth(#[serde(with = "crate::serde")] [u8; 32]);

    #[test_strategy::proptest]
    fn test_serde_roundtrip(data: Vec<u8>) {
        let test_struct = TestStruct { data };
        let serialized = serde_json::to_string(&test_struct).unwrap();
        let deserialized: TestStruct = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(test_struct, deserialized);
    }

    #[test_strategy::proptest]
    fn test_serde_roundtrip_fixed_width(data: [u8; 32]) {
        let test_struct = TestStructFixedWidth(data);
        let serialized = serde_json::to_string(&test_struct).unwrap();
        let deserialized: TestStructFixedWidth = serde_json::from_str(&serialized).unwrap();
        prop_assert_eq!(test_struct, deserialized);
    }

    #[test_strategy::proptest]
    fn test_serialize_parity(data: Vec<u8>) {
        let mut serializer = serde_json::Serializer::new(Vec::new());

        prop_assert_eq!(
            super::serialize(&data, &mut serializer).map_err(|_| ()),
            hex::serde::serialize(&data, &mut serde_json::Serializer::new(Vec::new()))
                .map_err(|_| ())
        );
    }

    #[test_strategy::proptest]
    fn test_deserialize_parity(data: Vec<u8>) {
        let hex_json = serde_json::to_string(&hex::encode(&data)).unwrap();

        let mut our_de = serde_json::Deserializer::from_str(&hex_json);
        let mut hex_de = serde_json::Deserializer::from_str(&hex_json);

        let our_result: Result<Vec<u8>, _> = super::deserialize(&mut our_de).map_err(|_| ());
        let hex_result: Result<Vec<u8>, _> = hex::serde::deserialize(&mut hex_de).map_err(|_| ());

        prop_assert_eq!(our_result, hex_result);
    }
}
