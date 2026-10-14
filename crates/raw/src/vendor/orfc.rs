//! Olympus predictive compression, adapted from RapidRAW's Rawler fork (LGPL-2.1).
//! Source: CyberTimon/RapidRAW-DngLab a32bc1ff1f4f3c56d4d3226082f6eb17414f9159,
//! rawler/src/decoders/orf.rs::decode_compressed; Daniel Vogelbacher,
//! Pedro Côrte-Real and DngLab/Rawler contributors (Rawloader lineage).
//! Raygun modifications (2026): checked bit reads, bounded allocations, wide predictor arithmetic,
//! explicit errors instead of wraparound, no unsafe/panics/camera tables. See LICENSE-LGPL-2.1.
use crate::{RawError, Result};

struct Bits<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Bits<'_> {
    fn peek(&self, n: u32) -> u32 {
        let byte = self.position / 8;
        let word = (0..3).fold(0u32, |v, i| (v << 8) | self.bytes.get(byte + i).copied().unwrap_or(0) as u32);
        (word >> (24 - self.position % 8 - n as usize)) & ((1 << n) - 1)
    }
    fn consume(&mut self, n: u32) -> Result<()> {
        let next = self.position.checked_add(n as usize).ok_or(RawError::Limit("ORF bit offset"))?;
        if next > self.bytes.len().saturating_mul(8) {
            return Err(RawError::Corrupt("truncated compressed ORF".into()));
        }
        self.position = next;
        Ok(())
    }
    fn read(&mut self, n: u32) -> Result<u32> {
        let value = self.peek(n);
        self.consume(n)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Default)]
struct Carry {
    encoded: u32,
    bias: i64,
    small: u32,
    left: i64,
    northwest: i64,
}

pub fn decode(bytes: &[u8], width: usize, height: usize, bits: u32) -> Result<Vec<u16>> {
    if !matches!(bits, 12 | 14) {
        return Err(RawError::Unsupported(format!("ORF {bits}-bit predictive compression")));
    }
    let pixels = width.checked_mul(height).filter(|n| *n > 0 && *n <= crate::MAX_SAMPLES).ok_or(RawError::Limit("compressed ORF dimensions"))?;
    if !width.is_multiple_of(2) {
        return Err(RawError::Corrupt("compressed ORF width must be even".into()));
    }
    let payload = bytes.get(if bits == 14 { 8.. } else { 7.. }).ok_or_else(|| RawError::Corrupt("compressed ORF header missing".into()))?;
    // Each pixel needs at least six bits. Reject impossible dimensions before allocating.
    if pixels.saturating_mul(6) > payload.len().saturating_mul(8) {
        return Err(RawError::Corrupt("compressed ORF payload too short".into()));
    }
    let mut output = Vec::new();
    output.try_reserve_exact(pixels).map_err(|_| RawError::Limit("compressed ORF allocation"))?;
    output.resize(pixels, 0u16);
    let mut pump = Bits { bytes: payload, position: 0 };
    for row in 0..height {
        let mut carries = [Carry::default(); 2];
        let offset = row.checked_mul(width).ok_or(RawError::Limit("ORF row offset"))?;
        let (previous, rest) = output.split_at_mut_checked(offset).ok_or_else(|| RawError::Corrupt("ORF row outside image".into()))?;
        let current = rest.get_mut(..width).ok_or_else(|| RawError::Corrupt("ORF row outside image".into()))?;
        for col in (0..width).step_by(2) {
            for (site, carry) in carries.iter_mut().enumerate() {
                let extra = if carry.small < 3 { 2 } else { 0 };
                let mut nbits = 2 + extra;
                while nbits < 16 && ((carry.encoded >> (nbits + extra)) & 0xffff) != 0 {
                    nbits += 1
                }
                let look = pump.peek(15);
                let sign = -((look >> 14) as i64);
                let low = ((look >> 12) & 3) as i64;
                let mut high = (look & 4095).leading_zeros().saturating_sub(20).min(12);
                if high == 12 {
                    pump.consume(15)?;
                    high = pump.read(16 - nbits)? >> 1;
                } else {
                    pump.consume(high + 4)?;
                }
                carry.encoded = (high << nbits) | pump.read(nbits)?;
                let diff = (carry.encoded as i64 ^ sign) + carry.bias;
                carry.bias = (diff * 3 + carry.bias) >> 5;
                carry.small = if carry.encoded > 16 { 0 } else { (carry.small + 1).min(3) };
                let column = col + site;
                let up = if row < 2 {
                    0
                } else {
                    let index = (row - 2).checked_mul(width).and_then(|i| i.checked_add(column)).ok_or(RawError::Limit("ORF predictor offset"))?;
                    previous.get(index).copied().ok_or_else(|| RawError::Corrupt("ORF predictor outside image".into()))? as i64
                };
                let predictor = if row < 2 {
                    carry.left
                } else if col < 2 {
                    up
                } else {
                    let dl = carry.left - carry.northwest;
                    let du = up - carry.northwest;
                    if (dl < 0 && du > 0) || (dl > 0 && du < 0) {
                        if dl.abs() > 32 || du.abs() > 32 { carry.left + du } else { (carry.left + up) >> 1 }
                    } else if dl.abs() > du.abs() {
                        carry.left
                    } else {
                        up
                    }
                };
                carry.left = predictor + ((diff << 2) | low);
                carry.northwest = up;
                if !(0..1i64 << bits).contains(&carry.left) {
                    return Err(RawError::Corrupt("compressed ORF sample outside sensor range".into()));
                }
                *current.get_mut(column).ok_or_else(|| RawError::Corrupt("ORF column outside image".into()))? = carry.left as u16;
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn zero_stream(width: usize, height: usize) -> Vec<u8> {
        let mut stream = Vec::new();
        for _ in 0..height {
            for x in 0..width {
                stream.extend([false, false, false, true]);
                stream.extend(std::iter::repeat_n(false, if x < 6 { 4 } else { 2 }));
            }
        }
        let mut bytes = vec![0; 7];
        bytes.extend(stream.chunks(8).map(|chunk| chunk.iter().enumerate().fold(0u8, |b, (i, v)| b | ((*v as u8) << (7 - i)))));
        bytes
    }
    #[test]
    fn predictive_zero_rows_and_truncated_escape_return_errors_without_panics() {
        let bytes = zero_stream(8, 4);
        assert_eq!(decode(&bytes, 8, 4, 12).unwrap(), vec![0; 32]);
        for cut in 0..bytes.len() - 1 {
            assert!(decode(&bytes[..cut], 8, 4, 12).is_err());
        }
        assert!(decode(&bytes, 7, 4, 12).is_err());
        assert!(decode(&bytes, usize::MAX, 4, 12).is_err());
        assert!(decode(&[0; 64], 8, 4, 12).is_err());
        assert!(decode(&bytes, 8, 4, 16).is_err());
    }
}
