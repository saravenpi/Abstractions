//! Herds and the ground they stand on.

use super::*;

/// Returns the site of unit direction `dir` of `globe` as herds read it, in full daylight.
pub fn site_of(globe: &abstractions::world::Globe, dir: DVec3) -> abstractions::life::census::Site {
    use abstractions::world::Surface;
    let height = globe.height(dir);
    let climate = globe.air(dir, height);
    let wet = matches!(
        globe.surface(dir),
        Surface::Water | Surface::Shallows | Surface::Reef | Surface::Pond | Surface::River
    );
    abstractions::life::census::Site {
        height,
        temperature: climate.temperature,
        moisture: climate.moisture.clamp(0.0, 1.0),
        biome: globe.biome(dir),
        liquid: height < globe.sea_level() || wet,
        daylight: 1.0,
    }
}

/// Finds the nearest herd of walkers at least 0.8 m at the hip within 3 km of the wake point, logs
/// its distance, stands 60 m from it and shoots it.
pub fn find(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let globe = world.resource::<Terrain>().globe().clone();
    let Some(life) = globe.life() else {
        return Act::Fail("no life".into());
    };
    if frame == 1 {
        let Some(base) = world.get_resource::<Base>().copied() else {
            return Act::Fail("no base".into());
        };
        let time = world.resource::<abstractions::space::Clock>().seconds;
        let sample = |dir: DVec3| site_of(&globe, dir);
        let light = |_: DVec3, _: f64| 1.0;
        let height = |dir: DVec3| globe.height(dir);
        let ground = abstractions::life::herd::Ground {
            sample: &sample,
            height: &height,
            light: &light,
        };
        let wake = base.wake.normalize();
        let search = abstractions::life::herd::Search {
            at: wake,
            radius: 3000.0,
            planet: globe.radius(),
            time: time + 4.0,
        };
        let herds = abstractions::life::herd::near(life, &ground, &search);
        let arc =
            |h: &abstractions::life::herd::Herd| h.centre.angle_between(wake) * globe.radius();
        let Some(herd) = herds
            .iter()
            .filter(|h| big_walker(&life.species[usize::from(h.species)]))
            .min_by(|a, b| arc(a).total_cmp(&arc(b)))
        else {
            eprintln!("PROBE no big walker herd within 3 km");
            return Act::Next;
        };
        let sp = &life.species[usize::from(herd.species)];
        eprintln!(
            "PROBE nearest big herd {} x{} {:.0} kg hip {:.2} m length {:.2} m, {:.0} m from the wake point",
            sp.name,
            herd.count,
            sp.history.mass,
            sp.body.height,
            sp.body.length,
            arc(herd)
        );
        let at = herd.centre * globe.radius();
        let me = robot(world);
        let feet = level_spot(world, ground_at(world, at), me, 60.0);
        stand(world, feet, at, -0.05);
        notes.animal = Some((Entity::PLACEHOLDER, feet));
        return Act::Wait;
    }
    match frame {
        90 => Act::Shoot("03-find".into()),
        _ => Act::Wait,
    }
}

