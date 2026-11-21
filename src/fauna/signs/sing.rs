//! The singing: the calls heard lately move the bodies that make them, each frame, to the rhythm the
//! sound follows. A call is placed at its universe time, so every player sees a herd call at the
//! same moment. One member makes a contact, young or distress call; the whole herd an alarm, a chase
//! or a threat.

use bevy::prelude::*;

use super::tune::CHORUS;
use crate::fauna::beast::Beast;
use crate::fauna::motion::Song;
use crate::fauna::want::Lead;
use crate::life::signs::calls::{Calling, Rhythm};
use crate::space::Clock;
use crate::universe::hash::{Rng, child};

/// The calls whose bodies may still be moving, with their rhythm.
#[derive(Resource, Default)]
pub struct Sung {
    /// Calls heard lately and their rhythm, oldest first.
    pub calls: Vec<(Calling, Rhythm)>,
}

impl Sung {
    /// Forgets the calls that ended before universe time `now`, then adds `heard` with the rhythm
    /// `rhythm` gives each.
    pub fn remember(
        &mut self,
        now: f64,
        heard: impl Iterator<Item = Calling>,
        rhythm: impl Fn(&Calling) -> Option<Rhythm>,
    ) {
        self.calls
            .retain(|(c, r)| c.at + f64::from(r.seconds()) + 1.0 > now);
        self.calls
            .extend(heard.filter_map(|c| rhythm(&c).map(|r| (c, r))));
    }

    /// Returns the latest call member `slot` of herd `herd` is making at universe time `now`, as the
    /// song its body moves to. A herd cry reaches each member up to [`CHORUS`] seconds after the
    /// first, seeded by its slot, so a herd calls as a ragged chorus and never in lockstep.
    pub fn song(&self, (herd, slot): (u64, u8), now: f64) -> Option<Song> {
        let (calling, rhythm) = self.calls.iter().rev().find(|(c, r)| {
            let age = now - c.at;
            c.herd == herd
                && (c.cry.whole() || c.slot == slot)
                && (0.0..f64::from(r.seconds())).contains(&age)
        })?;
        let lag = match calling.cry.whole() {
            true => CHORUS * Rng::new(child(herd, u64::from(slot))).unit(),
            false => 0.0,
        };
        Some(Song {
            age: (now - calling.at - lag) as f32,
            rhythm: *rhythm,
            cry: calling.cry,
        })
    }
}

/// Returns true while any call heard lately may still move a body.
pub fn sounding(sung: Res<Sung>) -> bool {
    !sung.calls.is_empty()
}

/// Sets the song of every drawn animal making one of the calls heard lately.
pub fn sing(clock: Res<Clock>, sung: Res<Sung>, mut beasts: Query<&mut Beast>) {
    let now = clock.seconds;
    for mut beast in &mut beasts {
        if matches!(beast.lead, Lead::Carcass { .. }) {
            continue;
        }
        if let Some(song) = sung.song(beast.lead.key(), now) {
            beast.motion.song = Some(song);
        }
    }
}
