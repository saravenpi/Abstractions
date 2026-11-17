//! Look tests inside the app: no node is rounded, and menus fall back to a plain dark veil
//! where the blurred backdrop is not available.

use bevy::prelude::*;

use super::app;
use super::flow::open;
use crate::menus::Menu;
use crate::ui::{Backdrop, Blurred, theme};

/// The menus that can be opened.
const MENUS: [Menu; 5] = [
    Menu::Main,
    Menu::Worlds,
    Menu::NewGame,
    Menu::Settings,
    Menu::Pause,
];

/// No node of any menu has a border radius.
#[test]
fn no_node_is_rounded() {
    let mut app = app("square");
    let mut seen = 0;
    for menu in MENUS {
        open(&mut app, menu);
        let mut nodes = app.world_mut().query::<&Node>();
        for node in nodes.iter(app.world()) {
            assert_eq!(node.border_radius, BorderRadius::ZERO, "{menu:?}");
            seen += 1;
        }
    }
    assert!(seen > 200, "only {seen} nodes were checked");
}

/// Without a renderer there is no blur material, and every menu keeps its dark veil.
#[test]
fn menus_fall_back_to_a_plain_veil() {
    let mut app = app("veil");
    assert!(app.world().resource::<Backdrop>().0.is_none());
    for menu in MENUS {
        open(&mut app, menu);
        let mut roots = app
            .world_mut()
            .query_filtered::<&BackgroundColor, With<Blurred>>();
        let veils: Vec<Color> = roots.iter(app.world()).map(|veil| veil.0).collect();
        assert_eq!(veils, [theme::SCRIM], "{menu:?}");
    }
}
