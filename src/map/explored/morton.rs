//! Morton codes of a leaf's column and row: the bits of the two numbers woven together.

/// Returns the leaves along one edge of a face at `depth`.
pub(super) fn along(depth: u64) -> f64 {
    (1u64 << depth) as f64
}

/// Spreads the low 32 bits of `value` to the even bits.
pub(super) fn spread(value: u64) -> u64 {
    let mut v = value & 0xffff_ffff;
    v = (v | (v << 16)) & 0x0000_ffff_0000_ffff;
    v = (v | (v << 8)) & 0x00ff_00ff_00ff_00ff;
    v = (v | (v << 4)) & 0x0f0f_0f0f_0f0f_0f0f;
    v = (v | (v << 2)) & 0x3333_3333_3333_3333;
    (v | (v << 1)) & 0x5555_5555_5555_5555
}

/// Gathers the even bits of `value`, the inverse of [`spread`].
pub(super) fn gather(value: u64) -> u64 {
    let mut v = value & 0x5555_5555_5555_5555;
    v = (v | (v >> 1)) & 0x3333_3333_3333_3333;
    v = (v | (v >> 2)) & 0x0f0f_0f0f_0f0f_0f0f;
    v = (v | (v >> 4)) & 0x00ff_00ff_00ff_00ff;
    v = (v | (v >> 8)) & 0x0000_ffff_0000_ffff;
    (v | (v >> 16)) & 0xffff_ffff
}
