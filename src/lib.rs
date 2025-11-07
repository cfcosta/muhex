#![feature(portable_simd)]

use std::{
    io::{Error, ErrorKind},
    simd::{
        cmp::SimdPartialOrd, simd_swizzle, u8x16, u8x32, u8x64, LaneCount,
        Simd, SupportedLaneCount,
    },
};

type SimdU8<const LANES: usize> = Simd<u8, LANES>;

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

    let interleaved: u8x64 = simd_swizzle!(
        hi_ascii,
        lo_ascii,
        [
            0, 32, 1, 33, 2, 34, 3, 35, 4, 36, 5, 37, 6, 38, 7, 39, 8, 40, 9,
            41, 10, 42, 11, 43, 12, 44, 13, 45, 14, 46, 15, 47, 16, 48, 17, 49,
            18, 50, 19, 51, 20, 52, 21, 53, 22, 54, 23, 55, 24, 56, 25, 57, 26,
            58, 27, 59, 28, 60, 29, 61, 30, 62, 31, 63
        ]
    );
    output.copy_from_slice(interleaved.as_array());
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

    let interleaved: u8x32 = simd_swizzle!(
        hi_ascii,
        lo_ascii,
        [
            0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23, 8, 24, 9,
            25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31
        ]
    );
    output.copy_from_slice(interleaved.as_array());
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

    while pos + 32 <= data.len() {
        encode_simd_32(
            &data[pos..pos + 32],
            &mut result[pos * 2..(pos + 32) * 2],
        );
        pos += 32;
    }

    while pos + 16 <= data.len() {
        encode_simd_16(
            &data[pos..pos + 16],
            &mut result[pos * 2..(pos + 16) * 2],
        );
        pos += 16;
    }

    if pos < data.len() {
        encode_scalar(&data[pos..], &mut result[pos * 2..]);
    }

    unsafe { String::from_utf8_unchecked(result) }
}

#[inline(always)]
fn decode_hex_nibbles<const LANES: usize>(
    n: SimdU8<LANES>,
) -> Result<SimdU8<LANES>, Error>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let zero = SimdU8::<LANES>::splat(b'0');
    let nine = SimdU8::<LANES>::splat(b'9');
    let upper_a = SimdU8::<LANES>::splat(b'A');
    let upper_f = SimdU8::<LANES>::splat(b'F');
    let lower_a = SimdU8::<LANES>::splat(b'a');
    let lower_f = SimdU8::<LANES>::splat(b'f');

    let is_digit = n.simd_ge(zero) & n.simd_le(nine);
    let is_upper = n.simd_ge(upper_a) & n.simd_le(upper_f);
    let is_lower = n.simd_ge(lower_a) & n.simd_le(lower_f);

    let valid = is_digit | is_upper | is_lower;
    if !valid.all() {
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
    let upper_val = n - upper_a + SimdU8::<LANES>::splat(10);
    // For lowercase 'a'..'f', subtract b'a' and add 10.
    let lower_val = n - lower_a + SimdU8::<LANES>::splat(10);

    Ok(is_digit.select(digit_val, is_upper.select(upper_val, lower_val)))
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

    while pos + 64 <= n {
        let chunk_vec: SimdU8<64> = Simd::from_slice(&input[pos..pos + 64]);
        let high_bytes: SimdU8<32> = simd_swizzle!(
            chunk_vec,
            [
                0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
                34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62
            ]
        );
        let low_bytes: SimdU8<32> = simd_swizzle!(
            chunk_vec,
            [
                1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33,
                35, 37, 39, 41, 43, 45, 47, 49, 51, 53, 55, 57, 59, 61, 63
            ]
        );

        let high_nibbles = decode_hex_nibbles::<32>(high_bytes)?;
        let low_nibbles = decode_hex_nibbles::<32>(low_bytes)?;

        let decoded = (high_nibbles << SimdU8::<32>::splat(4)) | low_nibbles;
        output.extend_from_slice(decoded.as_array());
        pos += 64;
    }

    while pos + 32 <= n {
        let chunk_vec: SimdU8<32> = Simd::from_slice(&input[pos..pos + 32]);
        let high_bytes: SimdU8<16> = simd_swizzle!(
            chunk_vec,
            [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]
        );
        let low_bytes: SimdU8<16> = simd_swizzle!(
            chunk_vec,
            [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31]
        );

        let high_nibbles = decode_hex_nibbles::<16>(high_bytes)?;
        let low_nibbles = decode_hex_nibbles::<16>(low_bytes)?;

        let decoded = (high_nibbles << SimdU8::<16>::splat(4)) | low_nibbles;
        output.extend_from_slice(decoded.as_array());
        pos += 32;
    }

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

        let high_nibbles = decode_hex_nibbles::<16>(high_simd)?;
        let low_nibbles = decode_hex_nibbles::<16>(low_simd)?;
        let decoded = (high_nibbles << SimdU8::<16>::splat(4)) | low_nibbles;
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
