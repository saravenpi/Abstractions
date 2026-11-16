//! Gameplay screen: crosshair and the pause key.

use bevy::prelude::*;

use super::{Screen, playing};
use crate::menus::Menu;
use crate::ui::theme;

/// Marks the crosshair dot.
#[derive(Component)]
struct Crosshair;

/// Spawns the centered crosshair dot.
fn enter(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Crosshair,
            Node {
                width: px(2),
                height: px(2),
                ..default()
            },
            BackgroundColor(theme::CROSSHAIR),
        )],
    ));
}

/// Opens the pause menu on Escape.
fn open_pause(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<NextState<Menu>>) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.set(Menu::Pause);
    }
}

/// Shows the crosshair only while no menu is open.
fn crosshair_visibility(menu: Res<State<Menu>>, mut dots: Query<&mut Visibility, With<Crosshair>>) {
    let wanted = if *menu.get() == Menu::None {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut dots {
        visibility.set_if_neq(wanted);
    }
}

/// Registers the gameplay screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), enter)
        .add_systems(
            Update,
            (
                open_pause.run_if(playing),
                crosshair_visibility.run_if(in_state(Screen::Gameplay)),
            ),
        );
}
