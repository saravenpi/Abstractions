//! The real lights of a ship: one at the tail that the thrust of the main engines lights up, one
//! under the belly for the lift nozzles, which lights the ground on a landing, and the landing
//! lamp in the nose, which is on while the gear is down. A light is a light: it is hidden, and
//! costs nothing, while its thrust is out.

use bevy::light::SpotLight;
use bevy::prelude::*;

use super::drive::look;
use super::jets::Kind;
use super::state::Exhaust;
use super::tune::{LANDING_LIGHT, LIFT_LIGHT, MAIN_LIGHT, OFF};
use crate::ship::legs::Leg;
use crate::ship::mesh::NOZZLE_Z;

/// A light of a ship, a child of the hull.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lamp {
    /// The glow of the main engines.
    Main,
    /// The glow of the lift nozzles.
    Lift,
    /// The landing lamp.
    Landing,
}

/// Adds the three lights of a ship to `entity`, out.
pub fn fit(entity: &mut EntityCommands) {
    let glow = |(intensity, range): (f32, f32), color: Color| PointLight {
        intensity,
        range,
        color,
        radius: 0.1,
        shadow_maps_enabled: false,
        ..default()
    };
    let main = glow((0.0, MAIN_LIGHT.1), Color::srgb(0.6, 0.75, 1.0));
    let lift = glow((0.0, LIFT_LIGHT.1), Color::srgb(0.8, 0.85, 1.0));
    let beam = SpotLight {
        intensity: 0.0,
        range: LANDING_LIGHT.1,
        outer_angle: LANDING_LIGHT.2,
        inner_angle: LANDING_LIGHT.2 * 0.5,
        shadow_maps_enabled: false,
        ..default()
    };
    let nose =
        Transform::from_xyz(0.0, -0.34, -3.45).looking_to(Vec3::new(0.0, -0.3, -1.0), Vec3::Y);
    let tail = Transform::from_xyz(0.0, 0.2, NOZZLE_Z + 0.6);
    let belly = Transform::from_xyz(0.0, -0.95, 0.2);
    entity.with_child((Lamp::Main, main, tail, Visibility::Hidden));
    entity.with_child((Lamp::Lift, lift, belly, Visibility::Hidden));
    entity.with_child((Lamp::Landing, beam, nose, Visibility::Hidden));
}

/// Returns the share of full power the lamp `lamp` of a ship with this exhaust burns at: the
/// brightness of the plume of its jets, and for the landing lamp whether the gear is `down`.
fn power(lamp: Lamp, exhaust: &Exhaust, down: bool) -> f32 {
    match lamp {
        Lamp::Main => look(Kind::Main, exhaust).bright,
        Lamp::Lift => look(Kind::Lift, exhaust).bright,
        Lamp::Landing => f32::from(u8::from(down)),
    }
}

/// The lights of every ship, with the hull each hangs from.
type Lamps<'w, 's> = Query<
    'w,
    's,
    (
        &'static ChildOf,
        &'static Lamp,
        &'static mut Visibility,
        Option<&'static mut PointLight>,
        Option<&'static mut SpotLight>,
    ),
>;

/// Sets the intensity and the visibility of the lights of every ship from its exhaust and gear.
pub fn glare(
    ships: Query<&Exhaust>,
    legs: Query<(&ChildOf, &Transform), (With<Leg>, Without<Lamp>)>,
    mut lamps: Lamps,
) {
    for (parent, lamp, mut visibility, point, spot) in &mut lamps {
        let Ok(exhaust) = ships.get(parent.parent()) else {
            continue;
        };
        let down = legs
            .iter()
            .any(|(leg, at)| leg.parent() == parent.parent() && at.scale.y >= 0.999);
        let share = power(*lamp, exhaust, down);
        let lumens = match lamp {
            Lamp::Main => MAIN_LIGHT.0,
            Lamp::Lift => LIFT_LIGHT.0,
            Lamp::Landing => LANDING_LIGHT.0,
        } * share;
        visibility.set_if_neq(if share > OFF {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if let Some(mut light) = point.filter(|light| (light.intensity - lumens).abs() > 1.0) {
            light.intensity = lumens;
        }
        if let Some(mut light) = spot.filter(|light| (light.intensity - lumens).abs() > 1.0) {
            light.intensity = lumens;
        }
    }
}
