//! Whole numbers as the wire writes them: the sign folded into the lowest bit, then seven bits
//! at a time.

/// Returns `value` with its sign in the lowest bit, so small numbers of both signs stay short.
pub(super) fn zigzag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

/// Returns the number [`zigzag`] made `value` from.
pub(super) fn unzigzag(value: u64) -> i64 {
    (value >> 1) as i64 ^ -((value & 1) as i64)
}

/// Adds `value` to `out` in seven-bit groups, low first, the high bit of a byte saying more
/// follow.
pub(super) fn put_varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push(value as u8 | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}
