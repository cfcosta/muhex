use std::io::{Error, ErrorKind};

const LOOKUP: [u8; 16] = *b"0123456789abcdef";

pub fn encode<T: AsRef<[u8]>>(v: T) -> String {
    let data = v.as_ref();
    let mut result = Vec::with_capacity(data.len() * 2);

    for byte in data.iter() {
        result.push(LOOKUP[(byte >> 4) as usize]);
        result.push(LOOKUP[(byte & 0xf) as usize]);
    }

    // Safe because we only used valid ASCII hex digits
    unsafe { String::from_utf8_unchecked(result) }
}

pub fn decode(input: &str) -> Result<Vec<u8>, Error> {
    if input.len() % 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "hex string length must be even",
        ));
    }

    let input = input.as_bytes();
    let mut bytes = Vec::with_capacity(input.len() / 2);

    for chunk in input.chunks_exact(2) {
        let hi = from_hex_digit(chunk[0])?;
        let lo = from_hex_digit(chunk[1])?;
        bytes.push((hi << 4) | lo);
    }

    Ok(bytes)
}

fn from_hex_digit(digit: u8) -> Result<u8, Error> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid hex digit: {}", digit as char),
        )),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    #[test_strategy::proptest(cases = 10000)]
    fn test_hex_parity(input: Vec<u8>) {
        prop_assert_eq!(super::encode(&input), hex::encode(input))
    }

    #[test_strategy::proptest(cases = 10000)]
    fn test_hex_roundtrip(input: Vec<u8>) {
        prop_assert_eq!(super::decode(&super::encode(&input)).unwrap(), input)
    }

    #[test_strategy::proptest(cases = 10000)]
    fn test_hex_roundtrip_parity(input: Vec<u8>) {
        prop_assert_eq!(
            super::decode(&super::encode(&input)).unwrap(),
            hex::decode(hex::encode(input)).unwrap()
        )
    }
}
