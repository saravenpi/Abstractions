//! The hand-over of the sea: how much of the near sea is drawn as the flat sea of the coarse planet mesh.
//!
//! The near sea mirrors the sky and the coarse sea is plainly lit, so from high up the near sea showed
//! as a disc of another colour under the camera. The near sea now turns into the coarse one, lit the same
//! way, with distance and with height, before it fades out. `water.wesl` reads these numbers.

use bevy::math::Vec4;

use super::follow::HANDOVER;

/// Share of [`HANDOVER`] at which the near sea starts to turn into the coarse sea with distance.
pub const TURN_START: f32 = 0.5;

/// Share of [`HANDOVER`] at which the near sea is the coarse sea, and starts to fade out (`HAND_SOFT`
/// in `water.wesl`).
pub const TURN_END: f32 = 0.83;

/// Heights of the camera above the sea in metres between which the whole near sea turns into the coarse
/// sea: from the air the two must be one colour under the camera too.
pub const ALOFT: (f32, f32) = (150.0, 600.0);

/// Hermite interpolation of `x` between `a` and `b`, clamped to 0..1, as `smoothstep` in a shader.
#[cfg(test)]
fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Returns the share of the coarse sea in the near sea `distance` metres from a camera `height` metres
/// above the sea: none close by from the ground, all of it where the near sea fades out and from the air.
/// It replays `afar` of `water.wesl`, which draws it.
#[cfg(test)]
pub fn afar(distance: f32, height: f32) -> f32 {
    let reach = smooth(HANDOVER * TURN_START, HANDOVER * TURN_END, distance);
    reach.max(smooth(ALOFT.0, ALOFT.1, height))
}

/// Returns the numbers of the hand-over as the shader reads them: the distances between which the near
/// sea turns with distance, then the heights between which it turns with height.
pub fn packed() -> Vec4 {
    Vec4::new(HANDOVER * TURN_START, HANDOVER * TURN_END, ALOFT.0, ALOFT.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On foot the sea close by is the near sea alone, and it is the coarse sea alone before it starts to
    /// fade out, whatever the height.
    #[test]
    fn the_near_sea_is_the_coarse_sea_where_it_fades_out() {
        assert_eq!(afar(0.0, 1.7), 0.0);
        assert_eq!(afar(HANDOVER * TURN_START, 100.0), 0.0);
        for height in [0.0, 1.7, 100.0, 400.0, 1500.0] {
            assert_eq!(afar(HANDOVER * TURN_END, height), 1.0);
            assert_eq!(afar(HANDOVER, height), 1.0);
        }
    }

    /// From the air the whole near sea is the coarse sea, straight down included, and the share never
    /// drops as the camera climbs or as the sea gets farther.
    #[test]
    fn from_the_air_the_whole_sea_is_the_coarse_sea() {
        for distance in [600.0, 800.0, 1500.0] {
            assert_eq!(afar(distance, ALOFT.1), 1.0);
        }
        let mut last = 0.0;
        for step in 0..=100 {
            let height = step as f32 * 8.0;
            let share = afar(height.max(1.0), height);
            assert!(share >= last, "{height}: {share} under {last}");
            last = share;
        }
        assert!(afar(400.0, 200.0) < afar(1000.0, 200.0));
    }

    /// The shader gets the same numbers the rule is written with.
    #[test]
    fn the_shader_reads_the_numbers_of_the_rule() {
        let p = packed();
        assert_eq!((p.x, p.y), (HANDOVER * TURN_START, HANDOVER * TURN_END));
        assert_eq!((p.z, p.w), ALOFT);
        assert!(p.x < p.y && p.y < HANDOVER && p.z < p.w && p.w < HANDOVER);
    }
}
