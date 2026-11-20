//! Main menu shown over the title screen.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::recall::{Entry, LastEntry};
use super::update::{check_item, offer};
use super::{Menu, Nav};
use crate::saves::{SaveDir, Starter, WorldEntry, list};
use crate::ui::{Icon, Tone, column, icon, item, item_with, label, root, rule, text, theme};

/// Resumes the world stored in this component.
#[derive(Component)]
struct Continue(WorldEntry);

/// The entries below Continue, in order: icon, label and what each does. The update check and
/// Quit come after them.
const ENTRIES: [(Icon, &str, Nav); 4] = [
    (Icon::Planet, "Worlds", Nav::To(Menu::Worlds)),
    (Icon::Plus, "New world", Nav::To(Menu::NewGame)),
    (Icon::Users, "Multiplayer", Nav::To(Menu::Multiplayer)),
    (Icon::Settings, "Settings", Nav::To(Menu::Settings)),
];

/// Position of the "Check for updates" entry. Quit is the one after it.
const CHECK: usize = ENTRIES.len() + 1;

/// The Continue item: its icon, its label and the muted name of the world it resumes.
fn continue_item(name: &str) -> impl Bundle {
    item_with(
        0,
        (
            Node {
                align_items: AlignItems::Center,
                column_gap: px(theme::GAP_S),
                overflow: Overflow::clip(),
                ..default()
            },
            children![
                icon(Icon::Play, Tone::Plain),
                label("Continue", theme::FONT_BODY, Tone::Plain),
                label(name, theme::FONT_SMALL, Tone::Muted),
            ],
        ),
    )
}

/// The overlay root with the title and a rule.
fn frame() -> impl Bundle {
    (
        DespawnOnExit(Menu::Main),
        root(),
        children![text("Abstractions", theme::FONT_TITLE, theme::TEXT), rule()],
    )
}

/// Spawns `entry` as entry `index` of the column `items`, and returns it with its index.
fn add(
    commands: &mut Commands,
    items: Entity,
    index: usize,
    entry: impl Bundle,
) -> (usize, Entity) {
    let spawned = commands.spawn((entry, Entry(index), ChildOf(items)));
    (index, spawned.id())
}

/// Spawns the title, a rule and the menu items, and focuses the entry used last. The line about
/// a newer release sits above the items, hidden until there is one, and never takes the focus.
fn spawn(mut commands: Commands, dir: Res<SaveDir>, last: Res<LastEntry>) {
    let menu = commands.spawn(frame()).id();
    let items = commands
        .spawn((column(px(theme::MENU_WIDTH), 0.0), ChildOf(menu)))
        .id();
    commands.spawn((offer(), ChildOf(items)));
    let mut entries = Vec::new();
    if let Some(entry) = list(&dir.0).into_iter().next() {
        let resume = (continue_item(&entry.save.name), Continue(entry));
        entries.push(add(&mut commands, items, 0, resume));
    }
    for (index, (symbol, name, nav)) in ENTRIES.into_iter().enumerate() {
        let entry = (item(symbol, name, Tone::Plain), nav);
        entries.push(add(&mut commands, items, index + 1, entry));
    }
    entries.push(add(&mut commands, items, CHECK, check_item()));
    let quit = (item(Icon::Power, "Quit", Tone::Plain), Nav::Quit);
    entries.push(add(&mut commands, items, CHECK + 1, quit));
    let wanted = last.of(Menu::Main, entries[0].0);
    let found = entries.iter().find(|(index, _)| *index == wanted);
    commands
        .entity(found.unwrap_or(&entries[0]).1)
        .insert(AutoFocus);
}

/// Resumes the most recent world.
fn on_activate(event: On<Activate>, resumes: Query<&Continue>, mut starter: Starter) {
    if let Ok(resume) = resumes.get(event.entity) {
        starter.start(resume.0.clone());
    }
}

/// Registers the main menu systems.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_activate)
        .add_systems(OnEnter(Menu::Main), spawn);
}
