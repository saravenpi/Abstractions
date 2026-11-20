//! Helpers shared by every stage: screenshots, pacing, keys, ground and the player pose.

use super::*;

/// Saves a screenshot of the window.
pub fn shot(world: &mut World, name: &str) {
    let dir = std::env::var("SMOKE_DIR").unwrap_or_default();
    world
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(format!("{dir}/{name}.png")));
}

/// Sets the simulated seconds per frame.
pub fn pace(world: &mut World, seconds: f64) {
    world.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        seconds,
    )));
}

/// Presses or releases `code`.
pub fn key(world: &mut World, code: KeyCode, pressed: bool) {
    let window = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .iter(world)
        .next()
        .unwrap_or(Entity::PLACEHOLDER);
    world.write_message(KeyboardInput {
        key_code: code,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state: [ButtonState::Released, ButtonState::Pressed][usize::from(pressed)],
        text: None,
        repeat: false,
        window,
    });
}

/// Returns the key the settings bind to what `pick` names.
pub fn bound(world: &World, pick: impl Fn(&Settings) -> KeyCode) -> KeyCode {
    pick(world.resource::<Persistent<Settings>>())
}

/// Returns the planet position on the ground of the streamed planet under `pos`, raised by `lift`.
pub fn ground(world: &World, pos: DVec3, lift: f64) -> DVec3 {
    let globe = &world.resource::<Planet>().globe;
    let dir = pos.normalize();
    dir * (globe.radius() + f64::from(globe.height(dir).max(0.0)) + lift)
}

/// Stands the player at planet position `feet` looking at planet position `at`, with `pitch`.
pub fn stand(world: &mut World, feet: DVec3, at: DVec3, pitch: f32) {
    let anchor = *world.resource::<Anchor>();
    let (feet, at) = (anchor.to_render(feet), anchor.to_render(at));
    place(world, feet, at - feet);
    let yaw = world
        .query_filtered::<&Motion, With<Player>>()
        .iter(world)
        .next()
        .map_or(0.0, |m| m.yaw);
    world.write_message(Cut {
        heading: Some(Heading { yaw, pitch }),
    });
}

/// Logs the meshes, triangles and lights in view.
pub fn hygiene(world: &mut World, label: &str) {
    let counted = world.run_system_once(
        |meshes: Query<(&Mesh3d, &ViewVisibility)>,
         tris: Res<Tris>,
         points: Query<&ViewVisibility, With<PointLight>>,
         spots: Query<&ViewVisibility, With<SpotLight>>| {
            let shown = meshes.iter().filter(|(_, v)| v.get());
            let (mut count, mut triangles) = (0, 0);
            for (mesh, _) in shown {
                count += 1;
                triangles += tris.0.get(&mesh.0.id()).copied().unwrap_or(0);
            }
            let lights = points
                .iter()
                .chain(spots.iter())
                .filter(|v| v.get())
                .count();
            (meshes.iter().count(), count, triangles, lights)
        },
    );
    if let Ok((all, shown, triangles, lights)) = counted {
        info!(
            "SMOKE hygiene {label}: {all} mesh entities, {shown} in view, {triangles} triangles in view, {lights} lights in view"
        );
    }
}
