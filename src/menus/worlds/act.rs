//! The actions of the worlds menu, shared by its keys and its buttons.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::rows::Row;
use super::{Mode, Notice, Worlds, ops};
use crate::saves::{SaveDir, Starter};
use crate::update::Updates;
use crate::update::older::OLDER_TAG;

/// What a key or a button of the worlds menu does.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Act {
    /// Start the selected world.
    Play,
    /// Start typing a new name for the selected world.
    Rename,
    /// Copy the selected world.
    Duplicate,
    /// Ask whether to delete the selected world.
    Delete,
    /// Switch between the two orders.
    Sort,
    /// Apply the typed name, or delete the world for good.
    Confirm,
    /// Drop the typed name, or keep the world.
    Cancel,
}

/// State the actions read and write.
#[derive(SystemParam)]
pub(super) struct Ctx<'w, 's> {
    pub(super) worlds: ResMut<'w, Worlds>,
    pub(super) mode: ResMut<'w, Mode>,
    notice: ResMut<'w, Notice>,
    dir: Res<'w, SaveDir>,
    starter: Starter<'w, 's>,
    updates: ResMut<'w, Updates>,
}

impl Ctx<'_, '_> {
    /// Runs an action. An action that does not fit the mode does nothing.
    pub(super) fn run(&mut self, act: Act) {
        self.notice.set_if_neq(Notice::default());
        let outcome = match (act, self.mode.clone()) {
            (Act::Play, Mode::Browse) => self.play(),
            (Act::Rename, Mode::Browse) => self.ask(|name| Mode::Rename(name.to_string())),
            (Act::Delete, Mode::Browse) => self.ask(|_| Mode::Delete { armed: false }),
            (Act::Duplicate, Mode::Browse) => self.duplicate(),
            (Act::Sort, Mode::Browse) => {
                self.worlds.resort();
                Ok(())
            }
            (Act::Confirm, Mode::Rename(typed)) => self.rename(&typed),
            (Act::Confirm, Mode::Delete { .. }) => self.delete(),
            (Act::Cancel, _) => {
                self.mode.set_if_neq(Mode::Browse);
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(message) = outcome {
            self.notice.0 = message;
        }
    }

    /// Starts the selected world. A world of an older release opens in that release, which the
    /// game downloads first when it is not there.
    fn play(&mut self) -> Result<(), String> {
        let Some(world) = self.worlds.selected().cloned() else {
            return Ok(());
        };
        if world.entry.save.made_with().is_none() {
            self.starter.start(world.entry);
            return Ok(());
        }
        let data = dirs::data_dir().unwrap_or_default().join("abstractions");
        self.updates.open_older(OLDER_TAG, &world.entry.id, &data);
        Ok(())
    }

    /// Enters the mode `mode` builds from the name of the selected world, when there is one.
    fn ask(&mut self, mode: impl Fn(&str) -> Mode) -> Result<(), String> {
        if let Some(world) = self.worlds.selected() {
            *self.mode = mode(&world.entry.save.name);
        }
        Ok(())
    }

    /// Reads the worlds again and selects the one with id `keep`.
    fn reload(&mut self, keep: Option<&str>) {
        let list = ops::load(&self.dir.0);
        self.worlds.fill(list, keep);
    }

    /// Copies the selected world and selects the copy.
    fn duplicate(&mut self) -> Result<(), String> {
        let Some(world) = self.worlds.selected().cloned() else {
            return Ok(());
        };
        let names: Vec<&str> = self
            .worlds
            .list
            .iter()
            .map(|w| w.entry.save.name.as_str())
            .collect();
        let copy = ops::duplicate(&self.dir.0, &world.entry, &names)
            .map_err(|error| format!("Could not duplicate: {error}"))?;
        self.reload(Some(&copy.id));
        Ok(())
    }

    /// Gives the selected world the typed name. A refused name keeps the field open.
    fn rename(&mut self, typed: &str) -> Result<(), String> {
        let at = self.worlds.window.cursor;
        let Some(mut entry) = self.worlds.selected().map(|world| world.entry.clone()) else {
            return Ok(());
        };
        let others = self
            .worlds
            .list
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != at);
        let others: Vec<&str> = others.map(|(_, w)| w.entry.save.name.as_str()).collect();
        ops::rename(&self.dir.0, &mut entry, typed, &others)?;
        *self.mode = Mode::Browse;
        self.reload(Some(&entry.id));
        Ok(())
    }

    /// Deletes the selected world for good.
    fn delete(&mut self) -> Result<(), String> {
        *self.mode = Mode::Browse;
        let Some(id) = self.worlds.selected().map(|world| world.entry.id.clone()) else {
            return Ok(());
        };
        let done =
            ops::remove(&self.dir.0, &id).map_err(|error| format!("Could not delete: {error}"));
        self.reload(None);
        done
    }
}

/// Runs the action of an activated button, or plays the activated row.
pub(super) fn on_activate(event: On<Activate>, acts: Query<&Act>, rows: Query<&Row>, mut ctx: Ctx) {
    if let Ok(act) = acts.get(event.entity) {
        ctx.run(*act);
    } else if let (Ok(row), Mode::Browse) = (rows.get(event.entity), ctx.mode.clone()) {
        let len = ctx.worlds.list.len();
        if let Some(index) = ctx.worlds.window.index(len, row.0) {
            ctx.worlds.window.cursor = index;
            ctx.run(Act::Play);
        }
    }
}
