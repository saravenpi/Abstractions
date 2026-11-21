//! Stones on the ground: the look of each kind and where they lie. The search hops around the wake
//! point, notes the nearest rock of every kind it sees, and shoots each from 2.5 m and from 8 m
//! (`s01` on). Then the steepest dry slope within 400 m (`s20`, `s21`) and the highest top within
//! 800 m (`s30`, `s31`), where no stone should rest. Run it with `PROBE_STAGES=stones` on
//! `SMOKE_SEED=57` and `SMOKE_SEED=7`. It is left out of a run of all groups.

use abstractions::world::Globe;
use abstractions::world::rocks::Plant;
use avian3d::prelude::SpatialQueryFilter;

use super::*;

/// The name of the stage in the log.
const STAGE: &str = "stones";

/// Frames a hop of the search waits for the chunks around the robot to stream in and grow their colliders.
const SETTLE: u32 = 90;

/// Hops of the search: the wake point, then eight points around it.
const HOPS: u32 = 9;

/// Metres from the wake point of the hops around it.
const HOP: f64 = 90.0;

/// Frames spent on each kind of rock: the stand, the near shot, the step back and the far shot.
const VISIT: u32 = 200;

/// The rocks the search found: for each kind, the planet position of its middle and of the ground
/// 2.5 m from it.
#[derive(Resource, Default)]
pub struct StoneFinds(Vec<(Plant, DVec3, DVec3)>);

/// The spot a slope or a top stage looks at: the planet position on the ground and the way down
/// from it.
#[derive(Resource)]
pub struct StoneSpot(DVec3, DVec3);

/// Returns the rock of every kind nearest the robot within 30 m: its kind, the planet position of its
/// middle and of the ground 2.5 m from it toward the robot. Rays cast down on a grid ask the loaded
/// chunks which hits are rocks, as the pick key does for the ray of the crosshair.
fn kinds((space, chunks, placed, anchor, players): Scan) -> Vec<(Plant, DVec3, DVec3)> {
    let Some(me) = players.iter().next().map(|p| p.translation()) else {
        return Vec::new();
    };
    let up = abstractions::space::up(&anchor, me);
    let (east, filter) = (up.any_orthonormal_vector(), SpatialQueryFilter::default());
    let north = up.cross(east);
    let Ok(down) = Dir3::new(-up) else {
        return Vec::new();
    };
    let mut best: Vec<(f32, Plant, Vec3, Vec3)> = Vec::new();
    for (x, z) in (-40..=40).flat_map(|x| (-40..=40).map(move |z| (x, z))) {
        let over = me + (east * x as f32 + north * z as f32) * 0.75 + up * 25.0;
        let Some(hit) = space.cast_ray(over, down, 60.0, true, &filter) else {
            continue;
        };
        let point = over - up * hit.distance;
        let Ok(props) = placed.get(hit.entity) else {
            continue;
        };
        let local = props.affine().inverse().transform_point3(point);
        let Some(rock) = chunks.rock_under(hit.entity, local) else {
            continue;
        };
        let away = (point - me).reject_from(up).length();
        let seen = (away, rock.plant, point, props.transform_point(rock.mid));
        match best.iter_mut().find(|b| b.1 == rock.plant) {
            Some(b) if away < b.0 => *b = seen,
            Some(_) => {}
            None => best.push(seen),
        }
    }
    let to = |(_, plant, point, mid): (f32, Plant, Vec3, Vec3)| {
        let back = (me - point).reject_from(up).normalize_or_zero();
        (
            plant,
            anchor.to_frame(mid),
            anchor.to_frame(point + back * 2.5),
        )
    };
    best.into_iter().map(to).collect()
}

/// Moves the robot to hop `(frame - 1) / SETTLE` of the search and, once the chunks there have
/// settled, notes the nearest rock of every kind not found yet.
fn hop(world: &mut World, base: &Base, frame: u32) {
    let (k, t) = ((frame - 1) / SETTLE, (frame - 1) % SETTLE);
    if t == 0 {
        let ways = compass(base.wake, 8);
        let at = match k {
            0 => base.wake,
            _ => base.wake + ways[(k as usize - 1) % 8] * HOP,
        };
        let feet = ground(world, at, 0.0);
        stand(world, feet, base.centre, 0.0);
    }
    if t == SETTLE - 1 {
        let seen = world.run_system_once(kinds).unwrap_or_default();
        let mut found = world.resource_mut::<StoneFinds>();
        for rock in seen {
            if !found.0.iter().any(|f| f.0 == rock.0) {
                found.0.push(rock);
            }
        }
    }
}

