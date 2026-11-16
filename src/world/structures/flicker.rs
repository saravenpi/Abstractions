//! Pure helpers of the fire and light systems: flicker, fading, daylight dimming and which lights stay on.

use super::glow::LightKind;

/// Lights farther than this from the camera stay off, in meters.
pub const ACTIVE_RANGE: f32 = 70.0;

/// Most lights lit at once.
pub const MAX_ACTIVE: usize = 32;

/// How fast a light fades in or out, in levels per second.
pub const FADE_RATE: f32 = 2.0;

/// Rate in radians per second, weight and phase factor of each flicker wave. The weights sum to one.
const WAVES: [(f32, f32, f32); 5] = [
    (2.3, 0.32, 1.0),
    (5.9, 0.26, 1.7),
    (9.7, 0.2, 2.9),
    (14.3, 0.14, 0.6),
    (23.1, 0.08, 4.1),
];

/// Returns the flicker factor at `seconds` for a light of `kind` with `phase`.
///
/// Open flames stay in `[0.75, 1.05]`: five incommensurate sines plus an occasional soft dip.
/// The enclosed lamp and the candle stay in `[0.96, 1.02]`.
pub fn flicker(kind: LightKind, seconds: f32, phase: f32) -> f32 {
    let wave: f32 = WAVES
        .iter()
        .map(|(rate, weight, shift)| weight * (seconds * rate + phase * shift).sin())
        .sum();
    let gust = (seconds * 0.9 + phase * 2.3).sin() * (seconds * 0.37 + phase).sin();
    let dip = ((gust - 0.6) / 0.4).clamp(0.0, 1.0);
    let swing = 0.1 * wave - 0.1 * dip * dip * (3.0 - 2.0 * dip);
    match kind {
        LightKind::Lamp | LightKind::Candle => 1.0 + 0.2 * swing,
        _ => 0.95 + swing,
    }
}

/// Returns the level of a light after `dt` seconds: it rises to 1 while `on` and falls to 0 otherwise.
pub fn fade(level: f32, on: bool, dt: f32) -> f32 {
    let step = FADE_RATE * dt.max(0.0);
    (level + if on { step } else { -step }).clamp(0.0, 1.0)
}

/// Returns the share of full strength a light of `kind` has: indoor fires and candles stay half lit by day.
pub fn lit_share(kind: LightKind, daylight: f32) -> f32 {
    let floor = f32::from(matches!(kind, LightKind::Hearth | LightKind::Candle)) * 0.5;
    (1.0 - daylight.clamp(0.0, 1.0)).powi(2).max(floor)
}

/// Returns luminous power in lumens and range in meters of a light kind.
///
/// A point light gives `lumens / (4 pi d^2)` lux at `d` meters, so 26 000 lm is about 230 lux at 3 m.
pub fn power(kind: LightKind) -> (f32, f32) {
    match kind {
        LightKind::Lamp => (38_000.0, 11.0),
        LightKind::Torch => (26_000.0, 9.0),
        LightKind::Campfire => (70_000.0, 14.0),
        LightKind::Brazier => (48_000.0, 12.0),
        LightKind::Hearth => (32_000.0, 8.0),
        LightKind::Candle => (9_000.0, 6.0),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Open flames flicker within a quarter, the lamp and the candle within four percent, and none is a plain pulse.
    #[test]
    fn flicker_stays_bounded_per_kind() {
        let kinds = [
            (LightKind::Torch, 0.75, 1.05),
            (LightKind::Campfire, 0.75, 1.05),
            (LightKind::Brazier, 0.75, 1.05),
            (LightKind::Hearth, 0.75, 1.05),
            (LightKind::Lamp, 0.96, 1.02),
            (LightKind::Candle, 0.96, 1.02),
        ];
        for (kind, low, high) in kinds {
            let samples: Vec<f32> = (0..4000)
                .map(|i| flicker(kind, i as f32 * 0.037, i as f32 * 0.01))
                .collect();
            let (min, max) = samples
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), f| (a.min(*f), b.max(*f)));
            assert!(
                min >= low - 1e-4 && max <= high + 1e-4,
                "{kind:?}: {min}..{max}"
            );
            assert!(max - min > (high - low) * 0.5, "{kind:?} barely moves");
        }
        assert!(
            (flicker(LightKind::Torch, 3.0, 0.0) - flicker(LightKind::Torch, 3.0, 2.0)).abs()
                > 1e-3
        );
    }

    /// A light takes half a second to fade either way, never jumps and never leaves 0 to 1.
    #[test]
    fn fade_is_gradual_and_bounded() {
        assert_eq!(fade(0.0, false, 1.0), 0.0);
        assert_eq!(fade(1.0, true, 1.0), 1.0);
        assert!((fade(0.0, true, 0.1) - 0.2).abs() < 1e-6);
        assert!((fade(1.0, false, 0.1) - 0.8).abs() < 1e-6);
        let mut level = 0.0;
        let frames = (0..120).take_while(|_| {
            level = fade(level, true, 1.0 / 60.0);
            level < 1.0
        });
        assert!((28..=31).contains(&frames.count()));
        assert_eq!(lit_share(LightKind::Torch, 0.0), 1.0);
        assert_eq!(lit_share(LightKind::Hearth, 1.0), 0.5);
        assert_eq!(lit_share(LightKind::Torch, 1.0), 0.0);
    }

    /// Only the nearest lights within range are picked.
    #[test]
    fn nearest_lights_win() {
        let on = pick_active(&[5.0, 90.0, 1.0, 30.0], 2, 70.0);
        assert_eq!(on, vec![true, false, true, false]);
    }
}
