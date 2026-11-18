//! The dither of a fade: the 4 by 4 Bayer pattern of Bevy's visibility ranges, drawn by the
//! opaque pass itself, with no blending, no sorting and no extra pass.
//!
//! A visibility range fades a mesh with the distance of the camera. A fade over time is the same
//! range with a margin so wide that the distance no longer matters, moved one level at a time.
//!
//! Bevy chooses the dithered pipeline of a mesh once, by whether it has a visibility range then,
//! and chooses again only when the mesh changes. A dithered pipeline with no range discards every
//! pixel, so a range that comes or goes marks the mesh changed.

use bevy::camera::visibility::VisibilityRange;
use bevy::prelude::*;

/// Seconds a fade lasts.
pub const FADE_SECONDS: f32 = 0.4;

/// Steps of a fade: the sixteen levels of the Bayer pattern.
pub const LEVELS: u8 = 16;

/// Width in metres of the made-up distance margin a fade is written as. It is so wide that the
/// true distance of the camera, a few thousand metres at most, never moves the dither level.
const SPAN: f32 = 1.0e7;

/// Which way something fades, and what becomes of it at the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    /// It appears and stays.
    In,
    /// It disappears and is hidden.
    Hide,
    /// It disappears and is despawned.
    Leave,
}

/// Returns the level of a fade that began `age` seconds ago: 0 at the start, [`LEVELS`] done.
pub fn level(age: f32) -> u8 {
    ((age / FADE_SECONDS).clamp(0.0, 1.0) * f32::from(LEVELS)) as u8
}

/// Returns the visibility range that draws `level` sixteenths of what fades in, or hides that
/// many sixteenths of what fades out, at any distance of the camera. Two things at the same
/// level, one each way, fill exactly every pixel between them.
pub fn range(way: Way, level: u8) -> VisibilityRange {
    let from = -SPAN * f32::from(level.min(LEVELS)) / f32::from(LEVELS);
    let margin = from..from + SPAN;
    match way {
        Way::In => VisibilityRange {
            start_margin: margin,
            end_margin: f32::MAX..f32::MAX,
            use_aabb: false,
        },
        Way::Hide | Way::Leave => VisibilityRange {
            start_margin: from - 1.0..from - 1.0,
            end_margin: margin,
            use_aabb: false,
        },
    }
}

/// Gives `part` the visibility range `range`, or takes its range away, and marks its mesh
/// changed so that Bevy draws it with or without the dither from the next frame on.
pub fn redither(commands: &mut Commands, part: Entity, range: Option<VisibilityRange>) {
    commands
        .entity(part)
        .queue_silenced(move |mut entity: EntityWorldMut| {
            match range {
                Some(range) => entity.insert(range),
                None => entity.remove::<VisibilityRange>(),
            };
            if let Some(mut mesh) = entity.get_mut::<Mesh3d>() {
                mesh.set_changed();
            }
        });
}
