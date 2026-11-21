//! The backdrop of the pause menu: the game behind it, blurred and dimmed, and no picture of
//! its own.

use bevy::prelude::*;

use super::app;
use super::fit_screens::under;
use super::flow::open;
use crate::menus::Menu;
use crate::ui::{Blurred, Overlay, theme};

/// The pause menu lays the blur over the game, dimmed by the scrim where the blur is not
/// ready, and nothing under it shows a picture: no slide of the title screen, no still image.
#[test]
fn the_pause_menu_blurs_the_game_and_shows_no_picture() {
    let mut app = app("pause-blur");
    open(&mut app, Menu::Pause);
    let mut roots = app
        .world_mut()
        .query_filtered::<(Entity, &DespawnOnExit<Menu>, &BackgroundColor, Has<Blurred>), With<Overlay>>();
    let found: Vec<(Entity, Color, bool)> = roots
        .iter(app.world())
        .filter(|(_, scope, ..)| scope.0 == Menu::Pause)
        .map(|(root, _, fill, blurred)| (root, fill.0, blurred))
        .collect();
    let &[(root, fill, blurred)] = found.as_slice() else {
        panic!("one pause root, found {}", found.len());
    };
    assert!(blurred, "the pause root blurs the game");
    assert_eq!(fill, theme::SCRIM);
    let world = app.world();
    let pictures = under(world, root)
        .into_iter()
        .filter(|node| world.get::<ImageNode>(*node).is_some())
        .count();
    assert_eq!(pictures, 0, "the pause menu shows a picture");
}
