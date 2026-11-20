//! Gameplay screen: crosshair and the pause key.

use bevy::prelude::*;

use super::{Screen, in_control, playing};
use crate::computer::Computer;
use crate::menus::Menu;
use crate::space::{Location, Travel};
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
            Visibility::Hidden,
        )],
    ));
}

/// Opens the pause menu on Escape.
///
/// It runs only while the board computer and the field guide are closed. They close on the same
/// key, and their state changes one frame later, so one press never does both.
fn open_pause(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<NextState<Menu>>) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.set(Menu::Pause);
    }
}

/// Shows the crosshair only on foot, while the player controls the game.
fn crosshair_visibility(
    In(control): In<bool>,
    location: Option<Res<Location>>,
    mut dots: Query<&mut Visibility, With<Crosshair>>,
) {
    let on_foot = location.is_none_or(|location| location.travel == Travel::OnFoot);
    let wanted = match control && on_foot {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
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
                open_pause
                    .run_if(playing)
                    .run_if(in_state(Computer::Closed))
                    .run_if(not(crate::fauna::guide::shown)),
                in_control
                    .pipe(crosshair_visibility)
                    .run_if(in_state(Screen::Gameplay)),
            ),
        );
}
