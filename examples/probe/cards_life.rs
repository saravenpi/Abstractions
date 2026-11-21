//! Postcards with the life of the planet: a herd grazing with the robot crouched in front of
//! it, a hunter or a large animal faced with a tool in hand, and the wingsuit over a valley.

use abstractions::life::Guild;

use super::*;

/// Metres from the robot to the animal of the herd it watches.
const HERD_RANGE: f64 = 13.0;

/// Metres between the robot and the animal it faces.
const BEAST_RANGE: f64 = 6.0;

/// Returns the ground under the centre of the herd of walkers nearest the wake point, within
/// 3 km: a herd of large ones first, of three members at least.
fn herd_near(world: &World) -> Option<DVec3> {
    let globe = world.resource::<Terrain>().globe().clone();
    let life = globe.life()?;
    let wake = world.get_resource::<Base>()?.wake.normalize();
    let sample = |dir: DVec3| site_of(&globe, dir);
    let (height, light) = (|dir: DVec3| globe.height(dir), |_: DVec3, _: f64| 1.0);
    let ground = abstractions::life::herd::Ground {
        sample: &sample,
        height: &height,
        light: &light,
    };
    let search = abstractions::life::herd::Search {
        at: wake,
        radius: 3000.0,
        planet: globe.radius(),
        time: world.resource::<abstractions::space::Clock>().seconds + 4.0,
    };
    let herds = abstractions::life::herd::near(life, &ground, &search);
    let kind = |index: u16| &life.species[usize::from(index)];
    let rank = |h: &&abstractions::life::herd::Herd| {
        let small = if big_walker(kind(h.species)) {
            0.0
        } else {
            1.0e7
        };
        small + h.centre.angle_between(wake) * globe.radius()
    };
    let walkers = herds
        .iter()
        .filter(|h| walker(kind(h.species)) && h.count >= 3);
    let herd = walkers.min_by(|a, b| rank(a).total_cmp(&rank(b)))?;
    Some(ground_at(world, herd.centre * globe.radius()))
}

/// A herd grazing in golden light, the robot crouched in the foreground on the left third, seen
/// from behind its shoulder, the light across the picture.
pub fn card_herd(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let crouch = bound(world, |s| s.controls.keys.crouch);
    if t == 1 {
        lens(world, 60);
        view(world, ViewMode::Third, ViewMode::First);
        clock_at(world, GOLD, true);
        let Some(herd) = herd_near(world) else {
            return Act::Fail("no herd of walkers within 3 km of the base".into());
        };
        let across = herd.normalize().cross(sunward(world, herd));
        let far = beside(world, herd, across, 40.0);
        stand(world, far, herd, 0.0);
        notes.spots = vec![herd];
    }
    if t == settle() - 40 {
        let herd = notes.spots[0];
        let list = animals(world);
        let near = list
            .iter()
            .map(|a| a.1)
            .filter(|at| at.distance(herd) < 150.0);
        let at = near
            .min_by(|a, b| a.distance(herd).total_cmp(&b.distance(herd)))
            .unwrap_or(herd);
        let up = at.normalize();
        let feet = beside(world, at, up.cross(sunward(world, at)), HERD_RANGE);
        let ahead = (at - feet)
            .reject_from(up)
            .normalize_or(up.any_orthonormal_vector());
        let right = ahead.cross(up);
        stand(world, feet, at, 0.0);
        key(world, crouch, true);
        let eye = ground(world, feet - ahead * 2.8 + right * 1.3, 0.75);
        let target = feet + ahead * 7.0 - right * 0.8 + up * 0.9;
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, target);
    }
    if t == settle() + 2 {
        key(world, crouch, false);
        return Act::Next;
    }
    still(world, t.min(settle() - 1), "p09-herd");
    if t == settle() {
        shot(world, "p09-herd");
    }
    Act::Wait
}

