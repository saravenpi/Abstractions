//! What a key does right now: one press does one thing, the thing the prompt names.
//!
//! Every system that acts on a key press offers its action each frame it is possible, with the
//! name of what it would act on. Before the next frame's systems run, [`settle`] keeps the one
//! offer that comes first in [`Verb::ALL`]. The prompt line shows it, and a system that shares
//! the interact key acts only when its own action is the one kept. So the key does what the
//! player read on the line, and nothing else.

use bevy::prelude::*;

/// Number of actions that can be offered.
const VERBS: usize = 7;

/// An action a key does to what the robot looks at or stands by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verb {
    /// Pull a spear out of the animal or the ground it stands in.
    Pull,
    /// Cut a part off a carcass.
    Carve,
    /// Take a thing off the ground into the pack.
    Take,
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
    /// Every action, the one the prompt prefers first. The five that share the interact key
    /// come first, so the one of them that is kept is also the one the line shows.
    pub const ALL: [Verb; VERBS] = [
        Verb::Pull,
        Verb::Carve,
        Verb::Take,
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
}

/// The actions offered this frame and the one kept from the last frame.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Interact {
    /// What each action would act on, for the actions possible this frame, in the order of
    /// [`Verb::ALL`].
    offers: [Option<&'static str>; VERBS],
    /// The action the prompt shows and the interact key does this frame.
    chosen: Option<Offer>,
}

impl Interact {
    /// Says that `verb` is possible right now on `thing`.
    pub fn offer(&mut self, verb: Verb, thing: &'static str) {
        self.offers[verb as usize] = Some(thing);
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
        let named = |verb: Verb| self.offers[verb as usize].map(|thing| Offer { verb, thing });
        let next = Verb::ALL.into_iter().find_map(named);
        let idle = self.offers.iter().all(Option::is_none);
        if next == self.chosen && idle {
            return false;
        }
        self.chosen = next;
        self.offers = [None; VERBS];
        true
    }
}

/// Keeps the action offered last frame that comes first, for the prompt and the keys of this
/// frame.
fn settle(mut interact: ResMut<Interact>) {
    let mut next = interact.clone();
    if next.settle() {
        *interact = next;
    }
}

/// Registers the interact resource and the system that settles it. Every area that offers or
/// acts on an action adds it, so that each works in a test app of its own.
pub struct Interacting;

impl Plugin for Interacting {
    fn build(&self, app: &mut App) {
        app.init_resource::<Interact>().add_systems(
            PreUpdate,
            settle.run_if(resource_exists_and_changed::<Interact>),
        );
    }
}

/// Adds [`Interacting`] to `app` unless an area already did.
pub fn interacting(app: &mut App) {
    if !app.is_plugin_added::<Interacting>() {
        app.add_plugins(Interacting);
    }
}
