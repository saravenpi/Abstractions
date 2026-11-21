//! Tests of the glow along whole flights: a craft falling from orbit into an Earth-like air and
//! into a thin air starts to glow and peaks at the heights the physics of a ballistic entry gives,
//! nothing glows in a vacuum or on a slow climb, and the glow is the same going up and going down.

/// Heat share at which a glow reads as begun.
const SHEATH_FROM: f32 = 0.02;

use super::{Gas, cool_speed, entry};

/// Mass over drag area of the ship flying nose first, in kilograms per square metre: one over
/// twice its nose drag of 1.3e-3 square metres per kilogram.
const BALLISTIC: f64 = 385.0;

/// Seconds per step of the flights.
const STEP: f64 = 0.05;

/// An air and the planet under it.
#[derive(Debug, Clone, Copy)]
struct Air {
    /// Density at sea level in kilograms per cubic metre.
    sea: f64,
    /// Scale height in metres.
    scale: f64,
    /// Top of the air in metres, twelve scale heights up as in the game.
    top: f64,
    /// Radius of the planet in metres.
    radius: f64,
    /// Gravity at the surface in metres per second squared.
    gravity: f64,
}

impl Air {
    /// Returns the density `height` metres up: exponential below the top, nothing above.
    fn density(&self, height: f64) -> f64 {
        match height < self.top {
            true => self.sea * (-height.max(0.0) / self.scale).exp(),
            false => 0.0,
        }
    }

    /// Returns the height of the peak heating of a ballistic entry at the path angle `slope`
    /// in radians below the horizon, from Allen and Eggers: where the density is
    /// `beta sin(slope) / 3 H`.
    fn peak(&self, slope: f64) -> f64 {
        let density = BALLISTIC * slope.sin() / (3.0 * self.scale);
        self.scale * (self.sea / density).ln()
    }
}

/// The air of the Earth, as the game builds it.
const EARTH: Air = Air {
    sea: 1.2,
    scale: 8420.0,
    top: 101_000.0,
    radius: 6.371e6,
    gravity: 9.81,
};

/// A thin, cold air on a small planet, like that of Mars.
const THIN: Air = Air {
    sea: 0.02,
    scale: 11_100.0,
    top: 133_000.0,
    radius: 3.39e6,
    gravity: 3.71,
};

/// Flies a ballistic fall from the top of `air` at `speed` metres per second, `slope` radians
/// below the horizon, until the craft is slow or lands, and returns the height, the speed and the glow of
/// each step.
fn fall(air: &Air, speed: f64, slope: f64) -> Vec<(f64, f64, f32)> {
    let mut pos = bevy::math::DVec2::new(0.0, air.radius + air.top - 1.0);
    let mut vel = bevy::math::DVec2::new(slope.cos(), -slope.sin()) * speed;
    let mut path = Vec::new();
    while vel.length() > 200.0 && pos.length() > air.radius && path.len() < 40_000 {
        let height = pos.length() - air.radius;
        let density = air.density(height);
        let pull = -pos.normalize() * air.gravity * (air.radius / pos.length()).powi(2);
        let drag = -vel * (density * vel.length() / (2.0 * BALLISTIC));
        vel += (pull + drag) * STEP;
        pos += vel * STEP;
        let glow = entry(density, vel.length(), Gas::NitrogenOxygen).heat;
        path.push((height, vel.length(), glow));
    }
    path
}

/// Returns the highest height at which the glow of `path` through `air` shows, and the height
/// of its strongest heating, judged by the heat flux itself and not by its share, which tops
/// out at 1.
fn marks(air: &Air, path: &[(f64, f64, f32)]) -> (f64, f64) {
    let start = path
        .iter()
        .find(|step| step.2 > SHEATH_FROM)
        .map_or(0.0, |step| step.0);
    let flux = |step: &(f64, f64, f32)| air.density(step.0).sqrt() * step.1.powi(3);
    let peak = path.iter().max_by(|a, b| flux(a).total_cmp(&flux(b)));
    (start, peak.map_or(0.0, |step| step.0))
}

/// From a low orbit into the air of the Earth the glow shows from the top of the air, which the
/// game puts at 101 km, and the heating peaks within a scale height of where Allen and Eggers put
/// it, near 56 km. Falling from its own low orbit into a thin air, the glow starts much lower,
/// near 55 km, and the heating peaks lower still, near 31 km.
#[test]
fn the_glow_starts_and_peaks_where_the_physics_says() {
    let slope = 6.0f64.to_radians();
    let (start, peak) = marks(&EARTH, &fall(&EARTH, 7800.0, slope));
    assert!(start > 95_000.0, "earth starts at {start}");
    assert!(
        (peak - EARTH.peak(slope)).abs() < EARTH.scale,
        "earth peaks at {peak}"
    );
    let (thin_start, thin_peak) = marks(&THIN, &fall(&THIN, 3500.0, slope));
    assert!(
        thin_start < 80_000.0 && thin_start > 40_000.0,
        "thin starts at {thin_start}"
    );
    assert!(
        (thin_peak - THIN.peak(slope)).abs() < THIN.scale,
        "thin peaks at {thin_peak}"
    );
    assert!(thin_start < start && thin_peak < peak);
}

/// A craft above the top of the air glows at no speed.
#[test]
fn nothing_glows_in_a_vacuum() {
    for speed in [300.0, 7800.0, 3.0e4] {
        let density = EARTH.density(EARTH.top + 1.0);
        assert_eq!(entry(density, speed, Gas::NitrogenOxygen).heat, 0.0);
    }
}

/// A climb that keeps under the cool speed of the air glows nowhere, from the sea to the top,
/// while a climb three times as fast glows low in the air: the same rule makes both.
#[test]
fn a_slow_climb_stays_dark_and_a_fast_one_glows() {
    let mut fast = 0.0f32;
    for metre in (0..100_000).step_by(500) {
        let density = EARTH.density(f64::from(metre));
        let cool = cool_speed(density).min(8000.0);
        assert_eq!(
            entry(density, cool * 0.95, Gas::NitrogenOxygen).heat,
            0.0,
            "{metre}"
        );
        fast = fast.max(entry(density, cool * 3.0, Gas::NitrogenOxygen).heat);
    }
    assert!(fast > 0.5, "a fast climb glows only {fast}");
}

/// Flown back up along the path of a fall, at the same speeds, the craft glows exactly as it did
/// on the way down at every height: only the air and the speed count, not the way.
#[test]
fn the_glow_is_the_same_down_and_up() {
    let path = fall(&EARTH, 7800.0, 6.0f64.to_radians());
    for (height, speed, down) in path.iter().rev().step_by(40) {
        let up = entry(EARTH.density(*height), *speed, Gas::NitrogenOxygen).heat;
        assert_eq!(up, *down, "at {height}");
    }
    assert!(path.iter().any(|step| step.2 > 0.9));
}
