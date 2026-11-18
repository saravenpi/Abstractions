//! Keyboard, wheel and button handling of the table of the computer screen.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::board::{Board, VISIBLE_ROWS};
use super::open::ready;
use super::pointer::MapView;
use super::route::{Destination, Engage};
use super::view::{Act, Slot, TabLink};
use crate::audio::AudioCue;
use crate::settings::{Persistent, Settings};
use crate::ui::theme;

/// Rows a jump to the first or the last row moves by: more than any table holds.
const FAR: isize = 1_000_000;

/// Rows scrolled by one notch of the wheel.
const WHEEL_ROWS: f32 = 3.0;

/// Rows the cursor moves by for each key.
const MOVES: [(KeyCode, isize); 6] = [
    (KeyCode::ArrowUp, -1),
    (KeyCode::ArrowDown, 1),
    (KeyCode::PageUp, -(VISIBLE_ROWS as isize)),
    (KeyCode::PageDown, VISIBLE_ROWS as isize),
    (KeyCode::Home, -FAR),
    (KeyCode::End, FAR),
];

/// The components that say what an activated button of the screen does.
type Role = (
    Option<&'static Slot>,
    Option<&'static TabLink>,
    Option<&'static Act>,
);

/// Moves the cursor with the arrows, Page Up and Down, Home and End, changes tab with Left and
/// Right, and scrolls with the wheel unless the pointer is over the map. A move that scrolls
/// the table under a cursor that stays on the same drawn row is sounded here, since the focus
/// does not move.
pub(super) fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    (wheel, view): (Res<AccumulatedMouseScroll>, Res<MapView>),
    mut board: ResMut<Board>,
    mut cues: MessageWriter<AudioCue>,
) {
    let pressed = MOVES.iter().filter(|(key, _)| keys.just_pressed(*key));
    let by: isize = pressed.map(|(_, by)| by).sum();
    if by != 0 {
        let (cursor, slot) = (board.cursor, board.cursor - board.top);
        board.step(by);
        if board.cursor != cursor && board.cursor - board.top == slot {
            cues.write(AudioCue::UiMove);
        }
    }
    let turn = isize::from(keys.just_pressed(KeyCode::ArrowRight))
        - isize::from(keys.just_pressed(KeyCode::ArrowLeft));
    if turn != 0 {
        let tab = board.tab.step(turn);
        board.show(tab);
    }
    let rows = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * WHEEL_ROWS,
        MouseScrollUnit::Pixel => wheel.delta.y / theme::ROW_HEIGHT,
    };
    if rows.round() != 0.0 && !view.over {
        board.scroll(-rows.round() as isize);
    }
}

/// Gives keyboard focus to the row under the cursor whenever that row changes.
fn focus_cursor(
    board: Res<Board>,
    slots: Query<(Entity, &Slot)>,
    mut focus: ResMut<InputFocus>,
    mut last: Local<Option<Entity>>,
) {
    let wanted = board.cursor - board.top;
    let row = slots
        .iter()
        .find(|(_, slot)| slot.0 == wanted)
        .map(|(entity, _)| entity);
    if let Some(entity) = row
        && row != *last
    {
        *last = row;
        focus.set(entity, FocusCause::Navigated);
    }
}

/// What the actions of the screen read and write.
#[derive(SystemParam)]
pub(super) struct Desk<'w, 's> {
    board: ResMut<'w, Board>,
    destination: ResMut<'w, Destination>,
    commands: Commands<'w, 's>,
}

impl Desk<'_, '_> {
    /// Runs an action of the screen: the course goes to the selected row, to the autopilot or
    /// away.
    fn run(&mut self, act: Act) {
        match (act, self.destination.0) {
            (Act::Set, _) => {
                let row = self.board.lines.get(self.board.cursor);
                self.destination.0 = row.map(|line| line.target).or(self.destination.0);
            }
            (Act::Engage, Some(target)) => self.commands.trigger(Engage(target)),
            (Act::Clear, Some(_)) => self.destination.0 = None,
            _ => {}
        }
    }
}

/// Sets the course to the selected row on Enter, hands the destination to the autopilot on the
/// engage key of the settings, unless a filter is being typed, and forgets the destination on
/// Delete.
fn action_keys(
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Persistent<Settings>>,
    mut desk: Desk,
) {
    if keys.just_pressed(KeyCode::Enter) {
        desk.run(Act::Set);
    }
    if keys.just_pressed(settings.controls.keys.engage) && desk.board.filter.is_empty() {
        desk.run(Act::Engage);
    }
    if keys.just_pressed(KeyCode::Delete) {
        desk.run(Act::Clear);
    }
}

/// Applies an activated button: a row is selected, a tab opens, an action runs. Enter on the
/// focused row selects it again; `action_keys` sets the course.
pub(super) fn on_activate(event: On<Activate>, roles: Query<Role>, mut desk: Desk) {
    let Ok((slot, link, act)) = roles.get(event.entity) else {
        return;
    };
    if let Some(Slot(slot)) = slot {
        desk.board.pick(*slot);
    }
    if let Some(TabLink(tab)) = link {
        desk.board.show(*tab);
    }
    if let Some(act) = act {
        desk.run(*act);
    }
}

/// Registers the input systems of the screen.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_activate).add_systems(
        Update,
        (
            navigate,
            focus_cursor,
            action_keys,
            super::typing::type_filter,
        )
            .chain()
            .in_set(super::Input)
            .run_if(ready),
    );
}
