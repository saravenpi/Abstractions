//! The start of the voyage: awake at the home base on Kodya, and into the seat of the ship.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::{here, key};
use crate::audio::AudioScene;
use crate::avatar::Shown;
use crate::home::Base;
use crate::player::place;
use crate::ship::landing::GEAR_DROP;
use crate::ship::tests::rig::{Game, Log};
use crate::ship::{Hatch, ShipEvent};
use crate::space::{Anchor, Location, Placed, Travel};
use crate::universe::{HOME_PLANET_NAME, Universe};
use crate::view::{ViewMode, ViewPrefs};

/// A new game wakes the player at the home base and parks the ship on its pad, nose along the
/// pad heading.
pub fn starts_at_the_base(game: &mut Game) {
    let (base, ship, walker) = (game.get::<Base>(), game.ship(), game.walker());
    let up = base.pad.normalize();
    assert_eq!(base.body, game.get::<Universe>().home());
    assert!(ship.landed && ship.pos.distance(base.pad + up * GEAR_DROP) < 1e-6);
    assert!(ship.forward().dot(base.heading) > 0.9999 && (ship.rot * DVec3::Y).dot(up) > 0.9999);
    let from_wake = walker.at - base.wake;
    let aside = (from_wake - up * from_wake.dot(up)).length();
    assert!(aside < 0.5, "the player woke {aside} m from the wake point");
}

/// A new game puts the player on foot on the ground of the home planet, Kodya.
pub fn stands_on_kodya(game: &mut Game) {
    let (body, planet) = here(game);
    assert_eq!(body, game.get::<Universe>().home());
    assert_eq!(planet.name, HOME_PLANET_NAME);
    assert_eq!(game.get::<Location>().travel, Travel::OnFoot);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    let walker = game.walker();
    assert_eq!(walker.body, Some(body));
    let on_ground = walker.gap > -0.3 && walker.gap < 1.5;
    assert!(on_ground, "feet {} m off the ground", walker.gap);
    assert!(walker.at.length() > planet.radius, "under the sea");
    assert!(!walker.seated);
}

/// Standing at the hatch of the landed ship, the board key seats the player.
pub fn boards(game: &mut Game) {
    let anchor = game.get::<Anchor>();
    let world = game.app.world_mut();
    let mut ships = world.query_filtered::<&Placed, With<crate::ship::Ship>>();
    let hull = *ships.single(world).expect("one ship");
    let beside = hull.pos + hull.rot * DVec3::new(-2.6, -1.2, -1.8);
    let (feet, toward) = (anchor.to_render(beside), anchor.to_render(hull.pos));
    place(world, feet, toward - feet);
    game.step(2);
    assert!(game.get::<Hatch>().near, "the hatch is out of reach");
    game.press(key(game, |keys| keys.ship.board));
    game.step(1);
    game.release(key(game, |keys| keys.ship.board));
    game.step(2);
    assert_eq!(game.get::<Location>().travel, Travel::InShip);
    assert!(game.walker().seated && !game.get::<Hatch>().near);
    assert!(game.get::<Log>().ship.contains(&ShipEvent::Boarded));
    assert_eq!(game.get::<AudioScene>().cabin, 1.0);
}

/// In the seat, the view key moves the camera behind the ship, which it shows whole, and the
/// camera stays above the ground the landed ship stands on.
pub fn looks_from_behind(game: &mut Game) {
    assert_eq!(game.get::<ViewPrefs>().ship, ViewMode::First);
    game.press(key(game, |keys| keys.ship.view));
    game.step(1);
    game.release(key(game, |keys| keys.ship.view));
    assert_eq!(game.get::<ViewPrefs>().ship, ViewMode::Third);
    assert_eq!(game.get::<ViewPrefs>().on_foot, ViewMode::First);
    for _ in 0..60 {
        game.step(1);
        let height = game.camera_height();
        assert!(height > 0.3, "the camera is {height} m over the ground");
    }
    assert!(game.hull_in_view(), "the landed ship is not all in view");
    assert_eq!(game.get::<Shown>(), Shown::Hidden);
}
