//! Tests of what hides the stars, through the values the star shader gets: in space a sunlit hull
//! hides none and the field shows whole but for the glare around the star, at noon on the ground
//! of a world with air none shows, and at night the stars the eye sees all do.

use bevy::prelude::*;

use super::hiding;
use super::material::StarUniform;
use super::specks::star_light;
use crate::render::exposure::{Metered, lifted, target_ev};
use crate::render::palette::Palette;
use crate::render::skylight::{BAND, FIELD_DAY, GLARE, GLARE_SKY, band, glare, shown, veil};
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

/// Returns how far a star of light `flux` at the unit direction `dir` shows under the values
/// `stars`, by the rule of `starfield.wesl`: the sky between the horizon and the zenith, or the
/// glare of the star where that is brighter.
fn seen(flux: f32, dir: Vec3, stars: &StarUniform) -> f32 {
    let height = dir.dot(stars.dome.truncate()).clamp(0.0, 1.0);
    let sky = stars.base.x + (stars.dome.w - stars.base.x) * height.sqrt();
    let glared = glare(dir.dot(stars.sun.truncate()), stars.sun.w);
    shown(flux, sky.max(glared))
}

/// In open space, the sunlit hull or cockpit filling the frame and metered as a day, or twice as
/// bright, under any exposure the camera takes, the field shows whole but for the stars in the
/// glare cone around the star: at least 95 of 100 stars show, on both seeds.
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
            let palette = Palette {
                zenith: lifted(space.zenith, gain),
                horizon: lifted(space.horizon, gain),
                ..space
            };
            let values = hiding(&palette, veil(hull, 0.0), &sun, Vec3::Y);
            let lit = stars
                .iter()
                .filter(|(dir, flux)| seen(*flux, *dir, &values) > 0.5);
            let count = lit.count();
            assert!(
                count * 100 >= stars.len() * 95,
                "seed {seed}: {count} of {}",
                stars.len()
            );
        }
    }
}

/// Inside the glare cone of the star a bright star hides, and the band too; out of it, or with
/// the star behind the planet, it shows.
#[test]
fn the_glare_of_the_star_hides_the_stars_around_it() {
    let black = Palette::lit(0.9, Vec3::ONE).seen(&SunDir::default(), 0.0, Vec3::ONE);
    let near = Vec3::new(3.0_f32.to_radians().cos(), 3.0_f32.to_radians().sin(), 0.0);
    let far = Vec3::new(0.0, 0.0, 1.0);
    let sun = |lit: f32| SunDir {
        dir: Vec3::X,
        height: 0.0,
        lit,
    };
    let full = hiding(&black, 0.0, &sun(1.0), Vec3::Y);
    assert_eq!(seen(0.5, near, &full), 0.0);
    assert_eq!(seen(0.5, far, &full), 1.0);
    let shadowed = hiding(&black, 0.0, &sun(0.0), Vec3::Y);
    assert_eq!(seen(0.5, near, &shadowed), 1.0);
    assert_eq!(band(glare(near.x, 1.0)), 0.0);
    assert_eq!(band(glare(far.x, 1.0)), 1.0);
    assert_eq!(glare(GLARE.0, 1.0), 0.0);
    assert!(glare(GLARE.1, 1.0) >= GLARE_SKY);
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
    let values = hiding(&noon, veil(FIELD_DAY, 1.0), &sun, Vec3::Y);
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
    let palette = Palette {
        zenith: lifted(night.zenith, 2.0),
        horizon: lifted(night.horizon, 2.0),
        ..night
    };
    let values = hiding(&palette, veil(0.05 * FIELD_DAY, 1.0), &sun, Vec3::Y);
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

/// Both shaders hold the glare of the rule tested here, and the sky shader the edges of the band.
#[test]
fn the_shaders_have_the_glare_of_the_rule() {
    let glare = [
        format!("const GLARE_OUT: f32 = {:?};", GLARE.0),
        format!("const GLARE_IN: f32 = {:?};", GLARE.1),
        format!("const GLARE_SKY: f32 = {GLARE_SKY:?};"),
    ];
    let edges = [
        format!("const BAND_FULL: f32 = {:?};", BAND.0),
        format!("const BAND_GONE: f32 = {:?};", BAND.1),
    ];
    let stars = include_str!("../starfield.wesl");
    let sky = include_str!("../sky.wesl");
    for line in &glare {
        assert!(
            stars.contains(line.as_str()),
            "starfield.wesl lacks `{line}`"
        );
    }
    for line in glare.iter().chain(&edges) {
        assert!(sky.contains(line.as_str()), "sky.wesl lacks `{line}`");
    }
}
