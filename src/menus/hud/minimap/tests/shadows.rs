//! The owner's rule on shadows: an icon never casts one, and text on a panel or a strip has none,
//! so nothing of the HUD or of the menus carries a `TextShadow` or a `BoxShadow`.

use bevy::prelude::*;

use super::rig::app;
use crate::menus::Menu;

/// Returns the entities of `app` that carry a shadow: any text, icons included, or any box.
fn shadowed(app: &mut App) -> Vec<Entity> {
    let mut found = Vec::new();
    let mut texts = app.world_mut().query_filtered::<Entity, With<TextShadow>>();
    found.extend(texts.iter(app.world()));
    let mut boxes = app.world_mut().query_filtered::<Entity, With<BoxShadow>>();
    found.extend(boxes.iter(app.world()));
    found
}

/// With the HUD up and then each of the title menus open, no entity draws a shadow.
#[test]
fn no_icon_and_no_text_casts_a_shadow() {
    let mut app = app("shadows");
    for _ in 0..4 {
        app.update();
    }
    assert!(shadowed(&mut app).is_empty(), "the HUD casts a shadow");
    for menu in [Menu::Main, Menu::Worlds, Menu::NewGame, Menu::Settings] {
        app.world_mut().resource_mut::<NextState<Menu>>().set(menu);
        for _ in 0..4 {
            app.update();
        }
        assert!(shadowed(&mut app).is_empty(), "{menu:?} casts a shadow");
    }
}
