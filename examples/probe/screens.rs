//! Stages that open the screens of the interface: the menus on the title screen, the pause menu, the
//! map at each level, the field guide and the pack, one shot each.

use abstractions::computer::{Board, Computer, Tab};
use abstractions::map::Places;
use abstractions::menus::{Menu, SettingsPage, SettingsTab};

use super::*;

/// Presses and releases `code` over two frames, the first being `frame`.
fn tap(world: &mut World, code: KeyCode, frame: u32, at: u32) {
    if frame == at {
        key(world, code, true);
    }
    if frame == at + 1 {
        key(world, code, false);
    }
}

/// Opens the menu `menu`.
fn open(world: &mut World, menu: Menu) {
    world.resource_mut::<NextState<Menu>>().set(menu);
}

/// The menus of the title screen: the worlds, a new world, the servers and each page of the settings.
pub fn menus(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let pages = [
        SettingsPage::Graphics,
        SettingsPage::Controls,
        SettingsPage::Ship,
        SettingsPage::Audio,
        SettingsPage::General,
        SettingsPage::Help,
    ];
    let named = [
        (Menu::Worlds, "a0-worlds"),
        (Menu::NewGame, "a1-new-world"),
        (Menu::Multiplayer, "a2-multiplayer"),
    ];
    let total = named.len() + pages.len();
    let index = (frame / 25) as usize;
    match (frame % 25, index) {
        (1, i) if i < named.len() => open(world, named[i].0),
        (20, i) if i < named.len() => shot(world, named[i].1),
        (1, i) if i < total => {
            open(world, Menu::Settings);
            world.resource_mut::<SettingsTab>().0 = pages[i - named.len()];
        }
        (20, i) if i < total => {
            let page = pages[i - named.len()];
            shot(world, &format!("a{i}-settings-{page:?}"));
        }
        (1, i) if i == total => open(world, Menu::Main),
        (_, i) if i > total => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// The pause menu over the game. It runs last: the menu stays open.
pub fn pause(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    tap(world, KeyCode::Escape, frame, 1);
    match frame {
        40 => Act::Shoot("b0-pause".into()),
        _ => Act::Wait,
    }
}

/// Opens or closes the map screen.
fn map(world: &mut World, shown: bool) {
    let wanted = [Computer::Closed, Computer::Open][usize::from(shown)];
    world.resource_mut::<NextState<Computer>>().set(wanted);
}

/// Returns the pack and guide keys of the settings, for the stages that press them.
fn keys(world: &World) -> (KeyCode, KeyCode) {
    (
        bound(world, |s| s.controls.keys.guide),
        bound(world, |s| s.controls.keys.pack),
    )
}

/// The map at the level of the ship, then of the system, the galaxy and the universe.
pub fn map_levels(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let levels = [
        (Tab::System, "c1-map-system"),
        (Tab::Galaxy, "c2-map-galaxy"),
        (Tab::Universe, "c3-map-universe"),
    ];
    let at = frame.saturating_sub(21) / 40;
    match (frame, frame.saturating_sub(21) % 40) {
        (1, _) => map(world, true),
        (20, _) => shot(world, "c0-map-open"),
        (f, 0) if f > 20 && at < 3 => world.resource_mut::<Board>().show(levels[at as usize].0),
        (f, 30) if f > 20 && at < 3 => shot(world, levels[at as usize].1),
        (150, _) => map(world, false),
        (170, _) => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// The map with a planet selected: the card with its Go button and the facts.
pub fn map_card(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => map(world, true),
        20 | 24 => key(world, KeyCode::ArrowDown, true),
        21 | 25 => key(world, KeyCode::ArrowDown, false),
        60 => shot(world, "c4-map-card"),
        61 => world.resource_mut::<Board>().step(3),
        100 => shot(world, "c5-map-card-last"),
        101 => map(world, false),
        120 => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// The map with a pin dropped and selected.
pub fn map_pin(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            let body = world.resource::<Location>().body;
            let at = world.resource::<Base>().wake;
            let mut places = world.resource_mut::<Places>();
            places.drop(body, at.normalize());
            places.selected = Some(0);
            map(world, true);
        }
        50 => shot(world, "c6-map-pin"),
        51 => map(world, false),
        70 => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// The field guide, then the pack.
pub fn books(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let (guide, pack) = keys(world);
    match frame {
        1 => key(world, guide, true),
        2 => key(world, guide, false),
        40 => shot(world, "d0-guide"),
        41 => key(world, guide, true),
        42 => key(world, guide, false),
        60 => key(world, pack, true),
        61 => key(world, pack, false),
        100 => shot(world, "d1-pack"),
        101 => key(world, pack, true),
        102 => key(world, pack, false),
        120 => return Act::Next,
        _ => {}
    }
    Act::Wait
}
