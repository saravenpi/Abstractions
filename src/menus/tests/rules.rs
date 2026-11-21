//! The design rules, held mechanically: every node of every converted page is walked in a built
//! app, and its spacing, its positions, its sizes, its type and its colours are checked against
//! the tokens of the theme. A page joins [`PAGES`] when it is converted to the page skeleton.

use bevy::prelude::*;
use bevy::text::{FontSize, FontSource};
use bevy::ui::{BoxShadow, Outline};

use super::app;
use super::flow::open;
use crate::menus::Menu;
use crate::menus::options::{SettingsTab, Tab};
use crate::ui::{ICON_FONT, Page, Thumb, theme};

/// The pages built on the page skeleton.
pub(super) const PAGES: [Menu; 2] = [Menu::Worlds, Menu::Settings];

/// The spacing scale, with nothing.
const SCALE: [f32; 7] = [0.0, 4.0, 8.0, 16.0, 24.0, 32.0, 48.0];

/// The type scale.
const TYPE: [f32; 4] = [16.0, 20.0, 32.0, 72.0];

/// Opens `menu` and returns its root with every node under it, the root first.
pub(super) fn nodes(app: &mut App, menu: Menu) -> Vec<Entity> {
    open(app, menu);
    let mut roots = app.world_mut().query_filtered::<Entity, With<Page>>();
    let root = roots.single(app.world()).expect("one page is open");
    let mut found = vec![root];
    let mut at = 0;
    while at < found.len() {
        if let Some(children) = app.world().get::<Children>(found[at]) {
            found.extend(children.into_iter().copied());
        }
        at += 1;
    }
    found
}

/// Runs `check` on the nodes of every converted page, and of every tab of the settings.
pub(super) fn each_page(tag: &str, check: impl Fn(&World, &[Entity], Menu)) {
    let mut app = app(tag);
    for menu in PAGES {
        let found = nodes(&mut app, menu);
        assert!(found.len() > 20, "{menu:?} has {} nodes", found.len());
        check(app.world(), &found, menu);
    }
    for tab in Tab::ALL {
        app.world_mut().resource_mut::<SettingsTab>().0 = tab;
        app.update();
        let found = nodes(&mut app, Menu::Settings);
        check(app.world(), &found, Menu::Settings);
    }
}

/// Whether a padding, a margin or a gap is nothing or a step of the spacing scale.
fn spaced(value: Val) -> bool {
    match value {
        Val::Auto => true,
        Val::Px(pixels) => SCALE.contains(&pixels),
        _ => false,
    }
}

/// Whether a position or a size is left to the layout, a share of its parent, the whole window,
/// or a whole number of 4 pixel steps.
fn on_grid(value: Val) -> bool {
    match value {
        Val::Auto | Val::Percent(_) => true,
        Val::Vw(share) | Val::Vh(share) => share == 100.0,
        Val::Px(pixels) => pixels % 4.0 == 0.0,
        _ => false,
    }
}

/// Every padding, margin and gap is on the spacing scale, every position and size on the 4
/// pixel grid, and no node has a border, a rounded corner, an outline or a shadow: a page keeps
/// its panel, and the HUD shadow is for the in-game HUD alone.
#[test]
fn every_node_is_on_the_scale_and_plain() {
    each_page("scale", |world, found, menu| {
        for entity in found {
            let Some(node) = world.get::<Node>(*entity) else {
                continue;
            };
            let sides = |rect: UiRect| [rect.left, rect.right, rect.top, rect.bottom];
            let spaces = [sides(node.padding), sides(node.margin)].concat();
            assert!(spaces.into_iter().all(spaced), "{menu:?} {node:?}");
            assert!(spaced(node.row_gap) && spaced(node.column_gap), "{menu:?}");
            let moving = world.get::<Thumb>(*entity).is_some();
            let places = [node.left, node.right, node.top, node.bottom];
            let sizes = [node.width, node.height, node.min_width, node.min_height];
            assert!(
                moving || places.into_iter().all(on_grid),
                "{menu:?} {node:?}"
            );
            assert!(
                moving || sizes.into_iter().all(on_grid),
                "{menu:?} {node:?}"
            );
            assert_eq!(node.border, UiRect::DEFAULT, "{menu:?}");
            assert_eq!(node.border_radius, BorderRadius::ZERO, "{menu:?}");
            assert!(world.get::<Outline>(*entity).is_none(), "{menu:?}");
            assert!(world.get::<BoxShadow>(*entity).is_none(), "{menu:?}");
            assert!(world.get::<TextShadow>(*entity).is_none(), "{menu:?}");
        }
    });
}

/// Every text is on the type scale, a page shows at most three sizes, and an icon is drawn on
/// its 24 pixel grid.
#[test]
fn every_text_is_on_the_type_scale() {
    each_page("type", |world, found, menu| {
        let mut sizes: Vec<f32> = Vec::new();
        for entity in found {
            let Some(font) = world.get::<TextFont>(*entity) else {
                continue;
            };
            let FontSize::Px(size) = font.font_size else {
                panic!("{menu:?}: a text is not sized in pixels");
            };
            if font.font == FontSource::Handle(ICON_FONT) {
                assert_eq!(size, theme::ICON, "{menu:?}");
                continue;
            }
            assert!(TYPE.contains(&size), "{menu:?}: {size}");
            if !sizes.contains(&size) {
                sizes.push(size);
            }
        }
        assert!(sizes.len() <= 3, "{menu:?} shows {sizes:?}");
    });
}
