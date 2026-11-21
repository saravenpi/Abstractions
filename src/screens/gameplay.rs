//! Gameplay screen: crosshair, the pause key and the key that hides the interface.

use bevy::prelude::*;

use super::{Screen, in_control, playing};
use crate::computer::Computer;
use crate::menus::Menu;
use crate::space::{Location, Travel};
use crate::ui::{WorldView, theme};

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

/// The key that hides the interface and shows it again, for a clean view or a screenshot.
pub const HIDE_KEY: KeyCode = KeyCode::F1;

/// Whether the player hid the interface. It is the one switch for every interface layer: the
/// HUD, the markers over the world, the prompts, the hotbar, the crosshair, the frame rate and
/// the tooltips. Menus and the map still show: it acts only while the player controls the game.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HideHud(pub bool);

/// Hides the interface, or shows it again, on [`HIDE_KEY`].
fn toggle_interface(keys: Res<ButtonInput<KeyCode>>, mut hidden: ResMut<HideHud>) {
    if keys.just_pressed(HIDE_KEY) {
        hidden.0 = !hidden.0;
    }
}

/// Draws the game image over every interface layer while the interface is hidden and the
/// player controls the game, and under them all otherwise.
fn veil(
    In(control): In<bool>,
    hidden: Res<HideHud>,
    mut views: Query<&mut GlobalZIndex, With<WorldView>>,
) {
    let wanted = match control && hidden.0 {
        true => GlobalZIndex(i32::MAX),
        false => GlobalZIndex(i32::MIN),
    };
    for mut layer in &mut views {
        layer.set_if_neq(wanted);
    }
}

/// Shows the interface again when the world is left, so the next one starts with it.
fn reveal(mut hidden: ResMut<HideHud>, mut views: Query<&mut GlobalZIndex, With<WorldView>>) {
    hidden.0 = false;
    for mut layer in &mut views {
        layer.set_if_neq(GlobalZIndex(i32::MIN));
    }
}

/// Registers the gameplay screen systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HideHud>()
        .add_systems(OnEnter(Screen::Gameplay), enter)
        .add_systems(OnExit(Screen::Gameplay), reveal)
        .add_systems(
            Update,
            (
                toggle_interface.run_if(in_control),
                in_control.pipe(veil).run_if(in_state(Screen::Gameplay)),
                open_pause
                    .run_if(playing)
                    .run_if(in_state(Computer::Closed))
                    .run_if(not(crate::fauna::guide::shown))
                    .run_if(not(crate::craft::ui::shown)),
                in_control
                    .pipe(crosshair_visibility)
                    .run_if(in_state(Screen::Gameplay)),
            ),
        );
}
