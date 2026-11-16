//! New world menu: a name and a seed, then create.

mod input;
mod view;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::{Menu, seed};
use crate::saves::{SaveDir, Starter, new_entry, write};

/// Default name offered for a new world.
const DEFAULT_NAME: &str = "New world";

/// The text typed so far.
#[derive(Resource)]
struct NewWorld {
    name: String,
    seed: String,
}

impl Default for NewWorld {
    fn default() -> Self {
        Self {
            name: DEFAULT_NAME.to_string(),
            seed: String::new(),
        }
    }
}

/// Which text a field row edits.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Name,
    Seed,
}

/// Buttons specific to this menu.
#[derive(Component, Clone, Copy)]
enum Action {
    Random,
    Create,
}

/// State needed to create a world and start it.
#[derive(SystemParam)]
struct CreateCtx<'w, 's> {
    input: ResMut<'w, NewWorld>,
    dir: Res<'w, SaveDir>,
    starter: Starter<'w, 's>,
}

impl CreateCtx<'_, '_> {
    /// Writes the new world file, then starts the world.
    fn create(&mut self) {
        let name = self.input.name.trim();
        let name = if name.is_empty() { DEFAULT_NAME } else { name };
        let seed = seed::parse(&self.input.seed).unwrap_or_else(seed::random);
        let entry = new_entry(&self.dir.0, name, seed);
        if let Err(error) = write(&self.dir.0, &entry.id, &entry.save) {
            warn!("could not save the new world {}: {error}", entry.id);
        }
        self.starter.start(entry);
    }
}

/// Handles the random and create buttons.
fn on_activate(event: On<Activate>, actions: Query<&Action>, mut ctx: CreateCtx) {
    match actions.get(event.entity) {
        Ok(Action::Random) => ctx.input.seed = seed::random().to_string(),
        Ok(Action::Create) => ctx.create(),
        Err(_) => {}
    }
}

/// Registers the new world menu systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<NewWorld>()
        .add_observer(on_activate)
        .add_systems(OnEnter(Menu::NewGame), view::spawn)
        .add_systems(
            Update,
            (
                input::type_text,
                input::switch_field,
                input::create_on_enter,
                input::show_fields,
            )
                .run_if(in_state(Menu::NewGame)),
        );
}
