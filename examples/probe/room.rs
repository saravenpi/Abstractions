//! The village room, its door and its roof.

use super::*;

/// Takes the walled house of the village nearest to its plaza from the village layout, and
/// stands in it while its chunk loads.
pub fn room(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    let Some(origin) = notes.village else {
        return Act::Fail("no village".into());
    };
    match frame {
        1 => {
            let terrain = world.resource::<Terrain>().clone();
            let sites = village_sites(&terrain, origin);
            let Some(site) = sites.first() else {
                return Act::Fail("no village site".into());
            };
            let view = terrain.around(site.origin.x, site.origin.y);
            let mut houses = entries(&view, site);
            houses.sort_by(|a, b| a.0.distance(origin).total_cmp(&b.0.distance(origin)));
            let Some((middle, door, out)) = houses.first().copied() else {
                return Act::Fail("the village has no walled house".into());
            };
            info!("SMOKE house middle {middle:?}, door {door:?}");
            (notes.room, notes.door) = (Some(middle), Some((door, out)));
            let up = middle.normalize();
            stand(world, middle + up * 0.1, door, 0.0);
            Act::Wait
        }
        90 => Act::Next,
        _ => Act::Wait,
    }
}

/// Stands `away` metres out of the door, looking at its frame.
pub fn at_door(world: &mut World, notes: &Notes, away: f64) -> bool {
    let Some((door, out)) = notes.door else {
        return false;
    };
    let up = door.normalize();
    let feet = ground(world, door + out * away, 0.0);
    let pitch = (0.6 / away).atan() as f32;
    stand(world, feet, door + up * 1.6, pitch);
    true
}

/// The door frame from 15 m outside.
pub fn door_far(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !at_door(world, notes, 15.0) => Act::Fail("no door".into()),
        60 => {
            hygiene(world, "house door");
            Act::Shoot("07-door-15m".into())
        }
        _ => Act::Wait,
    }
}

/// The door frame from 3 m outside.
pub fn door_near(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !at_door(world, notes, 3.0) => Act::Fail("no door".into()),
        60 => Act::Shoot("08-door-3m".into()),
        _ => Act::Wait,
    }
}

/// Stands 1.5 m inside the door, clear of the furniture in the middle of the room, looking up at
/// the roof, turned `turn` radians from the way out.
pub fn under_roof(world: &mut World, notes: &Notes, turn: f64) -> bool {
    let Some((door, out)) = notes.door else {
        return false;
    };
    let spot = door - out * 1.5;
    let up = spot.normalize();
    let toward = DQuat::from_axis_angle(up, turn) * -out;
    stand(world, spot + up * 0.1, spot + toward, 1.05);
    true
}

/// A drawn mesh: its entity, where it is and whether it is seen and shown.
pub type Drawn = (
    Entity,
    &'static GlobalTransform,
    &'static ViewVisibility,
    &'static InheritedVisibility,
);

/// The robot body and the companion.
pub type Bodies = Or<(
    With<abstractions::avatar::Avatar>,
    With<abstractions::companion::Companion>,
)>;

/// Logs the pose input, what is drawn of the body, and every drawn mesh of the body within 1.5 m
/// of the camera with its angle from the line of sight.
pub fn body_report(world: &mut World) {
    let Some(pose) = world
        .query::<&abstractions::avatar::AvatarPose>()
        .iter(world)
        .next()
        .copied()
    else {
        return;
    };
    let shown = *world.resource::<abstractions::avatar::Shown>();
    info!(
        "SMOKE body look {:?}, shown {shown:?}, mode {:?}",
        pose.look, pose.mode
    );
    let height = world.run_system_once(
        |camera: Single<&GlobalTransform, With<abstractions::render::GameCamera>>,
         body: Single<&GlobalTransform, With<Player>>| {
            let up = body.up().as_vec3();
            (camera.translation() - body.translation()).dot(up)
        },
    );
    info!("SMOKE camera over the body centre: {height:?}");
    let report = world.run_system_once(
        |camera: Single<&GlobalTransform, With<abstractions::render::GameCamera>>,
         meshes: Query<Drawn, With<Mesh3d>>,
         parents: Query<&ChildOf>,
         bodies: Query<(), Bodies>| {
            let (eye, ahead) = (camera.translation(), camera.forward().as_vec3());
            let mut lines = Vec::new();
            for (entity, at, seen, inherited) in &meshes {
                let owner = parents.iter_ancestors(entity).find(|e| bodies.contains(*e));
                let offset = at.translation() - eye;
                if owner.is_some() && offset.length() < 1.5 && inherited.get() {
                    let angle = offset.normalize_or_zero().dot(ahead).acos().to_degrees();
                    lines.push(format!(
                        "{entity:?} {:.2} m {angle:.0} deg seen {}",
                        offset.length(),
                        seen.get()
                    ));
                }
            }
            lines
        },
    );
    for line in report.unwrap_or_default() {
        info!("SMOKE body part {line}");
    }
}

/// Inside, looking up at the roof away from the door.
pub fn roof_a(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !under_roof(world, notes, 0.0) => Act::Fail("no room".into()),
        49 => {
            body_report(world);
            Act::Wait
        }
        50 => Act::Shoot("09-roof-in-a".into()),
        _ => Act::Wait,
    }
}

/// Inside, looking up at the roof across the room.
pub fn roof_b(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !under_roof(world, notes, 1.6) => Act::Fail("no room".into()),
        50 => Act::Shoot("10-roof-in-b".into()),
        _ => Act::Wait,
    }
}
