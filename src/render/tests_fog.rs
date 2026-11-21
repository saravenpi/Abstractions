//! The fog is the air alone: it shows tens of kilometres on foot whatever the render distance,
//! opens with height, closes in a cloud, and culls the view where it has hidden everything.

use bevy::pbr::FogFalloff;
use bevy::prelude::*;

use super::aerial::{CONTRAST, Haze};
use super::fog::{FogKey, cull_distance, extinction, fog_for};
use crate::world::globe::far::SIGHT;

/// Radius of the Earth in metres.
const EARTH: f32 = 6.371e6;

/// On foot in clear home air the camera sees between 60 and 110 km, far past any detailed ground,
/// and climbing never shortens the view.
#[test]
fn the_air_shows_tens_of_kilometres_on_foot() {
    let seen =
        |altitude: f32| cull_distance(extinction(&Haze::HOME, (altitude, altitude), 0.0), EARTH);
    assert!((60_000.0..110_000.0).contains(&seen(2.0)), "{}", seen(2.0));
    let sights: Vec<f32> = (0..=400).map(|k| seen(k as f32 * 100.0)).collect();
    assert!(sights.windows(2).all(|pair| pair[1] >= pair[0]));
    assert!(sights[400] > 3.0 * sights[0]);
}

/// A camera standing on a plateau sees through the thin air of the plateau, thinner than the air
/// a camera flying as high looks down through, which is thinner than the air at the sea.
#[test]
fn the_fog_is_the_air_between_the_camera_and_the_ground() {
    let sea = extinction(&Haze::HOME, (2.0, 2.0), 0.0);
    let plateau = extinction(&Haze::HOME, (3000.0, 2.0), 0.0);
    let flying = extinction(&Haze::HOME, (3000.0, 3000.0), 0.0);
    assert!(plateau.cmplt(flying).all() && flying.cmplt(sea).all());
    assert!(sea.z > sea.y && sea.y > sea.x);
}

/// In the middle of a cloud the camera sees 70 m, the view culls at Bevy's default and no
/// closer, and the fog takes each colour at its own rate and gives back as much of its own.
#[test]
fn a_cloud_whites_the_view_out() {
    let inside = extinction(&Haze::HOME, (2000.0, 2000.0), 1.0);
    assert!(inside.min_element() >= CONTRAST / 70.0);
    assert_eq!(cull_distance(inside, EARTH), 1000.0);
    let fog = fog_for(FogKey::home(Vec3::new(0.7, 0.8, 1.0)));
    let FogFalloff::Atmospheric {
        extinction: taken,
        inscattering: given,
    } = fog.falloff
    else {
        panic!("the fog of the camera is not atmospheric");
    };
    assert_eq!(taken, given);
    assert_eq!(taken, Haze::HOME.toward(0.0, 0.0));
}

/// Without air nothing is hidden and the view culls at 100 planet radii, where nothing of the
/// world is drawn any more; with air it culls far closer.
#[test]
fn the_view_is_counted_in_radii() {
    let most = SIGHT as f32 * EARTH;
    assert_eq!(cull_distance(Vec3::ZERO, EARTH), most);
    assert!(cull_distance(extinction(&Haze::HOME, (3.0e6, 3.0e6), 0.0), EARTH) < most);
}
