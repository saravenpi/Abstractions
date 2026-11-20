//! The robot in the village room next to its nearest wall.

use super::*;

/// Returns the unit horizontal direction from the room middle to its nearest wall and how far it is.
pub fn nearest_wall(world: &mut World, middle: DVec3) -> Option<(DVec3, f64)> {
    let anchor = *world.resource::<Anchor>();
    let up = middle.normalize();
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    let from = anchor.to_render(middle + up * 1.2);
    let dirs: Vec<DVec3> = (0..32)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 32.0;
            east * a.cos() + north * a.sin()
        })
        .collect();
    let rays: Vec<(Vec3, DVec3)> = dirs
        .iter()
        .map(|d| ((anchor.rotation * *d).as_vec3(), *d))
        .collect();
    let hits = world
        .run_system_once(
            move |query: avian3d::prelude::SpatialQuery, players: Query<Entity, With<Player>>| {
                let filter = avian3d::prelude::SpatialQueryFilter::default()
                    .with_excluded_entities(players.iter());
                rays.iter()
                    .filter_map(|(r, d)| {
                        let dir = Dir3::new(*r).ok()?;
                        query
                            .cast_ray(from, dir, 8.0, true, &filter)
                            .map(|h| (*d, f64::from(h.distance)))
                    })
                    .collect::<Vec<_>>()
            },
        )
        .ok()?;
    hits.into_iter().min_by(|a, b| a.1.total_cmp(&b.1))
}

/// Third person in the room: the robot `gap` metres from its nearest wall, facing it when `facing`,
/// with the camera turned to the face when `face`.
pub fn at_wall(world: &mut World, notes: &Notes, gap: f64, facing: bool, face: bool) -> bool {
    let Some(middle) = notes.room else {
        return false;
    };
    let Some((dir, far)) = nearest_wall(world, middle) else {
        return false;
    };
    info!("SMOKE nearest wall {far:.2} m");
    view(world, ViewMode::Third, ViewMode::First);
    let up = middle.normalize();
    let feet = middle + up * 0.1 + dir * (far - gap).max(0.0);
    let toward = if facing { feet + dir } else { feet - dir };
    stand(world, feet, toward, -0.05);
    if face {
        let yaw = world
            .query_filtered::<&Motion, With<Player>>()
            .iter(world)
            .next()
            .map_or(0.0, |m| m.yaw);
        world.write_message(Cut {
            heading: Some(Heading {
                yaw: yaw + std::f32::consts::PI,
                pitch: -0.05,
            }),
        });
    }
    true
}

/// Third person in the middle of the room, the camera turned to the face.
pub fn wall_open(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !at_wall(world, notes, 2.0, true, true) => Act::Fail("no room".into()),
        50 => Act::Shoot("w0-face-open".into()),
        _ => Act::Wait,
    }
}

/// Third person facing a wall 0.7 m away, the camera turned to the face between them.
pub fn wall_face(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !at_wall(world, notes, 0.7, true, true) => Act::Fail("no room".into()),
        50 => Act::Shoot("w1-face-wall".into()),
        _ => Act::Wait,
    }
}

/// Third person with the back to a wall 0.45 m away, the camera behind.
pub fn wall_back(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    match frame {
        1 if !at_wall(world, notes, 0.45, false, false) => Act::Fail("no room".into()),
        50 => {
            view(world, ViewMode::First, ViewMode::First);
            Act::Shoot("w2-back-wall".into())
        }
        _ => Act::Wait,
    }
}
