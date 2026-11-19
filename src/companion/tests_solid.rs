//! The companion among drawn solids: a wall, a house, a roof and the ship's hull stop it, and it
//! slides round what stands between it and its station.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::tests_yard::{Yard, outside};
use super::tune::SHELL_RADIUS;

/// Thickness of the wall and of the roof, in metres: thinner than the companion, so only a
/// sweep stops it.
const THIN: f32 = 0.1;

/// Returns a settled yard, the player looking north, where the companion is and where its
/// station will be once the player walked `ahead` metres north.
fn settled(ahead: f32) -> (Yard, Vec3, Vec3) {
    let yard = Yard::new(Vec3::NEG_Z);
    let (at, station) = (yard.at(), yard.station() + Vec3::NEG_Z * ahead);
    (yard, at, station)
}

/// Walks the player at `velocity` for `walk` seconds, stands still `rest` seconds and returns
/// the whole path of the companion.
fn walked(yard: &mut Yard, velocity: Vec3, (walk, rest): (f32, f32)) -> Vec<Vec3> {
    let mut path = yard.walk(velocity, walk);
    path.extend(yard.walk(Vec3::ZERO, rest));
    path
}

/// A thin wall between the companion and its station, as the player walks on through it,
/// stops it at the wall: it never crosses and ends pressed against it.
#[test]
fn a_wall_stops_it_short_of_its_station() {
    let (mut yard, at, station) = settled(8.0);
    let wall = Vec3::new(at.x, at.y, (at.z + station.z) * 0.5);
    yard.solid(wall, Vec3::new(30.0, 12.0, THIN), RigidBody::Static);
    let path = walked(&mut yard, Vec3::NEG_Z * 4.0, (2.0, 3.0));
    let face = wall.z + THIN * 0.5 + SHELL_RADIUS;
    let deepest = path.iter().map(|p| p.z).fold(f32::MAX, f32::min);
    assert!(
        deepest > face - 0.03,
        "it reached {deepest} past the wall face {face}"
    );
    let end = path.last().unwrap().z;
    assert!(
        end < face + 0.2,
        "it stopped {} m short of the wall",
        end - face
    );
    assert!(
        yard.station().z < wall.z,
        "the station is not behind the wall"
    );
}

/// A house stands square across its straight way to its station: it slides round it toward
/// the player, never enters it, and rejoins its station on the far side.
#[test]
fn it_goes_round_a_house_to_rejoin() {
    let (mut yard, at, _) = settled(16.0);
    let size = Vec3::new(5.0, 8.0, 5.0);
    let house = Vec3::new(at.x + 1.2, at.y, at.z - 8.0);
    yard.solid(house, size, RigidBody::Static);
    let path = walked(&mut yard, Vec3::NEG_Z * 4.0, (4.0, 10.0));
    let nearest = path
        .iter()
        .map(|p| outside(*p, house, size))
        .fold(f32::MAX, f32::min);
    assert!(
        nearest > SHELL_RADIUS - 0.03,
        "it came {nearest} m from the house"
    );
    let gap = yard.at().distance(yard.station());
    assert!(gap < 1.0, "still {gap} m from its station");
}

/// A thin roof above it while the player rises through it: it stays under the roof.
#[test]
fn it_does_not_rise_through_a_roof() {
    let (mut yard, at, _) = settled(0.0);
    let roof = at + Vec3::Y * 1.5;
    yard.solid(roof, Vec3::new(80.0, THIN, 80.0), RigidBody::Static);
    let path = walked(&mut yard, Vec3::Y * 3.0, (2.0, 2.0));
    let under = roof.y - THIN * 0.5 - SHELL_RADIUS;
    let highest = path.iter().map(|p| p.y).fold(f32::MIN, f32::max);
    assert!(
        highest < under + 0.03,
        "it rose to {highest} past the roof at {under}"
    );
    assert!(
        yard.station().y > roof.y,
        "the station is not above the roof"
    );
}

/// The ship's hull, a kinematic box across its way, stops it like a wall.
#[test]
fn it_does_not_pass_through_the_ship_hull() {
    let (mut yard, at, station) = settled(8.0);
    let size = Vec3::new(6.4, 1.3, 1.8);
    let hull = Vec3::new(at.x, at.y, (at.z + station.z) * 0.5);
    yard.solid(hull, size, RigidBody::Kinematic);
    let path = walked(&mut yard, Vec3::NEG_Z * 4.0, (2.0, 3.0));
    let nearest = path
        .iter()
        .map(|p| outside(*p, hull, size))
        .fold(f32::MAX, f32::min);
    assert!(
        nearest > SHELL_RADIUS - 0.03,
        "it came {nearest} m from the hull"
    );
}
