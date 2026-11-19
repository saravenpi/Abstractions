//! The path over the map: the Back button alone on its row, and under it the levels from the
//! universe down to the one shown, each a button that jumps to its level but the last. Black
//! and white, with no border. Backspace goes back as the button does.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Place;
use super::go::Nav;
use crate::computer::Typing;
use crate::ui::{Icon, Tone, entry, frame, label, theme};
use crate::universe::Universe;

/// Marks the Back button.
#[derive(Component)]
pub struct BackButton;

/// Marks the row the parts of the path are written in.
#[derive(Component)]
pub struct Parts;

/// A part of the path, which jumps to its level.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crumb(pub Place);

/// The node of a small button of the path.
fn button_node() -> Node {
    Node {
        align_items: AlignItems::Center,
        padding: UiRect::axes(px(theme::GAP_XS), px(theme::GAP_XS)),
        ..default()
    }
}

/// The path over the map, empty until [`write`] fills it: the row of the Back button, which
/// keeps its height while the button hides, over the row of the parts of the path.
pub fn path() -> impl Bundle {
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_XS),
        min_height: px(theme::ROW_COMPACT),
        ..default()
    };
    let back = (
        BackButton,
        frame(),
        button_node(),
        children![entry(Icon::Back, "Back", theme::FONT_SMALL, Tone::Plain)],
    );
    let column = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexStart,
        row_gap: px(theme::GAP_XS),
        ..default()
    };
    (
        column,
        children![(row.clone(), children![back]), (Parts, row)],
    )
}

/// Returns one part of the path: a button for a level above the one shown, plain words for it.
fn part(commands: &mut ChildSpawnerCommands, place: Place, name: String, last: bool) {
    if last {
        commands.spawn(label(name, theme::FONT_SMALL, Tone::Plain));
        return;
    }
    let words = label(name, theme::FONT_SMALL, Tone::Muted);
    commands.spawn((Crumb(place), frame(), button_node(), children![words]));
    commands.spawn(label(">", theme::FONT_SMALL, Tone::Muted));
}

/// Writes the path again when the level shown changes, and shows the Back button only while
/// there is somewhere to go back to.
pub(super) fn write(
    (nav, universe): (Nav, Option<Res<Universe>>),
    rows: Query<Entity, With<Parts>>,
    mut backs: Query<&mut Node, With<BackButton>>,
    mut commands: Commands,
    mut last: Local<Option<(Place, Entity)>>,
) {
    let (Some(universe), Some(row)) = (universe, rows.iter().next()) else {
        return;
    };
    let here = nav.here().place;
    let shown = [Display::None, Display::Flex][usize::from(nav.behind())];
    for mut node in &mut backs {
        if node.display != shown {
            node.display = shown;
        }
    }
    if *last == Some((here, row)) {
        return;
    }
    *last = Some((here, row));
    let path = here.path();
    commands
        .entity(row)
        .despawn_children()
        .with_children(|row| {
            for (at, place) in path.iter().enumerate() {
                part(row, *place, place.name(&universe), at + 1 == path.len());
            }
        });
}

/// Goes back on Backspace, unless something is typed.
pub(super) fn back_key(keys: Res<ButtonInput<KeyCode>>, typing: Res<Typing>, mut nav: Nav) {
    if keys.just_pressed(KeyCode::Backspace) && typing.is_off() {
        nav.back();
    }
}

/// Goes back on the Back button, and to the level of a part of the path clicked.
pub(super) fn on_press(
    event: On<Activate>,
    buttons: Query<(Has<BackButton>, Option<&Crumb>)>,
    mut nav: Nav,
) {
    match buttons.get(event.entity) {
        Ok((true, _)) => nav.back(),
        Ok((_, Some(Crumb(place)))) => nav.enter(*place),
        _ => {}
    }
}
