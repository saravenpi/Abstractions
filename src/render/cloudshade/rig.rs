//! The setup the tests of the cloud shadows share: a planet of the size of the Earth under a
//! cloud shell 3 km up, and a patch of ground on its equator.

use bevy::prelude::*;

use super::Cast;

/// Sea level radius of the planet of the tests, the size of the Earth, in metres.
pub const RADIUS: f32 = 6_371_000.0;

/// Returns a cast over a planet of the size of the Earth, under a cloud shell 3 km up, with the
/// sun at the sine `height` over the equator at +Z and the broad cover `cover`, at `seconds`.
pub fn cast(height: f32, cover: f32, seconds: f64) -> Cast {
    Cast {
        radius: RADIUS,
        shell: RADIUS + 3000.0,
        sun: Vec3::new(0.0, (1.0 - height * height).sqrt(), height),
        seconds,
        cover,
    }
}

/// Returns 400 ground directions within 4 km of the point of the equator at +Z.
pub fn ground() -> impl Iterator<Item = Vec3> {
    (0..400).map(|i| {
        let (x, z) = (
            (i % 20) as f32 * 400.0 - 4000.0,
            (i / 20) as f32 * 400.0 - 4000.0,
        );
        Vec3::new(x, z, RADIUS).normalize()
    })
}
