//! The flight as the effects see it this frame, read once so that every effect agrees.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::FlightFx;
use crate::gear::aero::grip;
use crate::gear::servo::ease;
use crate::gear::state::FlightState;
use crate::gear::{Gear, Mode};
use crate::player::Player;
use crate::render::{GameCamera, smoothstep as smooth};
use crate::space::{Anchor, altitude, on_foot, up};
use crate::world::Terrain;

/// Height of the boot soles under the body centre, in metres.
pub const SOLES: f32 = 0.9;

/// Rate at which the published load follows the felt one, per second.
const LOAD_RATE: f32 = 8.0;

/// Airspeeds between which the cloth starts to flutter and flutters fully.
const FLUTTER_SPEED: (f32, f32) = (10.0, 60.0);

/// Airspeeds between which the rush of air closes in on the view.
const RUSH_SPEED: (f32, f32) = (55.0, 110.0);

/// The body, the air and the camera of this frame, in render space.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct Scene {
    /// Whether a player on foot was seen this frame. Everything else is stale when false.
    pub active: bool,
    /// The gear state of the player.
    pub state: FlightState,
    /// Body centre.
    pub body: Vec3,
    /// Rotation from the upright frame of the body to render axes.
    pub tilt: Quat,
    /// Orientation of the body model.
    pub rot: Quat,
    /// Camera position.
    pub eye: Vec3,
    /// Camera orientation.
    pub view: Quat,
    /// Grip of the gear on the air, from 0 in vacuum to 1 at sea level.
    pub sigma: f32,
    /// Free height under the boot soles down to the ground or the sea, in metres.
    pub clearance: f32,
    /// Whether the sea is what lies nearest under the boots.
    pub wet: bool,
    /// Seconds since the last frame.
    pub dt: f32,
}

impl Scene {
    /// Returns the render position of a point given in the body model frame.
    pub fn at(&self, offset: Vec3) -> Vec3 {
        self.body + self.rot * offset
    }

    /// Returns the velocity in render axes.
    pub fn velocity(&self) -> Vec3 {
        self.tilt * self.state.velocity
    }

    /// Returns how hard the boosters push, from 0 to 1, in this air.
    pub fn push(&self) -> f32 {
        match self.active && self.state.mode.airborne() {
            true => self.state.power() * self.sigma,
            false => 0.0,
        }
    }
}

/// What the scene is read from.
#[derive(SystemParam)]
pub struct Sight<'w, 's> {
    /// Frame clock.
    time: Res<'w, Time>,
    /// The render anchor, absent before a world exists.
    anchor: Option<Res<'w, Anchor>>,
    /// The terrain, for the height above the sea.
    terrain: Option<Res<'w, Terrain>>,
    /// The player body and its gear.
    player: Query<'w, 's, (&'static Transform, &'static Gear), With<Player>>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// Returns the scene of a body at `pose` with `gear`, or an inactive one without a world.
fn read(sight: &Sight, pose: &Transform, gear: &Gear) -> Scene {
    let (Some(anchor), Some(terrain)) = (sight.anchor.as_deref(), sight.terrain.as_deref()) else {
        return Scene::default();
    };
    let vertical = up(anchor, pose.translation).as_dvec3();
    let tilt = DQuat::from_rotation_arc(DVec3::Y, vertical).as_quat();
    let sea = altitude(anchor, terrain.globe(), pose.translation) - SOLES;
    let ground = gear.input.clearance;
    let camera = sight
        .camera
        .single()
        .map(|at| at.compute_transform())
        .unwrap_or(*pose);
    Scene {
        active: true,
        state: gear.state,
        body: pose.translation,
        tilt,
        rot: tilt * gear.state.attitude,
        eye: camera.translation,
        view: camera.rotation,
        sigma: grip(gear.input.density),
        clearance: ground.min(sea).max(0.0),
        wet: sea < ground,
        dt: sight.time.delta_secs(),
    }
}

/// Returns the published values for `scene`, with the load eased from `last`.
fn publish(scene: &Scene, last: &FlightFx) -> FlightFx {
    if !scene.active {
        return FlightFx::default();
    }
    let (state, air) = (&scene.state, scene.sigma.min(1.0));
    let load = if state.mode == Mode::Off {
        1.0
    } else {
        state.load
    };
    let taut = state.spread() * smooth(FLUTTER_SPEED.0, FLUTTER_SPEED.1, state.airspeed);
    FlightFx {
        airspeed: state.airspeed,
        load: last.load + (load - last.load) * ease(LOAD_RATE, scene.dt),
        thrust: scene.push(),
        wings: state.spread(),
        mode: state.mode,
        flutter: taut * air * (0.6 + 0.4 * (state.load / 2.0).min(1.0)),
        rush: smooth(RUSH_SPEED.0, RUSH_SPEED.1, state.airspeed) * air,
    }
}

/// Reads the player, the air and the camera into [`Scene`] and publishes [`FlightFx`].
///
/// Without a player on foot both go back to their defaults, so every effect dies down.
pub fn observe(
    sight: Sight,
    walking: Option<Res<crate::space::Location>>,
    mut out: (ResMut<Scene>, ResMut<FlightFx>),
) {
    let seen = sight.player.single().ok().filter(|_| on_foot(walking));
    let scene = seen.map_or_else(Scene::default, |(pose, gear)| read(&sight, pose, gear));
    let fx = publish(&scene, &out.1);
    out.0.set_if_neq(scene);
    out.1.set_if_neq(fx);
}
