//! A repeatable roll: the same key always gives the same number, so every machine draws the same
//! scars on the same part and the same flicker at the same moment.

/// Returns a number from 0 to 1 that depends on `key` alone.
pub fn luck(key: u32) -> f32 {
    let mut h = key.wrapping_mul(0x27d4_eb2d);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / 16_777_216.0
}
