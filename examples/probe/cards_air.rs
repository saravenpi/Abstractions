//! Postcards with the ship: the cockpit, the lift-off, a banked turn over a coast, the peaks
//! from high up, low orbit, open space with the star at the limb, and the pad at night.

use abstractions::ship::SEAT;

use super::*;

/// Sets what a card in the ship shows: the field of view, and the pilot drawn whole from
/// outside or the cockpit from the seat.
fn aboard(world: &mut World, degrees: u32, outside: bool) {
    lens(world, degrees);
    let seen = [ViewMode::First, ViewMode::Third][usize::from(outside)];
    view(world, ViewMode::First, seen);
}

/// Flies the ship straight at `speed` metres per second with the main engines at full, for the
/// frame: the plume is lit and the ship keeps the attitude the card gave it.
fn cruise(world: &mut World, speed: f64) {
    with_ship(world, |ship| {
        ship.vel = ship.forward() * speed;
        (ship.spin, ship.throttle) = (DVec3::ZERO, 1.0);
    });
}

/// The end of a card that flies: after the settle time the ship flies straight for `run` frames
/// at `speed` and the shot `name` is taken. Returns nothing before that, while the card still
/// holds the ship in place.
fn flown(world: &mut World, t: u32, (name, run, speed): (&str, u32, f64)) -> Option<Act> {
    match t.saturating_sub(settle()) {
        0 => return None,
        1 => pace(world, FLIGHT),
        n if n == run => shot(world, name),
        n if n == run + 2 => {
            with_ship(world, |ship| ship.throttle = 0.0);
            return Some(Act::Next);
        }
        _ => {}
    }
    cruise(world, speed);
    Some(Act::Wait)
}

/// Returns the unit direction of the planet position `point` and the height over the sea of a
/// ship `lift` metres over the ground there, as `hover` takes them.
pub fn above_ground(world: &World, point: DVec3, lift: f64) -> (DVec3, f64) {
    let radius = world.resource::<Planet>().globe.radius();
    (
        point.normalize(),
        ground(world, point, lift).length() - radius,
    )
}

/// Returns the shore nearest the pad and the horizontal direction from the pad out to it.
fn coast(world: &World) -> Option<(DVec3, DVec3)> {
    let pad = world.get_resource::<Base>()?.pad;
    let sea = find_shore(world, pad)?;
    let shore = ground(world, sea * pad.length(), 0.0);
    let up = shore.normalize();
    let out = (shore - pad)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    Some((shore, out))
}

/// First person in the cockpit, the displays lit, flying low toward the coast in golden light.
pub fn card_cockpit(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let Some((shore, out)) = coast(world) else {
        return Act::Fail("no shore within 2 km of the pad".into());
    };
    if t == 1 {
        aboard(world, 80, false);
        side_light(world, GOLD, out);
        let eye = SEAT.as_vec3();
        let ahead = Quat::from_rotation_x(-0.18) * Vec3::NEG_Z;
        world.resource_mut::<Eye>().0 = Some((eye, eye + ahead));
    }
    if let Some(act) = flown(world, t, ("p05-cockpit", 12, 70.0)) {
        return act;
    }
    let (over, height) = above_ground(world, shore - out * 500.0, 120.0);
    hover(world, over, height, (out, 0.1));
    still(world, t.min(settle() - 1), "p05-cockpit")
}

