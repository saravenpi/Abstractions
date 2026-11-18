//! Small numbers in one or two bytes: shares from 0 to 1, and values inside a known range.
//!
//! Angles use the signed helpers with a range of pi or two pi. Out-of-range values are clamped.

/// Returns `value`, a share from 0 to 1, in 8 bits. The error is at most 1/510.
pub fn pack_unit(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Returns the share from 0 to 1 that [`pack_unit`] stored.
pub fn unpack_unit(packed: u8) -> f32 {
    f32::from(packed) / 255.0
}

/// Returns `value`, from `-range` to `range`, in 8 bits. The error is at most `range / 254`.
pub fn pack_i8(value: f32, range: f32) -> i8 {
    ((value / range).clamp(-1.0, 1.0) * 127.0).round() as i8
}

/// Returns the value from `-range` to `range` that [`pack_i8`] stored.
pub fn unpack_i8(packed: i8, range: f32) -> f32 {
    (f32::from(packed) / 127.0).max(-1.0) * range
}

/// Returns `value`, from `-range` to `range`, in 16 bits. The error is at most `range / 65534`.
pub fn pack_i16(value: f32, range: f32) -> i16 {
    ((value / range).clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Returns the value from `-range` to `range` that [`pack_i16`] stored.
pub fn unpack_i16(packed: i16, range: f32) -> f32 {
    (f32::from(packed) / 32767.0).max(-1.0) * range
}

/// Returns `value`, from 0 to `range`, in 16 bits. The error is at most `range / 131070`.
pub fn pack_u16(value: f32, range: f32) -> u16 {
    ((value / range).clamp(0.0, 1.0) * 65535.0).round() as u16
}

/// Returns the value from 0 to `range` that [`pack_u16`] stored.
pub fn unpack_u16(packed: u16, range: f32) -> f32 {
    f32::from(packed) / 65535.0 * range
}
