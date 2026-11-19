//! Keyboard, wheel and button handling of the search results of the map screen.

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::board::{Board, VISIBLE_ROWS};
use super::course::Act;
use super::home::{tab_of, toward};
use super::locate::here;
use super::open::ready;
use super::pointer::MapView;
use super::route::{Destination, Engage, RouteStatus};
use super::typing::Typing;
use super::view::Slot;
use crate::audio::AudioCue;
use crate::settings::{Persistent, Settings};
use crate::space::Anchor;
use crate::ui::theme;
use crate::universe::Universe;

/// Rows a jump to the first or the last row moves by: more than any table holds.
const FAR: isize = 1_000_000;

/// Rows scrolled by one notch of the wheel.
const WHEEL_ROWS: f32 = 3.0;

/// Rows the cursor moves by for each key. The Home key is not one of them: it sets the
/// course home.
const MOVES: [(KeyCode, isize); 5] = [
    (KeyCode::ArrowUp, -1),
    (KeyCode::ArrowDown, 1),
    (KeyCode::PageUp, -(VISIBLE_ROWS as isize)),
    (KeyCode::PageDown, VISIBLE_ROWS as isize),
    (KeyCode::End, FAR),
];

/// The key that sets the course home. It is not a letter, so that typing "home" still
/// narrows the list.
pub(super) const HOME_KEY: KeyCode = KeyCode::Home;

/// The components that say what an activated button of the screen does.
type Role = (Option<&'static Slot>, Option<&'static Act>);

/// Moves the cursor with the arrows, Page Up and Down and End, changes tab with Left and
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
    route: Option<Res<'w, RouteStatus>>,
    commands: Commands<'w, 's>,
    world: Option<(Res<'w, Universe>, Res<'w, Anchor>)>,
}

impl Desk<'_, '_> {
    /// Sets the course to the next place on the way home from where the ship is: the home
    /// galaxy, then the home star, then the home planet. The tab that lists it opens with
    /// the cursor on its row.
    fn home(&mut self) {
        let Some((universe, anchor)) = &self.world else {
            return;
        };
        let target = toward(here(universe, anchor.frame, anchor.origin), universe);
        self.destination.0 = Some(target);
        self.board.show(tab_of(target));
        self.board.seek = Some(target);
    }

    /// Runs an action of the screen: the course goes to the selected row, home, to the
    /// autopilot while the route is idle, as with the map closed, or away.
    fn run(&mut self, act: Act) {
        match (act, self.destination.0) {
            (Act::Home, _) => self.home(),
            (Act::Set, _) => {
                let row = self.board.lines.get(self.board.cursor);
                self.destination.0 = row.map(|line| line.target).or(self.destination.0);
            }
            (Act::Engage, Some(target)) if self.route.as_deref() == Some(&RouteStatus::Idle) => {
                self.commands.trigger(Engage(target));
            }
            (Act::Clear, Some(_)) => self.destination.0 = None,
            _ => {}
        }
    }
}

/// Sets the course to the selected row on Enter, hands the destination to the autopilot on the
/// engage key of the settings, forgets the destination on Delete and sets the course home on
/// the Home key. Enter and Home also work while a search is typed; nothing does while a
/// favourite is named, and the engage key and Delete type into a search.
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
        desk.run(Act::Set);
    }
    if keys.just_pressed(settings.controls.keys.engage) && off {
        desk.run(Act::Engage);
    }
    if keys.just_pressed(KeyCode::Delete) && off {
        desk.run(Act::Clear);
    }
    if keys.just_pressed(HOME_KEY) {
        desk.run(Act::Home);
    }
}

/// Seconds within which a second click on the same row sets the course to it.
const DOUBLE_CLICK: f64 = 0.45;

/// Applies an activated button: a row is selected or an action runs. A second
/// click of the pointer on the same row within [`DOUBLE_CLICK`] sets the course to it, as on
/// the map. Enter on the focused row selects it again; `action_keys` sets the course.
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
        let left = MouseButton::Left;
        let clicked = mouse.just_released(left) || mouse.just_pressed(left);
        let now = time.elapsed_secs_f64();
        let again = last.is_some_and(|(row, at)| row == *slot && now - at < DOUBLE_CLICK);
        *last = clicked.then_some((*slot, now)).filter(|_| !again);
        if clicked && again {
            desk.run(Act::Set);
        }
    }
    if let Some(act) = act {
        desk.run(*act);
    }
}

/// Ends any typing and clears the search, so neither outlives the screen: the keys of the
/// game work again once it closes, and Escape closes it once it opens again.
fn stop_typing(mut typing: ResMut<Typing>, mut board: ResMut<Board>) {
    *typing = Typing::Off;
    if !board.filter.is_empty() {
        (board.filter, board.stale) = (String::new(), true);
    }
}

/// Registers the input systems of the screen.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Typing>()
        .add_observer(on_activate)
        .add_observer(super::typing::on_search)
        .add_systems(OnEnter(super::Computer::Open), stop_typing)
        .add_systems(OnExit(super::Computer::Open), stop_typing)
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
