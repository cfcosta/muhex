#![feature(portable_simd)]

use std::{
    io::{Error, ErrorKind},
    mem::MaybeUninit,
    simd::{
        cmp::SimdPartialOrd, simd_swizzle, u8x16, u8x32, u8x64, LaneCount,
        Simd, SupportedLaneCount,
    },
};

type SimdU8<const LANES: usize> = Simd<u8, LANES>;

#[cfg(feature = "serde")]
pub mod serde;

mod buf;

pub use buf::*;

#[inline(always)]
fn encode_simd_32(input: &[u8], output: &mut [MaybeUninit<u8>]) {
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

    let interleaved: &[u8; 64] = interleaved.as_array();
    // SAFETY: &[u8;64] and &[MaybeUninit<u8>; 64] have the same layout
    let uninit_src: &[MaybeUninit<u8>; 64] =
        unsafe { std::mem::transmute(interleaved) };
    output.copy_from_slice(uninit_src);
}

#[inline(always)]
fn encode_simd_16(input: &[u8], output: &mut [MaybeUninit<u8>]) {
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

    let interleaved: &[u8; 32] = interleaved.as_array();
    // SAFETY: &[u8; 32] and &[MaybeUninit<u8>; 32] have the same layout
    let uninit_src: &[MaybeUninit<u8>; 32] =
        unsafe { std::mem::transmute(interleaved) };
    output.copy_from_slice(uninit_src);
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
fn encode_scalar(data: &[u8], result: &mut [MaybeUninit<u8>]) {
    for (i, byte) in data.iter().enumerate() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0xf) as usize;

        result[i * 2]
            .write(b'0' + hi as u8 + ((hi >= 10) as u8) * (b'a' - b'0' - 10));
        result[i * 2 + 1]
            .write(b'0' + lo as u8 + ((lo >= 10) as u8) * (b'a' - b'0' - 10));
    }
}

#[inline]
pub fn encode<T: AsRef<[u8]>>(v: T) -> String {
    let data = v.as_ref();
    let mut result = Vec::with_capacity(data.len() * 2);
    encode_to_buf(data, result.spare_capacity_mut())
        .expect("Len of result is always correct");
    unsafe {
        result.set_len(data.len() * 2);
    }
    unsafe { String::from_utf8_unchecked(result) }
}

#[inline]
pub fn encode_to_buf<T, Dst>(v: T, dst: &mut Dst) -> Result<(), Error>
where
    T: AsRef<[u8]>,
    Dst: Buf + ?Sized,
{
    let data = v.as_ref();
    let expected_len = data.len() * 2;
    // SAFETY: We only write fully initialized bytes through encode_simd_* and encode_scalar
    let dst = unsafe { dst.dst() };
    if dst.len() != expected_len {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "output slice has wrong length: expected {}, got {}",
                expected_len,
                dst.len()
            ),
        ));
    }

    let mut pos = 0;

    while pos + 32 <= data.len() {
        encode_simd_32(&data[pos..pos + 32], &mut dst[pos * 2..(pos + 32) * 2]);
        pos += 32;
    }

    while pos + 16 <= data.len() {
        encode_simd_16(&data[pos..pos + 16], &mut dst[pos * 2..(pos + 16) * 2]);
        pos += 16;
    }

    if pos < data.len() {
        encode_scalar(&data[pos..], &mut dst[pos * 2..]);
    }

    Ok(())
}

#[inline]
pub fn decode_to_buf<Dst>(input: &str, output: &mut Dst) -> Result<(), Error>
where
    Dst: Buf + ?Sized,
{
    let input = input.as_bytes();
    let expected_len = required_output_len(input.len())?;
    // SAFETY: We only write fully initialized bytes through decode_into
    let output = unsafe { output.dst() };
    if output.len() != expected_len {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "output slice has wrong length: expected {}, got {}",
                expected_len,
                output.len()
            ),
        ));
    }

    decode_into(input, output)
}

