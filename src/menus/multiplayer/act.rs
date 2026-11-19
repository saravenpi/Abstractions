//! The actions of the multiplayer menu, shared by its keys and its buttons.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::password::Retry;
use super::rows::Row;
use super::{Form, FormKind, Mode, Notice, Servers, Waiting, ops};
use crate::menus::{Menu, Trail};
use crate::net::{JoinServer, RefreshLobby};
use crate::settings::{Persistent, Settings};

/// What a key or a button of the multiplayer menu does.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::menus) enum Act {
    /// Join the selected server.
    Join,
    /// Type an address to join once.
    Direct,
    /// Type a new server to save.
    Add,
    /// Change the selected saved server.
    Edit,
    /// Forget the selected saved server.
    Remove,
    /// Ask every server for its status again.
    Refresh,
    /// Apply the form.
    Confirm,
    /// Drop the form.
    Cancel,
}

/// State the actions read and write.
#[derive(SystemParam)]
pub(super) struct Ctx<'w> {
    pub(super) servers: ResMut<'w, Servers>,
    pub(super) mode: ResMut<'w, Mode>,
    notice: ResMut<'w, Notice>,
    waiting: ResMut<'w, Waiting>,
    retry: ResMut<'w, Retry>,
    settings: ResMut<'w, Persistent<Settings>>,
    join: MessageWriter<'w, JoinServer>,
    refresh: MessageWriter<'w, RefreshLobby>,
    trail: ResMut<'w, Trail>,
    menu: ResMut<'w, NextState<Menu>>,
}

impl Ctx<'_> {
    /// Runs an action. An action that does not fit the mode does nothing.
    pub(super) fn run(&mut self, act: Act) {
        self.notice.set_if_neq(Notice::default());
        let outcome = match (act, self.mode.clone()) {
            (Act::Join, Mode::Browse) => self
                .servers
                .selected()
                .cloned()
                .map_or(Ok(()), |row| self.join(row.address, row.key, None)),
            (Act::Direct, Mode::Browse) => self.open(FormKind::Direct),
            (Act::Add, Mode::Browse) => self.open(FormKind::Add),
            (Act::Edit, Mode::Browse) => self
                .servers
                .saved()
                .map_or(Ok(()), |index| self.open(FormKind::Edit(index))),
            (Act::Remove, Mode::Browse) => self.remove(),
            (Act::Refresh, Mode::Browse) => {
                self.refresh.write(RefreshLobby);
                Ok(())
            }
            (Act::Confirm, Mode::Form(form)) => self.confirm(form),
            (Act::Cancel, Mode::Form(form)) => self.cancel(form.kind),
            _ => Ok(()),
        };
        if let Err(message) = outcome {
            self.notice.0 = message;
        }
    }

    /// Asks the network to join `address`, with the key a LAN beacon announced and the
    /// password typed, if any.
    fn join(
        &mut self,
        address: String,
        expected_key: Option<[u8; 32]>,
        password: Option<String>,
    ) -> Result<(), String> {
        *self.mode = Mode::Browse;
        self.waiting.0 = true;
        self.join.write(JoinServer {
            address,
            expected_key,
            password,
        });
        Ok(())
    }

    /// Opens a form of `kind`, filled with the saved server it edits or the current name.
    fn open(&mut self, kind: FormKind) -> Result<(), String> {
        let (name, address) = match kind {
            FormKind::Edit(index) => self
                .settings
                .servers
                .get(index)
                .map_or_else(Default::default, |server| {
                    (server.name.clone(), server.address.clone())
                }),
            FormKind::Player => (self.settings.player_name.clone(), String::new()),
            FormKind::Direct | FormKind::Add | FormKind::Password => Default::default(),
        };
        *self.mode = Mode::Form(Form::new(kind, name, address));
        Ok(())
    }

    /// Forgets the selected saved server. A network game cannot be removed.
    fn remove(&mut self) -> Result<(), String> {
        let Some(index) = self.servers.saved() else {
            return Ok(());
        };
        ops::remove(&mut self.settings.servers, index);
        self.persist()
    }

    /// Applies the form: stores the name or the server, or joins the typed address. A refused
    /// field keeps the form open.
    fn confirm(&mut self, form: Form) -> Result<(), String> {
        match form.kind {
            FormKind::Player => self.settings.player_name = ops::player(&form.name)?,
            FormKind::Direct => return self.join(ops::address(&form.address)?, None, None),
            FormKind::Password => {
                let Some((address, key)) = self.retry.0.take() else {
                    return Ok(());
                };
                return self.join(address, key, Some(form.name));
            }
            FormKind::Add | FormKind::Edit(_) => {
                let server = ops::server(&form.name, &form.address)?;
                let at = match form.kind {
                    FormKind::Edit(index) => Some(index),
                    _ => None,
                };
                self.servers.cursor = ops::put(&mut self.settings.servers, at, server);
            }
        }
        *self.mode = Mode::Browse;
        self.persist()
    }

    /// Drops the form. Dropping the player name leaves the menu, which needs one; dropping the
    /// password forgets the server that asked for it.
    fn cancel(&mut self, kind: FormKind) -> Result<(), String> {
        *self.mode = Mode::Browse;
        if kind == FormKind::Password {
            self.retry.0 = None;
        }
        if kind == FormKind::Player {
            let back = self.trail.back(Menu::Multiplayer);
            self.menu.set(back);
        }
        Ok(())
    }

    /// Saves the settings file.
    fn persist(&mut self) -> Result<(), String> {
        self.settings
            .persist()
            .map_err(|error| format!("Could not save the settings: {error}"))
    }
}

/// Runs the action of an activated button, or joins the activated row.
pub(super) fn on_activate(event: On<Activate>, acts: Query<&Act>, rows: Query<&Row>, mut ctx: Ctx) {
    if let Ok(act) = acts.get(event.entity) {
        ctx.run(*act);
    } else if let (Ok(row), Mode::Browse) = (rows.get(event.entity), ctx.mode.clone()) {
        ctx.servers.cursor = row.0;
        ctx.run(Act::Join);
    }
}
