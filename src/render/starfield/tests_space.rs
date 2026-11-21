//! Tests of what hides the stars, through the values the star shader gets: in space a sunlit hull
//! hides none and the field shows whole but where the glow of the star outshines it, at noon on
//! the ground of a world with air none shows, and at night the stars the eye sees all do.

use bevy::prelude::*;

use super::hiding;
use super::material::StarUniform;
use super::specks::{shine, star_light};
use crate::render::exposure::{Metered, lifted, target_ev};
use crate::render::glare::{glow, outshone, strength};
use crate::render::palette::Palette;
use crate::render::skylight::{FIELD_DAY, shown, veil};
use crate::render::sun::SunDir;
use crate::universe::{LY, Universe};

/// The seeds the probe plays: the postcard home and the default one.
const SEEDS: [u64; 2] = [57, 7];

/// Returns the stars the camera sees from the home star of `seed`, the home star left out: each
/// as its unit direction in galaxy axes and the light that arrives from it.
fn field(seed: u64) -> Vec<(Vec3, f32)> {
    let universe = Universe::new(seed);
    let home = universe.system(universe.home().star).star;
    let near = universe.stars_near(home.id.galaxy, home.pos, 40.0);
    near.iter()
        .filter(|star| (star.pos - home.pos).length() > 0.01 * LY)
        .map(|star| {
            let dir = (star.pos - home.pos).normalize().as_vec3();
            (dir, star_light(star, home.pos).1)
        })
        .collect()
}

/// Returns how far a white star of light `flux` at the unit direction `dir` shows under the
/// values `stars`, by the rule of `starfield.wesl`: the sky between the horizon and the zenith
/// hides it by its flux, and the glow of the star outshines what is left of its light.
fn seen(flux: f32, dir: Vec3, stars: &StarUniform) -> f32 {
    let height = dir.dot(stars.dome.truncate()).clamp(0.0, 1.0);
    let sky = stars.base.x + (stars.dome.w - stars.base.x) * height.sqrt();
    let kept = shown(flux, sky);
    let around = glow(dir.dot(stars.sun.truncate()), stars.sun.w);
    kept * outshone(shine(flux).1 * kept, around)
}

/// Returns `palette` with its sky lifted by the exposure gain `gain`, as the game draws it.
fn brightened(palette: Palette, gain: f32) -> Palette {
    Palette {
        zenith: lifted(palette.zenith, gain),
        horizon: lifted(palette.horizon, gain),
        ..palette
    }
}

/// In open space, the sunlit hull or cockpit filling the frame and metered as a day, or twice as
/// bright, under any exposure the camera takes, the field shows whole but for the stars the glow
/// of the star outshines close to it: at least 95 of 100 stars show, on both seeds.
#[test]
fn a_sunlit_hull_in_space_leaves_the_field_whole() {
    let sun = SunDir {
        dir: Vec3::X,
        height: 0.0,
        lit: 1.0,
    };
    let space = Palette::lit(0.9, Vec3::ONE).seen(&sun, 0.0, Vec3::ONE);
    let metered = Metered {
        ev: target_ev(Palette::SPACE_LUX),
    };
    for seed in SEEDS {
        let stars = field(seed);
        assert!(stars.len() > 100, "seed {seed}: {} stars", stars.len());
        for (gain, hull) in [(metered.gain(), FIELD_DAY), (2.0, 2.0 * FIELD_DAY)] {
            let palette = brightened(space, gain);
            let hides = (veil(hull, 0.0), strength(sun.lit, 0.0, gain));
            let values = hiding(&palette, hides, &sun, Vec3::Y);
            let count = stars
                .iter()
                .filter(|(d, f)| seen(*f, *d, &values) > 0.5)
                .count();
            assert!(
                count * 100 >= stars.len() * 95,
                "seed {seed}: {count} of {}",
                stars.len()
            );
        }
    }
}

/// At noon on the ground of a world with home air, the scene metered as the reference day, no
/// star of the field shows anywhere over the horizon, on both seeds.
#[test]
fn noon_on_the_ground_shows_no_star() {
    let sun = SunDir {
        dir: Vec3::Y,
        height: 1.0,
        lit: 1.0,
    };
    let noon = Palette::lit(1.0, Vec3::ONE).seen(&sun, 1.0, Vec3::ONE);
    let values = hiding(
        &noon,
        (veil(FIELD_DAY, 1.0), strength(1.0, 1.0, 1.0)),
        &sun,
        Vec3::Y,
    );
    for seed in SEEDS {
        for (dir, flux) in field(seed) {
            let up = Vec3::new(dir.x, dir.y.abs().max(0.02), dir.z).normalize();
            assert!(
                seen(flux, up, &values) < 0.05,
                "seed {seed}: flux {flux} at {up}"
            );
        }
    }
}

/// At night on the ground every star the eye sees high in the sky shows in full, the camera at
/// its brightest exposure and the dim ground at a twentieth of the day.
#[test]
fn night_on_the_ground_shows_the_stars() {
    let sun = SunDir {
        dir: Vec3::NEG_Y,
        height: -0.6,
        lit: 0.0,
    };
    let night = Palette::lit(-0.6, Vec3::ONE).seen(&sun, 1.0, Vec3::ONE);
    let palette = brightened(night, 2.0);
    let dim = (veil(0.05 * FIELD_DAY, 1.0), strength(0.0, 1.0, 2.0));
    let values = hiding(&palette, dim, &sun, Vec3::Y);
    for seed in SEEDS {
        let high = field(seed)
            .into_iter()
            .map(|(dir, flux)| (dir.with_y(dir.y.abs()), flux));
        for (dir, flux) in high.filter(|(dir, flux)| dir.y > 0.5 && *flux > -2.5) {
            assert!(
                seen(flux, dir, &values) > 0.99,
                "seed {seed}: flux {flux} at {dir}"
            );
        }
    }
}
