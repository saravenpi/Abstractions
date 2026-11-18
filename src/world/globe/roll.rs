//! Seeded rolls: a stable number for a planet seed, a grid cell and a slot, the only chance the ground uses.

/// Returns a number from 0 included to 1 excluded for `seed`, the grid cell `cell` and `slot`.
pub fn roll(seed: u64, cell: [i64; 3], slot: u32) -> f32 {
    (hash(seed, cell, slot) >> 40) as f32 / (1u64 << 24) as f32
}

/// Returns four numbers from 0 included to 1 excluded for `seed`, the grid cell `cell` and `slot`, from one
/// hash: each has 16 bits, enough to place and size a landform.
pub fn rolls(seed: u64, cell: [i64; 3], slot: u32) -> [f32; 4] {
    let h = hash(seed, cell, slot);
    [0, 16, 32, 48].map(|shift| ((h >> shift) & 0xFFFF) as f32 / 65_536.0)
}

/// Returns 64 scrambled bits for `seed`, the grid cell `cell` and `slot`.
fn hash(seed: u64, cell: [i64; 3], slot: u32) -> u64 {
    let mut h = seed ^ (cell[0] as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (cell[1] as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= (cell[2] as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
    h ^= u64::from(slot).wrapping_mul(0x1656_67B1_9E37_79F9);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^ (h >> 31)
}
