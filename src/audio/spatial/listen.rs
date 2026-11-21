//! The listener: where the ears of the player are, which way they face, and what surrounds them.
//!
//! The ears turn with the camera, so a sound comes from where it is on screen. They sit in the
//! head of the robot whenever it stands in the world, in first person and in third alike: its own
//! steps and its tool keep their distance, and a call is as far as it is from the body, not from
//! a camera that floats behind it. Seated in the ship, or with no body, the ears are the camera:
//! in the cabin with the view from the seat, outside with the chase view.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::air::Air;
use super::spot::{Between, Origin};
use super::tune::{DEEP, EARS, SIGHT};
use crate::audio::scene::AudioScene;
use crate::audio::sense::Here;
use crate::avatar::{Avatar, Own};
use crate::player::above_sea;
use crate::render::GameCamera;
use crate::space::{Anchor, Frame, Location, Travel, altitude, up};
use crate::view::{ViewMode, ViewPrefs};
use crate::world::Terrain;

/// The ears of the player.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Listener {
    /// Where they are, in render space.
    pub at: Vec3,
    /// Which way they face: the turn of the camera.
    pub turn: Quat,
    /// The way up from the ground there.
    pub up: Vec3,
    /// The centre of the listener's own body, while it stands in the world.
    pub body: Option<Vec3>,
    /// The air around them.
    pub air: Air,
    /// Whether they are under water.
    pub wet: bool,
    /// Whether they sit in the closed cabin of the ship: seated, with the view from the seat.
    pub cabin: bool,
}

impl Default for Listener {
    /// Ears at the origin facing ahead, in the air at sea level of home.
    fn default() -> Self {
        Self {
            at: Vec3::ZERO,
            turn: Quat::IDENTITY,
            up: Vec3::Y,
            body: None,
            air: Air::home(),
            wet: false,
            cabin: false,
        }
    }
}

/// What the ears are read from.
#[derive(SystemParam)]
pub struct Ears<'w, 's> {
    /// The frame of the render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The ground and the sea.
    terrain: Option<Res<'w, Terrain>>,
    /// Whether the player is seated.
    location: Option<Res<'w, Location>>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    /// The body of the player at this machine.
    own: Own<'w, 's>,
    /// Where every body is.
    bodies: Query<'w, 's, &'static GlobalTransform, With<Avatar>>,
    /// Whether the view from the seat is chosen.
    prefs: Option<Res<'w, ViewPrefs>>,
}

impl Ears<'_, '_> {
    /// Returns whether the player is seated in the ship, and whether the ears are in its closed
    /// cabin: seated with the view from the seat. A chase view puts them outside.
    fn seat(&self) -> (bool, bool) {
        let seated = self
            .location
            .as_ref()
            .is_some_and(|l| l.travel == Travel::InShip);
        let first = self
            .prefs
            .as_ref()
            .is_none_or(|p| p.ship == ViewMode::First);
        (seated, seated && first)
    }

    /// Returns the air at `at` on the planet `here` says, and whether it is under water: no air
    /// in space, the air of home without a world.
    fn medium(&self, here: &Here, at: Vec3) -> (Air, bool) {
        let (Some(anchor), Some(terrain)) = (self.anchor.as_deref(), self.terrain.as_deref())
        else {
            return (Air::home(), false);
        };
        let surface = matches!(anchor.frame, Frame::Surface(_));
        let air = match (surface, here.0.as_ref()) {
            (true, Some((_, spec))) => {
                Air::of(spec, f64::from(altitude(anchor, terrain.globe(), at)))
            }
            _ => Air::VACUUM,
        };
        let wet = surface && above_sea(anchor, terrain.globe(), at) < -DEEP;
        (air, wet)
    }
}

/// Writes where the ears are, the air around them, thinner and colder up high and none in space,
/// and whether they sit in the cabin, into the listener and the scene.
pub(in crate::audio) fn listen(
    ears: Ears,
    here: Res<Here>,
    mut listener: ResMut<Listener>,
    mut scene: ResMut<AudioScene>,
) {
    let Ok(camera) = ears.camera.single() else {
        return;
    };
    let (seated, cabin) = ears.seat();
    let own = ears
        .own
        .body()
        .and_then(|(_, body)| ears.bodies.get(body).ok());
    let body = own.map(GlobalTransform::translation).filter(|_| !seated);
    let near = body.unwrap_or(camera.translation());
    let up = ears
        .anchor
        .as_deref()
        .map_or(Vec3::Y, |anchor| up(anchor, near));
    let at = body.map_or(near, |body| body + up * EARS);
    let (air, wet) = ears.medium(&here, at);
    let turn = camera.rotation();
    listener.set_if_neq(Listener {
        at,
        turn,
        up,
        body,
        air,
        wet,
        cabin,
    });
    let inside = f32::from(u8::from(cabin));
    if (scene.absorb, scene.cabin) != (air.absorb, inside) {
        (scene.absorb, scene.cabin) = (air.absorb, inside);
    }
}

/// The ground between a source and the listener.
#[derive(SystemParam)]
pub struct Ground<'w> {
    /// The frame of the render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The ground and the sea.
    terrain: Option<Res<'w, Terrain>>,
}

impl Ground<'_> {
    /// Returns what lies between `from` and the `listener`: whether the source is under water and
    /// whether the ground hides it. A sound the body holds is in the listener's own medium.
    pub fn between(&self, listener: &Listener, from: Origin) -> Between {
        let at = match from {
            Origin::At(at, _) | Origin::Arrived(at, _) => at,
            Origin::Flat | Origin::Held(..) => return Between::default(),
        };
        let (Some(anchor), Some(terrain)) = (self.anchor.as_deref(), self.terrain.as_deref())
        else {
            return Between::default();
        };
        let surface = matches!(anchor.frame, Frame::Surface(_));
        let over = |p: Vec3| {
            let chart = terrain.locate(anchor.to_frame(p));
            chart.y - terrain.height(chart.x, chart.z)
        };
        Between {
            wet: surface && above_sea(anchor, terrain.globe(), at) < -DEEP,
            hidden: surface && hidden(over, listener.at, at),
        }
    }
}

/// Returns whether the ground hides `to` from `from`: whether a point read along the line between
/// them, away from both ends, lies under the ground. `over` gives the height of a point over it.
pub fn hidden(over: impl Fn(Vec3) -> f32, from: Vec3, to: Vec3) -> bool {
    let length = from.distance(to);
    let (points, margin) = SIGHT;
    (1..points).any(|k| {
        let along = k as f32 / points as f32;
        let metres = along * length;
        metres > margin && length - metres > margin && over(from.lerp(to, along)) < 0.0
    })
}
