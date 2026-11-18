//! Multiplayer menu: the saved servers and the games on the local network, with join, direct
//! connect, add, edit, remove and refresh, and the screen shown while a join waits.

mod act;
mod bars;
mod form;
mod form_view;
mod joining;
mod keys;
mod ops;
mod password;
mod rows;
#[cfg(test)]
mod tests;
mod view;

#[cfg(test)]
pub(in crate::menus) use act::Act;

use bevy::prelude::*;

use super::Menu;
use super::nav::MenuKeys;
use crate::net::RowStatus;

/// Which list a row belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    /// A saved server, at this index of the settings list.
    Saved(usize),
    /// A game found on the local network.
    Lan,
}

/// One server as the menu shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Listed {
    /// Its list, and its place in the settings for a saved one.
    section: Section,
    /// The name given to it. Empty shows the address.
    name: String,
    /// Where to reach it.
    address: String,
    /// What it answered last.
    status: RowStatus,
    /// The fingerprint a LAN beacon announced.
    key: Option<[u8; 32]>,
}

/// The rows shown, saved servers first, and the selected one.
#[derive(Resource, Debug, Default)]
struct Servers {
    /// Every row, in display order.
    list: Vec<Listed>,
    /// Index of the selected row.
    cursor: usize,
}

impl Servers {
    /// Returns the selected row.
    fn selected(&self) -> Option<&Listed> {
        self.list.get(self.cursor)
    }

    /// Returns the settings index of the selected row, when it is a saved server.
    fn saved(&self) -> Option<usize> {
        match self.selected()?.section {
            Section::Saved(index) => Some(index),
            Section::Lan => None,
        }
    }

    /// Returns how many saved servers and how many network games the list holds.
    fn shape(&self) -> (usize, usize) {
        let saved = self.list.iter().filter(|row| row.section != Section::Lan);
        let saved = saved.count();
        (saved, self.list.len() - saved)
    }

    /// Replaces the rows and keeps the cursor inside them.
    fn fill(&mut self, list: Vec<Listed>) {
        self.cursor = self.cursor.min(list.len().saturating_sub(1));
        self.list = list;
    }
}

/// What a form asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FormKind {
    /// The player name, before the first list.
    Player,
    /// An address to join once, without saving it.
    Direct,
    /// A new saved server.
    Add,
    /// The saved server at this index.
    Edit(usize),
    /// The password of the server that refused the last one.
    Password,
}

/// One text field of a form.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Field {
    /// The player name, or the name of a server.
    Name,
    /// The address of a server.
    Address,
    /// The password of a server, hidden as it is typed.
    Password,
}

/// A form being filled: what it asks for, what was typed and the field being typed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Form {
    /// What the form asks for.
    kind: FormKind,
    /// The typed name.
    name: String,
    /// The typed address.
    address: String,
    /// The field the keys type into.
    field: Field,
}

/// What the keyboard is doing in the menu.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub(super) enum Mode {
    /// Moving through the list.
    #[default]
    Browse,
    /// Filling a form.
    Form(Form),
}

/// What part of the menu a node is, which decides when it is drawn.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Pane {
    /// The list and its actions, drawn while browsing.
    List,
    /// The form and its actions, drawn while a form is filled.
    Form,
    /// The name field, drawn by every form but direct connect.
    Name,
    /// The address field, drawn by the forms of a server.
    Address,
    /// The password field, drawn by the password form only.
    Password,
}

/// The short line under the list: a refused field, a failed save or why a session ended.
#[derive(Resource, Debug, Default, PartialEq, Eq)]
struct Notice(String);

/// Whether a join was asked and the session has not answered it yet.
#[derive(Resource, Debug, Default)]
struct Waiting(bool);

/// Why the last session ended, until the menu shows it.
#[derive(Resource, Debug, Default)]
struct Ended(Option<String>);

/// Run condition: a form of the multiplayer menu is being filled.
pub(super) fn typing(mode: Res<Mode>) -> bool {
    *mode != Mode::Browse
}

/// Registers the multiplayer menu and joining screen systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Servers>()
        .init_resource::<Mode>()
        .init_resource::<Notice>()
        .init_resource::<Waiting>()
        .init_resource::<Ended>()
        .init_resource::<password::Retry>()
        .add_observer(act::on_activate)
        .add_systems(OnEnter(Menu::Multiplayer), view::spawn)
        .add_systems(
            Update,
            (
                (keys::follow_focus, keys::browse).chain().in_set(MenuKeys),
                form::type_keys.after(MenuKeys).run_if(typing),
                rows::refresh,
                view::layout,
                rows::show,
                keys::focus_cursor,
                form_view::show,
                view::show_notice,
                joining::follow_session,
                password::ask,
            )
                .chain()
                .run_if(in_state(Menu::Multiplayer)),
        )
        .add_systems(Update, password::remember);
    joining::plugin(app);
}
