#![feature(portable_simd)]

use std::{
    io::{Error, ErrorKind},
    simd::{cmp::SimdPartialOrd, u8x16, Simd},
};

const SIMD_CHUNK_SIZE: usize = 16;

#[inline(always)]
fn encode_simd(input: &[u8], output: &mut [u8]) {
    let raw: u8x16 = Simd::from_slice(input);

    let high_nibble = raw >> Simd::splat(4);
    let low_nibble = raw & Simd::splat(0x0F);

    let bias_0 = Simd::splat(b'0');
    let bias_a = Simd::splat(b'a' - 10);
    let cmp_9 = Simd::splat(9u8);

    let hi_ascii = nibble_to_ascii(high_nibble, bias_0, bias_a, cmp_9);
    let lo_ascii = nibble_to_ascii(low_nibble, bias_0, bias_a, cmp_9);

    for i in 0..16 {
        output[i * 2] = hi_ascii[i];
        output[i * 2 + 1] = lo_ascii[i];
    }
}

#[inline(always)]
fn nibble_to_ascii(
    n: u8x16,
    bias_0: u8x16,
    bias_a: u8x16,
    cmp_9: u8x16,
) -> u8x16 {
    let mask_gt_9 = n.simd_gt(cmp_9);
    let base_0 = n + bias_0;
    let base_a = n + bias_a;

    mask_gt_9.select(base_a, base_0)
}

#[inline(always)]
fn encode_scalar(data: &[u8], result: &mut [u8]) {
    for (i, byte) in data.iter().enumerate() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0xf) as usize;

        result[i * 2] =
            b'0' + hi as u8 + ((hi >= 10) as u8) * (b'a' - b'0' - 10);
        result[i * 2 + 1] =
            b'0' + lo as u8 + ((lo >= 10) as u8) * (b'a' - b'0' - 10);
    }
}

#[inline]
pub fn encode<T: AsRef<[u8]>>(v: T) -> String {
    let data = v.as_ref();
    let mut result = vec![0; data.len() * 2];

    let chunks = data.len() / SIMD_CHUNK_SIZE;
    let remainder = data.len() % SIMD_CHUNK_SIZE;

    if data.len() >= SIMD_CHUNK_SIZE {
        for i in 0..chunks {
            let start = i * SIMD_CHUNK_SIZE;
            let end = start + SIMD_CHUNK_SIZE;
            encode_simd(
                &data[start..end],
                &mut result[start * 2..(start + SIMD_CHUNK_SIZE) * 2],
            );
        }
    }

    // Handle remainder with scalar code
    if remainder > 0 {
        let start = chunks * SIMD_CHUNK_SIZE;
        let end = start + remainder;
        encode_scalar(&data[start..end], &mut result[start * 2..]);
    }

    // If input was too small for SIMD, use scalar
    if chunks == 0 {
        encode_scalar(data, &mut result);
    }

    unsafe { String::from_utf8_unchecked(result) }
}

#[inline]
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

#[inline(always)]
fn from_hex_digit(digit: u8) -> Result<u8, Error> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        b'A'..=b'F' => Ok(digit - b'A' + 10),
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
    fn test_hex_encode_parity(input: Vec<u8>) {
        prop_assert_eq!(super::encode(&input), hex::encode(input))
    }

    #[test_strategy::proptest(cases = 10000)]
    fn test_hex_decode_parity(input: String) {
        prop_assert_eq!(
            super::decode(&input).map_err(|_| ()),
            hex::decode(input).map_err(|_| ())
        )
    }

    #[test_strategy::proptest(cases = 10000)]
    fn test_hex_roundtrip(input: Vec<u8>) {
        prop_assert_eq!(super::decode(&super::encode(&input))?, input)
    }

    #[test_strategy::proptest(cases = 10000)]
    fn test_hex_roundtrip_parity(input: Vec<u8>) {
        prop_assert_eq!(
            super::decode(&super::encode(&input)).is_ok(),
            hex::decode(hex::encode(input)).is_ok()
        )
    }
}
