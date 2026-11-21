//! The real lights of a ship: one at the tail that the thrust of the main engines lights up, one
//! under the belly for the lift nozzles, which lights the ground on a landing, the headlight in
//! the nose, which the pilot switches, and the two navigation lamps at the tips of the winglets,
//! red on the left and green on the right. A light is a light: it is hidden, and costs nothing,
//! while it is out. Every ship has them, and they burn from its own exhaust and cockpit, so the
//! ship of another player lights the ground as its owner sees it.

use bevy::light::SpotLight;
use bevy::prelude::*;

use super::drive::look;
use super::jets::Kind;
use super::state::Exhaust;
use super::tune::{HEADLIGHT, LIFT_LIGHT, MAIN_LIGHT, OFF, TIP_LIGHT};
use crate::render::reach;
use crate::settings::{Persistent, Settings};
use crate::ship::body::Ship;
use crate::ship::cockpit::{Cockpit, Lamp as Signal};
use crate::ship::mesh::{HEAD_LAMP, NOZZLE_Z, TIP_LAMPS};

/// A light of a ship, a child of the hull.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lamp {
    /// The glow of the main engines.
    Main,
    /// The glow of the lift nozzles.
    Lift,
    /// The headlight in the nose.
    Head,
    /// The navigation lamp at the tip of a winglet: 0 for the left, 1 for the right.
    Tip(usize),
}

/// Linear colours of the navigation lamps: red on the left, green on the right, as on aircraft.
pub const TIP_COLORS: [[f32; 3]; 2] = [[1.0, 0.05, 0.05], [0.05, 1.0, 0.12]];

/// Returns where the lamp `lamp` stands in the axes of the hull. A navigation lamp stands in the
/// middle of its housing; the headlight stands just under and ahead of its lens, outside the
/// hull, so that its own nose casts no shadow in its beam.
pub fn place(lamp: Lamp) -> Vec3 {
    match lamp {
        Lamp::Main => Vec3::new(0.0, 0.2, NOZZLE_Z + 0.6),
        Lamp::Lift => Vec3::new(0.0, -0.95, 0.2),
        Lamp::Head => HEAD_LAMP + Vec3::new(0.0, -0.07, -0.04),
        Lamp::Tip(side) => TIP_LAMPS[side.min(1)],
    }
}

/// Returns a point light of `lumens` at full power and of `color`, out.
fn glow(lumens: f32, color: Color) -> PointLight {
    PointLight {
        intensity: 0.0,
        range: reach(lumens),
        color,
        radius: 0.1,
        shadow_maps_enabled: false,
        ..default()
    }
}

/// Adds the five lights of a ship to `entity`, out.
pub fn fit(entity: &mut EntityCommands) {
    let beam = SpotLight {
        intensity: 0.0,
        range: reach(HEADLIGHT.0),
        outer_angle: HEADLIGHT.1,
        inner_angle: HEADLIGHT.1 * 0.5,
        radius: HEADLIGHT.2,
        shadow_maps_enabled: false,
        ..default()
    };
    let at = |lamp: Lamp| Transform::from_translation(place(lamp));
    let nose = at(Lamp::Head).looking_to(Vec3::new(0.0, -0.3, -1.0), Vec3::Y);
    let main = glow(MAIN_LIGHT, Color::srgb(0.6, 0.75, 1.0));
    let lift = glow(LIFT_LIGHT, Color::srgb(0.8, 0.85, 1.0));
    entity.with_child((Lamp::Main, main, at(Lamp::Main), Visibility::Hidden));
    entity.with_child((Lamp::Lift, lift, at(Lamp::Lift), Visibility::Hidden));
    entity.with_child((Lamp::Head, beam, nose, Visibility::Hidden));
    for (side, [r, g, b]) in TIP_COLORS.into_iter().enumerate() {
        let lamp = Lamp::Tip(side);
        let light = glow(TIP_LIGHT, Color::linear_rgb(r, g, b));
        entity.with_child((lamp, light, at(lamp), Visibility::Hidden));
    }
}

/// Returns the share of full power the lamp `lamp` of a ship burns at: the brightness of the
/// plume of its jets, for the headlight whether it is switched `on`, and all of it for a
/// navigation lamp, which is always on.
pub fn power(lamp: Lamp, exhaust: &Exhaust, on: bool) -> f32 {
    match lamp {
        Lamp::Main => look(Kind::Main, exhaust).bright,
        Lamp::Lift => look(Kind::Lift, exhaust).bright,
        Lamp::Head => f32::from(u8::from(on)),
        Lamp::Tip(_) => 1.0,
    }
}

/// Returns the light of the lamp `lamp` at full power, in lumens.
fn full(lamp: Lamp) -> f32 {
    match lamp {
        Lamp::Main => MAIN_LIGHT,
        Lamp::Lift => LIFT_LIGHT,
        Lamp::Head => HEADLIGHT.0,
        Lamp::Tip(_) => TIP_LIGHT,
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

/// Sets the intensity and the visibility of the lights of every ship from its exhaust and from
/// the headlight lamp its cockpit reads. The headlight of the ship the player flies casts
/// shadows when the lighting setting lets the lamp of the robot cast them, and no other does.
pub fn glare(
    settings: Option<Res<Persistent<Settings>>>,
    ships: Query<(&Exhaust, &Cockpit, Has<Ship>)>,
    mut lamps: Lamps,
) {
    let allowed = settings.is_some_and(|settings| settings.graphics.lighting.local_shadows);
    for (parent, lamp, mut visibility, point, spot) in &mut lamps {
        let Ok((exhaust, cockpit, own)) = ships.get(parent.parent()) else {
            continue;
        };
        let on = cockpit.read.lamps[Signal::Light as usize];
        let share = power(*lamp, exhaust, on);
        let lumens = full(*lamp) * share;
        visibility.set_if_neq(if share > OFF {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if let Some(mut light) = point.filter(|light| (light.intensity - lumens).abs() > 1.0) {
            light.intensity = lumens;
        }
        if let Some(mut light) = spot {
            aim(&mut light, lumens, allowed && own);
        }
    }
}

/// Sets the power of the headlight `light` and whether it casts `shadows`, when either changed.
fn aim(light: &mut Mut<SpotLight>, lumens: f32, shadows: bool) {
    if (light.intensity - lumens).abs() > 1.0 {
        light.intensity = lumens;
    }
    if light.shadow_maps_enabled != shadows {
        light.shadow_maps_enabled = shadows;
    }
}
