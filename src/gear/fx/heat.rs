//! The heat of the robot in fast flight through air: what it flies through goes to [`Heat`],
//! which the sheath draws.

use bevy::prelude::*;

use super::scene::Scene;
use crate::render::{Craft, Heat, LocalSystem, MIN_SPEED, Vantage, entry};

/// Radius in metres of the sphere that holds the body of the robot.
const BODY_RADIUS: f32 = 0.9;

/// Run condition: the robot flies fast enough, through air, for its nose to heat.
pub fn fast(scene: Res<Scene>) -> bool {
    scene.active && scene.air_density > 0.0 && f64::from(scene.state.airspeed) >= MIN_SPEED
}

/// Offers the heat of the robot: the glow of its speed through the air it is in.
pub fn offer(
    scene: Res<Scene>,
    sky: (Option<Res<Vantage>>, Option<Res<LocalSystem>>),
    heat: Option<ResMut<Heat>>,
) {
    let (Some(vantage), Some(local), Some(mut heat)) = (sky.0, sky.1, heat) else {
        return;
    };
    let gas = local.gas(&vantage);
    let speed = f64::from(scene.state.airspeed);
    let glow = entry(f64::from(scene.air_density), speed, gas);
    let heading = scene.velocity().normalize_or(scene.rot * Vec3::NEG_Z);
    heat.offer(
        glow,
        Craft {
            centre: scene.body,
            heading,
            radius: BODY_RADIUS,
            roof: scene.rot * Vec3::Y,
        },
    );
}
