//! Postcards on foot at the home base: the robot and the companion, the walk to the ship, the
//! ship on its pad, the base at night and the base seen from the hill above it.

use super::*;

/// Returns the home base, the ground at its wake point and the horizontal direction from its
/// hatch out to that point: the robot stands there with the base behind it.
fn porch(world: &World) -> Option<(Base, DVec3, DVec3)> {
    let base = world.get_resource::<Base>().copied()?;
    let feet = ground(world, base.wake, 0.0);
    let up = feet.normalize();
    let out = (base.wake - base.door)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    Some((base, feet, out))
}

/// Returns the point a picture of the robot and the companion is framed on: between the chest
/// of the robot standing at `feet` and the companion, when it floats within 4 m.
fn pair(world: &mut World, feet: DVec3) -> DVec3 {
    let chest = feet + feet.normalize() * 1.1;
    match buddy(world).filter(|at| at.distance(chest) < 4.0) {
        Some(at) => (chest + at) * 0.5,
        None => chest,
    }
}

/// The robot and the companion at the home base in golden light, from a low three-quarter
/// angle: the robot on the left third, the base behind it on the right, the sky above.
pub fn card_home(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some((_, feet, out)) = porch(world) else {
        return Act::Fail("no home base".into());
    };
    let up = feet.normalize();
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::Third, ViewMode::First);
        let toward = DQuat::from_axis_angle(up, 0.25) * out;
        stand(world, feet, feet + toward * 10.0, 0.0);
        side_light(world, GOLD, -(DQuat::from_axis_angle(up, 0.6) * out));
    }
    if t == 1 || t == settle() - 10 {
        let subject = pair(world, feet);
        let low = Stand {
            turn: 0.6,
            back: 4.4,
            rise: -0.6,
            third: -1.0,
            sky: 0.7,
        };
        let (eye, target) = framed(world, subject, out, low);
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    still(world, t, "p01-home")
}

/// Close on the robot and the companion side by side, waist up, the faces lit by the low star.
pub fn card_pair(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some((_, feet, out)) = porch(world) else {
        return Act::Fail("no home base".into());
    };
    let up = feet.normalize();
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::Third, ViewMode::First);
        stand(world, feet, feet + out * 10.0, 0.0);
        clock_at(world, GOLD, true);
    }
    if t == settle() - 12 {
        let head = feet + up * 1.55;
        let friend = buddy(world).filter(|at| at.distance(head) < 5.0);
        let friend = friend.unwrap_or(head + up.cross(out) * 1.0);
        let apart = (friend - head).reject_from(up);
        let mut facing = up.cross(apart.normalize_or(out));
        if facing.dot(sunward(world, feet)) < 0.0 {
            facing = -facing;
        }
        let middle = (head + friend) * 0.5;
        let eye = middle + facing * (1.4 + 1.5 * apart.length()).max(2.0) + up * 0.05;
        stand(world, feet, eye, 0.0);
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, middle);
    }
    still(world, t, "p02-pair")
}

/// The ship on its pad, canopy open, and the robot walking to it, seen from behind its right
/// shoulder: the robot on the left third, the ship ahead on the right.
pub fn card_walk(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    let forward = bound(world, |s| s.controls.keys.forward);
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::Third, ViewMode::First);
        let start = ground(
            world,
            hull.pos + hull.rot * DVec3::new(-5.0, 0.0, -9.5),
            0.0,
        );
        let hatch = hull.pos + hull.rot * DVec3::new(-1.7, -1.2, -1.5);
        let up = start.normalize();
        let ahead = (hatch - start)
            .reject_from(up)
            .normalize_or(up.any_orthonormal_vector());
        let right = ahead.cross(up);
        stand(world, start, hatch, 0.0);
        side_light(world, GOLD, ahead);
        let eye = ground(world, start - ahead * 3.2 + right * 1.9, 0.8);
        let target = start + ahead * 7.0 + right * 2.0 + up * 1.2;
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    match t.saturating_sub(settle()) {
        0 => return still(world, t.min(settle() - 1), "p03-walk"),
        1 => key(world, forward, true),
        26 => shot(world, "p03-walk"),
        28 => {
            key(world, forward, false);
            return Act::Next;
        }
        _ => {}
    }
    Act::Wait
}

/// The ship on its pad from a low front three-quarter, the robot beside the cockpit, with the
/// star on the horizon.
pub fn card_ship(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some(hull) = ship(world) else {
        return Act::Fail("no ship".into());
    };
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::Third, ViewMode::Third);
        let beside = hull.pos + hull.rot * DVec3::new(-2.3, -1.2, -2.4);
        stand(
            world,
            beside,
            beside + hull.rot * DVec3::new(-4.0, 0.0, -10.0),
            0.0,
        );
        let (eye, aim) = (Vec3::new(-5.4, -0.6, -7.6), Vec3::new(0.9, 0.55, -0.4));
        side_light(world, RIM, hull.rot * (aim - eye).as_dvec3());
        world.resource_mut::<Eye>().0 = Some((eye, aim));
    }
    still(world, t, "p04-ship")
}

/// The base at night from 13 m, low: its lamps and the light of the companion on the lower
/// right third, the stars over it.
pub fn card_night_base(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some((base, feet, out)) = porch(world) else {
        return Act::Fail("no home base".into());
    };
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::Third, ViewMode::First);
        let up = feet.normalize();
        stand(
            world,
            feet,
            feet + DQuat::from_axis_angle(up, -0.3) * out * 10.0,
            0.0,
        );
        clock_at(world, NIGHT, false);
        let wide = Stand {
            turn: -0.45,
            back: 13.0,
            rise: -1.2,
            third: 1.0,
            sky: 1.0,
        };
        let subject = (base.door + feet) * 0.5 + up * 1.6;
        let (eye, target) = framed(world, subject, out, wide);
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    still(world, t, "p14-night-base")
}

/// The base from the highest ground within 350 m of it, in golden light: the base and the ship
/// on its pad small on the right third, the land and the sea behind. The robot stands behind
/// the camera, out of the picture.
pub fn card_overlook(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::First, ViewMode::First);
        let middle = (base.centre + base.pad) * 0.5;
        let rings = [120.0, 180.0, 250.0, 350.0];
        let around = compass(middle, 24).into_iter();
        let spots = around.flat_map(|d| rings.map(|r| ground(world, middle + d * r, 0.0)));
        let hill = spots
            .max_by(|a, b| a.length().total_cmp(&b.length()))
            .unwrap_or(middle);
        let up = hill.normalize();
        let away = (hill - middle)
            .reject_from(up)
            .normalize_or(up.any_orthonormal_vector());
        let feet = ground(world, hill + away * 6.0, 0.0);
        stand(world, feet, middle, 0.0);
        side_light(world, GOLD, -away);
        let from_hill = Stand {
            turn: 0.0,
            back: hill.distance(middle),
            rise: hill.length() - middle.length() + 2.2,
            third: 1.0,
            sky: -0.6,
        };
        let (eye, target) = framed(world, middle, away, from_hill);
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    still(world, t, "p19-overlook")
}
