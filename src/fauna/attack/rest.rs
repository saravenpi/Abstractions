//! A carcass at rest: it lies on the drawn ground under its body, tilted to the slope there, and lies
//! down on it again whenever that ground is drawn anew.
//!
//! A body that fell kept the height its root had when the animal was spawned, and a carcass drawn again
//! lay level on the finest ground under its middle: on a slope, or once its animal had walked uphill,
//! it lay in the ground.

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use crate::fauna::beast::Beast;
use crate::fauna::ground::Heights;
use crate::fauna::place::Spot;
use crate::space::Placed;
use crate::world::{Chunks, Terrain};

/// How a dead body lies in the flat frame of its animal, before it is laid on the ground.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Lain {
    /// The point of its pose it lies about.
    pub pivot: Vec3,
    /// How it is turned about that point on level ground.
    pub turn: Quat,
    /// Where in the frame, `(x, z)`, that point rests.
    pub at: Vec2,
    /// Where its middle lies from that point along the ground, `(x, z)`.
    pub middle: Vec2,
    /// Metres that point rests over the ground, along the normal of the ground.
    pub side: f32,
    /// Metres from its middle at which the ground under it is read: about half its length.
    pub reach: f32,
}

impl Lain {
    /// Returns the place of the root of the body lying as this says on `ground` of the frame `spot`: the
    /// plane of the ground read around its middle carries its resting point, and tilts the body with it.
    pub fn rest(&self, spot: &Spot, ground: &dyn Fn(Vec2) -> f32) -> Placed {
        let mid = self.at + self.middle;
        let reach = self.reach.max(0.05);
        let read = |d: Vec2| ground(mid + d * reach);
        let (east, west) = (read(Vec2::X), read(Vec2::NEG_X));
        let (north, south) = (read(Vec2::Y), read(Vec2::NEG_Y));
        let rise = Vec2::new(east - west, north - south) / (2.0 * reach);
        let height = 0.25 * (east + west + north + south) + rise.dot(self.at - mid);
        let normal = Vec3::new(-rise.x, 1.0, -rise.y).normalize();
        let turn = Quat::from_rotation_arc(Vec3::Y, normal) * self.turn;
        let land = Vec3::new(self.at.x, height, self.at.y) + normal * self.side;
        Placed {
            pos: spot.planet(land - turn * self.pivot),
            rot: spot.rot * turn.as_dquat(),
        }
    }

    /// Returns how a body that tipped over about `pivot` of its frame, turned by `turn`, lies: resting
    /// `side` metres over the ground there, its middle `height` metres from the pivot along its tipped up.
    pub fn tipped(pivot: Vec3, turn: Quat, (side, height): (f32, f32)) -> Self {
        Self {
            pivot,
            turn,
            at: pivot.xz(),
            middle: (turn * Vec3::Y * height).xz(),
            side,
            reach: height,
        }
    }
}

/// Lays every carcass on the drawn ground under it, as the chunks draw it this frame: a carcass stays on
/// the ground when a chunk under it streams in at another level.
pub fn settle(
    terrain: Res<Terrain>,
    chunks: Option<Res<Chunks>>,
    mut bodies: Query<(&Beast, &Lain, &mut Placed)>,
    heights: Local<Heights>,
) {
    if terrain.is_changed() {
        heights.clear();
    }
    heights.regrid();
    let chunks = chunks.as_deref();
    for (beast, lain, mut placed) in &mut bodies {
        let spot = beast.spot;
        let drawn = |x: f32, z: f32| heights.drawn(&terrain, chunks, x, z);
        let ground = |p: Vec2| spot.ground_by(&terrain, p, drawn);
        placed.set_if_neq(lain.rest(&spot, &ground));
    }
}

/// Registers the laying of the carcasses on the drawn ground: in play, after the collapse of the bodies
/// that just died, while there is a carcass.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        settle
            .after(super::slump::slump)
            .run_if(any_with_component::<Lain>)
            .run_if(crate::screens::playing),
    );
}
