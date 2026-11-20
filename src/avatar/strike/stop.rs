//! Hitstop: the robot and the animal it struck freeze together for a few frames, the animal
//! shaking from [`SHAKE`] to nothing, the camera too when the local robot is in it. Client-side
//! only: the server keeps simulating.

use bevy::prelude::*;

use super::hit::Hit;
use super::tune::{CAMERA_SHAKE, SHAKE, STOP, STOP_FPS};
use crate::fauna::{Frozen, Part};
use crate::render::GameCamera;
use crate::settings::{Persistent, Settings};

/// The hitstop under way, if any.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Hitstop {
    /// Seconds left.
    pub left: f32,
    /// Seconds it lasts in all.
    pub total: f32,
    /// The body that struck: its strike stands still meanwhile.
    pub attacker: Option<Entity>,
    /// The animal struck, frozen meanwhile.
    pub victim: Option<Entity>,
    /// Strength of the shake, 0 to 1, from the settings; 0 when the local robot is not in it for
    /// the camera.
    pub camera: f32,
    /// Offset laid on the parts of the animal last frame, in their frames.
    shaken: Vec3,
    /// Frames shown so far, which flip the shake.
    frame: u32,
    /// Local camera translation written last frame and the offset it held.
    lens: [Vec3; 2],
}

/// Returns the hitstop frames of a blow of `energy` joules at 60 fps: 3 + 0.4·√E, from 3 to 12.
pub fn frames(energy: f32) -> f32 {
    (STOP[0] + STOP[1] * energy.max(0.0).sqrt()).clamp(STOP[0], STOP[2])
}

impl Hitstop {
    /// Returns true while the strike of `body` stands still.
    pub fn holds(&self, body: Entity) -> bool {
        self.left > 0.0 && self.attacker == Some(body)
    }

    /// Returns the share of the shake left, 1 at the start to 0 at the end, flipping every frame.
    fn shake(&self) -> f32 {
        let flip = if self.frame % 2 == 0 { 1.0 } else { -1.0 };
        flip * (self.left / self.total.max(f32::EPSILON)).max(0.0)
    }
}

/// Run condition: a hitstop is under way or has a victim to let go.
pub fn active(stop: Res<Hitstop>) -> bool {
    active_now(&stop)
}

/// Starts a hitstop from the first blow that lands while none is under way, scaled by the hitstop
/// setting, and freezes its victim. `local` is the body of the local robot.
pub fn start(
    mut hits: MessageReader<Hit>,
    settings: Res<Persistent<Settings>>,
    mut stop: ResMut<Hitstop>,
    local: super::input::Robot,
    mut commands: Commands,
) {
    let strength = settings.gameplay.hitstop.clamp(0.0, 1.0);
    let Some(hit) = hits.read().last().copied() else {
        return;
    };
    if strength <= 0.0 || active_now(&stop) {
        return;
    }
    let seconds = frames(hit.blow.energy) / STOP_FPS * strength;
    let ours = local.body() == Some(hit.attacker);
    *stop = Hitstop {
        left: seconds,
        total: seconds,
        attacker: Some(hit.attacker),
        victim: Some(hit.victim),
        camera: if ours { strength } else { 0.0 },
        ..default()
    };
    commands.entity(hit.victim).try_insert(Frozen);
}

/// Returns true while a hitstop is under way.
fn active_now(stop: &Hitstop) -> bool {
    stop.left > 0.0 || stop.victim.is_some()
}

/// The parts of animals, which the shake moves.
type Parts<'w, 's> = Query<'w, 's, &'static mut Transform, (With<Part>, Without<GameCamera>)>;

/// The game camera, which the shake moves when the local robot is in it.
type Lens<'w, 's> =
    Option<Single<'w, 's, &'static mut Transform, (With<GameCamera>, Without<Part>)>>;

/// Moves the local `camera` by the shake of `stop` at `share`, on top of where the rig put it: a
/// camera the rig left alone since the last frame loses last frame's shake first.
fn steady(camera: &mut Transform, stop: &mut Hitstop, share: f32) {
    let shake = Vec3::new(1.0, -0.5, 0.0) * CAMERA_SHAKE * share * stop.camera;
    let [written, before] = stop.lens;
    let base = match camera.translation == written {
        true => written - before,
        false => camera.translation,
    };
    camera.translation = base + shake;
    stop.lens = [camera.translation, shake];
}

/// Runs the hitstop one frame: the animal and the camera shake less and less, and at the end both
/// come back to rest and the animal is let go.
pub fn hold(
    time: Res<Time>,
    mut stop: ResMut<Hitstop>,
    (children, mut parts): (Query<&Children>, Parts),
    lens: Lens,
    mut commands: Commands,
) {
    stop.left -= time.delta_secs();
    stop.frame += 1;
    let done = stop.left <= 0.0;
    let share = if done { 0.0 } else { stop.shake() };
    let offset = Vec3::new(SHAKE, 0.4 * SHAKE, 0.0) * share;
    let delta = offset - stop.shaken;
    stop.shaken = offset;
    let victim = stop.victim;
    for child in victim
        .iter()
        .flat_map(|v| children.get(*v).into_iter().flatten())
    {
        if let Ok(mut part) = parts.get_mut(*child) {
            part.translation += delta;
        }
    }
    if let Some(mut camera) = lens {
        steady(&mut camera, &mut stop, share);
    }
    if done {
        if let Some(victim) = victim {
            commands.entity(victim).try_remove::<Frozen>();
        }
        *stop = Hitstop::default();
    }
}
