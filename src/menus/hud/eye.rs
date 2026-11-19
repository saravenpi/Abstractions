//! The eye of the HUD: where the camera is and looks for this frame, worked out once after the
//! camera rig moved it and the anchor settled, so that the compass and the minimap agree.
//!
//! The pose comes from the `Transform` the rig wrote and the body that carries the camera,
//! through the anchor of this frame. Last frame's `GlobalTransform` is never read: after a
//! rebase it still holds the render origin of the frame before, which moves every mark.

use bevy::math::DVec3;
use bevy::prelude::*;

use crate::player::Player;
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::ship::ShipSet;
use crate::space::Anchor;
use crate::view::ViewSet;

use super::MarkerSet;

/// The camera and the body of the player for one frame, in the frame of `anchor`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Eye {
    /// The anchor of this frame, which every other field is given in.
    pub anchor: Anchor,
    /// Where the camera is, in metres in the anchor frame.
    pub at: DVec3,
    /// The way the camera looks, a unit vector in the axes of the anchor frame.
    pub forward: DVec3,
    /// The top of the view, a unit vector in the axes of the anchor frame.
    pub up: DVec3,
    /// The nose and the top of the body of the player, when it has one.
    pub body: Option<(DVec3, DVec3)>,
}

/// The eye of the HUD for this frame, or nothing without a camera or an anchor.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct HudPose(pub Option<Eye>);

/// The camera with the body that carries it, if any.
type Camera<'a> = (&'a Transform, Option<&'a ChildOf>);

/// Works out the eye of this frame from the camera `Transform`, composed with the one of the
/// body that carries it, and the anchor.
pub(super) fn capture(
    anchor: Option<Res<Anchor>>,
    cameras: Query<Camera, With<GameCamera>>,
    holders: Query<&Transform, Without<GameCamera>>,
    players: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut pose: ResMut<HudPose>,
) {
    let eye = anchor
        .zip(cameras.iter().next())
        .map(|(anchor, (local, parent))| {
            let holder = parent.and_then(|parent| holders.get(parent.parent()).ok());
            let world = holder.map_or(*local, |holder| holder.mul_transform(*local));
            let back = anchor.rotation.inverse();
            let axis = |dir: Dir3| back * dir.as_dvec3();
            Eye {
                anchor: *anchor,
                at: anchor.to_frame(world.translation),
                forward: axis(world.forward()),
                up: axis(world.up()),
                body: players
                    .iter()
                    .next()
                    .map(|body| (axis(body.forward()), axis(body.up()))),
            }
        });
    pose.set_if_neq(HudPose(eye));
}

/// Registers the eye, worked out after the rig and the ship placed everything, before the
/// compass and the minimap read it.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HudPose>().add_systems(
        Update,
        capture
            .after(ViewSet::Rig)
            .after(ShipSet::Place)
            .before(MarkerSet::Show)
            .run_if(in_state(Screen::Gameplay)),
    );
}
