//! A save written by 0.8.0 on an outer planet of the home system that is now a gas giant: the
//! world begins at home as a new game does, with a fresh ship on the pad of the base.

use bevy::math::{DQuat, DVec3};

use super::ground::starts_at_the_base;
use crate::saves::{Place, SaveDir, WorldEntry, WorldSave, new_entry};
use crate::ship::Ship;
use crate::ship::tests::rig::Game;
use crate::space::{Frame, Location, Travel};
use crate::universe::{BodyId, PlanetClass, Universe};

/// Returns the first gas giant of the home system of seed 7.
fn home_giant() -> BodyId {
    let universe = Universe::new(7);
    let system = universe.system(universe.home().star);
    let giant = system
        .planets
        .iter()
        .find(|p| p.record.class == PlanetClass::GasGiant);
    giant.expect("the home system of seed 7 has a gas giant").id
}

/// Returns a world file as 0.8.0 wrote it, standing on `body` with the ship 30 km under its
/// sea level radius: no `moon` key anywhere.
fn old_save(dir: &std::path::Path, body: BodyId, radius: f64) -> WorldEntry {
    let ship = Ship::at_rest(
        Frame::Surface(body),
        DVec3::Y * (radius - 3.0e4),
        DQuat::IDENTITY,
    );
    let mut entry = new_entry(dir, "Stranded", 7);
    entry.save.clock_seconds = 5.0e5;
    entry.save.place = Some(Place {
        body,
        dir: [0.0, 1.0, 0.0],
        altitude: 2.0,
    });
    entry.save.ship = Some(toml::Value::try_from(ship.state()).unwrap());
    let text = toml::to_string(&entry.save).unwrap();
    let old: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("moon ="))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(text.contains("moon = 0") && !old.contains("moon"));
    let save: WorldSave = toml::from_str(&old).unwrap();
    WorldEntry { save, ..entry }
}

/// Loading a 0.8.0 save that stands on what is now a gas giant puts the player on foot at the
/// home base, beside a fresh ship landed on its pad, not one left under the deck of the giant.
#[test]
fn a_save_on_a_new_giant_starts_at_the_base() {
    let mut game = Game::full("stranded");
    let giant = home_giant();
    let radius = Universe::new(7).planet(giant).radius;
    let dir = game.app.world().resource::<SaveDir>().0.clone();
    game.start("", Some(old_save(&dir, giant, radius)));
    let home = game.get::<Universe>().home();
    let location = game.get::<Location>();
    assert_eq!((location.body, location.travel), (home, Travel::OnFoot));
    assert_eq!(game.ship().frame, Frame::Surface(home));
    starts_at_the_base(&mut game);
}
