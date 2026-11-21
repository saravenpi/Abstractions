//! The shots of the planets seen from afar of the `far-bodies` stages: the home planet from three
//! heights and on both sides of the hand-over, the nearest planet and a moon from a few radii and
//! from far.

use bevy::math::DMat3;

use super::*;

/// Puts the ship in the frame of the star at `pos` metres from it, looking at `target`.
pub fn aim(world: &mut World, pos: DVec3, target: DVec3) {
    let star = world.resource::<Location>().body.star;
    let ahead = (target - pos).normalize();
    let up = ahead.any_orthonormal_vector();
    let rot = DQuat::from_mat3(&DMat3::from_cols(ahead.cross(up), up, -ahead));
    with_ship(world, |ship| {
        ship.frame = Frame::System(star);
        (ship.pos, ship.rot) = (pos, rot);
        (ship.vel, ship.spin, ship.landed) = (DVec3::ZERO, DVec3::ZERO, false);
    });
}

/// The home planet from 2,000 km.
pub fn far_home_2000km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    over_home(world, notes, frame, (2.0e6, "F1-home-2000km"))
}

/// The home planet from 20,000 km.
pub fn far_home_20000km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    over_home(world, notes, frame, (2.0e7, "F2-home-20000km"))
}

/// The home planet from 200,000 km.
pub fn far_home_200000km(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    over_home(world, notes, frame, (2.0e8, "F3-home-200000km"))
}

/// The nearest planet from three radii.
pub fn far_neighbour_near(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    visit(world, notes, frame, ("neighbour", 3.0, "F4-neighbour-3r"))
}

/// The nearest planet from three hundred radii.
pub fn far_neighbour_far(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    visit(
        world,
        notes,
        frame,
        ("neighbour", 300.0, "F5-neighbour-300r"),
    )
}

/// A moon from three radii.
pub fn far_moon_near(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    visit(world, notes, frame, ("moon", 3.0, "F6-moon-3r"))
}

/// A moon from three hundred radii.
pub fn far_moon_far(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    visit(world, notes, frame, ("moon", 300.0, "F7-moon-300r"))
}

/// Holds the ship over the day side of the home planet `radii` of its radii from its centre and
/// shoots `name`.
fn over_home_at(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    (radii, name): (f64, &str),
) -> Act {
    let radius = world.resource::<Planet>().globe.radius();
    over_home(world, notes, frame, ((radii - 1.0) * radius, name))
}

/// The home planet just inside the hand-over: the ground of the world over the sunk ball.
pub fn far_handover_in(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    over_home_at(world, notes, frame, (1.45, "F11-handover-in"))
}

/// The home planet just past the hand-over: the ball of the sky alone, to hold against F11.
pub fn far_handover_out(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    over_home_at(world, notes, frame, (1.6, "F12-handover-out"))
}