/// Returns the drawn walker to face: a hunter or an ambusher first, then the heaviest.
fn beast(world: &mut World) -> Option<(Entity, DVec3)> {
    let globe = world.resource::<Terrain>().globe().clone();
    let life = globe.life()?;
    let rank = |index: u16| {
        let kind = &life.species[usize::from(index)];
        let hunts = matches!(kind.role.guild, Guild::Hunter | Guild::Ambusher);
        kind.history.mass + if hunts { 1.0e6 } else { 0.0 }
    };
    let list = animals(world);
    let walkers = list
        .iter()
        .filter(|a| walker(&life.species[usize::from(a.2)]));
    let (entity, at, _) = walkers.max_by(|a, b| rank(a.2).total_cmp(&rank(b.2)))?;
    Some((*entity, *at))
}

/// A hunter, or the heaviest animal near the base, faced from 6 m with a spear in hand, in
/// golden light that comes three quarters from behind them. The spear is put away after.
pub fn card_beast(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let (found, end) = (settle() - 50, settle() + 2);
    match t {
        1 => {
            lens(world, 60);
            view(world, ViewMode::Third, ViewMode::First);
            let here = robot(world);
            let wake = world.get_resource::<Base>().map_or(here, |b| b.wake);
            let feet = ground(world, wake, 0.0);
            stand(
                world,
                feet,
                feet + feet.normalize().any_orthonormal_vector() * 10.0,
                0.0,
            );
            clock_at(world, GOLD, true);
        }
        _ if [20, end + 1, end + 4].contains(&t) => key(world, KeyCode::F8, true),
        _ if [21, end + 2, end + 5].contains(&t) => key(world, KeyCode::F8, false),
        _ if t == found => {
            let Some((entity, at)) = beast(world) else {
                return Act::Fail("no walking animal is drawn near the base".into());
            };
            let across = at.normalize().cross(sunward(world, at));
            notes.animal = Some((entity, beside(world, at, across, BEAST_RANGE)));
        }
        _ if t == end + 7 => return Act::Next,
        _ => {}
    }
    if let (true, Some((entity, feet))) = ((found..=settle()).contains(&t), notes.animal) {
        let list = animals(world);
        let Some((_, at, _)) = list.iter().find(|a| a.0 == entity) else {
            return Act::Fail("the animal left".into());
        };
        let middle = (feet + *at) * 0.5 + feet.normalize() * 0.9;
        let light = DQuat::from_axis_angle(middle.normalize(), 0.9) * sunward(world, middle);
        let eye = ground(world, middle - light * 8.5, 0.9);
        stand(world, feet, *at, 0.0);
        *world.resource_mut::<Tripod>() = Tripod::Fixed(eye, middle);
    }
    still(world, t.min(settle() - 1), "p10-beast");
    if t == settle() {
        shot(world, "p10-beast");
    }
    Act::Wait
}

/// The wingsuit glide in golden light, from the high ground beside the base down over the
/// valley toward it: the robot on the right third from behind, the land spread out ahead.
pub fn card_glide(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(t) = card_ready(world, notes, frame, false) else {
        return card_wait(frame);
    };
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    if t == 1 {
        lens(world, 70);
        view(world, ViewMode::Third, ViewMode::First);
        let ways = compass(base.centre, 16).into_iter();
        let high = ways
            .map(|d| ground(world, base.centre + d * 700.0, 0.0))
            .max_by(|a, b| a.length().total_cmp(&b.length()))
            .unwrap_or(base.centre);
        stand(world, high, base.centre, 0.0);
        side_light(world, GOLD, base.centre - high);
        notes.spots = vec![high];
    }
    let (high, drop) = (notes.spots[0], t.saturating_sub(settle()));
    let down = (base.centre - high).reject_from(high.normalize());
    match drop {
        0 => return still(world, t.min(settle() - 1), "p11-glide"),
        1 => {
            let above = high + high.normalize() * 240.0;
            stand(world, above, above + down, -0.25);
            *world.resource_mut::<Tripod>() = Tripod::Follow {
                robot: true,
                heading: down.normalize_or(DVec3::X),
                eye: Vec3::new(-2.2, 1.4, 4.5),
                aim: Vec3::new(-3.6, -0.6, -6.0),
            };
        }
        50 => shot(world, "p11-glide"),
        52 => return Act::Next,
        _ => {}
    }
    if drop > 4 {
        spread(world);
    }
    Act::Wait
}
