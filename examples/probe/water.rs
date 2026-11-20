//! The camera above, at and below the water surface.

use super::*;

/// Camera heights above the waterline in metres and pitches of the lens tour.
pub const LENS: [(f32, f32, &str); 12] = [
    (1.0, 0.35, "s1-above-up"),
    (1.0, -0.35, "s2-above-down"),
    (0.0, 0.0, "s3-surface-level"),
    (0.0, 0.35, "s4-surface-up"),
    (0.0, -0.35, "s5-surface-down"),
    (0.015, 0.0, "s6-skim-above"),
    (-0.015, 0.0, "s7-skim-below"),
    (-1.0, 0.35, "s8-below-up"),
    (-1.0, -0.35, "s9-below-down"),
    (-1.0, 0.0, "s10-below-level"),
    (-0.3, 1.2, "s11-below-steep-up"),
    (0.3, -1.2, "s12-above-steep-down"),
];

/// Returns a point of open water at least 4 m deep within 3 km of `from`.
pub fn deep_water(world: &World, from: DVec3) -> Option<DVec3> {
    let globe = &world.resource::<Planet>().globe;
    let up = from.normalize();
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    (0..16).find_map(|k| {
        let a = f64::from(k) * std::f64::consts::TAU / 16.0;
        let dir = east * a.cos() + north * a.sin();
        (1..300)
            .map(|i| (from + dir * f64::from(i) * 10.0).normalize() * from.length())
            .find(|p| globe.height(p.normalize()) < -5.0)
    })
}

/// Holds the game camera `height` metres above the sea level 4 m from the robot, pitched `pitch`.
pub fn hold_lens(world: &mut World, height: f32, pitch: f32) {
    let anchor = *world.resource::<Anchor>();
    let radius = world.resource::<Planet>().globe.radius();
    let Some(body) = world
        .query_filtered::<&Transform, With<Player>>()
        .iter(world)
        .next()
        .copied()
    else {
        return;
    };
    let centre = anchor.to_frame(body.translation);
    let up = centre.normalize();
    let east = up.any_orthonormal_vector();
    let spot = (centre + east * 4.0).normalize() * (radius + f64::from(height));
    let up_r = (anchor.rotation * spot.normalize()).as_vec3();
    let fwd_r = (anchor.rotation * east).as_vec3();
    let fwd = (fwd_r - up_r * fwd_r.dot(up_r)).normalize();
    let right = fwd.cross(up_r);
    let look = Quat::from_axis_angle(right, pitch) * fwd;
    let pose = Transform::from_translation(anchor.to_render(spot)).looking_to(look, up_r);
    let back = body.rotation.inverse();
    for mut t in world
        .query_filtered::<&mut Transform, With<abstractions::render::GameCamera>>()
        .iter_mut(world)
    {
        t.translation = back * (pose.translation - body.translation);
        t.rotation = back * pose.rotation;
    }
}

/// The camera above, at and below the water surface of deep open water, looking up and down.
pub fn lens_tour(world: &mut World, notes: &mut Notes, frame: u32) -> Act {
    const EACH: u32 = 25;
    if frame == 1 {
        let Some(base) = world.get_resource::<Base>().copied() else {
            return Act::Fail("no home base".into());
        };
        let Some(sea) = deep_water(world, base.centre) else {
            return Act::Fail("no deep water within 3 km".into());
        };
        notes.shore = Some(sea);
        let feet = sea.normalize() * world.resource::<Planet>().globe.radius();
        stand(
            world,
            feet,
            feet + sea.normalize().any_orthonormal_vector(),
            0.0,
        );
        noon(world);
        return Act::Wait;
    }
    if frame < 90 {
        return Act::Wait;
    }
    let step = ((frame - 90) / EACH) as usize;
    let Some(&(height, pitch, name)) = LENS.get(step) else {
        world.resource_mut::<abstractions::view::Enabled>().0 = true;
        return Act::Next;
    };
    world.resource_mut::<abstractions::view::Enabled>().0 = false;
    hold_lens(world, height, pitch);
    if (frame - 90) % EACH == EACH - 1 {
        shot(world, name);
    }
    Act::Wait
}
