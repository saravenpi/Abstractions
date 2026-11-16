//! Pure helpers of the fire and light systems: flicker, daylight dimming, which lights stay on, smoke puffs.

use super::glow::LightKind;

/// Lights farther than this from the camera stay off, in meters.
pub const ACTIVE_RANGE: f32 = 70.0;

/// Most lights lit at once.
pub const MAX_ACTIVE: usize = 32;

/// Returns the flicker factor in `[0.7, 1.0]` at `seconds` for a light with `phase`, a sum of two sines.
pub fn flicker(seconds: f32, phase: f32) -> f32 {
    let wave = (seconds * 7.3 + phase).sin() + (seconds * 11.9 + phase * 1.7).sin();
    0.85 + 0.075 * wave
}

/// Returns the share of full strength a light has when the daylight factor is `daylight`.
pub fn night_share(daylight: f32) -> f32 {
    (1.0 - daylight.clamp(0.0, 1.0)).powi(2)
}

/// Returns luminous power in lumens and range in meters of a light kind.
pub fn power(kind: LightKind) -> (f32, f32) {
    match kind {
        LightKind::Lamp => (30_000.0, 8.0),
        LightKind::Torch => (16_000.0, 6.0),
        LightKind::Campfire => (60_000.0, 10.0),
        LightKind::Brazier => (40_000.0, 9.0),
    }
}

/// Returns which of the lights at distances `dists` stay on: the nearest `max` within `range`.
pub fn pick_active(dists: &[f32], max: usize, range: f32) -> Vec<bool> {
    let mut order: Vec<usize> = (0..dists.len()).filter(|i| dists[*i] <= range).collect();
    order.sort_by(|a, b| dists[*a].total_cmp(&dists[*b]));
    let mut on = vec![false; dists.len()];
    order.into_iter().take(max).for_each(|i| on[i] = true);
    on
}

/// Returns the size of a smoke puff at `age` in `[0, 1)`: it swells, then shrinks away.
pub fn puff_size(age: f32) -> f32 {
    match age < 0.75 {
        true => 0.12 + age / 0.75 * 0.4,
        false => 0.52 * (1.0 - (age - 0.75) / 0.25).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flicker stays in range, and daylight dims lights to nothing.
    #[test]
    fn flicker_and_daylight_stay_bounded() {
        for i in 0..400 {
            let f = flicker(i as f32 * 0.37, i as f32);
            assert!((0.69..=1.01).contains(&f), "{f}");
        }
        assert_eq!(night_share(1.0), 0.0);
        assert_eq!(night_share(0.0), 1.0);
    }

    /// Only the nearest lights within range are picked, and puffs vanish at the end of their life.
    #[test]
    fn nearest_lights_win() {
        let on = pick_active(&[5.0, 90.0, 1.0, 30.0], 2, 70.0);
        assert_eq!(on, vec![true, false, true, false]);
        assert!(puff_size(0.999) < 0.01 && puff_size(0.7) > puff_size(0.0));
    }
}
