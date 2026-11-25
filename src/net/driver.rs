//! The network driver: every system that talks to the network, each gated so that single
//! player runs none of them. No thread, socket or listener exists until the player joins,
//! hosts or opens the multiplayer menu.

use bevy::prelude::*;

use super::link::{Dropped, FromServer, Link, Receive, drain};
use super::{
    CloseLan, JoinServer, LeaveSession, NetDir, OpenToLan, RefreshLobby, clock, host, in_session,
    join, joined, scan,
};
use crate::menus::Menu;
use crate::saves::SaveSet;
use crate::screens::{Screen, in_control};
use crate::sim::ShareChange;
use crate::space::ClockStep;

/// Registers the systems of a session that run every frame: joining, the link, the shared
/// log and clock, the records and leaving.
fn session(app: &mut App) {
    let linked = resource_exists::<Link>;
    app.add_systems(
        Update,
        (
            join::join.run_if(on_message::<JoinServer>),
            host::open_lan.run_if(on_message::<OpenToLan>),
            drain.in_set(Receive).run_if(linked),
            (
                join::welcome,
                join::roster,
                join::take_changes,
                clock::pong,
                join::end,
            )
                .chain()
                .distributive_run_if(linked),
            join::share.run_if(linked).run_if(on_message::<ShareChange>),
            clock::ping.run_if(linked).run_if(in_session),
            join::keep_record.run_if(linked).run_if(joined),
            join::leave
                .run_if(linked)
                .run_if(on_message::<LeaveSession>),
            join::notify.run_if(join::pending).run_if(in_control),
        )
            .chain(),
    );
}

/// Registers the end of hosting and the beacon, after the link is read and its end handled.
fn hosting(app: &mut App) {
    let hosted = resource_exists::<host::Hosted>;
    app.add_systems(
        Update,
        (
            host::close_lan
                .run_if(hosted)
                .run_if(on_message::<CloseLan>),
            host::resave.run_if(resource_exists::<host::Resave>),
            host::count.run_if(hosted),
        )
            .chain()
            .after(join::end),
    );
}

/// Registers the systems of a session outside `Update`: the clock right after it moved, and
/// the end of hosting and of a join before the world is saved on leaving or quitting.
fn edges(app: &mut App) {
    let linked = resource_exists::<Link>;
    let hosted = resource_exists::<host::Hosted>;
    app.add_systems(
        PreUpdate,
        clock::sync
            .after(ClockStep)
            .run_if(linked)
            .run_if(in_session),
    )
    .add_systems(
        OnExit(Screen::Gameplay),
        host::close_on_leave.before(SaveSet).run_if(hosted),
    )
    .add_systems(
        Last,
        (
            host::close_on_exit.run_if(hosted),
            join::quit.run_if(linked),
        )
            .chain()
            .before(SaveSet),
    );
}

/// Registers the systems of the multiplayer menu: the status of the saved servers and the
/// games heard on the network while the menu is open.
fn lobby(app: &mut App) {
    app.init_resource::<scan::Pings>()
        .add_systems(OnEnter(Menu::Multiplayer), scan::listen)
        .add_systems(OnExit(Menu::Multiplayer), scan::deafen)
        .add_systems(
            Update,
            (
                scan::refresh.run_if(on_message::<RefreshLobby>),
                scan::collect.run_if(scan::flying),
                scan::hear.run_if(resource_exists::<scan::Scan>),
            ),
        );
}

/// Registers the network driver. The game adds it; headless rigs of the interface do not.
pub fn plugin(app: &mut App) {
    app.init_resource::<NetDir>()
        .add_message::<FromServer>()
        .add_message::<Dropped>();
    session(app);
    hosting(app);
    edges(app);
    lobby(app);
    super::state::plugin(app);
    super::fauna::plugin(app);
    super::hunt::plugin(app);
    super::ground::plugin(app);
    super::chest::plugin(app);
    super::trees::plugin(app);
    super::digs::plugin(app);
    super::friends::plugin(app);
}
