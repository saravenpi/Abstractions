//! Sun tracking: once per frame, where the star stands for the camera and what time of day that makes.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::atmosphere::shadow;
use super::celestial::sun_direction;
use super::clock::{START_PHASE, Settled, catch_up};
use super::observer::Observer;
use super::vantage::{LocalSystem, Nearby, Vantage};
use super::{DayClock, GameCamera};
use crate::screens::Screen;
use crate::space::{Anchor, Clock, Frame, Location};
use crate::universe::{BodyId, Universe};

/// Where the star of the current system stands, the one source for the lights, the sky and the bodies.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct SunDir {
    /// Unit vector from the camera toward the star, in render axes.
    pub dir: Vec3,
    /// Sine of the height of the star above the local horizon, from -1 to 1.
    pub height: f32,
    /// Share of the starlight that reaches the camera past the planet: 0 in its shadow.
    pub lit: f32,
}

impl Default for SunDir {
    /// Stands where the early morning sun of a new game does.
    fn default() -> Self {
        let dir = sun_direction(START_PHASE);
        Self {
            dir,
            height: dir.y,
            lit: 1.0,
        }
    }
}

impl SunDir {
    /// Reads the star from an observer near a planet of `radius` metres.
    pub fn of(observer: &Observer, radius: f64) -> Self {
        let sun = observer.sun();
        Self {
            dir: (observer.to_render * sun).as_vec3(),
            height: sun.dot(observer.up()) as f32,
            lit: shadow(observer.eye - observer.center, sun, radius),
        }
    }
}

/// The world the camera is in.
#[derive(SystemParam)]
pub(super) struct Space<'w> {
    universe: Option<Res<'w, Universe>>,
    location: Option<Res<'w, Location>>,
    anchor: Option<Res<'w, Anchor>>,
    clock: ResMut<'w, Clock>,
    time: Res<'w, Time>,
}

impl Space<'_> {
    /// Returns the anchor and the planet the sky is seen from: the planet of a surface anchor, or
    /// the one the player is nearest to in space.
    fn footing(&self) -> Option<(Anchor, BodyId)> {
        let anchor = *self.anchor.as_deref()?;
        let body = match anchor.frame {
            Frame::Surface(body) => body,
            _ => self.location.as_deref()?.body,
        };
        Some((anchor, body))
    }
}

/// Everything the tracking writes.
#[derive(SystemParam)]
pub(super) struct Tracked<'w> {
    vantage: ResMut<'w, Vantage>,
    sun: ResMut<'w, SunDir>,
    day: ResMut<'w, DayClock>,
    local: ResMut<'w, LocalSystem>,
    settled: ResMut<'w, Settled>,
}

/// What the tracking remembers from the frame before.
#[derive(Default)]
pub(super) struct Memory {
    phase: Option<f32>,
    last: Option<(Frame, DVec3)>,
    velocity: Vec3,
}

impl Memory {
    /// Returns the phase somebody else wrote into the day clock since the last frame, if any.
    fn request(&self, phase: f32) -> Option<f32> {
        (self.phase != Some(phase)).then_some(phase)
    }

    /// Returns the camera velocity in render axes, measured in the anchor frame so that a rebase
    /// does not count as motion. A change of frame keeps the last velocity.
    fn velocity(&mut self, anchor: &Anchor, camera: Vec3, dt: f64) -> Vec3 {
        let at = anchor.to_frame(camera);
        if let Some((frame, was)) = self.last.replace((anchor.frame, at))
            && frame == anchor.frame
            && dt > 0.0
        {
            self.velocity = (anchor.rotation * ((at - was) / dt)).as_vec3();
        }
        self.velocity
    }
}

/// Finds the camera in its star system and derives the sun direction, the vantage and the day phase.
///
/// A phase written by anybody else moves universe time forward to that time of day, as long as
/// the camera stands on a planet. The phase a resumed world starts with is settled and moves
/// nothing. Runs after transform propagation so the camera pose is this frame's.
pub(super) fn track(
    mut space: Space,
    cameras: Query<&GlobalTransform, With<GameCamera>>,
    mut out: Tracked,
    mut memory: Local<Memory>,
) {
    let (Some((anchor, body)), Some(universe)) = (space.footing(), space.universe.as_deref())
    else {
        return;
    };
    let Some(camera) = cameras.iter().next().map(GlobalTransform::translation) else {
        return;
    };
    if out.local.0.as_ref().is_none_or(|l| l.id != body.star) {
        out.local.0 = Some(Nearby::read(universe, body.star));
    }
    let settled = std::mem::take(&mut out.settled.0);
    let wanted = memory.request(out.day.phase).filter(|_| !settled);
    let known = out.local.0.as_ref();
    let Some((local, planet)) = known.and_then(|l| Some((l, l.planet(body.index)?))) else {
        return;
    };
    let star = &local.system.star;
    let mut observer = Observer::new(&anchor, camera, planet, star, space.clock.seconds);
    if let Some(wanted) = wanted.filter(|_| matches!(anchor.frame, Frame::Surface(_))) {
        space.clock.seconds += catch_up(observer.phase(), wanted, planet);
        observer = Observer::new(&anchor, camera, planet, star, space.clock.seconds);
    }
    let velocity = memory.velocity(&anchor, camera, space.time.delta_secs_f64());
    *out.vantage = Vantage::of(&observer, local, planet, camera).moving(velocity);
    *out.sun = SunDir::of(&observer, planet.radius);
    out.day.phase = observer.phase();
    memory.phase = Some(out.day.phase);
}

/// Forgets the star system of the last world, so a new world reads its own.
fn forget(mut local: ResMut<LocalSystem>) {
    local.0 = None;
}

/// Registers the sun, the vantage and their tracking.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SunDir>()
        .init_resource::<Vantage>()
        .init_resource::<LocalSystem>()
        .add_systems(OnEnter(Screen::Loading), forget)
        .add_systems(PostUpdate, track.after(TransformSystems::Propagate));
}
