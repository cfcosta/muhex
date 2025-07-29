#![feature(portable_simd)]

use std::{
    io::{Error, ErrorKind},
    simd::{Simd, cmp::SimdPartialOrd, u8x16, u8x32},
};

#[cfg(feature = "serde")]
pub mod serde;

#[inline(always)]
fn encode_simd_32(input: &[u8], output: &mut [u8]) {
    let raw: u8x32 = Simd::from_slice(input);

    let high_nibble = raw >> Simd::splat(4);
    let low_nibble = raw & Simd::splat(0x0F);

    let bias_0 = Simd::splat(b'0');
    let bias_a = Simd::splat(b'a' - 10);
    let cmp_9 = Simd::splat(9u8);

    let hi_ascii = nibble_to_ascii_32(high_nibble, bias_0, bias_a, cmp_9);
    let lo_ascii = nibble_to_ascii_32(low_nibble, bias_0, bias_a, cmp_9);

    // Interleave high and low nibbles more efficiently
    let hi_array = hi_ascii.as_array();
    let lo_array = lo_ascii.as_array();

    for i in 0..32 {
        output[i * 2] = hi_array[i];
        output[i * 2 + 1] = lo_array[i];
    }
}

#[inline(always)]
fn encode_simd_16(input: &[u8], output: &mut [u8]) {
    let raw: u8x16 = Simd::from_slice(input);

    let high_nibble = raw >> Simd::splat(4);
    let low_nibble = raw & Simd::splat(0x0F);

    let bias_0 = Simd::splat(b'0');
    let bias_a = Simd::splat(b'a' - 10);
    let cmp_9 = Simd::splat(9u8);

    let hi_ascii = nibble_to_ascii(high_nibble, bias_0, bias_a, cmp_9);
    let lo_ascii = nibble_to_ascii(low_nibble, bias_0, bias_a, cmp_9);

    let hi_array = hi_ascii.as_array();
    let lo_array = lo_ascii.as_array();

    for i in 0..16 {
        output[i * 2] = hi_array[i];
        output[i * 2 + 1] = lo_array[i];
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
fn nibble_to_ascii_32(
    n: u8x32,
    bias_0: u8x32,
    bias_a: u8x32,
    cmp_9: u8x32,
) -> u8x32 {
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

    let mut pos = 0;

    // Process 32-byte chunks first
    while pos + 32 <= data.len() {
        encode_simd_32(
            &data[pos..pos + 32],
            &mut result[pos * 2..(pos + 32) * 2],
        );
        pos += 32;
    }

    // Process 16-byte chunks
    while pos + 16 <= data.len() {
        encode_simd_16(
            &data[pos..pos + 16],
            &mut result[pos * 2..(pos + 16) * 2],
        );
        pos += 16;
    }

    // Handle remainder with scalar code
    if pos < data.len() {
        encode_scalar(&data[pos..], &mut result[pos * 2..]);
    }

    unsafe { String::from_utf8_unchecked(result) }
}

#[inline(always)]
fn decode_hex_nibble_16(n: u8x16) -> Result<u8x16, Error> {
    // Define the boundaries.
    let zero = Simd::splat(b'0');
    let nine = Simd::splat(b'9');
    let upper_a = Simd::splat(b'A');
    let upper_f = Simd::splat(b'F');
    let lower_a = Simd::splat(b'a');
    let lower_f = Simd::splat(b'f');

    // Create masks for each valid range.
    let is_digit = n.simd_ge(zero) & n.simd_le(nine);
    let is_upper = n.simd_ge(upper_a) & n.simd_le(upper_f);
    let is_lower = n.simd_ge(lower_a) & n.simd_le(lower_f);

    // A byte is valid if it is a digit or a letter in either case.
    let valid = is_digit | is_upper | is_lower;
    if !valid.all() {
        // Find and report the first invalid digit.
        for &digit in n.as_array().iter() {
            if !(digit.is_ascii_digit()
                || (b'A'..=b'F').contains(&digit)
                || (b'a'..=b'f').contains(&digit))
            {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("invalid hex digit: {}", digit as char),
                ));
            }
        }
        unreachable!();
    }

    // For digits '0'..'9', subtract b'0'.
    let digit_val = n - zero;
    // For uppercase 'A'..'F', subtract b'A' and add 10.
    let upper_val = n - upper_a + Simd::splat(10);
    // For lowercase 'a'..'f', subtract b'a' and add 10.
    let lower_val = n - lower_a + Simd::splat(10);

    // Combine the values using the masks.
    let result =
        is_digit.select(digit_val, is_upper.select(upper_val, lower_val));

    Ok(result)
}