/// The lift-off from the pad with its dust, from a camera on the ground in front of the nose:
/// the ship on the right third, room above it to rise into.
pub fn card_liftoff(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let rise = bound(world, |s| s.controls.keys.ship.rise);
    if t == 1 {
        aboard(world, 60, true);
        let subject = hull.pos + hull.pos.normalize() * 1.2;
        let low = Stand {
            turn: 0.0,
            back: 9.5,
            rise: -3.0,
            third: 1.0,
            sky: 0.8,
        };
        let from = hull.rot * DVec3::new(-0.75, 0.0, -0.65);
        let (eye, target) = framed(world, subject, from, low);
        side_light(world, GOLD, subject - eye);
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    let climb = t.saturating_sub(settle());
    let high = world.resource::<ShipTelemetry>().altitude_ground;
    match (climb, notes.shot_at.take()) {
        (0, _) => return still(world, t.min(settle() - 1), "p06-liftoff"),
        (_, Some(_)) => {
            key(world, rise, false);
            return Act::Next;
        }
        (1, _) => pace(world, FLIGHT),
        (5, _) => key(world, rise, true),
        (n, _) if n > 15 && (high > 2.2 || n > 80) => {
            shot(world, "p06-liftoff");
            notes.shot_at = Some(t);
        }
        _ => {}
    }
    Act::Wait
}

/// The ship in a banked turn 160 m over the coast, the plume lit, from a level chase camera
/// above and to the sea side: the ship on the left third showing its back, the coast running
/// away ahead, in golden light.
pub fn card_coast(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let Some((shore, out)) = coast(world) else {
        return Act::Fail("no shore within 2 km of the pad".into());
    };
    let along = out.cross(shore.normalize());
    if t == 1 {
        aboard(world, 60, true);
        side_light(world, GOLD, along);
        *world.resource_mut::<Tripod>() = Tripod::Follow {
            robot: false,
            heading: along,
            eye: Vec3::new(-6.5, 2.6, 5.5),
            aim: Vec3::new(1.8, -0.3, -2.0),
        };
    }
    if let Some(act) = flown(world, t, ("p07-coast", 22, 105.0)) {
        return act;
    }
    let (over, height) = above_ground(world, shore, 160.0);
    hover(world, over, height, (along, 0.0));
    with_ship(world, |ship| ship.rot *= DQuat::from_rotation_z(0.55));
    still(world, t.min(settle() - 1), "p07-coast")
}

/// The ship 14 km over the highest ground within 60 km of the base, at noon, from behind and
/// above: the peaks under it and the curved horizon on the upper third.
pub fn card_peaks(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        aboard(world, 60, true);
        noon(world);
        let pad = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.pad);
        let rings = (1..=15).map(|k| f64::from(k) * 4000.0);
        let spots = rings.flat_map(|r| compass(pad, 24).into_iter().map(move |d| pad + d * r));
        let peak = spots
            .map(|p| ground(world, p, 0.0))
            .max_by(|a, b| a.length().total_cmp(&b.length()))
            .unwrap_or(pad);
        let heading = peak.normalize().cross(sunward(world, peak));
        notes.spots = vec![peak, heading];
        *world.resource_mut::<Tripod>() = Tripod::Follow {
            robot: false,
            heading,
            eye: Vec3::new(4.5, 3.2, 10.0),
            aim: Vec3::new(-2.0, -1.5, -8.0),
        };
    }
    if let Some(act) = flown(world, t, ("p08-peaks", 20, 300.0)) {
        return act;
    }
    let (peak, heading) = (notes.spots[0], notes.spots[1]);
    let (over, height) = above_ground(world, peak, 14.0e3);
    hover(world, over, height, (heading, 0.0));
    still(world, t.min(settle() - 1), "p08-peaks")
}

/// Holds the ship in space for a card, the engines lit, and shoots `name` after the settle time.
fn held(
    world: &mut World,
    t: u32,
    name: &str,
    (dir, height, toward, pitch): (DVec3, f64, DVec3, f64),
) -> Act {
    hover(world, dir, height, (toward, pitch));
    with_ship(world, |ship| ship.throttle = 1.0);
    if t > settle() {
        with_ship(world, |ship| ship.throttle = 0.0);
        return Act::Next;
    }
    still(world, t.min(settle() - 1), name);
    if t == settle() {
        shot(world, name);
    }
    Act::Wait
}

/// The planet from low orbit on its day side, the ship small on the upper left third, the
/// plume lit, the ground and the seas filling the rest.
pub fn card_orbit(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    let pad = world.get_resource::<Base>().map_or(DVec3::Y, |b| b.pad);
    if t == 1 {
        aboard(world, 60, true);
        noon(world);
        let eye = (Vec3::new(-34.0, 9.0, 30.0), Vec3::new(6.0, -9.0, -6.0));
        world.resource_mut::<Eye>().0 = Some(eye);
        notes.spots = vec![pad.normalize().cross(sunward(world, pad))];
    }
    let body = world.resource::<Location>().body;
    let air = world.resource::<Universe>().planet(body).atmosphere_height;
    held(
        world,
        t,
        "p12-orbit",
        (pad.normalize(), air * 1.6, notes.spots[0], 0.0),
    )
}

/// The ship in space from behind, flying toward the star where it stands just over the limb of
/// the planet: the limb on the lower third, the star on the upper left, the ship a silhouette
/// with its plume on the right.
pub fn card_star(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        aboard(world, 60, true);
        let (pad, light) = (
            world.get_resource::<Base>().map_or(DVec3::Y, |b| b.pad),
            star(world),
        );
        let across = pad
            .normalize()
            .reject_from(light)
            .normalize_or(light.any_orthonormal_vector());
        let over = (light * 0.12 + across * (1.0_f64 - 0.12 * 0.12).sqrt()).normalize();
        notes.spots = vec![over, light.reject_from(over).normalize_or(across)];
        let eye = (Vec3::new(3.2, 1.4, 9.5), Vec3::new(-2.2, 0.9, -30.0));
        world.resource_mut::<Eye>().0 = Some(eye);
    }
    let radius = world.resource::<Planet>().globe.radius();
    let (over, toward) = (notes.spots[0], notes.spots[1]);
    held(world, t, "p13-star", (over, radius * 0.08, toward, 0.14))
}

/// The ship on its pad at night from a low front three-quarter, the pilot in the seat and the
/// cockpit lit, the stars over it.
pub fn card_night_ship(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, true) else {
        return card_wait(frame);
    };
    if t == 1 {
        aboard(world, 60, true);
        clock_at(world, NIGHT, false);
        let eye = (Vec3::new(-5.6, -0.2, -7.4), Vec3::new(0.8, 1.1, -0.6));
        world.resource_mut::<Eye>().0 = Some(eye);
    }
    still(world, t, "p15-night-ship")
}
