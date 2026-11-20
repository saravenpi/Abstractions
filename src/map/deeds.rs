//! The Go card under the name of the selected place: one button, Go, with its key, that sends
//! the ship there and reads Stop while the route to the place is flown, and one sentence under
//! it that says how long the trip takes, what the ship does, or why it cannot start and what to
//! do. With no place selected the card says how to pick one.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::computer::{
    Act, Ask, Board, Destination, Keys, MapView, NO_PLACE, RouteStatus, Target, arrival, card_line,
    pace, running, same_galaxy, selected,
};
use crate::settings::{Keymap, Persistent, Settings};
use crate::ship::ShipTelemetry;
use crate::space::{Location, Travel};
use crate::ui::{Icon, Selected, Tone, column, entry, frame, key_cap, text, theme};
use crate::universe::Universe;

/// Marks the row the Go button is in.
#[derive(Component)]
pub struct Deeds;

/// Marks the line that says what Go does, or why it cannot.
#[derive(Component)]
pub struct Sentence;

/// The Go button as it is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    /// True when the button stops the route that is flown instead of starting one.
    pub stop: bool,
    /// True when pressing the button does something.
    pub enabled: bool,
}

/// What the card offers: its button, if a place is shown, and its sentence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    /// The button, or nothing while no place is shown.
    pub button: Option<Button>,
    /// The sentence under the button.
    pub sentence: String,
}

/// Returns what the card of the place `ask` describes offers, or how to pick a place for
/// nothing. The button stops the route while the route to the place is flown.
pub fn offer(ask: Option<&Ask>) -> Offer {
    let Some(ask) = ask else {
        let sentence = NO_PLACE.to_string();
        return Offer {
            button: None,
            sentence,
        };
    };
    let (enabled, sentence) = card_line(ask);
    let stop = ask.ours && running(ask.status);
    Offer {
        button: Some(Button {
            stop,
            enabled: enabled || stop,
        }),
        sentence,
    }
}

/// The Go button row and the sentence under it, empty until [`write`] fills them.
pub fn deeds() -> impl Bundle {
    let row = Node {
        width: percent(100),
        ..default()
    };
    (
        column(percent(100), theme::S2),
        children![
            (Deeds, row),
            (Sentence, text("", theme::FONT_SMALL, theme::TEXT))
        ],
    )
}

/// Spawns the Go button under `row`: a block with the key at its right end while it works, and
/// dim words while it cannot.
fn spawn(row: &mut ChildSpawnerCommands, button: Button, key: &str) {
    let (icon, words) = match button.stop {
        true => (Icon::Stop, "Stop"),
        false => (Icon::Go, "Go"),
    };
    let node = Node {
        width: percent(100),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::axes(px(theme::S3), px(theme::S2)),
        ..default()
    };
    if !button.enabled {
        row.spawn((
            node,
            children![text(words, theme::FONT_BODY, theme::TEXT_OFF)],
        ));
        return;
    }
    let shown = entry(icon, words, theme::FONT_BODY, Tone::Accent);
    row.spawn((
        frame(),
        Selected(true),
        Act::Go,
        node,
        children![shown, key_cap(key)],
    ));
}

/// What the card is worked out from.
#[derive(SystemParam)]
pub(super) struct Source<'w> {
    board: Res<'w, Board>,
    view: Res<'w, MapView>,
    destination: Res<'w, Destination>,
    route: Res<'w, RouteStatus>,
    location: Option<Res<'w, Location>>,
    settings: Option<Res<'w, Persistent<Settings>>>,
    telemetry: Option<Res<'w, ShipTelemetry>>,
    universe: Option<Res<'w, Universe>>,
}

impl Source<'_> {
    /// Returns the place the player picked in the table, if any.
    fn pick(&self) -> Option<Target> {
        let row = selected(&self.board, &self.view)?;
        self.board.lines.get(row).map(|line| line.target)
    }

    /// Returns true when what the card shows may have changed since `pick` was read last.
    fn moved(&self, pick: Option<Target>, last: Option<Target>) -> bool {
        let seat = self.location.as_ref().is_some_and(|at| at.is_changed());
        let (board, course, route) = (&self.board, &self.destination, &self.route);
        pick != last || board.is_changed() || course.is_changed() || route.is_changed() || seat
    }

    /// Returns the time to get to `target`: the trip of the warp drive from where the ship is,
    /// or at the speed of the ship when that is faster, as in warp.
    fn eta(&self, target: Target) -> Option<String> {
        let speed = self
            .telemetry
            .as_ref()
            .map_or(0.0, |ship| ship.orbital_speed);
        let line = self.board.lines.iter().find(|line| line.target == target)?;
        let tier = pace(line.distance, same_galaxy(self.board.here, target));
        arrival(line.distance, speed.max(tier))
    }

    /// Returns what the card offers for `shown`, and the name of the Go key.
    fn offer(&self, shown: Option<Target>) -> (Offer, String) {
        let map = self
            .settings
            .as_ref()
            .map_or(Keymap::default(), |s| s.controls.keys);
        let keys = Keys::of(&map);
        let (Some(target), Some(universe)) = (shown, self.universe.as_deref()) else {
            return (offer(None), keys.go);
        };
        let name = target.name(universe);
        let eta = self.eta(target);
        let seated = self
            .location
            .as_ref()
            .is_some_and(|at| at.travel == Travel::InShip);
        let ask = Ask {
            name: &name,
            ours: self.destination.0 == Some(target),
            here: self.board.here.contains(&Some(target)),
            seated,
            status: *self.route,
            eta: eta.as_deref(),
            keys: &keys,
        };
        (offer(Some(&ask)), keys.go)
    }
}

/// What the card showed last: the place it was written for and its button.
type Shown = (Option<Target>, Option<Button>, String);

/// Writes the Go button and its sentence when the table, the selection, the course, the route
/// or the seat changed, or the screen was just opened. The place is the one picked, or the
/// course while its route is flown.
pub(super) fn write(
    source: Source,
    (rows, fresh): (Query<Entity, With<Deeds>>, Query<(), Added<Deeds>>),
    mut sentences: Query<&mut Text, With<Sentence>>,
    mut commands: Commands,
    mut last: Local<Option<Shown>>,
) {
    let pick = source.pick();
    let before = last.as_ref().and_then(|(pick, ..)| *pick);
    let Some(row) = rows.iter().next() else {
        return;
    };
    if !(source.moved(pick, before) || !fresh.is_empty()) {
        return;
    }
    let shown = pick.or(source.destination.0.filter(|_| running(*source.route)));
    let (offered, key) = source.offer(shown);
    for mut text in &mut sentences {
        text.set_if_neq(Text(offered.sentence.clone()));
    }
    let now = (pick, offered.button, key);
    if last.as_ref() == Some(&now) && fresh.is_empty() {
        return;
    }
    commands.entity(row).despawn_children();
    if let Some(button) = offered.button {
        commands
            .entity(row)
            .with_children(|row| spawn(row, button, &now.2));
    }
    *last = Some(now);
}
