//! Navigation buttons shared by every menu.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Menu;
use super::options::capturing;
use super::slots::{move_slots, uses_slots};
use crate::screens::Screen;
use crate::ui::move_focus;

/// What a navigation button does when activated.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    /// Open another menu.
    To(Menu),
    /// Open settings and remember the current menu.
    Settings,
    /// Return to the menu that opened settings.
    Back,
    /// Leave the world and return to the title screen.
    ToTitle,
    /// Close the menu and keep playing.
    Resume,
    /// Exit the application.
    Quit,
}

/// The menu that opened the settings menu.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsOrigin(pub Menu);

impl Default for SettingsOrigin {
    fn default() -> Self {
        Self(Menu::Main)
    }
}

/// Keyboard systems of every open menu. They pause while a key is being rebound.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct MenuKeys;

/// State the navigation handler reads and writes.
#[derive(SystemParam)]
struct NavCtx<'w> {
    menu: Res<'w, State<Menu>>,
    origin: ResMut<'w, SettingsOrigin>,
    next_menu: ResMut<'w, NextState<Menu>>,
    next_screen: ResMut<'w, NextState<Screen>>,
    exit: MessageWriter<'w, AppExit>,
}

/// Applies a navigation action to the menu and screen states.
fn on_activate(event: On<Activate>, navs: Query<&Nav>, mut ctx: NavCtx) {
    let Ok(nav) = navs.get(event.entity) else {
        return;
    };
    match *nav {
        Nav::To(menu) => ctx.next_menu.set(menu),
        Nav::Settings => {
            ctx.origin.0 = *ctx.menu.get();
            ctx.next_menu.set(Menu::Settings);
        }
        Nav::Back => ctx.next_menu.set(ctx.origin.0),
        Nav::ToTitle => ctx.next_screen.set(Screen::Title),
        Nav::Resume => ctx.next_menu.set(Menu::None),
        Nav::Quit => {
            ctx.exit.write(AppExit::Success);
        }
    }
}

/// Goes back one menu on Escape. The pause menu closes itself.
fn back_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<State<Menu>>,
    origin: Res<SettingsOrigin>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    match menu.get() {
        Menu::Settings => next_menu.set(origin.0),
        Menu::NewGame => next_menu.set(Menu::Main),
        _ => {}
    }
}

/// Registers the navigation observer and keyboard systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SettingsOrigin>()
        .add_observer(on_activate)
        .configure_sets(
            Update,
            MenuKeys
                .run_if(not(in_state(Menu::None)))
                .run_if(not(capturing)),
        )
        .add_systems(
            Update,
            (
                move_focus.run_if(not(uses_slots)),
                move_slots.run_if(uses_slots),
                back_on_escape,
            )
                .in_set(MenuKeys),
        );
}
