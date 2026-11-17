//! The life of the base: lights that follow the day, the beacon seen from afar, and an operator
//! who notices the robot at the hatch.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::base::Base;
use super::glow::{Glow, Mood};
use super::spawn::{Home, Lamp, Sight};
use super::tune::{BEACON_GROWTH, NOTICE};
use super::works::Works;
use crate::render::{DayClock, daylight};
use crate::world::StreamAnchor;

/// Smallest change of a colour channel worth sending to the renderer.
const STEP: f32 = 0.01;

/// Returns the notice level after `dt` seconds: it rises to 1 while the robot is `near` the
/// hatch and falls back to 0 after it leaves.
pub fn noticed(level: f32, near: bool, dt: f32) -> f32 {
    let step = dt.max(0.0) / NOTICE.1;
    (level + if near { step } else { -step }).clamp(0.0, 1.0)
}

/// Returns how many times its size the beacon is drawn from `distance` metres, so that it
/// stays a dot on screen from the air.
pub fn swell(distance: f32) -> f32 {
    (distance / BEACON_GROWTH.0).clamp(1.0, BEACON_GROWTH.1)
}

/// Lets the operator notice the robot: the level rises while the focus stands near the hatch.
pub fn notice(time: Res<Time>, base: Res<Base>, sight: Sight, mut home: Single<&mut Home>) {
    let near = sight
        .distance(&base, base.door)
        .is_some_and(|d| d < NOTICE.0);
    let level = noticed(home.notice, near, time.delta_secs());
    if level != home.notice {
        home.notice = level;
    }
}

/// Reads the time and the time of day.
#[derive(SystemParam)]
pub struct Skies<'w> {
    /// Game time.
    time: Res<'w, Time>,
    /// Time of day, when a sky exists.
    clock: Option<Res<'w, DayClock>>,
}

impl Skies<'_> {
    /// Returns the mood the lights answer to this frame at the notice level of `home`.
    fn mood(&self, home: &Home) -> Mood {
        let day = self
            .clock
            .as_ref()
            .map_or(1.0, |clock| daylight(clock.phase));
        Mood {
            seconds: self.time.elapsed_secs(),
            night: 1.0 - day,
            notice: home.notice,
        }
    }
}

/// Returns true when the colours `a` and `b` differ enough to be worth sending to the renderer.
fn differs(a: Color, b: Color) -> bool {
    let apart = a.to_linear().to_vec3() - b.to_linear().to_vec3();
    apart.abs().max_element() > STEP
}

/// Sets the colour of every glow whose look changed, and the power of the lights they cast.
pub fn glow(
    skies: Skies,
    works: Res<Works>,
    home: Single<&Home>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lamps: Query<(&Lamp, &mut PointLight)>,
) {
    let mood = skies.mood(&home);
    for (lamp, mut light) in &mut lamps {
        light.intensity = lamp.0.power(mood);
    }
    let kit = works
        .kit
        .iter()
        .flat_map(|kit| Glow::ALL.into_iter().zip(&kit.glows));
    for (glow, (_, handle)) in kit {
        let wanted = glow.color(mood);
        let stale = materials
            .get(handle)
            .is_some_and(|m| differs(m.base_color, wanted));
        if stale && let Some(mut material) = materials.get_mut(handle) {
            material.base_color = wanted;
        }
    }
}

/// Grows the beacon with the distance of the focus, about the point it shines from.
pub fn beacon(
    base: Res<Base>,
    works: Res<Works>,
    sight: Sight,
    mut glows: Query<(&Glow, &mut Transform), Without<StreamAnchor>>,
) {
    let lamps = works.kit.iter().flat_map(|kit| &kit.lamps);
    let at = lamps.into_iter().find(|lamp| lamp.0 == Glow::Beacon);
    let (Some((_, at)), Some(distance)) = (at, sight.distance(&base, base.centre)) else {
        return;
    };
    let scale = swell(distance as f32);
    let wanted = Transform::from_translation(*at * (1.0 - scale)).with_scale(Vec3::splat(scale));
    for (glow, mut transform) in &mut glows {
        if *glow == Glow::Beacon {
            transform.set_if_neq(wanted);
        }
    }
}