#[inline]
pub fn decode_to_slice(input: &str, output: &mut [u8]) -> Result<(), Error> {
    decode_to_buf(input, output)
}

#[inline(always)]
fn invalid_hex_char_error() -> Error {
    Error::from(ErrorKind::InvalidData)
}

#[inline]
pub fn decode(input: &str) -> Result<Vec<u8>, Error> {
    let input = input.as_bytes();
    let n = input.len();

    if n % 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "input length must be even",
        ));
    }

    let mut output = Vec::with_capacity(n / 2);
    decode_into(input, output.spare_capacity_mut())?;
    unsafe { output.set_len(n / 2) };
    Ok(output)
}

#[inline(always)]
fn required_output_len(input_len: usize) -> Result<usize, Error> {
    if input_len % 2 != 0 {
        Err(Error::new(
            ErrorKind::InvalidInput,
            "hex string length must be even",
        ))
    } else {
        Ok(input_len / 2)
    }
}

const HEX_DECODE_LUT: [u8; 256] = {
    let mut lut = [255u8; 256]; // 255 = invalid
    let mut i = 0;
    while i < 256 {
        lut[i] = match i as u8 {
            b'0'..=b'9' => i as u8 - b'0',
            b'A'..=b'F' => i as u8 - b'A' + 10,
            b'a'..=b'f' => i as u8 - b'a' + 10,
            _ => 255,
        };
        i += 1;
    }
    lut
};

fn decode_into(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<(), Error> {
    let n = input.len();

    if n % 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "input length must be even",
        ));
    }

    let mut pos = 0;
    let mut out_pos = 0;

    // Process 64 bytes at a time with combined validation + decode
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

        let (high_nibbles, high_valid) = decode_hex_nibbles(high_bytes);
        let (low_nibbles, low_valid) = decode_hex_nibbles(low_bytes);

        if !(high_valid & low_valid) {
            return Err(invalid_hex_char_error());
        }
        let decoded = (high_nibbles << SimdU8::<32>::splat(4)) | low_nibbles;

        let decoded: &[u8; 32] = decoded.as_array();
        // SAFETY: &[u8; 32] and &[MaybeUninit<u8>; 32] have the same layout
        let uninit_src: &[MaybeUninit<u8>; 32] =
            unsafe { std::mem::transmute(decoded) };
        output[out_pos..out_pos + 32].copy_from_slice(uninit_src);

        pos += 64;
        out_pos += 32;
    }

    // Process 32 bytes at a time
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

        let (high_nibbles, high_valid) = decode_hex_nibbles(high_bytes);
        let (low_nibbles, low_valid) = decode_hex_nibbles(low_bytes);

        if !(high_valid & low_valid) {
            return Err(invalid_hex_char_error());
        }

        let decoded = (high_nibbles << SimdU8::<16>::splat(4)) | low_nibbles;
        let decoded: &[u8; 16] = decoded.as_array();
        // SAFETY: &[u8;16] and &[MaybeUninit<u8>; 16] have the same layout
        let uninit_src: &[MaybeUninit<u8>; 16] =
            unsafe { std::mem::transmute(decoded) };
        output[out_pos..out_pos + 16].copy_from_slice(uninit_src);
        pos += 32;
        out_pos += 16;
    }

    let remaining = n - pos;
    decode_remainder_lut(input, output, pos, out_pos, remaining)?;

    Ok(())
}

