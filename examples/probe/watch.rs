//! Standing near one animal and looking at it.

use super::*;

/// Which animals a stage looks at.
pub type Pick = fn(&abstractions::life::Species) -> bool;

/// Stands `metres` from the animal nearest the robot that `pick` takes, looking at it, and follows
/// it with the eyes.
pub fn near_animal(
    world: &mut World,
    notes: &mut Notes,
    (frame, metres, pick): (u32, f64, Pick),
) -> Option<f64> {
    let globe = world.resource::<Terrain>().globe().clone();
    let life = globe.life()?;
    if frame == 1 || notes.animal.is_none() {
        let me = robot(world);
        let list = animals(world);
        let (e, at, i) = *list
            .iter()
            .filter(|a| pick(&life.species[usize::from(a.2)]))
            .min_by(|a, b| a.1.distance(me).total_cmp(&b.1.distance(me)))?;
        let up = at.normalize();
        let away = (me - at)
            .reject_from(up)
            .normalize_or(up.any_orthonormal_vector());
        notes.animal = Some((e, clear(world, at, away, metres)));
        let s = &life.species[usize::from(i)];
        eprintln!(
            "PROBE looking at {} {:?} mass {:.1} kg length {:.2} m, {:.0} m away",
            s.name,
            s.body.locomotion,
            s.history.mass,
            s.body.length,
            at.distance(me)
        );
    }
    let (e, feet) = notes.animal?;
    let list = animals(world);
    let (_, at, _) = list.iter().find(|a| a.0 == e)?;
    let up = feet.normalize();
    let rel = (*at - (feet + up * 1.6)).normalize();
    let pitch = (rel.dot(up).asin() as f32).clamp(-1.2, 1.2);
    stand(world, feet, *at, pitch);
    Some(at.distance(feet))
}

/// Shoots the animal nearest the robot that `pick` takes from `metres`, in view `mode`.
pub fn animal_shot(
    world: &mut World,
    notes: &mut Notes,
    frame: u32,
    (metres, mode, name, pick): (f64, ViewMode, &str, Pick),
) -> Act {
    if frame == 1 {
        notes.animal = None;
        view(world, mode, ViewMode::First);
    }
    let Some(d) = near_animal(world, notes, (frame, metres, pick)) else {
        notes.animal = None;
        return match frame {
            f if f < 120 => Act::Wait,
            _ => Act::Fail("no animal".into()),
        };
    };
    if frame.is_multiple_of(20) {
        eprintln!("PROBE {name} frame {frame}: animal at {d:.1} m");
    }
    match frame {
        f if f >= 60 => Act::Shoot(name.into()),
        _ => Act::Wait,
    }
}

/// Takes animals that walk.
pub fn walker(s: &abstractions::life::Species) -> bool {
    s.body.locomotion == abstractions::life::body::Locomotion::Walk
}

/// Takes walkers at least 0.8 m at the hip.
pub fn big_walker(s: &abstractions::life::Species) -> bool {
    walker(s) && s.body.height >= 0.8
}

/// Takes flyers.
pub fn flyer(s: &abstractions::life::Species) -> bool {
    s.body.locomotion == abstractions::life::body::Locomotion::Fly
}

/// Takes swarms.
pub fn swarmer(s: &abstractions::life::Species) -> bool {
    s.body.locomotion == abstractions::life::body::Locomotion::Swarm
}

/// Takes glowing species.
pub fn glowing(s: &abstractions::life::Species) -> bool {
    s.glow.is_some() && s.body.length >= 0.4
}
