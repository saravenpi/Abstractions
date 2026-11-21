//! Pause menu: freezes physics while open in single player. In a session the world goes on, the
//! entries follow the session and the menu is built again when the session changes.

use avian3d::prelude::*;
use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::pause_hud::{corner_text, saved_label, show_saved, world_corner};
use super::recall::{Entry, LastEntry};
use super::{Menu, Nav};
use crate::net::{CloseLan, LeaveSession, OpenToLan, Session, in_session};
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::ui::{Icon, Tone, column, footer, heading, hints, item, panel, root, theme};
use crate::world::WorldSeed;

/// A pause entry that talks to the network.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Online {
    /// Open the world to the local network.
    OpenToLan,
    /// Stop hosting on the local network.
    CloseLan,
    /// Leave the server for the title screen.
    Disconnect,
}

/// What one pause entry does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Does {
    /// Navigate, as every menu entry does.
    Go(Nav),
    /// Ask the network.
    Net(Online),
}

/// Marks the root of the pause menu.
#[derive(Component)]
struct PauseRoot;

/// Returns the pause entries for `session`, in order: icon, label and what each does. Joined to
/// a server there is nothing to save, since the server saves.
pub(super) fn entries(session: &Session) -> Vec<(Icon, &'static str, Does)> {
    let resume = (Icon::Play, "Resume", Does::Go(Nav::Resume));
    let settings = (
        Icon::Settings,
        "Settings",
        Does::Go(Nav::To(Menu::Settings)),
    );
    let lan = match session {
        Session::Joined { .. } => {
            return vec![
                resume,
                (Icon::Users, "Players", Does::Go(Nav::To(Menu::Players))),
                settings,
                (Icon::LogOut, "Disconnect", Does::Net(Online::Disconnect)),
                (Icon::Power, "Quit the game", Does::Go(Nav::Quit)),
            ];
        }
        Session::Hosting { .. } => (Icon::Close, "Close LAN", Does::Net(Online::CloseLan)),
        _ => (Icon::Wifi, "Open to LAN", Does::Net(Online::OpenToLan)),
    };
    vec![
        resume,
        settings,
        lan,
        (
            Icon::LogOut,
            "Save and quit to menu",
            Does::Go(Nav::ToTitle),
        ),
        (Icon::Power, "Save and quit the game", Does::Go(Nav::Quit)),
    ]
}

/// What the pause menu shows besides its entries.
type Corner<'w> = (Option<Res<'w, WorldSeed>>, Option<Res<'w, ActiveWorld>>);

/// Spawns the pause menu over the blurred game, with the world or the session in the corner,
/// and focuses the entry used last. A pause menu already open is replaced.
fn spawn(
    mut commands: Commands,
    session: Res<Session>,
    (seed, active): Corner,
    last: Res<LastEntry>,
    open: Query<Entity, With<PauseRoot>>,
) {
    for root in &open {
        commands.entity(root).despawn();
    }
    let corner = corner_text(&session, active.as_deref(), seed.as_deref());
    let menu = commands
        .spawn((
            PauseRoot,
            DespawnOnExit(Menu::Pause),
            root(),
            children![
                footer(hints(&[("Resume", "Esc")])),
                world_corner(corner),
                saved_label()
            ],
        ))
        .id();
    let card = commands
        .spawn((panel(), ChildOf(menu), children![heading("Paused")]))
        .id();
    let items = commands
        .spawn((column(px(theme::MENU_WIDTH), 0.0), ChildOf(card)))
        .id();
    add_entries(&mut commands, items, &session, last.of(Menu::Pause, 0));
}

/// Spawns the entries of `session` in the column `items` and focuses entry `wanted`, or the
/// last one when there are fewer.
fn add_entries(commands: &mut Commands, items: Entity, session: &Session, wanted: usize) {
    let listed = entries(session);
    let wanted = wanted.min(listed.len() - 1);
    for (index, (symbol, name, does)) in listed.into_iter().enumerate() {
        let entry = (item(symbol, name, Tone::Plain), Entry(index));
        let mut spawned = commands.spawn((entry, ChildOf(items)));
        match does {
            Does::Go(nav) => spawned.insert(nav),
            Does::Net(online) => spawned.insert(online),
        };
        if index == wanted {
            spawned.insert(AutoFocus);
        }
    }
}

/// The requests the pause entries send.
type Wires<'w> = (
    MessageWriter<'w, OpenToLan>,
    MessageWriter<'w, CloseLan>,
    MessageWriter<'w, LeaveSession>,
);

/// Sends the request of an activated network entry. Disconnecting also leaves for the title.
fn on_online(
    event: On<Activate>,
    onlines: Query<&Online>,
    (mut open, mut close, mut leave): Wires,
    mut screen: ResMut<NextState<Screen>>,
) {
    match onlines.get(event.entity) {
        Ok(Online::OpenToLan) => {
            open.write(OpenToLan);
        }
        Ok(Online::CloseLan) => {
            close.write(CloseLan);
        }
        Ok(Online::Disconnect) => {
            leave.write(LeaveSession);
            screen.set(Screen::Title);
        }
        Err(_) => {}
    }
}

/// Freezes physics.
fn freeze(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

/// Resumes physics.
fn thaw(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

/// Registers the pause menu systems. Escape closes the menu through the shared navigation.
/// Physics freezes only out of a session, and follows the session while the menu is open.
pub(super) fn plugin(app: &mut App) {
    let follow = (
        spawn,
        freeze.run_if(not(in_session)),
        thaw.run_if(in_session),
    );
    app.add_observer(on_online)
        .add_systems(
            OnEnter(Menu::Pause),
            (spawn, freeze.run_if(not(in_session))),
        )
        .add_systems(OnEnter(Menu::None), thaw)
        .add_systems(OnExit(Screen::Gameplay), thaw)
        .add_systems(
            Update,
            (
                show_saved,
                follow.run_if(resource_exists_and_changed::<Session>),
            )
                .run_if(in_state(Menu::Pause)),
        );
}
