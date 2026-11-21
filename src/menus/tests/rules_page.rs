//! The rules of the page skeleton, of sections and of icons, held on every converted page: the
//! screen edge, the three zones at the same place on every page, more space above a section
//! title than below it, and no icon without its name in words.

use bevy::prelude::*;
use bevy::text::FontSource;

use super::rules::each_page;
use crate::ui::{ICON_FONT, Section, SectionTitle, Sections, Tip, Zone, theme};

/// Returns the node of `entity`.
fn node(world: &World, entity: Entity) -> &Node {
    world.get::<Node>(entity).expect("a node")
}

/// Returns the children of `entity`, or none.
fn kids(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .get::<Children>(entity)
        .map(|children| children.into_iter().copied().collect())
        .unwrap_or_default()
}

/// The root of a page pads by the screen edge on every side, and holds its three zones in
/// order: the heading zone of one heading line with one gap under it, the content zone that
/// takes the room, and the footer zone one gap under it. So the heading and the footer of every
/// page sit at the same place.
#[test]
fn every_page_has_the_same_skeleton() {
    each_page("skeleton", |world, found, menu| {
        let root = found[0];
        assert_eq!(
            node(world, root).padding,
            UiRect::all(px(theme::EDGE)),
            "{menu:?}"
        );
        let zones: Vec<Zone> = kids(world, root)
            .into_iter()
            .filter_map(|child| world.get::<Zone>(child).copied())
            .collect();
        assert_eq!(zones, [Zone::Head, Zone::Body, Zone::Foot], "{menu:?}");
        let parts = kids(world, root);
        let &[head, body, foot] = parts.as_slice() else {
            panic!("{menu:?} has other children than its zones");
        };
        let head = node(world, head);
        assert_eq!(head.height, px(theme::LINE_HEADING), "{menu:?}");
        assert_eq!(head.margin, UiRect::bottom(px(theme::S5)), "{menu:?}");
        assert_eq!(head.padding, UiRect::horizontal(px(theme::INSET)));
        assert_eq!(node(world, body).flex_grow, 1.0, "{menu:?}");
        let foot = node(world, foot);
        assert_eq!(foot.margin, UiRect::top(px(theme::S4)), "{menu:?}");
        assert_eq!(foot.min_height, px(theme::ROW_HEIGHT), "{menu:?}");
    });
}

/// Every section starts with its title, the gap under the title is the one step every screen
/// uses, and the column that holds the section sets a larger step above it, from the two the
/// system allows.
#[test]
fn a_section_title_is_nearer_to_its_content() {
    let seen = std::cell::Cell::new(0);
    each_page("sections", |world, found, menu| {
        for entity in found {
            if world.get::<Section>(*entity).is_none() {
                continue;
            }
            let below = node(world, *entity).row_gap;
            assert_eq!(below, px(theme::TITLE_BELOW), "{menu:?}");
            let first = kids(world, *entity)[0];
            assert!(world.get::<SectionTitle>(first).is_some(), "{menu:?}");
            let holder = world.get::<ChildOf>(*entity).expect("a holder").parent();
            assert!(world.get::<Sections>(holder).is_some(), "{menu:?}");
            let above = node(world, holder).row_gap;
            let allowed = [px(theme::SECTION_ABOVE), px(theme::GROUP_ABOVE)];
            assert!(allowed.contains(&above), "{menu:?}: {above:?}");
            seen.set(seen.get() + 1);
        }
    });
    assert!(
        seen.get() >= 12,
        "only {} sections were checked",
        seen.get()
    );
    const { assert!(theme::SECTION_ABOVE > theme::TITLE_BELOW) };
    const { assert!(theme::GROUP_ABOVE > theme::TITLE_BELOW) };
}

/// Whether `entity` is a text drawn in the icon font.
fn is_icon(world: &World, entity: Entity) -> bool {
    world
        .get::<TextFont>(entity)
        .is_some_and(|font| font.font == FontSource::Handle(ICON_FONT))
}

/// Whether `entity` or anything under it is a text in words.
fn has_words(world: &World, entity: Entity) -> bool {
    let worded = world.get::<Text>(entity).is_some() && !is_icon(world, entity);
    worded
        || kids(world, entity)
            .into_iter()
            .any(|child| has_words(world, child))
}

/// Whether the icon `entity` is named: one of the four nodes above it carries a tooltip or
/// holds a text in words.
fn named(world: &World, entity: Entity) -> bool {
    let mut at = entity;
    for _ in 0..4 {
        let Some(parent) = world.get::<ChildOf>(at) else {
            return false;
        };
        at = parent.parent();
        if world.get::<Tip>(at).is_some() || has_words(world, at) {
            return true;
        }
    }
    false
}

/// Every icon has its name in words beside it, in the same control or the same row, or its
/// control carries a tooltip.
#[test]
fn every_icon_is_named_in_words() {
    each_page("named", |world, found, menu| {
        let icons = found.iter().filter(|entity| is_icon(world, **entity));
        for entity in icons {
            assert!(named(world, *entity), "{menu:?}: an icon has no name");
        }
    });
}
