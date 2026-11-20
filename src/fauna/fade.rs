//! Fades of the drawn animals: they dither in when they come and out when they go, with the dither of
//! the world's chunks, so nothing pops.

use bevy::prelude::*;

use super::beast::Beast;
use crate::world::dither::{FADE_SECONDS, LEVELS, Way, level, range, redither};

/// The fade of one animal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fade {
    /// Which way it fades.
    pub way: Way,
    /// Game seconds it started.
    pub start: f32,
    /// Level shown, [`LEVELS`] once done.
    pub level: u8,
}

impl Fade {
    /// Returns a fade `way` starting at `now` game seconds.
    pub fn new(way: Way, now: f32) -> Self {
        Self {
            way,
            start: now,
            level: 0,
        }
    }

    /// Returns true while the animal is leaving.
    pub fn leaving(&self) -> bool {
        self.way == Way::Leave
    }
}

/// Starts fading `beast` out at `now` game seconds, from the level it is at, unless it already
/// leaves.
pub fn leave(commands: &mut Commands, beast: &mut Beast, now: f32) {
    if beast.fade.leaving() {
        return;
    }
    let shown = match beast.fade.way {
        Way::In => beast.fade.level,
        _ => LEVELS,
    };
    let hidden = LEVELS - shown;
    beast.fade = Fade {
        level: hidden,
        ..Fade::new(
            Way::Leave,
            now - f32::from(hidden) / f32::from(LEVELS) * FADE_SECONDS,
        )
    };
    for part in beast.parts.iter().flatten() {
        redither(commands, *part, Some(range(Way::Leave, beast.fade.level)));
    }
}

/// Moves every fade on: parts get the dither of their level, a finished fade in drops its range, and
/// a finished fade out despawns the animal.
pub fn fade(mut commands: Commands, time: Res<Time>, mut beasts: Query<(Entity, &mut Beast)>) {
    let now = time.elapsed_secs();
    for (root, mut beast) in &mut beasts {
        let fade = beast.fade;
        let next = level(now - fade.start);
        if next >= LEVELS && fade.way == Way::Leave {
            commands.entity(root).despawn();
            continue;
        }
        if fade.level >= LEVELS || next == fade.level {
            continue;
        }
        beast.fade.level = next;
        let shown = (next < LEVELS).then(|| range(fade.way, next));
        for part in beast.parts.iter().flatten() {
            redither(&mut commands, *part, shown.clone());
        }
    }
}