#[inline(always)]
fn decode_hex_nibble_32(n: u8x32) -> Result<u8x32, Error> {
    // Define the boundaries.
    let zero = Simd::splat(b'0');
    let nine = Simd::splat(b'9');
    let upper_a = Simd::splat(b'A');
    let upper_f = Simd::splat(b'F');
    let lower_a = Simd::splat(b'a');
    let lower_f = Simd::splat(b'f');

    // Create masks for each valid range.
    let is_digit = n.simd_ge(zero) & n.simd_le(nine);
    let is_upper = n.simd_ge(upper_a) & n.simd_le(upper_f);
    let is_lower = n.simd_ge(lower_a) & n.simd_le(lower_f);

    // A byte is valid if it is a digit or a letter in either case.
    let valid = is_digit | is_upper | is_lower;
    if !valid.all() {
        // Find and report the first invalid digit.
        for &digit in n.as_array().iter() {
            if !(digit.is_ascii_digit()
                || (b'A'..=b'F').contains(&digit)
                || (b'a'..=b'f').contains(&digit))
            {
                return Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("invalid hex digit: {}", digit as char),
                ));
            }
        }
        unreachable!();
    }

    // For digits '0'..'9', subtract b'0'.
    let digit_val = n - zero;
    // For uppercase 'A'..'F', subtract b'A' and add 10.
    let upper_val = n - upper_a + Simd::splat(10);
    // For lowercase 'a'..'f', subtract b'a' and add 10.
    let lower_val = n - lower_a + Simd::splat(10);

    // Combine the values using the masks.
    let result =
        is_digit.select(digit_val, is_upper.select(upper_val, lower_val));

    Ok(result)
}

#[inline(always)]
fn nibble_chunk_32(chunk: &[u8]) -> (u8x32, u8x32) {
    debug_assert_eq!(chunk.len(), 64);
    let mut high_bytes = [0u8; 32];
    let mut low_bytes = [0u8; 32];

    // Unroll the loop for better performance
    for i in 0..32 {
        high_bytes[i] = chunk[i * 2];
        low_bytes[i] = chunk[i * 2 + 1];
    }

    (u8x32::from_array(high_bytes), u8x32::from_array(low_bytes))
}

#[inline(always)]
fn nibble_chunk_16(chunk: &[u8]) -> (u8x16, u8x16) {
    debug_assert_eq!(chunk.len(), 32);
    let mut high_bytes = [0u8; 16];
    let mut low_bytes = [0u8; 16];

    for i in 0..16 {
        high_bytes[i] = chunk[i * 2];
        low_bytes[i] = chunk[i * 2 + 1];
    }

    (u8x16::from_array(high_bytes), u8x16::from_array(low_bytes))
}

#[inline]
pub fn decode(input: &str) -> Result<Vec<u8>, Error> {
    let input = input.as_bytes();

    if input.len() % 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "hex string length must be even",
        ));
    }

    let n = input.len();
    let mut output = Vec::with_capacity(n / 2);
    let mut pos = 0;

    // Process 64-byte chunks (32 output bytes)
    while pos + 64 <= n {
        let chunk = &input[pos..pos + 64];
        let (high_bytes, low_bytes) = nibble_chunk_32(chunk);

        let high_nibbles = decode_hex_nibble_32(high_bytes)?;
        let low_nibbles = decode_hex_nibble_32(low_bytes)?;

        let decoded = (high_nibbles << Simd::splat(4)) | low_nibbles;
        output.extend_from_slice(decoded.as_array());
        pos += 64;
    }

    // Process 32-byte chunks (16 output bytes)
    while pos + 32 <= n {
        let chunk = &input[pos..pos + 32];
        let (high_bytes, low_bytes) = nibble_chunk_16(chunk);

        let high_nibbles = decode_hex_nibble_16(high_bytes)?;
        let low_nibbles = decode_hex_nibble_16(low_bytes)?;

        let decoded = (high_nibbles << Simd::splat(4)) | low_nibbles;
        output.extend_from_slice(decoded.as_array());
        pos += 32;
    }

    // Handle remainder
    let remainder = n - pos;
    if remainder > 0 {
        let pairs = remainder / 2;
        let mut high_arr = [b'0'; 16];
        let mut low_arr = [b'0'; 16];
        for j in 0..pairs {
            high_arr[j] = input[pos + j * 2];
            low_arr[j] = input[pos + j * 2 + 1];
        }
        let high_simd = u8x16::from_array(high_arr);
        let low_simd = u8x16::from_array(low_arr);

        let high_nibbles = decode_hex_nibble_16(high_simd)?;
        let low_nibbles = decode_hex_nibble_16(low_simd)?;
        let decoded = (high_nibbles << Simd::splat(4)) | low_nibbles;
        output.extend_from_slice(&decoded.as_array()[..pairs]);
    }

    Ok(output)
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
