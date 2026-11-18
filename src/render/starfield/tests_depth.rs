//! Depth tests: the stars slide past a moving camera by their true parallax, so the sky before
//! and after a warp is the true sky and a new build of the mesh changes nothing on screen.

use bevy::math::DVec3;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::Starfield;
use super::depth::{EXTENDED, Sighted, lit};
use super::specks::{GALAXY_ANGLE, Speck, shine, star_speck};
use crate::render::vantage::Vantage;
use crate::universe::{GLY, GalaxyId, StarSpec, Universe};

impl Sighted {
    /// Returns the speck as it shows once the camera has moved `drift` game light-years, in
    /// galaxy axes, from the place it was sighted from: where it stands now, and how large and
    /// bright the new distance makes it. This is the rule `starfield.wesl` applies to every
    /// vertex, written again here to be tested.
    pub fn drifted(&self, drift: Vec3) -> Speck {
        let far = self.speck.dir * self.range - drift;
        let now = far.length().max(1.0e-4);
        let (size, gain) = if self.flux >= EXTENDED {
            let size = self.speck.size * self.range / now;
            (size.min(GALAXY_ANGLE), 1.0)
        } else {
            let (size, shown) = shine(self.flux - 2.0 * (now / self.range).log10());
            (size, shown / shine(self.flux).1)
        };
        Speck {
            dir: far / now,
            size,
            color: self.speck.color * gain,
            soft: self.speck.soft,
        }
    }
}

/// Returns the home star of seed 11 and the stars within 40 light-years of it.
fn neighbourhood() -> (StarSpec, Vec<StarSpec>) {
    let universe = Universe::new(11);
    let star = universe.system(universe.home().star).star;
    let near = universe.stars_near(star.id.galaxy, star.pos, 40.0);
    (star, near)
}

/// Returns a speck one unit wide, `range` light-years away along `dir`, with the light `flux`.
fn sighted(dir: Vec3, range: f32, flux: f32) -> Sighted {
    let speck = Speck {
        dir,
        size: 0.01,
        color: Vec3::ONE,
        soft: 1.0,
    };
    Sighted { speck, range, flux }
}

/// A star sighted from one place and slid by the way to another is the star sighted from
/// there: same direction, same size, same colour. The mesh of the next build swaps in unseen.
#[test]
fn a_new_build_changes_nothing_on_screen() {
    let (home, stars) = neighbourhood();
    let from = home.pos + DVec3::X * 2.0 * GLY;
    let moved = DVec3::new(1.5, -0.7, 2.0);
    let others = stars.iter().filter(|star| star.id != home.id).take(300);
    let mut checked = 0;
    for star in others {
        let slid = Sighted::star(star, from).drifted(moved.as_vec3());
        let fresh = star_speck(star, from + moved * GLY);
        assert!(slid.dir.angle_between(fresh.dir) < 2e-4, "{star:?}");
        assert!((slid.size - fresh.size).abs() < 1e-5, "{star:?}");
        assert!((slid.color - fresh.color).length() < 2e-3 * fresh.color.length());
        checked += 1;
    }
    assert!(checked > 20, "only {checked} stars around the home star");
}

/// Without drift every speck is the speck that was sighted.
#[test]
fn nothing_moves_at_the_place_of_the_build() {
    let (home, stars) = neighbourhood();
    for star in stars.iter().filter(|star| star.id != home.id).take(100) {
        let seen = Sighted::star(star, home.pos);
        let still = seen.drifted(Vec3::ZERO);
        assert!((still.dir - seen.dir).length() < 1e-6 && still.size == seen.size);
        assert!((still.color - seen.color).length() < 1e-5 && still.soft == seen.soft);
    }
    let galaxy = sighted(Vec3::X, 3.0e5, EXTENDED);
    assert_eq!(galaxy.drifted(Vec3::ZERO), galaxy.speck);
}

/// A ship that flies five light-years sees a star two light-years off its path swing from
/// ahead to behind, a star thirty light-years off turn a little, and a far galaxy stand still.
/// Everything slides aft, and what comes closer grows.
#[test]
fn near_stars_pass_and_far_galaxies_stand() {
    let ahead = Vec3::NEG_Z;
    let beside = |off: f32, flux: f32| {
        let at = Vec3::new(off, 0.0, -2.5);
        sighted(at.normalize(), at.length(), flux)
    };
    let swing = |seen: &Sighted| {
        let (before, after) = (seen.speck.dir, seen.drifted(ahead * 5.0).dir);
        assert!(after.angle_between(ahead) > before.angle_between(ahead));
        assert!(after.x > 0.0 && after.y.abs() < 1e-6);
        before.angle_between(after)
    };
    let (near, far, galaxy) = (beside(2.0, 0.0), beside(30.0, 0.0), beside(3.0e5, EXTENDED));
    assert!(swing(&near) > 1.5 && swing(&far) < 0.2 && swing(&galaxy) < 2e-5);
    assert!(near.speck.dir.z < 0.0 && near.drifted(ahead * 5.0).dir.z > 0.0);
    let closing = sighted(ahead, 10.0, -3.0);
    let closer = closing.drifted(ahead * 8.0);
    assert!(closer.size > shine(closing.flux).0 && closer.color.x > closing.speck.color.x);
    let grown = sighted(ahead, 3.0e5, EXTENDED).drifted(ahead * 1.0e5);
    assert!((grown.size - 0.015).abs() < 1e-6 && grown.color == Vec3::ONE);
}

/// The mesh carries the range and the light of each speck on its four corners, and the drift
/// the shader slides them by is the way from the place of the build, in light-years.
#[test]
fn the_mesh_and_the_drift_carry_the_depth() {
    let specks = [
        sighted(Vec3::X, 4.0, -1.0),
        sighted(Vec3::Y, 2.0e5, EXTENDED),
    ];
    let mesh = lit(&specks);
    let Some(VertexAttributeValues::Float32x2(depths)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        panic!("depths missing");
    };
    assert_eq!(depths.len(), 8);
    assert!(depths[..4].iter().all(|depth| *depth == [4.0, -1.0]));
    assert!(depths[4..].iter().all(|depth| *depth == [2.0e5, EXTENDED]));
    let at = DVec3::new(3.0e12, -1.0e12, 5.0e11);
    let vantage = Vantage {
        galaxy: GalaxyId::default(),
        galactic: at + DVec3::new(2.0, 0.0, -1.0) * GLY,
        ..Vantage::default()
    };
    let field = |built| Starfield { built };
    assert_eq!(field(None).drift(&vantage), Vec3::ZERO);
    let here = field(Some((GalaxyId::default(), at))).drift(&vantage);
    assert!((here - Vec3::new(2.0, 0.0, -1.0)).length() < 1e-5);
}
