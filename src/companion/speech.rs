//! When the companion speaks: on what matters, and rarely on its own.
//!
//! Every chirp has a reason. A warning, under water, always gets through, with a pause between two.
//! An event, a greeting as it wakes, a discovery, a takeoff or a lift-off, gets through once nothing
//! was said for a moment. What it says on its own, a hum or a change of face, waits minutes between two,
//! never sounds while the player flies or sits in the ship, and never at all when the player turned its
//! voice off; off silences everything.

use super::idle::Idle;
use super::mind::Cue;
use super::mood::ChirpKind;
use super::sense::CompanionSense;
use super::tune::{CHATTER_GAP, SPEAK_GAP, WARN_GAP, WORRY_DEPTH};
use crate::settings::Chatter;

/// Why the companion would speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// A danger to the player: water over the head.
    Warning,
    /// Something happened: it woke, the player found something, took off or lifted off.
    Event,
    /// Nothing in particular: a hum, a change of face.
    Chatter,
}

/// What the companion would say, and why.
pub type Said = Option<(ChirpKind, Why)>;

/// The seconds since the companion last spoke, last warned and last chattered.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speech {
    /// Seconds since anything was said.
    spoke: f32,
    /// Seconds since the last warning.
    warned: f32,
    /// Seconds since the last chatter.
    chattered: f32,
}

impl Default for Speech {
    /// Starts ready to speak and to warn, with the first chatter minutes away.
    fn default() -> Self {
        Self {
            spoke: f32::MAX,
            warned: f32::MAX,
            chattered: 0.0,
        }
    }
}

/// Returns what the mind says when `cue` happens: a greeting as it wakes, the first notes of a hum.
pub fn spoken(cue: Cue) -> Said {
    match cue {
        Cue::Woke => Some((ChirpKind::Wake, Why::Event)),
        Cue::Started(Idle::Hum) => Some((ChirpKind::Hum, Why::Chatter)),
        Cue::Started(_) | Cue::Dozed => None,
    }
}

/// Returns why the companion would say `kind` on a change of face under `sense`: a worried face is a
/// warning under water, any other face is chatter.
pub fn reason(kind: ChirpKind, sense: &CompanionSense) -> Why {
    match kind == ChirpKind::Worried && sense.depth > WORRY_DEPTH {
        true => Why::Warning,
        false => Why::Chatter,
    }
}

/// Returns the seconds between two chatters for `voice`.
fn chatter_gap(voice: Chatter) -> f32 {
    match voice {
        Chatter::Often => CHATTER_GAP.0,
        Chatter::Rare => CHATTER_GAP.1,
        Chatter::Off => f32::INFINITY,
    }
}

impl Speech {
    /// Moves the clocks on by `dt` seconds and returns the chirp to send of what the companion would
    /// say, if it may: `voice` is the setting, `aloft` true while the player flies or sits in the ship.
    pub fn hear(
        &mut self,
        said: Said,
        (voice, aloft): (Chatter, bool),
        dt: f32,
    ) -> Option<ChirpKind> {
        self.spoke += dt;
        self.warned += dt;
        self.chattered += dt;
        let (kind, why) = said?;
        let quiet = self.spoke >= SPEAK_GAP;
        let may = match (voice, why) {
            (Chatter::Off, _) => false,
            (_, Why::Warning) => self.warned >= WARN_GAP,
            (_, Why::Event) => quiet,
            (_, Why::Chatter) => !aloft && quiet && self.chattered >= chatter_gap(voice),
        };
        if !may {
            return None;
        }
        self.spoke = 0.0;
        match why {
            Why::Warning => self.warned = 0.0,
            Why::Chatter => self.chattered = 0.0,
            Why::Event => {}
        }
        Some(kind)
    }
}
