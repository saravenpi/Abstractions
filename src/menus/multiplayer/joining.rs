//! The joining screen, shown while a server has not answered, and the way back to the
//! multiplayer menu when a session ends.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::{Ended, Waiting};
use crate::menus::{Menu, Nav, Trail};
use crate::net::{LeaveSession, Session, SessionEnded};
use crate::screens::Screen;
use crate::ui::{Cancels, Icon, Tone, column, footer, heading, item, panel, root, text, theme};

/// Returns the line of the joining screen: "Connecting to <address>".
pub(super) fn connecting(session: &Session) -> String {
    match session {
        Session::Joining { address } => format!("Connecting to {address}"),
        _ => "Connecting".to_string(),
    }
}

/// The muted line under the address, with room around it.
fn waiting_line() -> impl Bundle {
    (
        text(
            "Waiting for the server to answer",
            theme::FONT_SMALL,
            theme::TEXT_2,
        ),
        Node {
            margin: UiRect::vertical(px(theme::GAP_M)),
            ..default()
        },
    )
}

/// Spawns the joining screen with its one entry, which cancels.
fn spawn(mut commands: Commands, session: Res<Session>) {
    let cancel = (
        item(Icon::Close, "Cancel", Tone::Plain),
        Nav::Back,
        Cancels,
        AutoFocus,
    );
    let line = text(connecting(&session), theme::FONT_HEADING, theme::TEXT);
    let entries = (column(px(theme::MENU_WIDTH), 0.0), children![cancel]);
    commands.spawn((
        DespawnOnExit(Menu::Joining),
        root(),
        children![
            (
                panel(),
                children![heading("Multiplayer"), line, waiting_line(), entries]
            ),
            footer("Esc cancels"),
        ],
    ));
}

/// Opens the joining screen once the network answers a join from this menu with `Joining`.
pub(super) fn follow_session(
    session: Res<Session>,
    mut waiting: ResMut<Waiting>,
    mut trail: ResMut<Trail>,
    mut menu: ResMut<NextState<Menu>>,
) {
    if waiting.0 && matches!(*session, Session::Joining { .. }) {
        waiting.0 = false;
        trail.0.push(Menu::Multiplayer);
        menu.set(Menu::Joining);
    }
}

/// Stops joining when the screen closes before the server answered.
fn leave_if_joining(session: Res<Session>, mut leave: MessageWriter<LeaveSession>) {
    if matches!(*session, Session::Joining { .. }) {
        leave.write(LeaveSession);
    }
}

/// The screen state, to leave the world.
type Screens<'w> = (Res<'w, State<Screen>>, ResMut<'w, NextState<Screen>>);

/// Keeps why a join or a session ended, and leaves the world for the title screen. An end that
/// concerns no join, as when hosting stops, is ignored.
fn watch_end(
    mut ends: MessageReader<SessionEnded>,
    session: Res<Session>,
    mut was_client: Local<bool>,
    (mut waiting, mut ended): (ResMut<Waiting>, ResMut<Ended>),
    (screen, mut next): Screens,
) {
    let last = ends.read().last().cloned();
    let ours = *was_client || waiting.0 || session.client();
    *was_client = session.client();
    let Some(end) = last.filter(|_| ours) else {
        return;
    };
    waiting.0 = false;
    ended.0 = Some(end.reason);
    if *screen.get() != Screen::Title {
        next.set(Screen::Title);
    }
}

/// The menu state, to open the multiplayer menu.
type Menus<'w> = (Res<'w, State<Menu>>, ResMut<'w, NextState<Menu>>);

/// Opens the multiplayer menu over the title screen while an ended session waits to be shown.
fn reopen(
    ended: Res<Ended>,
    screen: Res<State<Screen>>,
    (menu, mut next): Menus,
    mut trail: ResMut<Trail>,
) {
    let away = *menu.get() != Menu::Multiplayer;
    if ended.0.is_some() && *screen.get() == Screen::Title && away {
        trail.0 = vec![Menu::Main];
        next.set(Menu::Multiplayer);
    }
}

/// Registers the joining screen and the return to the menu.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Joining), spawn)
        .add_systems(OnExit(Menu::Joining), leave_if_joining)
        .add_systems(Update, (watch_end, reopen).chain());
}
