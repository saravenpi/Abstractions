//! The light of the cockpit: a glow of the panel that is on while the pilot sits, a cabin
//! light that is on while the ship is landed and the hatch is in use, and the warm light of
//! an entry that comes through the glass. The lit faces of the instruments are dimmed by day
//! so they do not wash out. No cockpit light casts a shadow, and only the ship the player
//! flies has lights: the others show emissive faces alone.

use bevy::prelude::*;

use super::fit::Lit;
use super::state::Cockpit;
use super::tune::{DAY_DIM, LUMENS, SCREEN_GLOW};
use crate::render::{DayClock, Heat, daylight, reach};
use crate::ship::board::Hatch;
use crate::ship::body::Ship;

/// A light of the cockpit.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beam {
    /// The glow of the panel, cool and close, on the pilot and the dashboard.
    Panel,
    /// The cabin light, warm, which also takes the colour of an entry.
    Cabin,
}

/// What the lights answer to.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Mood {
    /// Daylight from 0 at night to 1 at noon.
    pub day: f32,
    /// Whether a pilot sits in the seat.
    pub seated: bool,
    /// Whether the ship rests on the ground.
    pub landed: bool,
    /// Whether the player stands at the hatch.
    pub hatch: bool,
    /// Heat of the entry from 0 to 1, and the linear colour of its glow.
    pub heat: (f32, Vec3),
}

/// Returns the colour and the power in lumens of `beam` for `mood`.
pub fn beam(beam: Beam, mood: &Mood) -> (Color, f32) {
    match beam {
        Beam::Panel => {
            let on = f32::from(u8::from(mood.seated));
            let power = LUMENS.0 * on * (1.0 - DAY_DIM.0 * mood.day);
            (Color::linear_rgb(0.55, 0.8, 1.0), power)
        }
        Beam::Cabin => {
            let landed = mood.landed && (mood.hatch || mood.seated);
            let base = LUMENS.1 * f32::from(u8::from(landed)) * (1.0 - DAY_DIM.1 * mood.day);
            let entry = LUMENS.2 * mood.heat.0.clamp(0.0, 1.0) * f32::from(u8::from(mood.seated));
            let warm = Vec3::new(1.0, 0.93, 0.8).lerp(mood.heat.1, mood.heat.0.clamp(0.0, 1.0));
            (Color::linear_rgb(warm.x, warm.y, warm.z), base + entry)
        }
    }
}

/// Returns how bright the lit faces of the instruments are at `day`, as a share of white.
pub fn glow(day: f32) -> f32 {
    SCREEN_GLOW.1 + (SCREEN_GLOW.0 - SCREEN_GLOW.1) * day.clamp(0.0, 1.0)
}

/// Adds the lights to the ship the player flies when its cockpit is fitted: the glow of the
/// panel above the dashboard and the cabin light over the seat.
pub fn rig(mut commands: Commands, ships: Query<Entity, (Added<Cockpit>, With<Ship>)>) {
    for ship in &ships {
        for (kind, at, range) in [
            (Beam::Panel, Vec3::new(0.0, 0.62, -1.75), reach(LUMENS.0)),
            (
                Beam::Cabin,
                Vec3::new(0.0, 0.85, -1.2),
                reach(LUMENS.1 + LUMENS.2),
            ),
        ] {
            let light = PointLight {
                intensity: 0.0,
                range,
                radius: 0.05,
                shadow_maps_enabled: false,
                ..default()
            };
            commands.spawn((kind, light, Transform::from_translation(at), ChildOf(ship)));
        }
    }
}

/// What the lights read besides the cockpit.
type Sky<'w> = (
    Option<Res<'w, DayClock>>,
    Option<Res<'w, Heat>>,
    Option<Res<'w, Hatch>>,
);

/// Sets the colour and the power of the lights from the state of the ship, the time of day, the
/// heat of the entry and the hatch. A light that did not change is not written.
pub fn shine(
    (clock, heat, hatch): Sky,
    ships: Query<(&Ship, &Cockpit)>,
    mut lights: Query<(&Beam, &ChildOf, &mut PointLight)>,
) {
    let day = clock.map_or(1.0, |clock| daylight(clock.phase));
    let entry = heat.map_or((0.0, Vec3::ZERO), |heat| (heat.glow.heat, heat.glow.color));
    for (kind, parent, mut light) in &mut lights {
        let Ok((ship, cockpit)) = ships.get(parent.parent()) else {
            continue;
        };
        let mood = Mood {
            day,
            seated: cockpit.pilot.is_some(),
            landed: ship.landed,
            hatch: hatch.as_ref().is_some_and(|hatch| hatch.near),
            heat: entry,
        };
        let (color, power) = beam(*kind, &mood);
        if light.intensity != power {
            light.intensity = power;
        }
        if light.color != color {
            light.color = color;
        }
    }
}

/// Dims the lit faces of every instrument to the brightness of the time of day, and writes a
/// material only when its colour changed.
pub fn dim(
    clock: Option<Res<DayClock>>,
    lit: Query<(&Lit, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let level = glow(clock.map_or(1.0, |clock| daylight(clock.phase)));
    for (lit, handle) in &lit {
        let tone = lit.0 * level;
        let wanted = Color::linear_rgb(tone.x, tone.y, tone.z);
        if materials
            .get(&handle.0)
            .is_some_and(|m| m.base_color != wanted)
            && let Some(mut material) = materials.get_mut(&handle.0)
        {
            material.base_color = wanted;
        }
    }
}

/// Run condition: the cockpit of the ship the player flies was just fitted.
pub fn fitting(cockpits: Query<(), (Added<Cockpit>, With<Ship>)>) -> bool {
    !cockpits.is_empty()
}