/// Searches for rocks of every kind around the wake point, then stands by the nearest of each kind,
/// looking at it, and shoots it from 2.5 m and from 8 m.
pub fn stones_near(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let Some(base) = world.get_resource::<Base>().copied() else {
        return Act::Fail("no home base".into());
    };
    if frame == 1 {
        world.insert_resource(StoneFinds::default());
    }
    let search = HOPS * SETTLE;
    if frame <= search {
        hop(world, &base, frame);
        return Act::Wait;
    }
    let found = world.resource::<StoneFinds>().0.clone();
    let (visit, t) = ((frame - search - 1) / VISIT, (frame - search - 1) % VISIT);
    let Some(&(plant, mid, feet)) = found.get(visit as usize) else {
        let names: Vec<Plant> = found.iter().map(|f| f.0).collect();
        say(
            STAGE,
            &format!("{} kinds of rock found: {names:?}", found.len()),
        );
        return Act::Next;
    };
    let name = format!("{plant:?}").to_lowercase();
    match t {
        0 => {
            let feet = ground(world, feet, 0.0);
            stand(world, feet, mid, pitch_to(feet, mid));
        }
        80 => {
            say(
                STAGE,
                &format!(
                    "{plant:?} at {:.0} m from the wake point",
                    mid.distance(base.wake)
                ),
            );
            shot(world, &format!("s{:02}-{name}-near", 1 + 2 * visit));
        }
        100 => {
            let up = mid.normalize();
            let away = (feet - mid)
                .reject_from(up)
                .normalize_or(up.any_orthonormal_vector());
            let back = ground(world, mid + away * 8.0, 0.0);
            stand(world, back, mid, pitch_to(back, mid));
        }
        180 => shot(world, &format!("s{:02}-{name}-far", 2 + 2 * visit)),
        _ => {}
    }
    Act::Wait
}

/// Returns the planet positions on the sea level sphere of a grid of `step` metres over `reach` metres
/// around `centre`, each with the height of the ground there.
fn field(globe: &Globe, centre: DVec3, reach: f64, step: f64) -> Vec<(DVec3, f64)> {
    let up = centre.normalize();
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let north = up.cross(east);
    let n = (reach / step) as i32;
    let cells = (-n..=n).flat_map(|i| (-n..=n).map(move |j| (i, j)));
    cells
        .map(|(i, j)| {
            let dir = (centre + (east * f64::from(i) + north * f64::from(j)) * step).normalize();
            (dir * globe.radius(), f64::from(globe.height(dir)))
        })
        .collect()
}

/// Returns the way the ground falls at planet position `p`: a tangent as long as the slope, rise over run.
fn downhill(globe: &Globe, p: DVec3) -> DVec3 {
    let up = p.normalize();
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let north = up.cross(east);
    let h = |o: DVec3| f64::from(globe.height((p + o * 2.0).normalize()));
    let (dx, dz) = ((h(east) - h(-east)) / 4.0, (h(north) - h(-north)) / 4.0);
    -(east * dx + north * dz)
}

/// Stands `back` metres down the way `spot` falls from it, on the ground, looking at it, and logs it.
fn look_up(world: &mut World, spot: &StoneSpot, back: f64, what: &str) {
    let globe = world.resource::<Planet>().globe.clone();
    let top = ground(world, spot.0, 0.0);
    let feet = ground(world, spot.0 + spot.1 * back, 0.0);
    let slope = downhill(&globe, spot.0).length();
    say(
        STAGE,
        &format!("{what}: slope {slope:.2}, {back:.0} m below it"),
    );
    stand(world, feet, top, pitch_to(feet, top));
}

/// Shoots the spot of a slope or a top stage: from `near` metres down its fall, then from `far`.
fn shoot_spot(world: &mut World, frame: u32, (near, far): (f64, f64), name: &str) -> Act {
    let Some(spot) = world.remove_resource::<StoneSpot>() else {
        return Act::Fail("no spot".into());
    };
    let act = match frame {
        1 => Act::Wait,
        2 => {
            look_up(world, &spot, near, name);
            Act::Wait
        }
        150 => {
            shot(world, &format!("{name}-near"));
            Act::Wait
        }
        160 => {
            look_up(world, &spot, far, name);
            Act::Wait
        }
        300 => Act::Shoot(format!("{name}-far")),
        _ => Act::Wait,
    };
    world.insert_resource(spot);
    act
}

/// The steepest dry slope within 400 m of the wake point, looked up at from 12 m and 40 m below it: no
/// stone rests on it, a few lie at its foot.
pub fn stones_slope(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        let Some(base) = world.get_resource::<Base>().copied() else {
            return Act::Fail("no home base".into());
        };
        let globe = world.resource::<Planet>().globe.clone();
        let dry = field(&globe, base.wake, 400.0, 10.0)
            .into_iter()
            .filter(|(_, h)| *h > 1.0);
        let fall = |p: &DVec3| downhill(&globe, *p).length();
        let Some((top, _)) = dry.max_by(|a, b| fall(&a.0).total_cmp(&fall(&b.0))) else {
            return Act::Fail("no dry ground".into());
        };
        let way = downhill(&globe, top).normalize_or(DVec3::X);
        world.insert_resource(StoneSpot(top, way));
    }
    shoot_spot(world, frame, (12.0, 40.0), "s20-slope")
}

/// The highest top within 800 m of the wake point, looked up at from 20 m and 60 m below it: no stone is
/// perched on it.
pub fn stones_peak(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        let Some(base) = world.get_resource::<Base>().copied() else {
            return Act::Fail("no home base".into());
        };
        let globe = world.resource::<Planet>().globe.clone();
        let land = field(&globe, base.wake, 800.0, 16.0);
        let Some((top, height)) = land.into_iter().max_by(|a, b| a.1.total_cmp(&b.1)) else {
            return Act::Fail("no ground".into());
        };
        let way = compass(top, 8)
            .into_iter()
            .min_by(|a, b| {
                let low = |w: &DVec3| globe.height((top + *w * 30.0).normalize());
                low(a).total_cmp(&low(b))
            })
            .unwrap_or(DVec3::X);
        say(STAGE, &format!("the top is {height:.0} m above the sea"));
        world.insert_resource(StoneSpot(top, way));
    }
    shoot_spot(world, frame, (20.0, 60.0), "s30-peak")
}
