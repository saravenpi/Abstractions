//! Keyboard, wheel and button handling of the search results of the map screen.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Activate;

use super::board::{Board, VISIBLE_ROWS};
use super::chart::Dot;
use super::coast::arrow;
use super::course::Act;
use super::go::Go;
use super::open::ready;
use super::pointer::{MapView, aimed, selected};
use super::readout::More;
use super::route::Target;
use super::typing::Typing;
use super::view::Slot;
use crate::audio::AudioCue;
use crate::settings::{Persistent, Settings};
use crate::ui::{Scroller, theme};

/// Rows a jump to the first or the last row moves by: more than any table holds.
const FAR: isize = 1_000_000;

/// Rows scrolled by one notch of the wheel.
const WHEEL_ROWS: f32 = 3.0;

/// Rows the cursor moves by for each key.
const MOVES: [(KeyCode, isize); 5] = [
    (KeyCode::ArrowUp, -1),
    (KeyCode::ArrowDown, 1),
    (KeyCode::PageUp, -(VISIBLE_ROWS as isize)),
    (KeyCode::PageDown, VISIBLE_ROWS as isize),
    (KeyCode::End, FAR),
];

/// The key that flies the ship home. It types a letter while a search is typed.
pub const HOME_KEY: KeyCode = KeyCode::KeyH;

/// The components that say what an activated button of the screen does.
type Role = (Option<&'static Slot>, Option<&'static Act>);

/// Moves the cursor with the arrows, Page Up and Down and End, and scrolls the results with the
/// wheel unless the pointer is over the map, where the arrows and the wheel move the map instead,
/// or over the part of the panel that scrolls, which the wheel then scrolls. A
/// key that moves the cursor picks its row. A move that scrolls the table under a cursor that
/// stays on the same drawn row is sounded here, since the focus does not move.
pub(super) fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    (wheel, mut view): (Res<AccumulatedMouseScroll>, ResMut<MapView>),
    scrolling: Query<&RelativeCursorPosition, With<Scroller>>,
    mut board: ResMut<Board>,
    mut cues: MessageWriter<AudioCue>,
) {
    let over = view.over;
    let elsewhere = over || scrolling.iter().any(|pointer| pointer.cursor_over);
    let held = |key: KeyCode| keys.just_pressed(key) && !(over && arrow(key));
    let pressed = MOVES.iter().filter(|(key, _)| held(*key));
    let by: isize = pressed.map(|(_, by)| by).sum();
    if by != 0 {
        let (cursor, slot) = (board.cursor, board.cursor - board.top);
        board.step(by);
        view.picked = true;
        if board.cursor != cursor && board.cursor - board.top == slot {
            cues.write(AudioCue::UiMove);
        }
    }
    let rows = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * WHEEL_ROWS,
        MouseScrollUnit::Pixel => wheel.delta.y / theme::ROW_HEIGHT,
    };
    if rows.round() != 0.0 && !elsewhere {
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
    view: ResMut<'w, MapView>,
    more: ResMut<'w, More>,
    go: Go<'w, 's>,
}

impl Desk<'_, '_> {
    /// Eases the chart onto the row under the cursor and asks to go into its place.
    fn focus(&mut self) {
        if let Some(lens) = aimed(&self.board, &self.view, Dot::Row(self.board.cursor)) {
            self.view.goal = lens;
        }
        self.view.into = self
            .board
            .lines
            .get(self.board.cursor)
            .map(|line| line.target);
    }

    /// Returns the place of the row under the cursor.
    fn cursor(&self) -> Option<Target> {
        self.board
            .lines
            .get(self.board.cursor)
            .map(|line| line.target)
    }

    /// Runs an action of the screen: Go on the selected row, or on the course when no row is
    /// selected, the way home, or the rarer facts of the card.
    fn run(&mut self, act: Act) {
        match act {
            Act::Go => {
                let row = selected(&self.board, &self.view);
                let pick = row.map(|row| self.board.lines[row].target);
                self.go.press(pick);
            }
            Act::Home => self.go.home(),
            Act::More => self.more.0 = !self.more.0,
        }
    }
}

/// Sends the ship to the row under the cursor on Enter, presses Go on the engage key of the
/// settings, which stops a route that is flown, and flies home on the home key. Enter also works
/// while a search is typed; nothing does while a pin is named, and the other keys type into a
/// search.
fn action_keys(
    keys: Res<ButtonInput<KeyCode>>,
    (settings, typing): (Res<Persistent<Settings>>, Res<Typing>),
    mut desk: Desk,
) {
    if matches!(*typing, Typing::Name(..)) {
        return;
    }
    let off = typing.is_off();
    if keys.just_pressed(KeyCode::Enter) {
        let cursor = desk.cursor();
        desk.go.press(cursor);
    }
    if keys.just_pressed(settings.controls.keys.engage) && off {
        desk.run(Act::Go);
    }
    if keys.just_pressed(HOME_KEY) && off {
        desk.run(Act::Home);
    }
}

/// Seconds within which a second click on the same row focuses the map on it.
const DOUBLE_CLICK: f64 = 0.45;

/// Applies an activated button: a row is picked or an action runs. A second
/// click of the pointer on the same row within [`DOUBLE_CLICK`] goes into its place, as on
/// the map. Enter on the focused row selects it again; `action_keys` sends the ship there.
pub(super) fn on_activate(
    event: On<Activate>,
    roles: Query<Role>,
    (mouse, time): (Res<ButtonInput<MouseButton>>, Res<Time<Real>>),
    mut last: Local<Option<(usize, f64)>>,
    mut desk: Desk,
) {
    let Ok((slot, act)) = roles.get(event.entity) else {
        return;
    };
    if let Some(Slot(slot)) = slot {
        desk.board.pick(*slot);
        desk.view.picked = true;
        let left = MouseButton::Left;
        let clicked = mouse.just_released(left) || mouse.just_pressed(left);
        let now = time.elapsed_secs_f64();
        let again = last.is_some_and(|(row, at)| row == *slot && now - at < DOUBLE_CLICK);
        *last = clicked.then_some((*slot, now)).filter(|_| !again);
        if clicked && again {
            desk.focus();
        }
    }
    if let Some(act) = act {
        desk.run(*act);
    }
}

/// Registers the input systems of the screen.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Typing>()
        .init_resource::<More>()
        .add_observer(on_activate)
        .add_observer(super::typing::on_search)
        .add_systems(OnEnter(super::Computer::Open), super::typing::stop_typing)
        .add_systems(OnExit(super::Computer::Open), super::typing::stop_typing)
        .add_systems(
            Update,
            (
                navigate,
                focus_cursor,
                action_keys,
                super::typing::type_filter,
                super::typing::show_search,
            )
                .chain()
                .in_set(super::Input)
                .run_if(ready),
        );
}
