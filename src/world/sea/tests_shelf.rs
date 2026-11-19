//! Tests of the shelf of the near sea: over the flat sea of the coarse planet mesh it reads deep
//! water at every distance, so the straight cells of the mesh draw no rings.

use super::mesh::lines;

/// The shader source, to pin the formulas mirrored here.
const SHADER: &str = include_str!("water.wesl");

/// The coarse mesh source, to pin how far below sea level it draws its flat sea.
const COARSE: &str = include_str!("../mesh/tris.rs");

/// Metres below sea level of the flat sea of the coarse mesh, as `SEA_DROP` in `tris.rs`.
const SEA_DROP: f64 = 1.5;

/// Water depths in metres between which the near sea turns from a shallow shelf to deep water,
/// as `SHELF_START` and `SHELF_END` in the shader.
const SHELF: (f64, f64) = (1.1, 1.4);

/// Depth in metres the shader caps a water column at, as `DEEP`.
const DEEP: f64 = 12.0;

/// Mirrors the shelf of the fragment shader: how much deep water a fragment of the near sea
/// shows over a bed `measured` metres under it, the fragment lying `sag` metres under the sea
/// level sphere on the straight cell of its mesh.
fn shelf(measured: f64, sag: f64) -> f64 {
    let column = (measured + sag.max(0.0)).min(DEEP);
    let t = ((column - SHELF.0) / (SHELF.1 - SHELF.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns how far the middle of a straight edge `width` metres long between two points of the
/// sea level sphere of `radius` metres lies under that sphere.
fn chord_sag(width: f64, radius: f64) -> f64 {
    radius * (1.0 - (0.5 * width / radius).cos())
}

/// Over the coarse flat sea every cell of the near sea mesh is deep water, on the smallest and
/// the largest planet, although its straight cells sag far enough to read a shallow shelf from
/// the depth buffer alone.
#[test]
fn the_coarse_sea_reads_deep_under_every_cell() {
    let edges = lines();
    let widest = edges
        .windows(2)
        .map(|w| f64::from(w[1] - w[0]))
        .fold(0.0, f64::max);
    for radius in [3000.0, 6000.0, 40_000.0] {
        for pair in edges.windows(2) {
            let sag = chord_sag(f64::from(pair[1] - pair[0]), radius);
            assert_eq!(shelf(SEA_DROP - sag, sag), 1.0, "{radius} {pair:?}");
        }
    }
    let sag = chord_sag(widest, 3000.0);
    assert!(
        shelf(SEA_DROP - sag, 0.0) < 1.0,
        "the bare depth reads a shelf"
    );
    for line in [
        "let sag = max(water.globe.w - length(world - water.globe.xyz), 0.0);",
        "let column = min(column_depth(in.position, rise * dist) + sag, DEEP);",
        "let shelf = smoothstep(SHELF_START, SHELF_END, column);",
        "const SHELF_START: f32 = 1.1;",
        "const SHELF_END: f32 = 1.4;",
        "const DEEP: f32 = 12.0;",
    ] {
        assert!(SHADER.contains(line), "{line}");
    }
    assert!(COARSE.contains("const SEA_DROP: f32 = 1.5;"));
}
