//! The scene sensed in the game on planets with other air than the home planet.

use super::app::{app, enter_world};
use crate::audio::scene::AudioScene;
use crate::audio::sense::Here;
use crate::gear::FlightFx;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::wind::{Wind, samples};

/// Puts the player in fast flight in a gale and returns the scene sensed on a planet whose
/// record is replaced by `record`.
fn sensed(name: &str, record: PlanetRecord) -> AudioScene {
    let mut app = app(name);
    enter_world(&mut app);
    app.update();
    let mut here = app.world_mut().resource_mut::<Here>();
    here.0.as_mut().expect("a planet was sensed").1.record = record;
    app.world_mut().resource_mut::<Wind>().strength = 0.8;
    app.insert_resource(FlightFx {
        airspeed: 60.0,
        ..FlightFx::default()
    });
    app.update();
    *app.world().resource::<AudioScene>()
}

/// In the game the scene follows the record: the home record gives the wind and the rush as
/// they are, and a planet without air gives neither, whatever blows and however fast one falls.
#[test]
fn the_scene_follows_the_record_of_the_planet() {
    let home = sensed("air-home", PlanetRecord::TEMPERATE);
    assert_eq!((home.wind, home.class), (0.8, PlanetClass::Island));
    assert!(home.rush > 0.5);
    let bare = sensed("air-bare", samples()[5]);
    assert_eq!((bare.wind, bare.rush, bare.shore), (0.0, 0.0, 0.0));
    assert_eq!((bare.cloud_depth, bare.class), (0.0, PlanetClass::Barren));
    let thin = sensed("air-thin", samples()[2]);
    assert_eq!(thin.wind, 0.8);
    assert!((thin.rush - 0.4 * home.rush).abs() < 1e-6 && thin.shore == 0.0);
}
