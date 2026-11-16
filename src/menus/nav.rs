//! Navigation buttons shared by every menu.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Menu;
use crate::screens::Screen;

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

/// Registers the navigation observer.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SettingsOrigin>()
        .add_observer(on_activate);
}
