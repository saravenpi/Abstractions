//! Opening, closing and drawing the menu of a right click. It has no border and no rounded
//! corner: a block a tone lighter than the map, with one white line per choice, inverted
//! under the pointer.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;

use super::spot::Pointed;
use super::{Choice, ContextMenu, Opened, PAD, ROW, Under, WIDTH, choices, place, words};
use crate::computer::{Computer, PANEL};
use crate::map::level::Level;
use crate::ui::{Tone, entry, frame, theme};

/// Marks the menu on the screen.
#[derive(Component)]
pub struct MenuRoot;

/// Layer the menu is drawn on, over the whole map screen.
const LAYER: i32 = 50;

/// Returns true while the pointer is over the open menu.
fn over(roots: &Query<&RelativeCursorPosition, With<MenuRoot>>) -> bool {
    roots.iter().any(|root| root.cursor_over)
}

/// Closes the menu on a press of a button outside it, and when the level shown changes under
/// it.
pub(super) fn dismiss(
    buttons: Res<ButtonInput<MouseButton>>,
    level: Res<Level>,
    roots: Query<&RelativeCursorPosition, With<MenuRoot>>,
    mut menu: ResMut<ContextMenu>,
) {
    let pressed = buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]);
    let away = (pressed && !over(&roots)) || level.is_changed();
    if away && menu.0.is_some() {
        menu.0 = None;
    }
}

/// Opens the menu at the pointer on a right click on the map, for what is under the pointer.
pub(super) fn summon(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    pointed: Pointed,
    mut menu: ResMut<ContextMenu>,
) {
    if !buttons.just_pressed(MouseButton::Right) || menu.0.is_some() {
        return;
    }
    let Some(window) = windows.iter().next() else {
        return;
    };
    let (Some(at), Some(under)) = (window.cursor_position(), pointed.under()) else {
        return;
    };
    let at = place(at, window.size(), choices(&under).len());
    menu.0 = Some(Opened { under, at });
}

/// One line of the menu for `under`.
fn line(choice: Choice, under: Under) -> impl Bundle {
    let (icon, said) = words(choice, &under);
    let node = Node {
        width: percent(100),
        height: px(ROW),
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(px(theme::GAP_S)),
        ..default()
    };
    (
        choice,
        frame(),
        node,
        children![entry(icon, said, theme::FONT_SMALL, Tone::Plain)],
    )
}

/// Draws the menu again whenever it opens, closes or moves.
pub(super) fn draw(
    menu: Res<ContextMenu>,
    roots: Query<Entity, With<MenuRoot>>,
    mut commands: Commands,
) {
    if !menu.is_changed() {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some(Opened { under, at }) = menu.0 else {
        return;
    };
    let node = Node {
        position_type: PositionType::Absolute,
        left: px(at.x),
        top: px(at.y),
        width: px(WIDTH),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(PAD)),
        ..default()
    };
    let lines = choices(&under)
        .iter()
        .map(move |choice| line(*choice, under));
    commands.spawn((
        (MenuRoot, DespawnOnExit(Computer::Open), GlobalZIndex(LAYER)),
        (
            node,
            BackgroundColor(PANEL),
            RelativeCursorPosition::default(),
        ),
        Children::spawn(SpawnIter(lines)),
    ));
}
