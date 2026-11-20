//! Fades: what arrives where the player can see it dithers in over a moment instead of popping,
//! and what it replaces dithers out. A fade that ends removes its visibility range, so it costs
//! nothing afterwards.

use std::collections::HashMap;

use bevy::prelude::*;

use super::dither::{LEVELS, SPAN, Way, level, ranged, redither};

/// One thing that fades: its mesh entities, which way it goes and where it stands.
struct Fade {
    parts: Vec<Entity>,
    way: Way,
    start: f64,
    level: u8,
}

/// Everything that fades now, by the entity that owns it.
pub struct Fades {
    live: HashMap<Entity, Fade>,
    /// Width in metres of the margin the fades are written with, see [`super::dither::span`].
    pub span: f32,
}

impl Default for Fades {
    /// Returns no fades, with the margin of a small planet.
    fn default() -> Self {
        Self {
            live: HashMap::new(),
            span: SPAN,
        }
    }
}

impl Fades {
    /// Returns which way `root` fades, or nothing when it does not.
    pub fn way(&self, root: Entity) -> Option<Way> {
        self.live.get(&root).map(|fade| fade.way)
    }

    /// Returns true while nothing fades.
    #[cfg(test)]
    pub fn idle(&self) -> bool {
        self.live.is_empty()
    }

    /// Forgets every fade, and despawns what was fading out to leave.
    pub fn clear(&mut self, commands: &mut Commands) {
        for (root, fade) in self.live.drain() {
            if fade.way == Way::Leave {
                commands.entity(root).try_despawn();
            }
        }
    }

    /// Starts the fade of `root`: its mesh entities `parts` go the way of `at` from its time in
    /// seconds. A fade `root` was in stops first.
    pub fn begin(
        &mut self,
        commands: &mut Commands,
        root: Entity,
        parts: Vec<Entity>,
        at: (Way, f64),
    ) {
        let (way, start) = at;
        self.cancel(commands, root);
        for part in &parts {
            redither(commands, *part, Some(ranged(way, 0, self.span)));
        }
        let fade = Fade {
            parts,
            way,
            start,
            level: 0,
        };
        self.live.insert(root, fade);
    }

    /// Fades `root` in with what it draws now while `ghost`, which draws what `root` drew a
    /// moment ago, fades out and goes, both from `now` seconds.
    pub fn cross(&mut self, commands: &mut Commands, root: Entity, ghost: Entity, now: f64) {
        self.begin(commands, ghost, vec![ghost], (Way::Leave, now));
        self.begin(commands, root, vec![root], (Way::In, now));
    }

    /// Stops the fade of `root` where it stands and draws it whole again.
    pub fn cancel(&mut self, commands: &mut Commands, root: Entity) {
        for part in self.live.remove(&root).iter().flat_map(|fade| &fade.parts) {
            redither(commands, *part, None);
        }
    }

    /// Ends the fade of `root`: what faded in is drawn whole, what faded out is hidden or gone.
    fn end(&mut self, commands: &mut Commands, root: Entity) {
        let way = self.live.get(&root).map(|fade| fade.way);
        self.cancel(commands, root);
        match way {
            Some(Way::Hide) => {
                commands.entity(root).try_insert(Visibility::Hidden);
            }
            Some(Way::Leave) => commands.entity(root).try_despawn(),
            Some(Way::In) | None => {}
        }
    }

    /// Moves every fade on to `now` seconds: a new level is written to its entities, and a fade
    /// that reached the last level ends.
    pub fn run(&mut self, commands: &mut Commands, now: f64) {
        let mut done = Vec::new();
        for (root, fade) in self.live.iter_mut() {
            let next = level((now - fade.start) as f32);
            if next >= LEVELS {
                done.push(*root);
            } else if next != fade.level {
                fade.level = next;
                for part in &fade.parts {
                    commands
                        .entity(*part)
                        .try_insert(ranged(fade.way, next, self.span));
                }
            }
        }
        for root in done {
            self.end(commands, root);
        }
    }
}
