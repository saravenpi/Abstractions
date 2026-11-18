//! The update menu, and the update entries of the main menu.
//!
//! The main menu always has "Check for updates", and shows a quiet line with an "Update" entry
//! once a newer release is known. The update menu walks one install: the offer with the release
//! notes, the download with its progress, then the restart. Nothing downloads before the player
//! chooses it, and every step until the swap can be cancelled.

mod format;
pub(super) mod home;
mod pages;
pub(super) mod parts;
mod steps;
#[cfg(test)]
mod tests;
pub(super) mod view;

pub(super) use home::{check_item, offer};

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use self::parts::Notice;
use super::Menu;
use crate::settings::{Persistent, Settings};
use crate::update::{Failure, Updates};

/// What an update button does when activated.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateAction {
    /// Look for a newer release.
    Check,
    /// Download the newer release and install it.
    Install,
    /// Stop the download.
    Cancel,
    /// Start the new version and leave this one.
    Restart,
    /// Open the downloaded disk image.
    OpenImage,
    /// Open the download page in the browser.
    OpenPage,
}

/// State the update buttons read and write.
#[derive(SystemParam)]
struct Ctx<'w, 's> {
    /// The updater the buttons drive.
    updates: ResMut<'w, Updates>,
    /// The settings, saved before a restart.
    settings: ResMut<'w, Persistent<Settings>>,
    /// Where the game is told to exit after it started the new version.
    exit: MessageWriter<'w, AppExit>,
    /// The line of the open page that reports a failed restart or opening.
    notices: Query<'w, 's, &'static mut Text, With<Notice>>,
}

impl Ctx<'_, '_> {
    /// Saves the settings, starts the new version and leaves this one.
    fn restart(&mut self) -> Result<(), Failure> {
        if let Err(error) = self.settings.persist() {
            warn!("could not save settings before the restart: {error}");
        }
        self.updates.restart()?;
        self.exit.write(AppExit::Success);
        Ok(())
    }

    /// Writes why something outside the game could not be started on the line of the page.
    fn report(&mut self, done: Result<(), Failure>) {
        let Err(failure) = done else {
            return;
        };
        for mut text in &mut self.notices {
            text.0 = format!("{}. Do it by hand", failure.reason());
        }
    }
}

/// Applies an update button press.
fn on_activate(event: On<Activate>, actions: Query<&UpdateAction>, mut ctx: Ctx) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    match action {
        UpdateAction::Check => ctx.updates.check(),
        UpdateAction::Install => ctx.updates.install(),
        UpdateAction::Cancel => ctx.updates.cancel(),
        UpdateAction::Restart => {
            let done = ctx.restart();
            ctx.report(done);
        }
        UpdateAction::OpenImage => {
            let done = ctx.updates.open_image();
            ctx.report(done);
        }
        UpdateAction::OpenPage => {
            let done = ctx.updates.open_page();
            ctx.report(done);
        }
    }
}

/// Registers the update menu and the update entries of the main menu.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Updates>()
        .add_observer(on_activate)
        .add_systems(OnEnter(Menu::Update), view::spawn)
        .add_systems(Update, home::show.run_if(in_state(Menu::Main)))
        .add_systems(
            Update,
            (view::refresh, steps::tick, parts::page_keys)
                .chain()
                .run_if(in_state(Menu::Update)),
        );
}
