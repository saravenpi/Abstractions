//! Lamp tests: the navigation lamps sit at the tips of the winglets, where their real lights
//! are, mirrored left and right, and the headlight of any ship, flown here or only seen, burns
//! when the lamp of its cockpit says it is on.

use bevy::light::SpotLight;
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;

use super::lights::{Lamp, fit, glare, place, power};
use super::state::Exhaust;
use super::tune::HEADLIGHT;
use crate::ship::cockpit::{Cockpit, Lamp as Signal};
use crate::ship::mesh::lamp_meshes;

/// Returns the middle of the box around the vertices of `mesh`.
fn middle(mesh: &Mesh) -> Vec3 {
    let Some(VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return Vec3::NAN;
    };
    let points = points.iter().map(|p| Vec3::from(*p));
    let grow = |(low, high): (Vec3, Vec3), p: Vec3| (low.min(p), high.max(p));
    let (low, high) = points.fold((Vec3::MAX, Vec3::MIN), grow);
    (low + high) * 0.5
}

/// The red lamp is at the top rear tip of the left winglet and the green one at the tip of the
/// right winglet, each within a centimetre of its real light, and the two mirror each other.
#[test]
fn the_navigation_lamps_sit_at_the_winglet_tips() {
    let [red, green, _] = lamp_meshes();
    let (left, right) = (place(Lamp::Tip(0)), place(Lamp::Tip(1)));
    assert!(middle(&red).distance(left) < 0.01, "{}", middle(&red));
    assert!(middle(&green).distance(right) < 0.01, "{}", middle(&green));
    assert!(left.distance(Vec3::new(-right.x, right.y, right.z)) < 1.0e-6);
    assert!(left.x < -3.1 && (left.y - 0.5).abs() < 0.1 && (left.z - 2.75).abs() < 0.15);
}

/// A navigation lamp always burns, and the headlight burns only when it is switched on.
#[test]
fn the_headlight_burns_when_it_is_switched_on() {
    let exhaust = Exhaust::default();
    assert_eq!(power(Lamp::Head, &exhaust, false), 0.0);
    assert_eq!(power(Lamp::Head, &exhaust, true), 1.0);
    assert_eq!(power(Lamp::Tip(0), &exhaust, false), 1.0);
}

/// Returns the intensity of the headlight of a hull that is only seen, with no flight state,
/// whose cockpit reads the headlight `on`, and whether the light shows.
fn seen(on: bool) -> (f32, Visibility) {
    let mut app = App::new();
    app.add_systems(Update, glare);
    let mut cockpit = Cockpit::default();
    cockpit.read.lamps[Signal::Light as usize] = on;
    let world = app.world_mut();
    let place = (Transform::default(), Visibility::default());
    fit(&mut world.commands().spawn((Exhaust::default(), cockpit, place)));
    world.flush();
    app.update();
    let mut lights = app.world_mut().query::<(&Lamp, &SpotLight, &Visibility)>();
    let (_, light, shown) = lights.single(app.world()).expect("one headlight");
    (light.intensity, *shown)
}

/// The ship of another player, drawn from its record, has a real headlight: its light is at
/// full power and shows when the record says the headlight is on, and is out and hidden when not.
#[test]
fn a_ship_that_is_only_seen_lights_its_headlight() {
    assert_eq!(seen(true), (HEADLIGHT.0, Visibility::Inherited));
    assert_eq!(seen(false), (0.0, Visibility::Hidden));
}
