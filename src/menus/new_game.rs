//! New game menu: type or randomise a seed, then start.

use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::{Menu, Nav, seed};
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};
use crate::ui::{button, heading, label, panel, root, row, theme};
use crate::world::WorldSeed;

/// The digits typed so far.
#[derive(Resource, Default)]
struct SeedInput(String);

/// Marks the text that shows the typed seed.
#[derive(Component)]
struct SeedText;

/// Buttons specific to this menu.
#[derive(Component, Clone, Copy)]
enum SeedAction {
    Random,
    Start,
}

/// State the seed buttons read and write.
#[derive(SystemParam)]
struct SeedCtx<'w> {
    input: ResMut<'w, SeedInput>,
    settings: ResMut<'w, Persistent<Settings>>,
    screen: ResMut<'w, NextState<Screen>>,
    menu: ResMut<'w, NextState<Menu>>,
}

/// Spawns the menu and clears the typed seed.
fn spawn(mut commands: Commands, mut input: ResMut<SeedInput>) {
    input.0.clear();
    commands.spawn((
        DespawnOnExit(Menu::NewGame),
        root(),
        children![(
            panel(),
            children![
                heading("New game", theme::FONT_HEADING),
                (label(""), SeedText),
                (
                    row(),
                    children![
                        (
                            button("Random seed", theme::BUTTON_WIDTH),
                            SeedAction::Random
                        ),
                        (button("Start", theme::BUTTON_WIDTH), SeedAction::Start),
                    ]
                ),
                (button("Back", theme::BUTTON_WIDTH), Nav::To(Menu::Main)),
            ]
        )],
    ));
}

/// Reads typed digits and backspace into the seed.
fn type_seed(mut keys: MessageReader<KeyboardInput>, mut input: ResMut<SeedInput>) {
    for key in keys.read().filter(|key| key.state == ButtonState::Pressed) {
        let backspace = key.logical_key == Key::Backspace;
        seed::edit(&mut input.0, backspace, key.text.as_deref().unwrap_or(""));
    }
}

/// Shows the typed seed, or a hint when it is empty.
fn show_seed(input: Res<SeedInput>, mut texts: Query<&mut Text, With<SeedText>>) {
    let shown = if input.0.is_empty() {
        "Seed: (random)".to_string()
    } else {
        format!("Seed: {}_", input.0)
    };
    for mut text in &mut texts {
        text.set_if_neq(Text(shown.clone()));
    }
}

/// Handles the random and start buttons.
fn on_activate(
    event: On<Activate>,
    actions: Query<&SeedAction>,
    mut commands: Commands,
    mut ctx: SeedCtx,
) {
    let Ok(action) = actions.get(event.entity) else {
        return;
    };
    match action {
        SeedAction::Random => ctx.input.0 = seed::random().to_string(),
        SeedAction::Start => {
            let seed = seed::parse(&ctx.input.0).unwrap_or_else(seed::random);
            if let Err(error) = ctx.settings.update(|s| s.world.last_seed = Some(seed)) {
                warn!("could not save the last seed: {error}");
            }
            commands.insert_resource(WorldSeed(seed));
            ctx.screen.set(Screen::Loading);
            ctx.menu.set(Menu::None);
        }
    }
}

/// Registers the new game menu systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SeedInput>()
        .add_observer(on_activate)
        .add_systems(OnEnter(Menu::NewGame), spawn)
        .add_systems(
            Update,
            (type_seed, show_seed).run_if(in_state(Menu::NewGame)),
        );
}
