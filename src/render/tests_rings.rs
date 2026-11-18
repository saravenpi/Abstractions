//! Ring tests: the rings cover their band in ringlets, the planet shades the rings behind it,
//! the rings lay a band of shadow on the planet, and a moon hides the star from its planet.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::atmosphere::shadow;
use super::moons::{coverage, eclipse, ring_shadow};
use crate::universe::{BodyId, PlanetSpec, Rings, Universe};

/// Radius in metres of the ringed planet of these tests.
const RADIUS: f64 = 70_000.0;

/// Returns rings from 1.4 to 2.4 planet radii, as dense as `density`.
fn rings(density: f32) -> Rings {
    Rings {
        inner: 1.4 * RADIUS,
        outer: 2.4 * RADIUS,
        density,
        color: Vec3::new(0.8, 0.72, 0.6),
    }
}

/// No ring stops light inside the inner edge or past the outer one; inside the band the cover
/// never passes the density, and the ringlets make it swing from thin to thick.
#[test]
fn rings_cover_their_band() {
    let rings = rings(0.8);
    for radius in [0.0, 1.0, 1.3, 1.399, 2.401, 3.0, 100.0] {
        assert_eq!(coverage(&rings, radius * RADIUS), 0.0, "{radius}");
    }
    let band: Vec<f32> = (1..200)
        .map(|i| coverage(&rings, (1.4 + f64::from(i) / 200.0) * RADIUS))
        .collect();
    assert!(band.iter().all(|cover| (0.0..=0.8).contains(cover)));
    let (least, most) = band[20..180]
        .iter()
        .fold((1.0_f32, 0.0_f32), |(a, b), c| (a.min(*c), b.max(*c)));
    assert!(most > 0.6 && least < 0.5 * most, "{least} {most}");
    assert_eq!(
        coverage(
            &Rings {
                density: 0.0,
                ..rings
            },
            1.9 * RADIUS
        ),
        0.0
    );
}

/// A point of the rings straight behind the planet from the star lies in its shadow, the same
/// point on the day side does not, and the edge of the shadow is the edge of the planet.
#[test]
fn planet_shadows_its_rings() {
    let sun = DVec3::new(1.0, 0.25, 0.3).normalize();
    let flat = DVec3::new(sun.x, 0.0, sun.z).normalize();
    let behind = -flat * 1.9 * RADIUS;
    assert_eq!(shadow(behind, sun, RADIUS), 0.0);
    assert_eq!(shadow(-behind, sun, RADIUS), 1.0);
    let side = DVec3::new(-flat.z, 0.0, flat.x) * 1.9 * RADIUS;
    assert_eq!(shadow(side, sun, RADIUS), 1.0);
}

/// With the star above the ring plane, a point of the southern hemisphere whose line to the
/// star crosses the rings is darker, a northern point is not, and a point whose line crosses
/// the gap inside the rings is not either.
#[test]
fn rings_shadow_the_planet() {
    let sun = DVec3::new(1.0, 0.5, 0.0).normalize();
    let rings = rings(0.8);
    let through = |radius: f64| {
        let hit = DVec3::new(radius * RADIUS, 0.0, 0.0);
        let below = hit - sun * (hit.length() * 0.3);
        (below, ring_shadow(below, sun, &rings))
    };
    let shaded = (150..230).map(|i| through(f64::from(i) / 100.0).1);
    assert!(shaded.clone().all(|share| share < 1.0) && shaded.fold(1.0, f32::min) < 0.5);
    assert_eq!(through(1.2).1, 1.0);
    let south = DVec3::new(0.3, -0.6, 0.0).normalize() * RADIUS;
    let north = DVec3::new(0.3, 0.6, 0.0).normalize() * RADIUS;
    assert!(ring_shadow(south, sun, &rings) < 1.0);
    assert_eq!(ring_shadow(north, sun, &rings), 1.0);
}

/// A moon between its planet and the star hides the star from the point of the planet behind
/// it and from nothing on the far side; without the moon nothing is hidden, and the planet and
/// its rings, which lie in the plane of the star, leave the moon in full light.
#[test]
fn bodies_of_one_planet_shade_each_other() {
    let universe = Universe::new(7);
    let mut planet = universe.planet(universe.home());
    planet.radius = RADIUS;
    planet.rings = Some(rings(0.9));
    let center = planet.position(0.0);
    let sun = (-center).normalize();
    let moon = PlanetSpec {
        id: BodyId {
            moon: 1,
            ..planet.id
        },
        radius: RADIUS * 0.4,
        orbit_radius: RADIUS * 10.0,
        orbit_phase: sun.z.atan2(sun.x),
        host: Some(planet.orbit()),
        rings: None,
        ..planet.clone()
    };
    let family = [planet.clone(), moon.clone()];
    let noon = center + sun * RADIUS;
    assert!((moon.position(0.0) - center).normalize().dot(sun) > 0.9999);
    assert_eq!(eclipse(noon, family.iter(), planet.id, 0.0), 0.0);
    let dusk = center + DVec3::Y.cross(sun) * RADIUS;
    assert_eq!(eclipse(dusk, family.iter(), planet.id, 0.0), 1.0);
    assert_eq!(eclipse(noon, family[..1].iter(), planet.id, 0.0), 1.0);
    assert_eq!(
        eclipse(moon.position(0.0), family.iter(), moon.id, 0.0),
        1.0
    );
}
