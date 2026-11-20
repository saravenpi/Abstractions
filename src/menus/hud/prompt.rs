//! The one prompt line: a key and what it does, bottom centre at the screen edge. Exactly one
//! row, the highest priority wins: a warning or block sentence, a course that runs, a course
//! ready to go, the board key, a first-time hint, nothing.

use bevy::prelude::*;

use super::context::{Context, Element, Where};
use super::course::{Course, Words, words};
use super::fade::gate;
use super::hints::{Hint, Hinted};
use crate::computer::{Destination, Keys, RouteStatus};
use crate::menus::key_name;
use crate::settings::{Persistent, Settings};
use crate::ship::{Hatch, ShipTelemetry};
use crate::space::{Location, Travel};
use crate::ui::{Tone, label, text, theme};
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
    /// A course that runs: its key and what it is doing.
    pub running: Option<(KeyCode, String)>,
    /// A course ready to go: its key and what it will do.
    pub ready: Option<(KeyCode, String)>,
    /// The board key and its action.
    pub board: Option<(KeyCode, &'static str)>,
    /// A first-time hint: its key and its words.
    pub hint: Option<(KeyCode, &'static str)>,
}

/// Returns the one row the line shows: its key, if it has one, and its words. A block beats a
/// running course, which beats a course ready to go, which beats the board key, which beats a
/// hint.
pub fn choose(state: &State) -> Option<(Option<KeyCode>, String)> {
    let key = |(key, words): &(KeyCode, String)| (Some(*key), words.clone());
    state
        .block
        .as_ref()
        .map(|sentence| (None, sentence.clone()))
        .or_else(|| state.running.as_ref().map(key))
        .or_else(|| state.ready.as_ref().map(key))
        .or_else(|| {
            state
                .board
                .map(|(key, words)| (Some(key), words.to_string()))
        })
        .or_else(|| {
            state
                .hint
                .map(|(key, words)| (Some(key), words.to_string()))
        })
}

/// Returns what the board key does right now, or nothing when it does nothing: boarding from
/// the hatch of the landed ship, or leaving the landed ship.
pub fn board_prompt(travel: Travel, at_hatch: bool, landed: bool) -> Option<&'static str> {
    match travel {
        Travel::OnFoot if at_hatch => Some("Board"),
        Travel::InShip if landed => Some("Leave the ship"),
        _ => None,
    }
}

/// The row of the prompt: a key and its words on a panel, hidden until there is something to say.
fn row() -> impl Bundle {
    (
        Prompt,
        gate(Element::Prompt),
        Node {
            display: Display::None,
            align_items: AlignItems::Center,
            column_gap: px(theme::S2),
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        BackgroundColor(theme::PANEL),
        children![
            (PromptKey, label("", theme::FONT_SMALL, Tone::Plain)),
            (
                PromptAction,
                text("", theme::FONT_SMALL, theme::TEXT),
                TextLayout::no_wrap()
            ),
        ],
    )
}

/// The hidden prompt, bottom centre at the screen edge. `update` fills it.
pub(super) fn line() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            bottom: px(theme::EDGE),
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
    hatch: Option<Res<'w, Hatch>>,
    telemetry: Option<Res<'w, ShipTelemetry>>,
    settings: Res<'w, Persistent<Settings>>,
    destination: Res<'w, Destination>,
    route: Res<'w, RouteStatus>,
    universe: Option<Res<'w, Universe>>,
    hinted: Res<'w, Hinted>,
    now: Res<'w, Where>,
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

    /// Returns the board row and the hint row: what the board key does and the hint in turn.
    fn keys(&self, travel: Option<Travel>) -> (Keyed, Keyed) {
        let keys = &self.settings.controls.keys;
        let at_hatch = self.hatch.as_deref().is_some_and(|hatch| hatch.near);
        let landed = self.telemetry.as_deref().is_some_and(|ship| ship.landed);
        let board = travel
            .and_then(|travel| board_prompt(travel, at_hatch, landed))
            .map(|action| (keys.ship.board, action));
        let hint = self
            .hinted
            .0
            .map(|hint: Hint| (hint.key(keys), hint.words()));
        (board, hint)
    }

    /// Returns the state of the line for this frame: empty while gliding or in a menu.
    fn state(&self) -> State {
        let travel = self.location.as_deref().map(|at| at.travel);
        let mut state = State::default();
        if matches!(self.now.0, Context::Gliding | Context::Menu) {
            return state;
        }
        (state.board, state.hint) = self.keys(travel);
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
    Query<'w, 's, &'static mut Text, (With<PromptKey>, Without<PromptAction>)>,
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
    let cap = key.map_or(String::new(), |key| format!("[{}]", key_name(key)));
    let tone = [theme::WARN, theme::TEXT][usize::from(key.is_some())];
    for (mut text, mut color) in &mut actions {
        text.set_if_neq(Text(words.clone()));
        color.set_if_neq(TextColor(tone));
    }
    for mut text in &mut keys {
        text.set_if_neq(Text(cap.clone()));
    }
}
