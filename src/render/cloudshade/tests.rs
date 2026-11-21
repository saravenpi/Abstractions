//! Tests of the cloud shadows: none under a clear sky, more as the cover grows, none with the sun
//! on the horizon, the camera takes the shadow under it, and the shader reads what the CPU writes.

use bevy::prelude::*;

use super::rig::{RADIUS, cast, ground};
use super::tune::{HEAD, OPTICAL_DEPTH, SUN_EDGE};
use super::{Cast, CloudShade};
use crate::render::PlaceUniform;

/// The shader that works the shadow out at every pixel.
const SHADER: &str = include_str!("../daylight.wesl");

/// Returns the mean share of the sun over the ground of [`ground`].
fn mean(cast: &Cast) -> f32 {
    ground().map(|dir| cast.through(dir)).sum::<f32>() / 400.0
}

/// The sky lets the sun through under a clear cover, takes more of it as the cover grows, and
/// never takes more than a full puff does.
#[test]
fn the_shadow_falls_with_the_cover() {
    let (clear, half, full) = (
        mean(&cast(0.8, 0.0, 1234.0)),
        mean(&cast(0.8, 0.5, 1234.0)),
        mean(&cast(0.8, 1.0, 1234.0)),
    );
    assert!(clear > 0.95, "{clear}");
    assert!(clear > half && half > full, "{clear} {half} {full}");
    assert!(full < 0.7, "{full}");
    let least = (-OPTICAL_DEPTH).exp();
    assert!(ground().all(|dir| cast(0.8, 1.0, 1234.0).through(dir) >= least - 1.0e-4));
}

/// With the sun on the horizon the clouds shade nothing.
#[test]
fn a_low_sun_is_not_shaded() {
    let low = cast(0.0, 1.0, 1234.0);
    assert!(ground().all(|dir| (low.through(dir) - 1.0).abs() < 1.0e-6));
}

/// Without a cast the camera takes the sun as it is, and with one it takes the shadow of the
/// ground under it.
#[test]
fn the_camera_reads_the_cast() {
    let empty = CloudShade::default();
    assert_eq!(empty.eye(Vec3::Y), 1.0);
    let c = cast(0.8, 1.0, 1234.0);
    let shade = CloudShade {
        cast: Some(c),
        ..default()
    };
    for dir in ground().take(20) {
        assert_eq!(shade.eye(dir), c.through(dir));
    }
}

/// The block holds the header a float at a time, and the shader reads it from the texel after
/// the fields of the place block, with the same depth and sun heights, and shades by the same
/// field the shell draws.
#[test]
fn the_shader_reads_the_shadow_where_it_is_written() {
    let mut shade = CloudShade::default();
    shade.head[3] = Vec4::new(RADIUS + 3000.0, 3540.0, 0.5, 1.0);
    let mut out = vec![0u8; 16 * CloudShade::TEXELS];
    shade.write(&mut out);
    let number = |at: usize| f32::from_le_bytes(out[4 * at..4 * at + 4].try_into().unwrap());
    assert_eq!((number(13), number(15)), (3540.0, 1.0));
    assert_eq!(HEAD, 5);
    let lines = [
        format!("const SHADE_AT: i32 = {};", PlaceUniform::FIELDS),
        format!("const SHADE_DEPTH: f32 = {OPTICAL_DEPTH:?};"),
        format!(
            "const SHADE_SUN: vec2<f32> = vec2<f32>({:?}, {:?});",
            SUN_EDGE.0, SUN_EDGE.1
        ),
        "overhead(normalize(hit) * shell.y, shade_at(map, 4), shell.z, footprint)".to_string(),
    ];
    for line in lines {
        assert!(SHADER.contains(&line), "{line}");
    }
}
