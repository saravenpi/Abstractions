//! The flight as the effects see it this frame, read once so that every effect agrees.

use bevy::ecs::system::SystemParam;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::FlightFx;
use super::under::under;
use crate::avatar::Marks;
use crate::gear::aero::grip;
use crate::gear::servo::ease;
use crate::gear::state::FlightState;
use crate::gear::{Gear, Mode};
use crate::player::Player;
use crate::render::{GameCamera, smoothstep as smooth};
use crate::space::{Anchor, altitude, on_foot, up};
use crate::view::{CameraPose, Enabled};
use crate::world::Terrain;

/// Height of the boot soles under the body centre, in metres.
pub const SOLES: f32 = 0.9;

/// Rate at which the published load follows the felt one, per second.
const LOAD_RATE: f32 = 8.0;

/// Airspeeds between which the cloth starts to flutter and flutters fully.
const FLUTTER_SPEED: (f32, f32) = (10.0, 60.0);

/// Airspeeds between which the rush of air closes in on the view.
const RUSH_SPEED: (f32, f32) = (55.0, 110.0);

/// Where the gear sits on the body as it is drawn, in render space. Left comes first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limbs {
    /// The wingtips.
    pub tips: [Vec3; 2],
    /// The nozzles under the soles.
    pub soles: [Vec3; 2],
    /// Unit direction the exhaust leaves the boots along.
    pub jet: Vec3,
}

impl Limbs {
    /// Returns the marks of the drawn body, which stands at `pose`, in render space.
    pub fn of(marks: &Marks, pose: &Transform) -> Self {
        let jet = pose.rotation * (marks.jets[0] + marks.jets[1]);
        Self {
            tips: marks.tips.map(|tip| pose.transform_point(tip)),
            soles: marks.soles.map(|sole| pose.transform_point(sole)),
            jet: jet.normalize_or(pose.rotation * Vec3::NEG_Y),
        }
    }
}

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
    /// Grip of the gear at this height, from 0 above the air to 1 at sea level. The boots
    /// follow it alone.
    pub sigma: f32,
    /// How much thinner than at home the air of the planet is: 0 with the home air or thicker,
    /// 1 without air. What the air shows, sounds and feels like is `sigma * (1 - thin)`.
    pub thin: f32,
    /// Density of the air at this height in kilograms per cubic metre, thickness of the planet
    /// included: zero above the air and on a planet without any.
    pub air_density: f32,
    /// Free height under the boot soles down to the ground or the sea, in metres.
    pub clearance: f32,
    /// Whether the sea is what lies nearest under the boots.
    pub wet: bool,
    /// Seconds since the last frame.
    pub dt: f32,
    /// Where the gear sits on the drawn body, when a body is drawn. Without one the effects
    /// fall back on fixed offsets in the frame of the gear.
    pub limbs: Option<Limbs>,
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

    /// Returns how hard the boosters push, from 0 to 1, at this height.
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
    /// The marks of the drawn body, when the game draws one.
    marks: Option<Res<'w, Marks>>,
    /// The camera the rig placed this frame.
    rig: Option<Res<'w, CameraPose>>,
    /// Whether the rig drives the camera.
    driven: Option<Res<'w, Enabled>>,
}

impl Sight<'_, '_> {
    /// Returns where the camera is this frame: the pose the rig made, which the transforms
    /// only show from the next frame, or else the camera itself, or else the body.
    fn camera(&self, pose: &Transform) -> Transform {
        let driven = self.driven.as_deref().is_some_and(|on| on.0);
        if let Some(rig) = self.rig.as_deref().filter(|_| driven) {
            return Transform::from_translation(rig.position).with_rotation(rig.rotation);
        }
        self.camera
            .single()
            .map(|at| at.compute_transform())
            .unwrap_or(*pose)
    }
}

/// Returns the scene of a body at `pose` with `gear`, or an inactive one without a world.
fn read(sight: &Sight, pose: &Transform, gear: &Gear) -> Scene {
    let (Some(anchor), Some(terrain)) = (sight.anchor.as_deref(), sight.terrain.as_deref()) else {
        return Scene::default();
    };
    let vertical = up(anchor, pose.translation).as_dvec3();
    let tilt = DQuat::from_rotation_arc(DVec3::Y, vertical).as_quat();
    let height = altitude(anchor, terrain.globe(), pose.translation) - SOLES;
    let (clearance, wet) = under(terrain.globe(), gear.input.clearance, height);
    let camera = sight.camera(pose);
    Scene {
        active: true,
        state: gear.state,
        body: pose.translation,
        tilt,
        rot: tilt * gear.state.attitude,
        eye: camera.translation,
        view: camera.rotation,
        sigma: grip(gear.input.density),
        thin: 1.0 - gear.input.air.clamp(0.0, 1.0),
        air_density: gear.input.density * gear.input.air.max(0.0),
        clearance,
        wet,
        dt: sight.time.delta_secs(),
        limbs: sight.marks.as_deref().map(|marks| Limbs::of(marks, pose)),
    }
}

/// Returns the published values for `scene`, with the load eased from `last`.
fn publish(scene: &Scene, last: &FlightFx) -> FlightFx {
    if !scene.active {
        return FlightFx::default();
    }
    let (state, air) = (&scene.state, scene.sigma.min(1.0) * (1.0 - scene.thin));
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
/// Without a player on foot, and while a menu or the boot screen is up, both go back to their
/// defaults, so every effect and every sound of the flight dies down. The gear itself keeps
/// its state: the flight goes on where it was when the game does.
pub fn observe(
    In(live): In<bool>,
    sight: Sight,
    walking: Option<Res<crate::space::Location>>,
    mut out: (ResMut<Scene>, ResMut<FlightFx>),
) {
    let seen = sight.player.single().ok();
    let seen = seen.filter(|_| live && on_foot(walking));
    let scene = seen.map_or_else(Scene::default, |(pose, gear)| read(&sight, pose, gear));
    let fx = publish(&scene, &out.1);
    out.0.set_if_neq(scene);
    out.1.set_if_neq(fx);
}
