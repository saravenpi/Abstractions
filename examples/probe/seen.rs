//! Animals in the world: the census, the clock and what the probe sees.

use super::*;

/// What the probe reads of a creature: entity, poses, visibility and first part.
type Seen = (
    Entity,
    &'static GlobalTransform,
    &'static Transform,
    Option<&'static Visibility>,
    Option<&'static InheritedVisibility>,
    Option<&'static Children>,
);

/// Logs the creatures nearest the player and what their first part looks like, every 120 frames.
pub fn probe(
    mut n: Local<u32>,
    players: Query<&GlobalTransform, With<abstractions::player::Player>>,
    creatures: Query<Seen, With<abstractions::fauna::Creature>>,
    parts: Query<(&GlobalTransform, Option<&ViewVisibility>, Option<&Mesh3d>)>,
    tour: Res<Tour>,
) {
    *n += 1;
    if !(*n).is_multiple_of(120) {
        return;
    }
    let Some(me) = players.iter().next().map(|g| g.translation()) else {
        return;
    };
    let mut list: Vec<_> = creatures
        .iter()
        .map(|(e, g, t, v, iv, ch)| {
            (
                g.translation().distance(me),
                e,
                g.translation() - me,
                t.scale,
                v.copied(),
                iv.map(|i| i.get()),
                ch.map(|c| c.len()).unwrap_or(0),
                ch.and_then(|c| c.first().copied()),
            )
        })
        .collect();
    list.sort_by(|a, b| a.0.total_cmp(&b.0));
    eprintln!(
        "PROBE stage={} me={me:?} n={}",
        STAGES.get(tour.stage).map_or("end", |s| s.0),
        list.len()
    );
    for (d, _e, rel, sc, v, iv, nch, first) in list.iter().take(3) {
        let part = first.and_then(|f| parts.get(f).ok()).map(|(g, vv, m)| {
            (
                g.translation() - me,
                g.scale(),
                vv.map(|x| x.get()),
                m.is_some(),
            )
        });
        eprintln!(
            "PROBE   d={d:.1} rel={rel:?} scale={sc:?} vis={v:?} inh={iv:?} parts={nch} part0={part:?}"
        );
    }
}

/// Returns the planet position of every drawn animal, its entity and its species index.
pub fn animals(world: &mut World) -> Vec<(Entity, DVec3, u16)> {
    let anchor = *world.resource::<Anchor>();
    let roots: Vec<(Entity, Option<Entity>, u16)> = world
        .query::<(Entity, Option<&Children>, &abstractions::fauna::Creature)>()
        .iter(world)
        .map(|(e, ch, c)| (e, ch.and_then(|c| c.first().copied()), c.species.index))
        .collect();
    roots
        .into_iter()
        .filter_map(|(e, torso, i)| {
            let at = world.get::<GlobalTransform>(torso?)?.translation();
            Some((e, anchor.to_frame(at), i))
        })
        .collect()
}

/// Logs every drawn species with its size, its guild and its colour.
pub fn census(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 1 {
        noon(world);
    }
    if frame < 30 {
        return Act::Wait;
    }
    let me = robot(world);
    let list = animals(world);
    let globe = world.resource::<Terrain>().globe().clone();
    let Some(life) = globe.life() else {
        return Act::Fail("no life".into());
    };
    let mut seen = std::collections::BTreeMap::<u16, (usize, f64)>::new();
    for (_, at, i) in &list {
        let e = seen.entry(*i).or_insert((0, f64::MAX));
        e.0 += 1;
        e.1 = e.1.min(at.distance(me));
    }
    for s in &life.species {
        eprintln!(
            "PROBE life {} {:?} {:?} {:?} mass {:.2} kg length {:.2} hip {:.2} guild {:?} habitat {:?} level {:.1} group {}-{} glow {:?} parts {}",
            s.name,
            s.plan,
            s.body.locomotion,
            s.role.activity,
            s.history.mass,
            s.body.length,
            s.body.height,
            s.role.guild,
            s.role.habitat,
            s.role.level,
            s.history.group_min,
            s.history.group_max,
            s.glow.map(|g| g.strength),
            s.body.parts.len()
        );
    }
    for (i, (n, d)) in seen {
        let s = &life.species[usize::from(i)];
        eprintln!(
            "PROBE species {} x{n} nearest {d:.0} m: {:?} {:?} mass {:.1} kg length {:.2} m hip {:.2} m guild {:?} level {:.1} colour {:?}",
            s.name,
            s.plan,
            s.body.locomotion,
            s.history.mass,
            s.body.length,
            s.body.height,
            s.role.guild,
            s.role.level,
            s.tissues.first().map(|t| t.colour)
        );
    }
    Act::Next
}

/// Returns the ground `metres` from planet position `at` that stands nearest the height of the
/// ground under `at`, out of the sea, preferring the side towards `me`.
pub fn level_spot(world: &World, at: DVec3, me: DVec3, metres: f64) -> DVec3 {
    let base = ground(world, at, 0.0).length();
    let up = at.normalize();
    let toward = (me - at)
        .reject_from(up)
        .normalize_or(up.any_orthonormal_vector());
    let mut ways = compass(at, 16);
    ways.insert(0, toward);
    let score = |p: DVec3| {
        let height = world.resource::<Planet>().globe.height(p.normalize());
        let wet = if height <= 0.5 { 100.0 } else { 0.0 };
        (p.length() - base).abs() + wet
    };
    ways.into_iter()
        .map(|d| ground(world, at + d * metres, 0.0))
        .min_by(|a, b| score(*a).total_cmp(&score(*b)))
        .unwrap_or(at)
}

/// Moves the clock on to the first time in the next day when the star stands high over the robot.
pub fn noon(world: &mut World) {
    let body = world.resource::<Planet>().body;
    let spec = world.resource::<Universe>().planet(body);
    let here = robot(world).normalize();
    let now = world.resource::<abstractions::space::Clock>().seconds;
    let high = |t: f64| {
        let toward = -spec.position(t).normalize_or(DVec3::X);
        here.dot(abstractions::world::spin(spec.day_seconds, t).inverse() * toward)
    };
    let best = (0..96)
        .map(|k| now + spec.day_seconds * f64::from(k) / 96.0)
        .find(|t| high(*t) > 0.6)
        .unwrap_or(now);
    eprintln!("PROBE clock moved {:.0} s on to daylight", best - now);
    world.resource_mut::<abstractions::space::Clock>().seconds = best;
}

/// Returns the ground `metres` from planet position `at` along `away`, pushed further out until no
/// drawn animal stands within half that distance of it, so no other animal blocks the view.
pub fn clear(world: &mut World, at: DVec3, away: DVec3, metres: f64) -> DVec3 {
    let list = animals(world);
    let mut far = metres;
    for _ in 0..12 {
        let feet = ground(world, at + away * far, 0.0);
        if list
            .iter()
            .all(|a| a.1.distance(feet) > 0.5 * metres.max(4.0))
        {
            return feet;
        }
        far += 3.0;
    }
    ground(world, at + away * far, 0.0)
}
