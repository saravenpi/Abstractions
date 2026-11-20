//! Tests of the cloud shadows: none under a clear sky, more as the cover grows, none with the sun
//! on the horizon, and the patch reads the same on the CPU as in the shader.

use bevy::prelude::*;

use super::tune::{EDGE_FADE, OPTICAL_DEPTH, REACH, SIDE, TEXELS};
use super::{Cast, CloudShade};
use crate::render::PlaceUniform;

/// The shader that reads the patch.
const SHADER: &str = include_str!("../daylight.wesl");

/// Returns a cast over a planet of the size of the Earth, under a cloud deck 3 km up, with the
/// sun at the sine `height` and the broad cover `cover`.
fn cast(height: f32, cover: f32) -> Cast {
    let sun = Vec3::new((1.0 - height * height).sqrt(), height, 0.0);
    Cast {
        centre: Vec3::Y,
        east: Vec3::X,
        north: Vec3::NEG_Z,
        radius: 6_371_000.0,
        shell: 6_374_000.0,
        sun,
        seconds: 1234.0,
        cover,
    }
}

/// Returns the mean share of the sun over the patch.
fn mean(cast: &Cast) -> f32 {
    let grid = cast.grid();
    grid.iter().sum::<f32>() / grid.len() as f32
}

/// The sky lets the sun through under a clear cover, takes more of it as the cover grows, and
/// never takes more than a full puff does.
#[test]
fn the_shadow_falls_with_the_cover() {
    let (clear, half, full) = (
        mean(&cast(0.8, 0.0)),
        mean(&cast(0.8, 0.5)),
        mean(&cast(0.8, 1.0)),
    );
    assert!(clear > 0.95, "{clear}");
    assert!(clear > half && half > full, "{clear} {half} {full}");
    assert!(full < 0.7, "{full}");
    let least = (-OPTICAL_DEPTH).exp();
    assert!(
        cast(0.8, 1.0)
            .grid()
            .iter()
            .all(|share| *share >= least - 1.0e-4)
    );
}

/// With the sun on the horizon the clouds shade nothing, and the patch is a whole grid.
#[test]
fn a_low_sun_is_not_shaded() {
    let low = cast(0.0, 1.0);
    assert_eq!(low.grid().len(), SIDE * SIDE);
    assert!(low.grid().iter().all(|share| (share - 1.0).abs() < 1.0e-6));
}

/// Without a patch the camera and the shader take the sun as it is, and with one the camera takes
/// the grid value under it, and nothing outside the patch.
#[test]
fn the_camera_reads_the_patch() {
    let empty = CloudShade::default();
    assert_eq!(empty.eye(Vec3::Y), 1.0);
    let c = cast(0.8, 1.0);
    let head = [
        Vec4::ZERO,
        Vec4::ZERO,
        Vec4::ZERO,
        c.centre.extend(REACH),
        c.east.extend(0.0),
        c.north.extend(0.0),
    ];
    let shade = CloudShade {
        head,
        grid: c.grid(),
        radius: c.radius,
        revision: 1,
    };
    let (lo, hi) = (SIDE / 2 - 1, SIDE / 2);
    let middle = [(lo, lo), (hi, lo), (lo, hi), (hi, hi)].map(|(i, j)| shade.grid[j * SIDE + i]);
    let want = middle.iter().sum::<f32>() / 4.0;
    assert!((shade.eye(Vec3::Y) - want).abs() < 1.0e-4);
    assert_eq!(shade.eye(Vec3::X), 1.0);
}

/// The block holds the header then the grid, a float at a time, and the shader reads them from
/// the texel after the fields of the place block with the same side and size.
#[test]
fn the_shader_reads_the_patch_where_it_is_written() {
    let mut shade = CloudShade::default();
    shade.head[3] = Vec4::new(0.0, 1.0, 0.0, REACH);
    shade.grid[0] = 0.25;
    let mut out = vec![0u8; 16 * TEXELS];
    shade.write(&mut out);
    let number = |at: usize| f32::from_le_bytes(out[4 * at..4 * at + 4].try_into().unwrap());
    assert_eq!((number(13), number(15)), (1.0, REACH));
    assert_eq!((number(24), number(27)), (0.25, 1.0));
    assert_eq!(super::tune::HEAD, 6);
    let at = format!("const SHADE_AT: i32 = {};", PlaceUniform::FIELDS);
    let grid = format!(
        "const SHADE_GRID: i32 = {};",
        PlaceUniform::FIELDS + super::tune::HEAD
    );
    let side = format!("const SHADE_SIDE: i32 = {SIDE};");
    for line in [at, grid, side] {
        assert!(SHADER.contains(&line), "{line}");
    }
    let fade = format!("const SHADE_REACH_FADE: f32 = {EDGE_FADE:?};");
    assert!(SHADER.contains(&fade));
}
