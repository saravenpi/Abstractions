//! The one prompt line: a key and what it does, bottom centre above the hotbar. Exactly one
//! row, the highest priority wins: a warning or block sentence, what was just picked up, a course
//! or an orbital cruise that runs, a course or a cruise ready to go, the action a key does right now to what the robot looks at or
//! stands by ([`crate::player::Interact`]: pull a spear, cut a part, board, leave, break stone off,
//! mend), or nothing.

use bevy::prelude::*;

use super::context::{Context, Element, Where};
use super::course::{Course, words};
use super::cruise::{merge, read};
use super::fade::gate;
use super::glide::{Bodies, mine, spoken};
use crate::computer::{Destination, Keys, RouteStatus};
use crate::craft::pack::{Notice, PICK_KEY};
use crate::home::Base;
use crate::menus::key_name;
use crate::player::{Interact, Offer, Verb};
use crate::settings::{Keymap, Persistent, Settings};
use crate::ship::{Hatch, Orbital};
use crate::space::{Location, Travel};
use crate::ui::{KEY_GAP, text, theme};
use crate::universe::Universe;
use crate::wind::Thickness;

/// What the line says while the pilot of a landed ship cannot step out yet: the ground beside
/// the hatch has no collider.
pub const UNREADY: &str = "Ground not ready.";

/// Marks the prompt row.
#[derive(Component)]
pub(super) struct Prompt;

/// Marks the text that names the action of the prompt.
#[derive(Component)]
pub(super) struct PromptAction;

/// Marks the text that names the key of the prompt.
#[derive(Component)]
pub(super) struct PromptKey;

/// Everything that can claim the prompt line, one field per priority.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct State {
    /// A warning or block sentence, with no key.
    pub block: Option<String>,
    /// What the robot just picked up, with the key that picked it.
    pub notice: Option<(KeyCode, String)>,
    /// A course that runs: its key and what it is doing.
    pub running: Option<(KeyCode, String)>,
    /// A course ready to go: its key and what it will do.
    pub ready: Option<(KeyCode, String)>,
    /// The action a key does right now: its key and its words.
    pub act: Option<(KeyCode, String)>,
}

/// Returns the one row the line shows: its key, if it has one, and its words. A block beats the
/// notice of what was just picked up, which beats a running course, which beats a course ready to
/// go, which beats the action of a key.
pub fn choose(state: &State) -> Option<(Option<KeyCode>, String)> {
    let key = |(key, words): &(KeyCode, String)| (Some(*key), words.clone());
    state
        .block
        .as_ref()
        .map(|sentence| (None, sentence.clone()))
        .or_else(|| state.notice.as_ref().map(key))
        .or_else(|| state.running.as_ref().map(key))
        .or_else(|| state.ready.as_ref().map(key))
        .or_else(|| state.act.as_ref().map(key))
}

/// Returns the key that does the action of `offer` under the bindings `keys`, and the words that
/// name it and what it acts on.
pub fn offered(offer: Offer, keys: &Keymap) -> (KeyCode, String) {
    let thing = offer.thing;
    match offer.verb {
        Verb::Pull => (keys.ship.board, format!("Pull the {thing}")),
        Verb::Carve => (keys.ship.board, format!("Cut the {thing}")),
        Verb::Board => (keys.ship.board, "Board".to_string()),
        Verb::Leave => (keys.ship.board, "Leave the ship".to_string()),
        Verb::Stone => (PICK_KEY, "Break stone off".to_string()),
        Verb::Mend => (keys.mend, format!("Mend the {}", thing.to_lowercase())),
    }
}

/// The row of the prompt: its words, then the key that does it, with no panel behind them,
/// hidden until there is something to say.
fn row() -> impl Bundle {
    (
        Prompt,
        gate(Element::Prompt),
        Node {
            display: Display::None,
            align_items: AlignItems::Center,
            column_gap: px(KEY_GAP),
            ..default()
        },
        children![
            (
                PromptAction,
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
            (PromptKey, Node::default(), super::switch::cap()),
        ],
    )
}

/// The hidden prompt, bottom centre, above the row of the hotbar. `update` fills it.
pub(super) fn line() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            bottom: px(theme::EDGE + crate::craft::ui::hotbar::SIDE + theme::S2),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![row()],
    )
}

