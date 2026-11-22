//! Guards the legacy plane: its scatter and merged output must stay bit for bit what v0.5 produced.

use bevy::mesh::VertexAttributeValues;

use super::*;
use crate::world::propmesh::merge_props;
use crate::world::props::{body_collider, build_grass};

/// Folds the bits of `values` into an FNV-1a digest.
fn fold(digest: u64, values: impl IntoIterator<Item = u32>) -> u64 {
    values.into_iter().fold(digest, |d, v| {
        (d ^ u64::from(v)).wrapping_mul(0x0000_0100_0000_01B3)
    })
}

/// Returns the bits of the vertex positions of a mesh.
fn positions(mesh: &Mesh) -> Vec<u32> {
    let Some(VertexAttributeValues::Float32x3(v)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        return Vec::new();
    };
    v.iter().flatten().map(|c| c.to_bits()).collect()
}

/// Returns the digest of everything the legacy entry points produce for a block of chunks of `seed`.
fn digest(seed: u64) -> u64 {
    let t = Terrain::legacy(seed);
    let mut d = 0xCBF2_9CE4_8422_2325;
    for coord in (-4..4).flat_map(|x| (-4..4).map(move |y| IVec2::new(x, y))) {
        let s = scatter_chunk(&t, coord);
        for p in s.props.iter().chain(&s.grass) {
            let bits = [p.pos.x, p.pos.y, p.pos.z, p.yaw, p.scale].map(f32::to_bits);
            d = fold(
                d,
                bits.into_iter()
                    .chain([p.plant as u32, u32::from(p.variant)]),
            );
        }
        d = fold(d, positions(&merge_props(&s.props)));
        d = fold(d, positions(&build_grass(&s)));
        d = fold(
            d,
            [body_collider(&s.props).map_or(0, |c| {
                c.shape()
                    .as_compound()
                    .map_or(0, |c| c.shapes().len() as u32)
            })],
        );
    }
    d
}

/// The legacy plane scatters, merges and builds exactly what it did before the globe.
#[test]
fn flat_output_is_frozen() {
    let frozen = [
        (1, 0xC3EA_0634_7238_DA83_u64),
        (5, 0xEA5F_7BC3_BA3A_089B),
        (42, 0x84EE_3F0C_960C_7136),
    ];
    let got: Vec<(u64, u64)> = frozen
        .iter()
        .map(|&(seed, _)| (seed, digest(seed)))
        .collect();
    let changed: Vec<String> = frozen
        .iter()
        .zip(&got)
        .filter(|((_, want), (_, got))| got != want)
        .map(|((seed, want), (_, got))| format!("seed {seed}: got {got:#018X}, want {want:#018X}"))
        .collect();
    assert!(changed.is_empty(), "{}", changed.join("\n"));
}
