//! Navigation shared by every menu: buttons that open a menu, and going back one level.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::Menu;
use super::multiplayer::typing;
use super::options::capturing;
use super::slots::{move_slots, uses_slots};
use super::worlds::editing;
use crate::screens::Screen;
use crate::ui::move_focus;

/// What a navigation button does when activated.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    /// Open another menu, one level deeper.
    To(Menu),
    /// Go back one level.
    Back,
    /// Leave the world and return to the title screen.
    ToTitle,
    /// Close the menu and keep playing.
    Resume,
    /// Exit the application.
    Quit,
}

/// The menus that led to the open one, oldest first. Going back pops the last.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct Trail(pub Vec<Menu>);

impl Trail {
    /// Returns the menu one level up from `from` and forgets it. Without a trail, the pause
    /// menu closes, the main menu stays and every other menu returns to the main menu.
    pub fn back(&mut self, from: Menu) -> Menu {
        let fallback = match from {
            Menu::Pause | Menu::None => Menu::None,
            _ => Menu::Main,
        };
        self.0.pop().unwrap_or(fallback)
    }
}

/// Keyboard systems of every open menu. They pause while a key is being rebound, a name typed
/// or a server form filled.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct MenuKeys;

/// State the navigation handler reads and writes.
#[derive(SystemParam)]
struct NavCtx<'w> {
    menu: Res<'w, State<Menu>>,
    trail: ResMut<'w, Trail>,
    next_menu: ResMut<'w, NextState<Menu>>,
    next_screen: ResMut<'w, NextState<Screen>>,
    exit: MessageWriter<'w, AppExit>,
}

/// Applies a navigation action to the menu and screen states.
fn on_activate(event: On<Activate>, navs: Query<&Nav>, mut ctx: NavCtx) {
    let Ok(nav) = navs.get(event.entity) else {
        return;
    };
    let here = *ctx.menu.get();
    match *nav {
        Nav::To(menu) => {
            ctx.trail.0.push(here);
            ctx.next_menu.set(menu);
        }
        Nav::Back => ctx.next_menu.set(ctx.trail.back(here)),
        Nav::ToTitle => ctx.next_screen.set(Screen::Title),
        Nav::Resume => ctx.next_menu.set(Menu::None),
        Nav::Quit => {
            ctx.exit.write(AppExit::Success);
        }
    }
}

/// Goes back one level on Escape. The main menu is the top level and stays.
fn back_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<State<Menu>>,
    mut trail: ResMut<Trail>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    let here = *menu.get();
    if keys.just_pressed(KeyCode::Escape) && here != Menu::Main {
        next_menu.set(trail.back(here));
    }
}

/// Forgets the trail once no menu or the top menu is open.
fn clear_trail(mut trail: ResMut<Trail>) {
    trail.0.clear();
}

/// Registers the navigation observer and keyboard systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Trail>()
        .add_observer(on_activate)
        .add_systems(OnEnter(Menu::None), clear_trail)
        .add_systems(OnEnter(Menu::Main), clear_trail)
        .configure_sets(
            Update,
            MenuKeys
                .run_if(not(in_state(Menu::None)))
                .run_if(not(capturing))
                .run_if(not(editing))
                .run_if(not(typing)),
        )
        .add_systems(
            Update,
            (
                move_focus
                    .run_if(not(uses_slots))
                    .run_if(not(in_state(Menu::Worlds))),
                move_slots.run_if(uses_slots),
                back_on_escape,
            )
                .in_set(MenuKeys),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Going back follows the trail, then falls back to the level above.
    #[test]
    fn back_pops_the_trail_then_falls_back() {
        let mut trail = Trail(vec![Menu::Main, Menu::Worlds]);
        assert_eq!(trail.back(Menu::NewGame), Menu::Worlds);
        assert_eq!(trail.back(Menu::Worlds), Menu::Main);
        assert_eq!(trail.back(Menu::Settings), Menu::Main);
        assert_eq!(trail.back(Menu::Pause), Menu::None);
    }
}
