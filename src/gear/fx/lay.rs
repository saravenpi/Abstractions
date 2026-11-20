//! Laying one trail: feeding it from its emitter and writing the mesh of its ribbon around
//! the head of the trail. The trails of the player and those of every other body share it.

use bevy::prelude::*;

use super::ribbon::{Look, Strip, fill, write};
use super::scene::Scene;
use super::trail::Trail;
use super::trails::{Ribbon, emitter, look};
use crate::render::Palette;
use crate::space::Anchor;

/// What a trail is drawn against: the flight that feeds it, the render anchor and the sky.
pub type Against<'a> = (&'a Scene, &'a Anchor, Option<&'a Palette>);

/// Feeds `ribbon` from its emitter and fills `strip` with it around the head of its trail.
/// Returns the render position of that head, or nothing when there is nothing to write.
///
/// A trail nobody feeds, with the player in the ship or gone, is emptied: left as it was it
/// would hang where it was last drawn and, fed again from another place, run straight there.
/// A trail with nothing left to show is written once more, to clear it, and then left alone.
fn lay(ribbon: &mut Ribbon, (scene, anchor, palette): Against, strip: &mut Strip) -> Option<Vec3> {
    match scene.active {
        true => {
            let (at, heat) = emitter(ribbon.source, scene);
            ribbon.trail.feed(anchor.to_frame(at), heat, scene.dt);
        }
        false => ribbon.trail = Trail::default(),
    }
    let style = look(ribbon.source, scene.eye, palette);
    let live = ribbon.trail.live(style.life);
    if !live && !ribbon.drawn {
        return None;
    }
    ribbon.drawn = live;
    let head = anchor.to_render(ribbon.trail.tip().at);
    let around = Look {
        eye: style.eye - head,
        ..style
    };
    fill(
        &ribbon.trail,
        &around,
        |point| anchor.to_render(point) - head,
        strip,
    );
    Some(head)
}

/// Feeds the trail of `ribbon`, moves its entity to the head of the trail and rewrites its
/// mesh when it glows, through `strip`.
pub fn redraw(
    (ribbon, handle, transform): (&mut Ribbon, &Mesh3d, &mut Transform),
    against: Against,
    strip: &mut Strip,
    meshes: &mut Assets<Mesh>,
) {
    let Some(head) = lay(ribbon, against, strip) else {
        return;
    };
    transform.translation = head;
    if let Some(mut mesh) = meshes.get_mut(&handle.0) {
        write(&mut mesh, strip);
    }
}
