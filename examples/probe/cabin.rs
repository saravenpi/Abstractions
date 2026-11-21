//! Looking at the cockpit from the seat: the view ahead, down at the dashboard, to both sides,
//! up through the canopy and down at the feet, by day and at night, and a report of what the
//! displays draw.

use abstractions::ship::SEAT;
use abstractions::ship::cockpit::{Display, Screen};

use super::*;

/// Where the pilot looks from the seat: the name of the view, the turn to the left and the
/// pitch up, in radians.
const LOOKS: [(&str, f32, f32); 7] = [
    ("ahead", 0.0, 0.0),
    ("dash", 0.0, -0.55),
    ("left", 0.95, -0.35),
    ("right", -0.95, -0.35),
    ("up", 0.0, 1.1),
    ("feet", 0.0, -1.2),
    ("back", 2.5, -0.1),
];

/// Where the pilot looks to see the hands on the controls: down at both, down to the left at
/// the throttle and down to the right at the stick.
const HANDS: [(&str, f32, f32); 3] = [
    ("both", 0.0, -1.0),
    ("throttle", 0.8, -0.95),
    ("stick", -0.8, -0.95),
];

/// Returns the views from the eye of the pilot, one for each look of `list`.
fn aimed(list: &[(&'static str, f32, f32)]) -> Vec<View> {
    let eye = SEAT.as_vec3();
    let aim = |yaw: f32, pitch: f32| {
        Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * Vec3::NEG_Z
    };
    list.iter()
        .map(|&(name, yaw, pitch)| (name, eye, eye + aim(yaw, pitch)))
        .collect()
}

/// Returns the views from the eye of the pilot, one for each of [`LOOKS`].
fn looks() -> Vec<View> {
    aimed(&LOOKS)
}

/// The hands on the controls in first person, by day: at rest, then with the stick rolled full
/// right and twisted full left and the throttle full forward. The ship stays on the ground: it
/// turns on the spot, and the camera turns with it.
pub fn hands(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let held = [
        bound(world, |s| s.controls.keys.ship.bank_right),
        bound(world, |s| s.controls.keys.ship.turn_left),
    ];
    let list = aimed(&HANDS);
    if frame <= 50 {
        views(world, frame, "37-hands-rest", &list);
        return Act::Wait;
    }
    if frame == 51 {
        for code in held {
            key(world, code, true);
        }
    }
    with_ship(world, |ship| ship.throttle = 1.0);
    if frame < 80 || !views(world, frame - 79, "37-hands-full", &list) {
        return Act::Wait;
    }
    for code in held {
        key(world, code, false);
    }
    with_ship(world, |ship| ship.throttle = 0.0);
    Act::Next
}

/// Logs, for each display of the cockpit, whether it is in view, how bright its material is
/// and how many of its pixels are lit.
fn report(world: &mut World) {
    let lines = world.run_system_once(
        |screens: Query<(
            &Screen,
            &Display,
            &ViewVisibility,
            &MeshMaterial3d<StandardMaterial>,
        )>,
         images: Res<Assets<Image>>,
         materials: Res<Assets<StandardMaterial>>| {
            let mut lines = Vec::new();
            for (screen, display, seen, material) in &screens {
                let lit = images
                    .get(&display.image)
                    .and_then(|image| image.data.as_ref());
                let lit = lit.map_or(0, |data| data.chunks(4).filter(|p| p[0] > 60).count());
                let tone = materials.get(&material.0).map(|m| m.base_color.to_linear());
                lines.push(format!(
                    "{screen:?}: seen {}, tone {tone:?}, {lit} pixels lit",
                    seen.get()
                ));
            }
            lines
        },
    );
    for line in lines.unwrap_or_default() {
        info!("SMOKE display {line}");
    }
}

/// The cockpit from the seat, by day.
pub fn cockpit(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    if frame == 12 {
        report(world);
    }
    match views(world, frame, "32-seat", &looks()) {
        true => Act::Next,
        false => Act::Wait,
    }
}

/// The views of the landed ship from outside, at night.
const OUT: [View; 2] = [
    ("out", Vec3::new(-4.4, 1.2, -6.4), Vec3::new(0.0, 0.2, -0.6)),
    (
        "canopy",
        Vec3::new(-2.4, 1.5, -3.3),
        Vec3::new(0.0, 0.6, -1.5),
    ),
];

/// Half a day later: the landed ship from outside, with the whole pilot in the seat, and the
/// cockpit from the seat, at night. The clock is then put on by another half day.
pub fn night(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let half = world.resource::<Planet>().day_seconds * 0.5;
    match frame {
        1 => {
            world.resource_mut::<abstractions::space::Clock>().seconds += half;
            view(world, ViewMode::First, ViewMode::Third);
        }
        60..=99 => {
            views(world, frame - 59, "33-night", &OUT);
        }
        100 => view(world, ViewMode::First, ViewMode::First),
        140.. if views(world, frame - 139, "33-night", &looks()[..4]) => {
            world.resource_mut::<abstractions::space::Clock>().seconds += half;
            return Act::Next;
        }
        _ => {}
    }
    Act::Wait
}
