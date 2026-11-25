//! What a key does right now: one press does one thing, the thing the prompt names.
//!
//! Every system that acts on a key press offers its action each frame it is possible, with the
//! name of what it would act on. Before the next frame's systems run, [`Interact::settle`] keeps
//! the one offer that comes first in [`Verb::ALL`]. The prompt line shows it, and a system that
//! shares the interact key acts only when its own action is the one kept. So the key does what the
//! player read on the line, and nothing else.

mod plugin;

pub use plugin::{Interacting, interacting};

use bevy::prelude::*;

/// Number of actions that can be offered.
const VERBS: usize = 8;

/// An action a key does to what the robot looks at or stands by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verb {
    /// Pull a spear out of the animal or the ground it stands in.
    Pull,
    /// Cut a part off a carcass.
    Carve,
    /// Take a thing off the ground into the pack.
    Take,
    /// Open a placed storage chest.
    Open,
    /// Take the seat of the landed ship.
    Board,
    /// Step out of the landed ship.
    Leave,
    /// Break a piece off a rock.
    Stone,
    /// Patch a damaged part of the robot.
    Mend,
}

impl Verb {
    /// Every action, the one the prompt prefers first. The six that share the interact key
    /// come first, so the one of them that is kept is also the one the line shows.
    pub const ALL: [Verb; VERBS] = [
        Verb::Pull,
        Verb::Carve,
        Verb::Take,
        Verb::Open,
        Verb::Board,
        Verb::Leave,
        Verb::Stone,
        Verb::Mend,
    ];
}

/// One action a key would do now, and the name of what it would do it to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offer {
    /// The action.
    pub verb: Verb,
    /// What it acts on, in lower case, or nothing to name.
    pub thing: &'static str,
    /// The mass of what it acts on, in grams, when the prompt says it.
    pub grams: Option<u32>,
}

impl Offer {
    /// Returns the mass of what the action acts on as the prompt says it after its name, such as
    /// " 1.2 kg", or nothing when it has none.
    pub fn weight(&self) -> String {
        let mass = |grams: u32| format!(" {}", crate::ui::units::mass(f64::from(grams) / 1e3));
        self.grams.map_or(String::new(), mass)
    }
}

/// The actions offered this frame and the one kept from the last frame.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Interact {
    /// The offer of each action possible this frame, by the number of its verb.
    offers: [Option<Offer>; VERBS],
    /// The action the prompt shows and the interact key does this frame.
    chosen: Option<Offer>,
}

impl Interact {
    /// Says that `verb` is possible right now on `thing`.
    pub fn offer(&mut self, verb: Verb, thing: &'static str) {
        self.weighed(verb, thing, None);
    }

    /// Says that `verb` is possible right now on `thing`, which weighs `grams`.
    pub fn weighed(&mut self, verb: Verb, thing: &'static str, grams: Option<u32>) {
        self.offers[verb as usize] = Some(Offer { verb, thing, grams });
    }

    /// Returns the action the prompt shows this frame.
    pub fn chosen(&self) -> Option<Offer> {
        self.chosen
    }

    /// Returns true when `verb` is the action the prompt shows this frame: the only action of
    /// the interact key that may act on a press.
    pub fn is(&self, verb: Verb) -> bool {
        self.chosen.is_some_and(|offer| offer.verb == verb)
    }

    /// Returns true on the frame `key` goes down while `verb` is the action the prompt shows.
    pub fn pressed(&self, verb: Verb, keys: &ButtonInput<KeyCode>, key: KeyCode) -> bool {
        self.is(verb) && keys.just_pressed(key)
    }

    /// Keeps the first offer as the chosen action and forgets the offers. Returns false when
    /// nothing changed.
    pub fn settle(&mut self) -> bool {
        let next = Verb::ALL
            .into_iter()
            .find_map(|verb| self.offers[verb as usize]);
        let idle = self.offers.iter().all(Option::is_none);
        if next == self.chosen && idle {
            return false;
        }
        self.chosen = next;
        self.offers = [None; VERBS];
        true
    }
}
