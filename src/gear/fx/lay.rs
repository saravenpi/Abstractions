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

/// Fills `strip` with the ribbon of `trail` around the head of the trail, drawn as `style`,
/// whose eye is in render space. Returns the render position of that head, or nothing when
/// there is nothing to write.
///
/// A trail with nothing left to show is written once more, to clear it, and then left alone:
/// `drawn` remembers whether the mesh shows anything. Every trail of the game is drawn through
/// here, whatever lays it.
pub fn trace(
    trail: &Trail,
    drawn: &mut bool,
    (style, anchor): (Look, &Anchor),
    strip: &mut Strip,
) -> Option<Vec3> {
    let live = trail.live(style.life);
    if !live && !*drawn {
        return None;
    }
    *drawn = live;
    let head = anchor.to_render(trail.tip().at);
    let around = Look {
        eye: style.eye - head,
        ..style
    };
    fill(
        trail,
        &around,
        |point| anchor.to_render(point) - head,
        strip,
    );
    Some(head)
}

/// Feeds `ribbon` from its emitter and fills `strip` with it around the head of its trail.
/// Returns the render position of that head, or nothing when there is nothing to write.
///
/// A trail nobody feeds, with the player in the ship or gone, is emptied: left as it was it
/// would hang where it was last drawn and, fed again from another place, run straight there.
fn lay(ribbon: &mut Ribbon, (scene, anchor, palette): Against, strip: &mut Strip) -> Option<Vec3> {
    match scene.active {
        true => {
            let (at, heat) = emitter(ribbon.source, scene);
            ribbon.trail.feed(anchor.to_frame(at), heat, scene.dt);
        }
        false => ribbon.trail = Trail::default(),
    }
    let style = look(ribbon.source, scene.eye, palette);
    trace(&ribbon.trail, &mut ribbon.drawn, (style, anchor), strip)
}

/// Moves the entity of a ribbon to the `head` of its trail and writes `strip` into its mesh.
pub fn show(
    head: Vec3,
    (handle, transform): (&Mesh3d, &mut Transform),
    strip: &Strip,
    meshes: &mut Assets<Mesh>,
) {
    transform.translation = head;
    if let Some(mut mesh) = meshes.get_mut(&handle.0) {
        write(&mut mesh, strip);
    }
}

/// Feeds the trail of `ribbon`, moves its entity to the head of the trail and rewrites its
/// mesh when it glows, through `strip`.
pub fn redraw(
    (ribbon, handle, transform): (&mut Ribbon, &Mesh3d, &mut Transform),
    against: Against,
    strip: &mut Strip,
    meshes: &mut Assets<Mesh>,
) {
    if let Some(head) = lay(ribbon, against, strip) {
        show(head, (handle, transform), strip, meshes);
    }
}
