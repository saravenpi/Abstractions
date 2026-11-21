//! Fit tests: every text of the converted pages fits its box, measured with the interface font,
//! and a page is the size of the window and fits the narrowest layout any window gives it.

use bevy::prelude::*;

use super::rig::{content, need, overflows};
use super::rules::each_page;
use super::stage::{lines, narrowest};
use crate::ui::{HINT_GAP, Legend, theme};

/// Most lines the key legend of a page may take beside its back action.
const LEGEND_LINES: usize = 2;

/// Every node of a fixed width holds what it contains: the name and the value of every row, the
/// key of every line of the reference, every tab, every section title, every word of the line
/// under the rows, on every tab and on every page.
#[test]
fn every_text_fits_its_box() {
    each_page("fit", |world, found, menu| {
        let over = overflows(world, found);
        assert!(over.is_empty(), "{menu:?}: {over:#?}");
    });
}

/// A page is as large as the window in window units, so Bevy cannot grow it past the window with
/// its content, and what it needs across fits the narrowest layout of every window at every
/// interface size.
#[test]
fn every_page_is_the_window_and_fits_it() {
    let room = narrowest();
    assert!(room >= crate::ui::MIN_WIDTH, "{room}");
    each_page("window", |world, found, menu| {
        let root = world.get::<Node>(found[0]).expect("a root node");
        assert_eq!((root.width, root.height), (vw(100), vh(100)), "{menu:?}");
        let needed = content(world, found[0]);
        assert!(needed <= room, "{menu:?} needs {needed} of {room}");
    });
}

/// The key legend of a page takes two lines at most beside its back action in the narrowest
/// layout, and wraps rather than running past the window.
#[test]
fn the_legend_takes_two_lines_at_most() {
    let room = narrowest() - 2.0 * theme::EDGE;
    each_page("legend", |world, found, menu| {
        for legend in found
            .iter()
            .filter(|entity| world.get::<Legend>(**entity).is_some())
        {
            let foot = world.get::<ChildOf>(*legend).expect("a footer").parent();
            let first = world
                .get::<Children>(foot)
                .and_then(|children| children.first().copied());
            let back = first.expect("a footer starts with its back action");
            let left = room - need(world, back) - theme::GUTTER;
            let hints: Vec<f32> = world
                .get::<Children>(*legend)
                .map(|children| {
                    children
                        .into_iter()
                        .map(|hint| need(world, *hint))
                        .collect()
                })
                .unwrap_or_default();
            let taken = lines(&hints, HINT_GAP, left);
            assert!(taken <= LEGEND_LINES, "{menu:?}: {taken} lines in {left}");
        }
    });
}
