//! The one prompt line: a key and what it does, bottom centre above the hotbar. Exactly one
//! row, the highest priority wins: a warning or block sentence, what was just picked up, a course
//! that runs, a course ready to go, the action a key does right now to what the robot looks at or stands by
//! ([`crate::player::Interact`]: pull a spear, cut a part, board, leave, break stone off, mend), a
//! first-time hint, nothing.

use bevy::prelude::*;

use super::context::{Context, Element, Where};
use super::course::{Course, Words, words};
use super::fade::gate;
use super::hints::{Hint, Hinted};
use crate::computer::{Destination, Keys, RouteStatus};
use crate::craft::pack::{Notice, PICK_KEY};
use crate::menus::key_name;
use crate::player::{Interact, Offer, Verb};
use crate::settings::{Keymap, Persistent, Settings};
use crate::space::{Location, Travel};
use crate::ui::{KEY_GAP, key_cap, text, theme};
use crate::universe::Universe;

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
    /// A first-time hint: its key and its words.
    pub hint: Option<(KeyCode, &'static str)>,
}

/// Returns the one row the line shows: its key, if it has one, and its words. A block beats the
/// notice of what was just picked up, which beats a running course, which beats a course ready to
/// go, which beats the action of a key, which beats a hint.
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
        .or_else(|| {
            state
                .hint
                .map(|(key, words)| (Some(key), words.to_string()))
        })
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

/// The row of the prompt: its words, then the key that does it in the muted tone of a key, on a
/// panel, hidden until there is something to say.
fn row() -> impl Bundle {
    (
        Prompt,
        gate(Element::Prompt),
        Node {
            display: Display::None,
            align_items: AlignItems::Center,
            column_gap: px(KEY_GAP),
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        children![
            (
                PromptAction,
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
            (PromptKey, Node::default(), key_cap("")),
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
pub(super) struct Reach<'w> {
    location: Option<Res<'w, Location>>,
    interact: Option<Res<'w, Interact>>,
    settings: Res<'w, Persistent<Settings>>,
    destination: Res<'w, Destination>,
    route: Res<'w, RouteStatus>,
    universe: Option<Res<'w, Universe>>,
    hinted: Res<'w, Hinted>,
    now: Res<'w, Where>,
    notice: Option<Res<'w, Notice>>,
}

/// A key with the words of what it does.
type Keyed = Option<(KeyCode, &'static str)>;

impl Reach<'_> {
    /// Returns the course as the adapter reads it, when one is set.
    fn course(&self) -> Option<Course> {
        let target = self.destination.0?;
        Some(Course {
            name: target.name(self.universe.as_deref()?),
            route: *self.route,
            gap: None,
        })
    }

    /// Returns the action row and the hint row: what a key does right now and the hint in turn.
    fn keys(&self) -> (Option<(KeyCode, String)>, Keyed) {
        let keys = &self.settings.controls.keys;
        let chosen = self.interact.as_deref().and_then(Interact::chosen);
        let act = chosen.map(|offer| offered(offer, keys));
        let hint = self
            .hinted
            .0
            .map(|hint: Hint| (hint.key(keys), hint.words()));
        (act, hint)
    }

    /// Returns the state of the line for this frame: empty while gliding or in a menu.
    fn state(&self) -> State {
        let travel = self.location.as_deref().map(|at| at.travel);
        let mut state = State::default();
        if matches!(self.now.0, Context::Gliding | Context::Menu) {
            return state;
        }
        (state.act, state.hint) = self.keys();
        let picked = self.notice.as_deref().and_then(Notice::shown);
        state.notice = picked.map(|words| (PICK_KEY, words.to_string()));
        let keys = &self.settings.controls.keys;
        let seated = travel == Some(Travel::InShip);
        let said = self
            .course()
            .map(|course| words(&course, seated, &Keys::of(keys)));
        let Words {
            block,
            running,
            ready,
        } = said.unwrap_or_default();
        state.block = block;
        state.running = running.map(|words| (keys.engage, words));
        state.ready = ready.map(|words| (keys.engage, words));
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
    mut rows: Query<(&mut Node, &mut BackgroundColor), With<Prompt>>,
    (mut actions, mut keys): Texts,
) {
    let wanted = choose(&reach.state());
    if wanted == *shown && fresh.is_empty() {
        return;
    }
    shown.clone_from(&wanted);
    let display = [Display::None, Display::Flex][usize::from(wanted.is_some())];
    let fill = [theme::PANEL_DEEP, theme::PANEL]
        [usize::from(wanted.as_ref().is_none_or(|(key, _)| key.is_some()))];
    for (mut node, mut back) in &mut rows {
        node.display = display;
        back.set_if_neq(BackgroundColor(fill));
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
