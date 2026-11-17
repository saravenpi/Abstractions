//! Keyboard, wheel and button handling of the computer screen.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::board::{Board, VISIBLE_ROWS};
use super::open::ready;
use super::route::{Destination, Engage};
use super::view::{Act, Slot, TabLink};
use crate::ui::theme;

/// Key that hands the destination to the autopilot.
pub const ENGAGE_KEY: KeyCode = KeyCode::KeyG;

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
/// Right, and scrolls with the wheel.
fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    wheel: Res<AccumulatedMouseScroll>,
    mut board: ResMut<Board>,
) {
    let pressed = MOVES.iter().filter(|(key, _)| keys.just_pressed(*key));
    let by: isize = pressed.map(|(_, by)| by).sum();
    if by != 0 {
        board.step(by);
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

/// Hands the destination to the autopilot on the engage key.
fn engage_key(
    keys: Res<ButtonInput<KeyCode>>,
    destination: Res<Destination>,
    mut commands: Commands,
) {
    if let (true, Some(target)) = (keys.just_pressed(ENGAGE_KEY), destination.0) {
        commands.trigger(Engage(target));
    }
}

/// Applies an activated button: a row sets the destination, a tab opens, an action runs.
fn on_activate(
    event: On<Activate>,
    roles: Query<Role>,
    mut board: ResMut<Board>,
    mut destination: ResMut<Destination>,
    mut commands: Commands,
) {
    let Ok((slot, link, act)) = roles.get(event.entity) else {
        return;
    };
    if let Some(line) = slot.and_then(|slot| board.at(slot.0)) {
        destination.0 = Some(line.target);
    }
    if let Some(TabLink(tab)) = link {
        board.show(*tab);
    }
    match (act, destination.0) {
        (Some(Act::Engage), Some(target)) => commands.trigger(Engage(target)),
        (Some(Act::Clear), _) => destination.0 = None,
        _ => {}
    }
}

/// Registers the input systems of the screen.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_activate).add_systems(
        Update,
        (follow_focus, navigate, focus_cursor, engage_key)
            .chain()
            .in_set(super::Input)
            .run_if(ready),
    );
}
