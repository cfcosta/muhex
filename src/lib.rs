#![feature(portable_simd)]
#![cfg_attr(target_arch = "x86_64", feature(stdarch_x86_avx512))]

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::{
    io::{Error, ErrorKind},
    mem::MaybeUninit,
    simd::{
        LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialOrd,
        simd_swizzle,
    },
};

#[cfg(not(target_arch = "x86_64"))]
use std::simd::{u8x16, u8x32, u8x64};

type SimdU8<const LANES: usize> = Simd<u8, LANES>;

#[cfg(feature = "serde")]
pub mod serde;

mod buf;

pub use buf::*;

const HEX_ENCODE_LUT: [u8; 16] = *b"0123456789abcdef";

// ─── Encode: x86_64 PSHUFB-based fast paths ────────────────────────────

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn encode_simd_64(input: &[u8], output: &mut [MaybeUninit<u8>]) {
    // Process 64 input bytes → 128 output bytes using AVX-512
    unsafe {
        let raw = _mm512_loadu_si512(input.as_ptr().cast());
        let mask = _mm512_set1_epi8(0x0F);
        let lut = _mm512_broadcast_i32x4(_mm_loadu_si128(
            HEX_ENCODE_LUT.as_ptr().cast(),
        ));

        // Extract nibbles
        let hi = _mm512_and_si512(_mm512_srli_epi16(raw, 4), mask);
        let lo = _mm512_and_si512(raw, mask);

        // LUT lookup: single vpshufb per nibble set
        let hi_ascii = _mm512_shuffle_epi8(lut, hi);
        let lo_ascii = _mm512_shuffle_epi8(lut, lo);

        // Interleave hi[i], lo[i] → output[2i], output[2i+1]
        // using vpermi2b (AVX-512 VBMI cross-lane byte permute)
        //
        // First 64 output bytes: hi[0],lo[0],hi[1],lo[1],...,hi[31],lo[31]
        // Second 64 output bytes: hi[32],lo[32],...,hi[63],lo[63]
        //
        // vpermi2b uses 7-bit indices: bit 6 selects source (0=first, 1=second)
        let perm_lo = _mm512_set_epi8(
            // Bytes 63..0 (set_epi8 is high-to-low)
            95, 31, 94, 30, 93, 29, 92, 28, 91, 27, 90, 26, 89, 25, 88, 24,
            87, 23, 86, 22, 85, 21, 84, 20, 83, 19, 82, 18, 81, 17, 80, 16,
            79, 15, 78, 14, 77, 13, 76, 12, 75, 11, 74, 10, 73, 9, 72, 8,
            71, 7, 70, 6, 69, 5, 68, 4, 67, 3, 66, 2, 65, 1, 64, 0,
        );
        let perm_hi = _mm512_set_epi8(
            127, 63, 126, 62, 125, 61, 124, 60, 123, 59, 122, 58, 121, 57,
            120, 56, 119, 55, 118, 54, 117, 53, 116, 52, 115, 51, 114, 50,
            113, 49, 112, 48, 111, 47, 110, 46, 109, 45, 108, 44, 107, 43,
            106, 42, 105, 41, 104, 40, 103, 39, 102, 38, 101, 37, 100, 36,
            99, 35, 98, 34, 97, 33, 96, 32,
        );

        let mut out_lo = perm_lo;
        let mut out_hi = perm_hi;
        out_lo = _mm512_permutex2var_epi8(hi_ascii, out_lo, lo_ascii);
        out_hi = _mm512_permutex2var_epi8(hi_ascii, out_hi, lo_ascii);

        _mm512_storeu_si512(output.as_mut_ptr().cast(), out_lo);
        _mm512_storeu_si512(
            output.as_mut_ptr().add(64).cast(),
            out_hi,
        );
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn encode_simd_32(input: &[u8], output: &mut [MaybeUninit<u8>]) {
    unsafe {
        let raw = _mm256_loadu_si256(input.as_ptr().cast());
        let mask = _mm256_set1_epi8(0x0F);
        let lut = _mm256_broadcastsi128_si256(_mm_loadu_si128(
            HEX_ENCODE_LUT.as_ptr().cast(),
        ));

        // Extract nibbles
        let hi = _mm256_and_si256(_mm256_srli_epi16(raw, 4), mask);
        let lo = _mm256_and_si256(raw, mask);

        // LUT lookup: single vpshufb per nibble set
        let hi_ascii = _mm256_shuffle_epi8(lut, hi);
        let lo_ascii = _mm256_shuffle_epi8(lut, lo);

        // Interleave using unpack + lane fix
        let interleaved_lo = _mm256_unpacklo_epi8(hi_ascii, lo_ascii);
        let interleaved_hi = _mm256_unpackhi_epi8(hi_ascii, lo_ascii);
        let final_lo =
            _mm256_permute2x128_si256(interleaved_lo, interleaved_hi, 0x20);
        let final_hi =
            _mm256_permute2x128_si256(interleaved_lo, interleaved_hi, 0x31);

        _mm256_storeu_si256(output.as_mut_ptr().cast(), final_lo);
        _mm256_storeu_si256(output.as_mut_ptr().add(32).cast(), final_hi);
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn encode_simd_16(input: &[u8], output: &mut [MaybeUninit<u8>]) {
    unsafe {
        let raw = _mm_loadu_si128(input.as_ptr().cast());
        let mask = _mm_set1_epi8(0x0F);
        let lut = _mm_loadu_si128(HEX_ENCODE_LUT.as_ptr().cast());

        let hi = _mm_and_si128(_mm_srli_epi16(raw, 4), mask);
        let lo = _mm_and_si128(raw, mask);

        let hi_ascii = _mm_shuffle_epi8(lut, hi);
        let lo_ascii = _mm_shuffle_epi8(lut, lo);

        let interleaved_lo = _mm_unpacklo_epi8(hi_ascii, lo_ascii);
        let interleaved_hi = _mm_unpackhi_epi8(hi_ascii, lo_ascii);

        _mm_storeu_si128(output.as_mut_ptr().cast(), interleaved_lo);
        _mm_storeu_si128(output.as_mut_ptr().add(16).cast(), interleaved_hi);
    }
}

// ─── Encode: portable_simd fallback for non-x86 ────────────────────────

#[cfg(not(target_arch = "x86_64"))]
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

    let interleaved: u8x64 = simd_swizzle!(hi_ascii, lo_ascii, [
        0, 32, 1, 33, 2, 34, 3, 35, 4, 36, 5, 37, 6, 38, 7, 39, 8, 40, 9,
        41, 10, 42, 11, 43, 12, 44, 13, 45, 14, 46, 15, 47, 16, 48, 17, 49,
        18, 50, 19, 51, 20, 52, 21, 53, 22, 54, 23, 55, 24, 56, 25, 57, 26,
        58, 27, 59, 28, 60, 29, 61, 30, 62, 31, 63
    ]);

    let interleaved: &[u8; 64] = interleaved.as_array();
    let uninit_src: &[MaybeUninit<u8>; 64] =
        unsafe { std::mem::transmute(interleaved) };
    output.copy_from_slice(uninit_src);
}

#[cfg(not(target_arch = "x86_64"))]
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

    let interleaved: u8x32 = simd_swizzle!(hi_ascii, lo_ascii, [
        0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23, 8, 24, 9,
        25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31
    ]);

    let interleaved: &[u8; 32] = interleaved.as_array();
    let uninit_src: &[MaybeUninit<u8>; 32] =
        unsafe { std::mem::transmute(interleaved) };
    output.copy_from_slice(uninit_src);
}

#[cfg(not(target_arch = "x86_64"))]
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

#[cfg(not(target_arch = "x86_64"))]
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

// ─── Encode: scalar fallback ────────────────────────────────────────────

#[inline(always)]
fn encode_scalar(data: &[u8], result: &mut [MaybeUninit<u8>]) {
    for (i, byte) in data.iter().enumerate() {
        let hi = (byte >> 4) as usize;
        let lo = (byte & 0xf) as usize;
        result[i * 2].write(HEX_ENCODE_LUT[hi]);
        result[i * 2 + 1].write(HEX_ENCODE_LUT[lo]);
    }
}

// ─── Encode: public API ─────────────────────────────────────────────────

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

    // Main loop: process 64 bytes at a time on x86_64 (AVX-512)
    #[cfg(target_arch = "x86_64")]
    while pos + 64 <= data.len() {
        encode_simd_64(
            &data[pos..pos + 64],
            &mut dst[pos * 2..(pos + 64) * 2],
        );
        pos += 64;
    }

    // Process remaining 32-byte chunks
    while pos + 32 <= data.len() {
        encode_simd_32(&data[pos..pos + 32], &mut dst[pos * 2..(pos + 32) * 2]);
        pos += 32;
    }

    // Handle remainder with overlapping SIMD reads to avoid
    // the scalar fallback for inputs >= 16 bytes
    if pos < data.len() {
        if data.len() >= 32 {
            // Re-encode last 32 bytes with overlapping SIMD
            let start = data.len() - 32;
            encode_simd_32(&data[start..], &mut dst[start * 2..]);
        } else if data.len() >= 16 {
            // 16-31 bytes: one 16-byte SIMD + overlapping 16
            encode_simd_16(&data[0..16], &mut dst[0..32]);
            if data.len() > 16 {
                let start = data.len() - 16;
                encode_simd_16(&data[start..], &mut dst[start * 2..]);
            }
        } else {
            // < 16 bytes: scalar fallback
            encode_scalar(&data[pos..], &mut dst[pos * 2..]);
        }
    }

    Ok(())
}

// ─── Decode ─────────────────────────────────────────────────────────────

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

#[inline(always)]
fn decode_simd_64(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<(), Error> {
    let chunk_vec: SimdU8<64> = Simd::from_slice(input);

    let high_bytes: SimdU8<32> = simd_swizzle!(chunk_vec, [
        0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34,
        36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62
    ]);
    let low_bytes: SimdU8<32> = simd_swizzle!(chunk_vec, [
        1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33, 35,
        37, 39, 41, 43, 45, 47, 49, 51, 53, 55, 57, 59, 61, 63
    ]);

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
    output.copy_from_slice(uninit_src);
    Ok(())
}

#[inline(always)]
fn decode_simd_32(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<(), Error> {
    let chunk_vec: SimdU8<32> = Simd::from_slice(input);
    let high_bytes: SimdU8<16> = simd_swizzle!(chunk_vec, [
        0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30
    ]);
    let low_bytes: SimdU8<16> = simd_swizzle!(chunk_vec, [
        1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31
    ]);

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
    output.copy_from_slice(uninit_src);
    Ok(())
}

#[inline]
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

        let high_bytes: SimdU8<32> = simd_swizzle!(chunk_vec, [
            0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
            34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60, 62
        ]);
        let low_bytes: SimdU8<32> = simd_swizzle!(chunk_vec, [
            1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31, 33,
            35, 37, 39, 41, 43, 45, 47, 49, 51, 53, 55, 57, 59, 61, 63
        ]);

        let (high_nibbles, high_valid) = decode_hex_nibbles(high_bytes);
        let (low_nibbles, low_valid) = decode_hex_nibbles(low_bytes);

        if !(high_valid & low_valid) {
            return Err(invalid_hex_char_error());
        }
        let decoded =
            (high_nibbles << SimdU8::<32>::splat(4)) | low_nibbles;

        let decoded: &[u8; 32] = decoded.as_array();
        // SAFETY: &[u8; 32] and &[MaybeUninit<u8>; 32] have the
        // same layout
        let uninit_src: &[MaybeUninit<u8>; 32] =
            unsafe { std::mem::transmute(decoded) };
        output[out_pos..out_pos + 32].copy_from_slice(uninit_src);

        pos += 64;
        out_pos += 32;
    }

    // Handle remainder with overlapping SIMD reads to avoid
    // the LUT fallback for inputs >= 32 hex chars
    if pos < n {
        if n >= 64 {
            // Re-decode last 64 hex bytes via overlapping SIMD
            let start = n - 64;
            let out_start = start / 2;
            decode_simd_64(
                &input[start..start + 64],
                &mut output[out_start..out_start + 32],
            )?;
        } else {
            // n < 64: process 32 bytes at a time
            while pos + 32 <= n {
                let chunk_vec: SimdU8<32> =
                    Simd::from_slice(&input[pos..pos + 32]);
                let high_bytes: SimdU8<16> = simd_swizzle!(chunk_vec, [
                    0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28,
                    30
                ]);
                let low_bytes: SimdU8<16> = simd_swizzle!(chunk_vec, [
                    1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29,
                    31
                ]);

                let (high_nibbles, high_valid) =
                    decode_hex_nibbles(high_bytes);
                let (low_nibbles, low_valid) =
                    decode_hex_nibbles(low_bytes);

                if !(high_valid & low_valid) {
                    return Err(invalid_hex_char_error());
                }

                let decoded =
                    (high_nibbles << SimdU8::<16>::splat(4)) | low_nibbles;
                let decoded: &[u8; 16] = decoded.as_array();
                // SAFETY: &[u8;16] and &[MaybeUninit<u8>; 16] have
                // the same layout
                let uninit_src: &[MaybeUninit<u8>; 16] =
                    unsafe { std::mem::transmute(decoded) };
                output[out_pos..out_pos + 16].copy_from_slice(uninit_src);
                pos += 32;
                out_pos += 16;
            }

            if pos < n {
                if n >= 32 {
                    // Re-decode last 32 hex bytes via overlapping
                    let start = n - 32;
                    let out_start = start / 2;
                    decode_simd_32(
                        &input[start..start + 32],
                        &mut output[out_start..out_start + 16],
                    )?;
                } else {
                    // < 32 hex bytes: LUT fallback
                    let remaining = n - pos;
                    decode_remainder_lut(
                        input, output, pos, out_pos, remaining,
                    )?;
                }
            }
        }
    }

    Ok(())
}

#[inline(always)]
fn decode_hex_nibbles<const LANES: usize>(
    n: SimdU8<LANES>,
) -> (SimdU8<LANES>, bool)
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let zero = SimdU8::<LANES>::splat(b'0');
    let nine = SimdU8::<LANES>::splat(b'9');
    let upper_a = SimdU8::<LANES>::splat(b'A');
    let gap = SimdU8::<LANES>::splat(b'A' - b'9' - 1);
    let lower_gap = SimdU8::<LANES>::splat(b'a' - b'A');

    let mut val = n - zero;

    // If > '9', subtract the gap to 'A'
    let gt_nine = n.simd_gt(nine);
    val = gt_nine.select(val - gap, val);

    // If >= 'a', subtract additional gap
    let ge_lower_a = n.simd_ge(SimdU8::<LANES>::splat(b'a'));
    val = ge_lower_a.select(val - lower_gap, val);

    // Validation: result must be in [0, 15] AND chars in the ':'-'@'
    // gap (0x3A-0x40) must be rejected. These produce values 3-9 after
    // gap subtraction, which would falsely pass val <= 15.
    let ge_upper_a = n.simd_ge(upper_a);
    let in_gap = gt_nine & !ge_upper_a;
    let valid = val.simd_le(SimdU8::<LANES>::splat(15)) & !in_gap;

    (val, valid.all())
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

    /// Regression test: characters ':' through '@' (0x3A-0x40) must be
    /// rejected as invalid hex. These sit in the gap between '9' and 'A'.
    #[test]
    fn test_decode_rejects_gap_chars_in_simd_path() {
        for ch in b':' ..= b'@' {
            // 64-char input exercises the 64-byte SIMD decode path
            let mut input = vec![b'0'; 64];
            input[0] = ch;
            let input_str = std::str::from_utf8(&input).unwrap();

            assert!(
                super::decode(input_str).is_err(),
                "char '{}' (0x{:02X}) should be rejected as invalid hex",
                ch as char,
                ch,
            );
        }
    }

    /// Verify gap char rejection also works in the 32-byte SIMD path
    #[test]
    fn test_decode_rejects_gap_chars_in_32byte_path() {
        for ch in b':' ..= b'@' {
            // 32-char input exercises the 32-byte SIMD decode path
            let mut input = vec![b'0'; 32];
            input[0] = ch;
            let input_str = std::str::from_utf8(&input).unwrap();

            assert!(
                super::decode(input_str).is_err(),
                "char '{}' (0x{:02X}) should be rejected in 32-byte path",
                ch as char,
                ch,
            );
        }
    }
}
