//! The dither of a fade: the 4 by 4 Bayer pattern of Bevy's visibility ranges, drawn by the
//! opaque pass itself, with no blending, no sorting and no extra pass.
//!
//! A visibility range fades a mesh with the distance of the camera. A fade over time is the same
//! range with a margin so wide that the distance no longer matters, moved one level at a time.
//!
//! Bevy chooses the dithered pipeline of a mesh once, by whether it has a visibility range then,
//! and chooses again only when the mesh changes. A dithered pipeline with no range discards every
//! pixel, so a range that comes or goes marks the mesh changed.
//!
//! The shadow pass does not dither. A mesh that fades out would cast its whole shadow onto what
//! replaces it until the last frame, so it casts none while it fades.

use bevy::camera::visibility::VisibilityRange;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

/// Seconds a fade lasts.
pub const FADE_SECONDS: f32 = 0.4;

/// Steps of a fade: the sixteen levels of the Bayer pattern.
pub const LEVELS: u8 = 16;

/// Narrowest width in metres of the made-up distance margin a fade is written as: so wide that
/// the true distance of the camera to a node of a small planet never moves the dither level.
pub const SPAN: f32 = 1.0e7;

/// How many times the farthest distance a node can lie from the camera the margin of a fade is
/// wide: at the half of a level, 1/32 of the margin, the distance would move the level.
const SPAN_SHARE: f32 = 64.0;

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

/// Largest power of two a margin is rounded up to, so that it stays a finite `f32`.
const WIDEST: f64 = 126.0;

/// Returns the width in metres of the margin of a fade seen from `altitude` metres over a planet
/// of `radius` metres: [`SPAN`] or, when the planet is big, [`SPAN_SHARE`] times the farthest
/// point of it from the camera, which is its width and the camera's altitude away, rounded up to
/// a power of two.
///
/// Bevy keeps one GPU slot for every distinct visibility range it has ever seen and never frees
/// one: past 65,535 every new range shares slot 0, and chunks blink and vanish. A margin that
/// followed the altitude metre by metre made new ranges on every frame of a flight. Rounded to a
/// power of two, a flight from the ground to the far hand-over uses a handful of margins.
pub fn span(altitude: f64, radius: f64) -> f32 {
    let farthest = 2.0 * radius + altitude.max(0.0);
    let wide = f64::from(SPAN_SHARE) * farthest;
    if wide.is_nan() || wide <= f64::from(SPAN) {
        return SPAN;
    }
    wide.log2().ceil().min(WIDEST).exp2() as f32
}

/// Returns the visibility range that draws `level` sixteenths of what fades in, or hides that
/// many sixteenths of what fades out, at any distance of the camera. Two things at the same
/// level, one each way, fill exactly every pixel between them.
pub fn range(way: Way, level: u8) -> VisibilityRange {
    ranged(way, level, SPAN)
}

/// Returns the visibility range of [`range`] with a margin of `span` metres, which must be wider
/// than [`SPAN_SHARE`] times the distance of the farthest thing that fades, see [`span`].
pub fn ranged(way: Way, level: u8, span: f32) -> VisibilityRange {
    let from = -span * f32::from(level.min(LEVELS)) / f32::from(LEVELS);
    let margin = from..from + span;
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

/// Stops `part` casting shadows while it fades out, `way` being the fade it is in, and lets it
/// cast them again when it is in no fade or fades in.
pub fn casting(commands: &mut Commands, part: Entity, way: Option<Way>) {
    match way {
        Some(Way::Hide | Way::Leave) => {
            commands.entity(part).try_insert(NotShadowCaster);
        }
        Some(Way::In) | None => {
            commands.entity(part).try_remove::<NotShadowCaster>();
        }
    }
}
