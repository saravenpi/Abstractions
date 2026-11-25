//! Placing chests and torches onto planet surfaces and breaking placed props back into items.

use abstractions_net::wire::ChestAct;
use avian3d::prelude::*;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::show::PropEntity;
use super::{ChestWorld, codec};
use crate::craft::ground::{Shed, Thing};
use crate::craft::matter::{PlaceableKind, Stuff};
use crate::craft::pack::{Pack, SlotEntry, Stack};
use crate::player::Player;
use crate::render::GameCamera;
use crate::space::{Anchor, Frame};
use crate::universe::BodyId;

/// Maximum raycast reach in metres for placing a chest or torch.
const PLACE_REACH: f32 = 6.0;

/// Maximum raycast reach in metres for breaking a placed chest or torch.
const BREAK_REACH: f32 = 5.0;

/// Computes prop yaw in radians around `up_frame` so its front faces the player looking along `look`.
fn facing_yaw(anchor: &Anchor, up_frame: DVec3, look: Vec3) -> f32 {
    let base = bevy::math::DQuat::from_rotation_arc(DVec3::Y, up_frame);
    let toward = -(anchor.rotation.inverse() * look.as_dvec3());
    let local = base.inverse() * toward;
    local.x.atan2(local.z) as f32
}

/// Consumes one held placeable from `pack` and emits [`ChestAct::Place`] into `world`.
pub fn place_prop(
    pack: &mut Pack,
    world: &mut ChestWorld,
    body: BodyId,
    dir: [f32; 3],
    yaw: f32,
) -> Option<PlaceableKind> {
    let Some(SlotEntry::Stack {
        stuff: Stuff::Placeable(kind),
        ..
    }) = pack.held_entry().copied()
    else {
        return None;
    };
    if !pack.take_count(&Stuff::Placeable(kind), 1) {
        return None;
    }
    world.emit(ChestAct::Place {
        body,
        dir,
        yaw,
        kind: codec::to_net_kind(kind),
    });
    Some(kind)
}

/// Breaks placed prop `id`, spilling all stored chest items plus the prop item itself into `shed`.
pub fn break_placed(id: u64, world: &mut ChestWorld, shed: &mut Shed) -> bool {
    let Some(prop) = world.chests.find(id).cloned() else {
        return false;
    };
    for item in prop.slots.iter().flatten() {
        if let Some(thing) = codec::decode_to_thing(item) {
            shed.0.push(thing);
        }
    }
    let kind = codec::from_net_kind(prop.kind);
    shed.0.push(Thing::Heap(Stack {
        stuff: Stuff::Placeable(kind),
        material: kind.material(),
        mass: kind.unit_mass(),
    }));
    world.emit(ChestAct::Break { id });
    true
}

/// Camera and player entity queries for prop placement and breaking.
type AimQueries<'w, 's> = (
    Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    Query<'w, 's, Entity, With<Player>>,
);

/// Places a held chest or torch where the camera aims on right-click.
pub(super) fn place(
    buttons: Res<ButtonInput<MouseButton>>,
    anchor: Option<Res<Anchor>>,
    (camera, player): AimQueries,
    space: SpatialQuery,
    (mut pack, mut world): (ResMut<Pack>, ResMut<ChestWorld>),
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    let (Some(anchor), Ok(eye), Ok(me)) = (anchor, camera.single(), player.single()) else {
        return;
    };
    let Frame::Surface(body) = anchor.frame else {
        return;
    };
    let filter = SpatialQueryFilter::default();
    let hit = space.cast_ray_predicate(
        eye.translation(),
        eye.forward(),
        PLACE_REACH,
        true,
        &filter,
        &|e| e != me,
    );
    let Some(hit) = hit else {
        return;
    };
    let at = eye.translation() + eye.forward().as_vec3() * hit.distance;
    let frame_pos = anchor.to_frame(at);
    let dir = frame_pos.normalize_or(DVec3::Y);
    let up_frame = if anchor.flat { DVec3::Y } else { dir };
    let yaw = facing_yaw(&anchor, up_frame, eye.forward().as_vec3());
    let _ = place_prop(&mut pack, &mut world, body, dir.as_vec3().to_array(), yaw);
}

/// Breaks an aimed placed chest or torch on middle-click, spilling contents into [`Shed`].
pub(super) fn break_prop(
    buttons: Res<ButtonInput<MouseButton>>,
    (camera, player): AimQueries,
    props: Query<&PropEntity>,
    space: SpatialQuery,
    (mut world, mut shed): (ResMut<ChestWorld>, ResMut<Shed>),
) {
    if !buttons.just_pressed(MouseButton::Middle) {
        return;
    }
    let (Ok(eye), Ok(me)) = (camera.single(), player.single()) else {
        return;
    };
    let filter = SpatialQueryFilter::default();
    let hit = space.cast_ray_predicate(
        eye.translation(),
        eye.forward(),
        BREAK_REACH,
        true,
        &filter,
        &|e| e != me,
    );
    if let Some(hit) = hit
        && let Ok(prop) = props.get(hit.entity)
    {
        break_placed(prop.id, &mut world, &mut shed);
    }
}
