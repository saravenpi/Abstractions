//! The zoo in motion: gaits, circles, the orbit camera and the shots of each gait.

use abstractions::fauna::motion::{Detail, Drive};
use abstractions::life::body::Locomotion;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use super::*;

/// Returns the gait shown at `time` seconds: its index in `GAITS`.
pub fn gait(time: f32) -> usize {
    (time / GAIT_SECONDS) as usize % GAITS.len()
}

/// Returns the velocity that keeps `beast` on its circle at `speed`, turning left.
pub fn round(beast: &Beast, speed: f32) -> Vec3 {
    let off = (beast.motion.root.at - beast.centre).with_y(0.0);
    let r = off.length().max(1e-3);
    let out = off / r;
    let ahead = Vec3::Y.cross(out);
    let back = (beast.radius - r) / beast.radius;
    (ahead + out * back).normalize_or_zero() * speed
}

/// Steps every beast, poses its parts and names the gait.
pub fn walk(
    time: Res<Time>,
    mut zoo: ResMut<Zoo>,
    mut parts: Query<&mut Transform, Without<Orbit>>,
    mut label: Query<&mut Text, With<GaitLabel>>,
) {
    let dt = time.delta_secs().clamp(1e-4, 0.1);
    zoo.time += dt;
    let (name, fr, lengths) = GAITS[gait(zoo.time)];
    let gravity = zoo.gravity;
    for beast in &mut zoo.beasts {
        let speed = match beast.body.locomotion {
            Locomotion::Walk => (fr * gravity * beast.body.height.max(0.05)).sqrt(),
            _ => lengths * beast.body.length.max(0.2),
        };
        let drive = Drive {
            velocity: round(beast, speed),
            look: None,
            gravity,
            excite: 0.0,
            detail: Detail::Full,
        };
        beast.motion.step(&beast.body, &drive, &|_| 0.0, dt);
        let pose = beast.motion.pose(&beast.body);
        for (entity, (at, turn, scale)) in beast.parts.iter().zip(pose) {
            if let Some(mut t) = entity.and_then(|e| parts.get_mut(e).ok()) {
                *t = Transform {
                    translation: at,
                    rotation: turn,
                    scale,
                };
            }
        }
    }
    let count = zoo.beasts.len();
    let title = format!("{}, {count} animals: {name}", zoo.title);
    for mut text in &mut label {
        text.0.clone_from(&title);
    }
}

/// Orbits the camera with a left drag and zooms it with the wheel.
pub fn orbit(
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
) {
    for (mut o, mut t) in &mut cameras {
        if buttons.pressed(MouseButton::Left) {
            o.yaw -= 0.005 * motion.delta.x;
            o.pitch = (o.pitch + 0.005 * motion.delta.y).clamp(0.05, 1.5);
        }
        let wheel = match scroll.unit {
            MouseScrollUnit::Line => scroll.delta.y,
            MouseScrollUnit::Pixel => scroll.delta.y / 40.0,
        };
        o.distance = (o.distance * (1.0 - 0.1 * wheel)).clamp(1.0, 500.0);
        let way = Quat::from_euler(EulerRot::YXZ, o.yaw, -o.pitch, 0.0) * Vec3::Z;
        *t = Transform::from_translation(o.target + way * o.distance).looking_at(o.target, Vec3::Y);
    }
}

/// In shots mode, saves one screenshot late in each gait, then exits once they are written.
pub fn shoot(mut commands: Commands, mut zoo: ResMut<Zoo>, mut exit: MessageWriter<AppExit>) {
    let Some(dir) = zoo.shots.clone() else {
        return;
    };
    let shown = (zoo.time / GAIT_SECONDS) as usize;
    if shown >= GAITS.len() {
        zoo.done += 1;
        let written = GAITS
            .iter()
            .all(|g| dir.join(format!("{}.png", g.0)).exists());
        if (written && zoo.done > 10) || zoo.done > 300 {
            exit.write(AppExit::Success);
        }
        return;
    }
    let late = zoo.time % GAIT_SECONDS;
    if (GAIT_SECONDS - 0.5..GAIT_SECONDS - 0.5 + STEP).contains(&late) {
        let path = dir.join(format!("{}.png", GAITS[shown].0));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}