/// What decides the prompt.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Reach<'w, 's> {
    location: Option<Res<'w, Location>>,
    interact: Option<Res<'w, Interact>>,
    hatch: Option<Res<'w, Hatch>>,
    settings: Res<'w, Persistent<Settings>>,
    destination: Res<'w, Destination>,
    route: Res<'w, RouteStatus>,
    universe: Option<Res<'w, Universe>>,
    now: Res<'w, Where>,
    air: Option<Res<'w, Thickness>>,
    players: Bodies<'w, 's>,
    notice: Option<Res<'w, Notice>>,
    orbital: Option<Res<'w, Orbital>>,
    base: Option<Res<'w, Base>>,
}

impl Reach<'_, '_> {
    /// Returns the course as the adapter reads it, when one is set.
    fn course(&self) -> Option<Course> {
        let target = self.destination.0?;
        Some(Course {
            name: target.name(self.universe.as_deref()?),
            route: *self.route,
            gap: None,
        })
    }

    /// Returns the rows of the wings and of the action key, see [`spoken`]: in a fall the key
    /// that spreads the wings comes before any other action.
    fn act(&self) -> State {
        let keys = &self.settings.controls.keys;
        let air = self.air.as_deref().map_or(1.0, |air| air.0);
        let (block, wings) = spoken(mine(&self.players), air, keys.jump, self.now.0);
        let chosen = self.interact.as_deref().and_then(Interact::chosen);
        let act = wings.or_else(|| chosen.map(|offer| offered(offer, keys)));
        State {
            block,
            act,
            ..State::default()
        }
    }

    /// Returns the state of the line: empty in a menu, the wings alone while gliding. The cruise
    /// takes the course rows while it runs; one ready to start comes after a course ready to go.
    fn state(&self) -> State {
        let mut state = self.act();
        if matches!(self.now.0, Context::Gliding | Context::Menu) {
            state.act = None;
            return state;
        }
        let keys = &self.settings.controls.keys;
        let picked = self.notice.as_deref().and_then(Notice::shown);
        state.notice = picked.map(|words| (PICK_KEY, words.to_string()));
        let seated = self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip);
        let told = self
            .course()
            .map(|course| words(&course, seated, &Keys::of(keys)));
        let found = (self.location.as_deref(), self.base.as_deref());
        let cruise = read(
            self.orbital.as_deref(),
            found,
            self.destination.0.is_some(),
            keys,
        );
        let (block, running, ready) = merge(cruise, told.unwrap_or_default(), keys.engage);
        let waiting = self
            .hatch
            .as_deref()
            .is_some_and(|hatch| hatch.waiting && seated);
        let unready = || waiting.then(|| UNREADY.to_string());
        state.block = state.block.or(block).or_else(unready);
        state.running = running;
        state.ready = ready;
        state
    }
}

/// The texts of the prompt: the action and the key.
type Texts<'w, 's> = (
    Query<
        'w,
        's,
        (&'static mut Text, &'static mut TextColor),
        (With<PromptAction>, Without<PromptKey>),
    >,
    Query<
        'w,
        's,
        (&'static mut Text, &'static mut Node),
        (With<PromptKey>, Without<PromptAction>, Without<Prompt>),
    >,
);

/// Shows the row the line chooses, and rewrites it only when it changes.
pub(super) fn update(
    reach: Reach,
    mut shown: Local<Option<(Option<KeyCode>, String)>>,
    fresh: Query<(), Added<Prompt>>,
    mut rows: Query<&mut Node, With<Prompt>>,
    (mut actions, mut keys): Texts,
) {
    let wanted = choose(&reach.state());
    if wanted == *shown && fresh.is_empty() {
        return;
    }
    shown.clone_from(&wanted);
    let display = [Display::None, Display::Flex][usize::from(wanted.is_some())];
    for mut node in &mut rows {
        node.display = display;
    }
    let Some((key, words)) = wanted else {
        return;
    };
    let cap = key.map_or(String::new(), key_name);
    let tone = [theme::WARN, theme::TEXT][usize::from(key.is_some())];
    for (mut text, mut color) in &mut actions {
        text.set_if_neq(Text(words.clone()));
        color.set_if_neq(TextColor(tone));
    }
    let display = [Display::Flex, Display::None][usize::from(cap.is_empty())];
    for (mut text, mut node) in &mut keys {
        text.set_if_neq(Text(cap.clone()));
        node.display = display;
    }
}
