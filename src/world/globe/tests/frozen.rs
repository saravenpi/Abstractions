//! The legacy plane is frozen: its heights, biomes, surfaces, ponds and spawns never change by a bit.

use crate::world::Terrain;

/// Folds `value` into the running FNV-1a hash `hash`.
fn fold(hash: u64, value: u32) -> u64 {
    (hash ^ u64::from(value)).wrapping_mul(0x0100_0000_01B3)
}

/// Returns the hash of everything the flat terrain of `seed` says over a grid of 96 by 96 points.
fn digest(seed: u64) -> u64 {
    let t = Terrain::new(seed);
    let spawn = t.find_spawn();
    let start = spawn
        .to_array()
        .iter()
        .fold(0xCBF2_9CE4_8422_2325, |h, v| fold(h, v.to_bits()));
    (0..96 * 96).fold(start, |h, i| {
        let (x, z) = (
            (i % 96) as f32 * 41.37 - 1985.0,
            (i / 96) as f32 * 40.61 - 1950.0,
        );
        let h = fold(h, t.height(x, z).to_bits());
        let h = fold(h, t.biome(x, z) as u32);
        let h = fold(h, t.surface(x, z) as u32);
        fold(h, u32::from(t.pond_at(x, z)))
    })
}

/// The flat terrain gives the exact values it gave before the globe was sampled in 3D.
#[test]
fn flat_path_is_frozen() {
    let got = [1, 2, 42].map(digest);
    assert_eq!(
        got,
        [
            0xA6DD_372B_E9AA_7ACF,
            0x82B4_2CD6_90ED_85FA,
            0xB8EC_22AE_8742_BA68
        ]
    );
}
