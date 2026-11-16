//! Main menu shown over the title screen.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::{Menu, Nav};
use crate::saves::{SaveDir, Starter, WorldEntry, list};
use crate::ui::{Tone, column, item, item_with, label, root, rule, text, theme};

/// Resumes the world stored in this component.
#[derive(Component)]
struct Continue(WorldEntry);

/// The Continue item: its label and the muted name of the world it resumes.
fn continue_item(name: &str) -> impl Bundle {
    item_with(
        0,
        (
            Node {
                align_items: AlignItems::Baseline,
                column_gap: px(theme::SPACE * 1.5),
                ..default()
            },
            children![
                label("Continue", theme::FONT_BODY, Tone::Plain),
                label(name, theme::FONT_SMALL, Tone::Muted),
            ],
        ),
    )
}

/// The overlay root with the title and an accent rule.
fn frame() -> impl Bundle {
    (
        DespawnOnExit(Menu::Main),
        root(),
        children![text("Abstractions", theme::FONT_TITLE, theme::TEXT), rule()],
    )
}

/// Spawns the title, an accent rule and the menu items.
fn spawn(mut commands: Commands, dir: Res<SaveDir>) {
    let menu = commands.spawn(frame()).id();
    let items = commands
        .spawn((column(px(theme::MENU_WIDTH), 0.0), ChildOf(menu)))
        .id();
    let resume = list(&dir.0).into_iter().next().map(|entry| {
        commands
            .spawn((
                continue_item(&entry.save.name),
                Continue(entry),
                ChildOf(items),
            ))
            .id()
    });
    let worlds = commands
        .spawn((item("Worlds", Tone::Plain), Nav::To(Menu::Worlds)))
        .id();
    commands
        .entity(items)
        .add_child(worlds)
        .with_children(|parent| {
            parent.spawn((item("New world", Tone::Plain), Nav::To(Menu::NewGame)));
            parent.spawn((item("Settings", Tone::Plain), Nav::Settings));
            parent.spawn((item("Quit", Tone::Plain), Nav::Quit));
        });
    commands.entity(resume.unwrap_or(worlds)).insert(AutoFocus);
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