/// Stands 4 m beside the nearest big walker, square to its heading, and shoots it from the side.
pub fn side(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let globe = world.resource::<Terrain>().globe().clone();
    let Some(life) = globe.life() else {
        return Act::Fail("no life".into());
    };
    if frame == 1 {
        view(world, ViewMode::First, ViewMode::First);
        let me = robot(world);
        let list = animals(world);
        let Some((e, at, _)) = list
            .iter()
            .filter(|a| big_walker(&life.species[usize::from(a.2)]))
            .min_by(|a, b| a.1.distance(me).total_cmp(&b.1.distance(me)))
            .copied()
        else {
            return Act::Fail("no big walker".into());
        };
        let anchor = *world.resource::<Anchor>();
        let Some(turn) = world
            .get::<GlobalTransform>(e)
            .map(|g| g.forward().as_vec3())
        else {
            return Act::Fail("no root".into());
        };
        let root = world
            .get::<GlobalTransform>(e)
            .map_or(Vec3::ZERO, |g| g.translation());
        let ahead = anchor.to_frame(root + turn) - anchor.to_frame(root);
        let up = at.normalize();
        let beside = ahead.cross(up).normalize_or(up.any_orthonormal_vector());
        notes.animal = Some((e, clear(world, at, beside, 4.0)));
    }
    let Some((e, feet)) = notes.animal else {
        return Act::Fail("lost".into());
    };
    let list = animals(world);
    let Some((_, at, _)) = list.iter().find(|a| a.0 == e) else {
        return Act::Fail("gone".into());
    };
    let up = feet.normalize();
    let rel = (*at - (feet + up * 1.6)).normalize();
    let pitch = (rel.dot(up).asin() as f32).clamp(-1.2, 1.2);
    stand(world, feet, *at, pitch);
    match frame {
        30 => Act::Shoot("04-side-4m".into()),
        _ => Act::Wait,
    }
}

/// Returns the rise over run of the ground of `globe` across 20 m around unit direction `dir`.
pub fn grade(globe: &abstractions::world::Globe, dir: DVec3) -> f32 {
    let east = DVec3::Y.cross(dir).normalize_or(DVec3::X);
    let north = dir.cross(east);
    let step = 10.0 / globe.radius();
    let at = |d: DVec3| globe.height((dir + d * step).normalize());
    let (dx, dz) = (at(east) - at(-east), at(north) - at(-north));
    (dx * dx + dz * dz).sqrt() / 20.0
}

/// Finds the herd of walkers on the steepest ground within 2 km of the wake point, stands 40 m
/// downhill of it and shoots it three times, four seconds apart.
pub fn hill(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let globe = world.resource::<Terrain>().globe().clone();
    let Some(life) = globe.life() else {
        return Act::Fail("no life".into());
    };
    if frame == 1 {
        let Some(base) = world.get_resource::<Base>().copied() else {
            return Act::Fail("no base".into());
        };
        let time = world.resource::<abstractions::space::Clock>().seconds;
        let sample = |dir: DVec3| site_of(&globe, dir);
        let height = |dir: DVec3| globe.height(dir);
        let light = |_: DVec3, _: f64| 1.0;
        let ground = abstractions::life::herd::Ground {
            sample: &sample,
            height: &height,
            light: &light,
        };
        let search = abstractions::life::herd::Search {
            at: base.wake.normalize(),
            radius: 2000.0,
            planet: globe.radius(),
            time: time + 4.0,
        };
        let herds = abstractions::life::herd::near(life, &ground, &search);
        let Some(herd) = herds
            .iter()
            .filter(|h| walker(&life.species[usize::from(h.species)]) && h.count > 2)
            .max_by(|a, b| grade(&globe, a.centre).total_cmp(&grade(&globe, b.centre)))
        else {
            return Act::Fail("no herd of walkers".into());
        };
        let sp = &life.species[usize::from(herd.species)];
        eprintln!(
            "PROBE hill herd {} x{} hip {:.2} m on a grade of {:.2}",
            sp.name,
            herd.count,
            sp.body.height,
            grade(&globe, herd.centre)
        );
        let at = ground_at(world, herd.centre * globe.radius());
        let me = robot(world);
        let feet = level_spot(world, at, me, 40.0);
        stand(world, feet, at, -0.1);
        notes.animal = Some((Entity::PLACEHOLDER, feet));
        return Act::Wait;
    }
    match frame {
        60 => {
            shot(world, "05a-hill");
            Act::Wait
        }
        180 => {
            shot(world, "05b-hill");
            Act::Wait
        }
        300 => Act::Shoot("05c-hill".into()),
        _ => Act::Wait,
    }
}

/// Returns the ground under planet position `at`.
pub fn ground_at(world: &World, at: DVec3) -> DVec3 {
    ground(world, at, 0.0)
}
