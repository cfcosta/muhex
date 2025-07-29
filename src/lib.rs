#![feature(portable_simd)]

use std::{
    io::{Error, ErrorKind},
    simd::{cmp::SimdPartialOrd, u8x16, u8x32, Simd},
};

#[cfg(feature = "serde")]
pub mod serde;

const SIMD_CHUNK_SIZE: usize = 16;
const SIMD_DECODE_CHUNK_SIZE: usize = 32;

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

#[inline(always)]
fn decode_hex_nibble(n: u8x16) -> Result<u8x16, Error> {
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
            if !((digit >= b'0' && digit <= b'9')
                || (digit >= b'A' && digit <= b'F')
                || (digit >= b'a' && digit <= b'f'))
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

#[inline]
fn nibble_chunck(chunk: &[u8]) -> (u8x16, u8x16) {
    let parsed_chunk = u8x32::from_slice(chunk);
    let mut high_bytes_vec: Vec<u8> = vec![];
    let mut low_bytes_vec: Vec<u8> = vec![];
    for (index, piece) in parsed_chunk.to_array().iter().enumerate() {
        if index % 2 == 0 {
            high_bytes_vec.push(piece.clone())
        } else {
            low_bytes_vec.push(piece.clone())
        }
    }
    (
        u8x16::from_slice(&high_bytes_vec),
        u8x16::from_slice(&low_bytes_vec),
    )
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

    // Process full chunks of 32 input bytes (16 output bytes)
    let chunks = n / SIMD_DECODE_CHUNK_SIZE;
    for i in 0..chunks {
        let chunk = &input
            [i * SIMD_DECODE_CHUNK_SIZE..(i + 1) * SIMD_DECODE_CHUNK_SIZE];
        let (high_bytes, low_bytes) = nibble_chunck(chunk);

        let high_nibbles = decode_hex_nibble(high_bytes)?;
        let low_nibbles = decode_hex_nibble(low_bytes)?;

        let decoded = (high_nibbles << Simd::splat(4)) | low_nibbles;
        output.extend_from_slice(decoded.as_array());
    }

    let remainder = n % SIMD_DECODE_CHUNK_SIZE;
    if remainder > 0 {
        let pairs = remainder / 2;
        let start = chunks * SIMD_DECODE_CHUNK_SIZE;
        let mut high_arr = [b'0'; 16];
        let mut low_arr = [b'0'; 16];
        for j in 0..pairs {
            high_arr[j] = input[start + j * 2];
            low_arr[j] = input[start + j * 2 + 1];
        }
        let high_simd = u8x16::from_array(high_arr);
        let low_simd = u8x16::from_array(low_arr);

        let high_nibbles = decode_hex_nibble(high_simd)?;
        let low_nibbles = decode_hex_nibble(low_simd)?;
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
