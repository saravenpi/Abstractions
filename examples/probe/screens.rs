//! Stages that open the screens of the interface: the menus on the title screen, the pause menu, the
//! map at each level, the field guide and the pack, one shot each.

use abstractions::computer::{Board, Computer, MapView};
use abstractions::map::Places;
use abstractions::menus::{Menu, SettingsPage, SettingsTab};

use super::*;

/// Presses and releases `code` over two frames, the first being `frame`.
pub fn tap(world: &mut World, code: KeyCode, frame: u32, at: u32) {
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

/// The map as it opens on the ground, at the closest zoom on the player, then one level out for
/// each Backspace, the key that leaves a level: the system, the galaxy and the universe.
pub fn map_levels(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let levels = ["c1-map-system", "c2-map-galaxy", "c3-map-universe"];
    match frame {
        1 => map(world, true),
        90 => shot(world, "c0-map-open"),
        100..250 => {
            let at = (frame - 100) / 50;
            tap(world, KeyCode::Backspace, frame, 100 + at * 50);
            if (frame - 100) % 50 == 40 {
                shot(world, levels[at as usize]);
            }
        }
        250 => map(world, false),
        270 => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// Selects the first row of the list of the map that `want` takes, given the planet underfoot,
/// the way a key or a click on the row does: the cursor goes to it and it is picked. The
/// arrows cannot do it here, since the pointer of the probe rests on the map, where they pan.
pub fn choose_row(world: &mut World, shot: &str, want: impl Fn(Target, BodyId) -> bool) {
    let here = world.resource::<Location>().body;
    let mut board = world.resource_mut::<Board>();
    let Some(row) = board.lines.iter().position(|l| want(l.target, here)) else {
        eprintln!(
            "PROBE {shot}: no row of the list fits, {} rows",
            board.lines.len()
        );
        return;
    };
    let by = row as isize - board.cursor as isize;
    board.step(by);
    let title = board.lines[row].title.clone();
    world.resource_mut::<MapView>().picked = true;
    eprintln!("PROBE {shot}: selected the row of {title}");
}

/// The map with a planet selected: the card of another planet of the system with its Go button
/// and its facts, then the card of the planet underfoot.
pub fn map_card(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let other = |target: Target, here: BodyId| matches!(target, Target::Body(body) if body != here && body.moon == 0);
    match frame {
        1 => map(world, true),
        30 => choose_row(world, "c4-map-card", other),
        80 => shot(world, "c4-map-card"),
        81 => choose_row(world, "c5-map-card-last", |target, here| {
            target == Target::Body(here)
        }),
        130 => shot(world, "c5-map-card-last"),
        131 => map(world, false),
        150 => return Act::Next,
        _ => {}
    }
    Act::Wait
}

/// Drops a pin `metres` from the wake point of the base along its first axis, on the ground,
/// the way a right click on the map does, and returns its index.
pub fn pin_near_base(world: &mut World, metres: f64) -> Option<usize> {
    let base = world.get_resource::<Base>().copied()?;
    let at = ground(world, base.wake + base.east * metres, 0.0);
    world
        .resource_mut::<Places>()
        .drop(base.body, at.normalize())
}

/// The map with a pin dropped 400 m from the base and selected. The pin is selected once the
/// map is open: opening the map lets go of the selection.
pub fn map_pin(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    match frame {
        1 => {
            let dropped = pin_near_base(world, 400.0);
            eprintln!("PROBE c6-map-pin: dropped pin {dropped:?}");
            map(world, true);
        }
        15 => {
            let mut places = world.resource_mut::<Places>();
            places.selected = places.pins.len().checked_sub(1);
        }
        90 => shot(world, "c6-map-pin"),
        91 => map(world, false),
        110 => return Act::Next,
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
