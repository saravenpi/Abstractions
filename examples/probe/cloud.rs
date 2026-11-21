//! Flying through a cloud: the ship climbs through the base of an overcast cloud band at 150 m/s,
//! seen from the chase camera and from the seat, with a shot every 0.2 s of the game, then again
//! at night. The puffs should stream past at their places, thicken to a lit white-grey inside and
//! open with gaps on the way; drops gather on the canopy and stream back; nothing goes dark by
//! day, and at night the lamps of the ship light the mist near it.

use abstractions::render::{CloudCover, CloudCrossing};
use abstractions::ship::SEAT;

use super::*;

/// Speed of the climb through the cloud, in metres per second, and its slope in radians.
const CLIMB: (f64, f64) = (150.0, 0.5);

/// Metres under the base of the band the climb starts and over it that it ends.
const SPAN: (f64, f64) = (200.0, 600.0);

/// Flies the ship straight through the base of the band, `t` frames into the stage, with the
/// cover forced overcast. Returns its height above the sea.
fn fly(world: &mut World, t: u32) -> f64 {
    let body = world.resource::<Location>().body;
    let spec = world.resource::<Universe>().planet(body);
    let up = world
        .get_resource::<Base>()
        .map_or(DVec3::Y, |b| b.pad)
        .normalize();
    let east = DVec3::Y.cross(up).normalize_or(up.any_orthonormal_vector());
    let along = f64::from(t) * FLIGHT * CLIMB.0;
    let height = spec.cloud_base - SPAN.0 + along * CLIMB.1.sin();
    let radius = world.resource::<Planet>().globe.radius();
    let way = (east * CLIMB.1.cos() + up * CLIMB.1.sin()).normalize();
    let over = (up * (radius + height) + east * along * CLIMB.1.cos()).normalize();
    hover(world, over, height, (east, -CLIMB.1));
    with_ship(world, |ship| ship.vel = way * CLIMB.0);
    height - spec.cloud_base
}

/// Climbs through the cloud with the camera at `eye` in hull axes, from outside when `out`,
/// shooting `name` every 0.2 s: by day, or at night with the headlight on when `night`.
fn through(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    ((eye, out, name), night): (((Vec3, Vec3), bool, &str), bool),
) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        match night && nightfall(world, notes) {
            true => with_ship(world, |ship| ship.headlight = true),
            false => noon(world),
        }
        world.resource_mut::<Eye>().0 = Some(eye);
        view(
            world,
            ViewMode::First,
            [ViewMode::First, ViewMode::Third][usize::from(out)],
        );
        world.resource_mut::<CloudCover>().0 = Some(0.95);
        pace(world, FLIGHT);
    }
    let above = fly(world, t);
    if t % 2 == 0 {
        let depth = world
            .get_resource::<CloudCrossing>()
            .map_or(0.0, |c| c.depth);
        eprintln!("PROBE {name} {t}: {above:.0} m from the base, in cloud {depth:.2}");
        shot(world, &format!("{name}-{t:04}"));
    }
    match above > SPAN.1 {
        true => {
            world.resource_mut::<CloudCover>().0 = None;
            with_ship(world, |ship| ship.headlight = false);
            daybreak(world, notes);
            landed(world, notes)
        }
        false => Act::Wait,
    }
}

/// The climb through the cloud from the chase camera.
pub fn cloud(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    through(world, notes, frame, ((CHASE, true, "C-chase"), false))
}

/// The climb through the cloud from the seat, looking ahead through the canopy.
pub fn cloud_seat(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let eye = SEAT.as_vec3();
    let ahead = (eye, eye + Quat::from_rotation_x(-0.1) * Vec3::NEG_Z);
    through(world, notes, frame, ((ahead, false, "C-seat"), false))
}

/// The climb through the cloud at night with the headlight on, from the chase camera: the nav
/// lights, the strobes and the beam should light the puffs near the ship.
pub fn cloud_night(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    through(world, notes, frame, ((CHASE, true, "C-night"), true))
}
