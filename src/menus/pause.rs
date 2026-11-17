//! Pause menu: freezes physics while open.

use avian3d::prelude::*;
use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::pause_hud::{saved_label, show_saved, world_corner};
use super::recall::{Entry, LastEntry};
use super::{Menu, Nav};
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::ui::{Icon, Tone, column, footer, heading, item, panel, root, theme};
use crate::world::WorldSeed;

/// The pause menu entries, in order: icon, label and what each does.
const ENTRIES: [(Icon, &str, Nav); 4] = [
    (Icon::Play, "Resume", Nav::Resume),
    (Icon::Settings, "Settings", Nav::To(Menu::Settings)),
    (Icon::LogOut, "Save and quit to menu", Nav::ToTitle),
    (Icon::Power, "Save and quit the game", Nav::Quit),
];

/// Returns the corner text: the world name and seed, or the seed alone.
fn corner_text(active: Option<&ActiveWorld>, seed: Option<&WorldSeed>) -> String {
    match (active, seed) {
        (Some(active), _) => format!("{} - seed {}", active.0.save.name, active.0.save.seed),
        (None, Some(seed)) => format!("Seed {}", seed.0),
        (None, None) => String::new(),
    }
}

/// What the pause menu shows besides its entries.
type Corner<'w> = (Option<Res<'w, WorldSeed>>, Option<Res<'w, ActiveWorld>>);

/// Spawns the pause menu over the blurred game, with the world name and seed in the
/// corner, and focuses the entry used last.
fn spawn(mut commands: Commands, (seed, active): Corner, last: Res<LastEntry>) {
    let corner = corner_text(active.as_deref(), seed.as_deref());
    let menu = commands
        .spawn((
            DespawnOnExit(Menu::Pause),
            root(),
            children![footer("Esc resumes"), world_corner(corner), saved_label()],
        ))
        .id();
    let card = commands
        .spawn((panel(), ChildOf(menu), children![heading("Paused")]))
        .id();
    let items = commands
        .spawn((column(px(theme::MENU_WIDTH), 0.0), ChildOf(card)))
        .id();
    let wanted = last.of(Menu::Pause, 0);
    for (index, (symbol, name, nav)) in ENTRIES.into_iter().enumerate() {
        let entry = (item(symbol, name, Tone::Plain), nav, Entry(index));
        let mut spawned = commands.spawn((entry, ChildOf(items)));
        if index == wanted {
            spawned.insert(AutoFocus);
        }
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
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), (spawn, freeze))
        .add_systems(OnEnter(Menu::None), thaw)
        .add_systems(OnExit(Screen::Gameplay), thaw)
        .add_systems(Update, show_saved.run_if(in_state(Menu::Pause)));
}
