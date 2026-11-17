//! Keyboard, wheel and button handling of the computer screen.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::board::{Board, VISIBLE_ROWS};
use super::open::{Computer, ready};
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

/// Moves the cursor to the row the pointer gave focus to.
fn follow_focus(focus: Res<InputFocus>, slots: Query<&Slot>, mut board: ResMut<Board>) {
    let Some(slot) = focus.get().and_then(|entity| slots.get(entity).ok()) else {
        return;
    };
    if board.top + slot.0 != board.cursor && board.at(slot.0).is_some() {
        board.pick(slot.0);
    }
}

/// Moves the cursor with the arrows, Page Up and Down, Home and End, changes tab with Left and
/// Right, and scrolls with the wheel. A move that scrolls the table under a cursor that stays
/// on the same drawn row is sounded here, since the focus does not move.
fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    wheel: Res<AccumulatedMouseScroll>,
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
    if rows.round() != 0.0 {
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
struct Desk<'w, 's> {
    board: ResMut<'w, Board>,
    destination: ResMut<'w, Destination>,
    next: ResMut<'w, NextState<Computer>>,
    commands: Commands<'w, 's>,
}

impl Desk<'_, '_> {
    /// Runs an action of the screen.
    fn run(&mut self, act: Act) {
        match (act, self.destination.0) {
            (Act::Engage, Some(target)) => {
                self.commands.trigger(Engage(target));
                self.next.set(Computer::Closed);
            }
            (Act::Clear, Some(_)) => self.destination.0 = None,
            (Act::Close, _) => self.next.set(Computer::Closed),
            _ => {}
        }
    }
}

/// Hands the destination to the autopilot on the engage key of the settings, and forgets the
/// destination on Delete.
fn action_keys(
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<Persistent<Settings>>,
    mut desk: Desk,
) {
    if keys.just_pressed(settings.controls.keys.engage) {
        desk.run(Act::Engage);
    }
    if keys.just_pressed(KeyCode::Delete) {
        desk.run(Act::Clear);
    }
}

/// Applies an activated button: a row sets the destination, a tab opens, an action runs.
fn on_activate(event: On<Activate>, roles: Query<Role>, mut desk: Desk) {
    let Ok((slot, link, act)) = roles.get(event.entity) else {
        return;
    };
    if let Some(target) = slot
        .and_then(|slot| desk.board.at(slot.0))
        .map(|line| line.target)
    {
        desk.destination.0 = Some(target);
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
        (follow_focus, navigate, focus_cursor, action_keys)
            .chain()
            .in_set(super::Input)
            .run_if(ready),
    );
}