#[inline(always)]
fn decode_hex_nibbles<const LANES: usize>(
    n: SimdU8<LANES>,
) -> (SimdU8<LANES>, bool)
where
    LaneCount<LANES>: SupportedLaneCount,
{
    // Branchless computation
    let zero = SimdU8::<LANES>::splat(b'0');
    let nine = SimdU8::<LANES>::splat(b'9');
    let gap = SimdU8::<LANES>::splat(b'A' - b'9' - 1);
    let lower_gap = SimdU8::<LANES>::splat(b'a' - b'A');

    // Normalize: convert to 0-15 range assuming valid input
    // '0'-'9' -> 0-9
    // 'A'-'F' -> 10-15 (after subtracting gap)
    // 'a'-'f' -> 10-15 (after subtracting both gaps)

    let mut val = n - zero;

    // If > '9', subtract the gap to 'A'
    let gt_nine = n.simd_gt(nine);
    val = gt_nine.select(val - gap, val);

    // If >= 'a', subtract additional gap
    let ge_lower_a = n.simd_ge(SimdU8::<LANES>::splat(b'a'));
    val = ge_lower_a.select(val - lower_gap, val);

    // Validation: check if result is in valid range [0, 15]
    let valid = val.simd_le(SimdU8::<LANES>::splat(15));

    (val, valid.all())
}

#[cfg(feature = "test-util")]
#[inline(always)]
pub fn decode_remainder_simd_bench(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
    pos: usize,
    out_pos: usize,
    remaining: usize,
) -> Result<(), Error> {
    decode_remainder_simd(input, output, pos, out_pos, remaining)
}

#[allow(dead_code)]
#[inline(always)]
fn decode_remainder_simd(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
    pos: usize,
    out_pos: usize,
    remaining: usize,
) -> Result<(), Error> {
    let pairs = remaining / 2;

    // Use 16-byte SIMD (works on both x86 SSE and ARM NEON)
    let mut high_arr = [b'0'; 16];
    let mut low_arr = [b'0'; 16];

    // Unrolled for better performance
    let pairs_to_process = pairs.min(16);
    for j in 0..pairs_to_process {
        unsafe {
            high_arr[j] = *input.get_unchecked(pos + j * 2);
            low_arr[j] = *input.get_unchecked(pos + j * 2 + 1);
        }
    }

    let high_simd = Simd::<u8, 16>::from_array(high_arr);
    let low_simd = Simd::<u8, 16>::from_array(low_arr);

    let (high_nibbles, high_valid) = decode_hex_nibbles(high_simd);
    let (low_nibbles, low_valid) = decode_hex_nibbles(low_simd);

    if !(high_valid & low_valid) {
        return Err(Error::from(ErrorKind::InvalidData));
    }

    let decoded = (high_nibbles << Simd::splat(4)) | low_nibbles;
    let decoded: &[u8; 16] = decoded.as_array();
    let uninit_src: &[MaybeUninit<u8>; 16] =
        unsafe { std::mem::transmute(decoded) };

    output[out_pos..out_pos + pairs_to_process]
        .copy_from_slice(&uninit_src[..pairs_to_process]);

    Ok(())
}

#[cfg(feature = "test-util")]
#[inline(always)]
pub fn decode_remainder_lut_bench(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
    pos: usize,
    out_pos: usize,
    remaining: usize,
) -> Result<(), Error> {
    decode_remainder_lut(input, output, pos, out_pos, remaining)
}

#[inline(always)]
fn decode_remainder_lut(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
    mut pos: usize,
    mut out_pos: usize,
    remaining: usize,
) -> Result<(), Error> {
    let end = pos + remaining;

    while pos < end {
        let hi = HEX_DECODE_LUT[input[pos] as usize];
        let lo = HEX_DECODE_LUT[input[pos + 1] as usize];

        if (hi | lo) == 255 {
            return Err(Error::from(ErrorKind::InvalidData));
        }

        output[out_pos].write((hi << 4) | lo);
        pos += 2;
        out_pos += 1;
    }

    Ok(())
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
    fn test_decode_to_slice_roundtrip(input: Vec<u8>) {
        let encoded = hex::encode(&input);
        let mut buffer = vec![0u8; input.len()];
        prop_assert!(super::decode_to_slice(&encoded, &mut buffer).is_ok());
        prop_assert_eq!(buffer, input);
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

    #[test]
    fn test_decode_to_slice_len_mismatch() {
        let mut buffer = [0u8; 1];
        let err =
            super::decode_to_slice("00", &mut buffer[..0]).expect_err("err");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}
