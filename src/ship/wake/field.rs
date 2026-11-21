//! What the trails of the ships are read from: the frame clock, the anchor, the air and the sky
//! of the planet, the weather and the time of day at the camera.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::lay::Source;
use super::{Air, Flown, lamp_heat, lamp_life, vortex_heat};
use crate::gear::{Look, air_density};
use crate::render::{CloudCrossing, DayClock, GameCamera, Palette, daylight};
use crate::ship::drag::SEA_DENSITY;
use crate::ship::exhaust::TIP_COLORS;
use crate::space::{Anchor, Frame, altitude, up};
use crate::speed::moisture;
use crate::universe::heights;
use crate::weather::Weather;
use crate::wind::Thickness;
use crate::world::Terrain;

/// Seconds a vortex trail glows for.
const VORTEX_LIFE: f32 = 2.5;

/// What the trails are read from, besides the ships.
#[derive(SystemParam)]
pub struct Field<'w, 's> {
    /// Frame clock.
    time: Res<'w, Time>,
    /// The render anchor.
    pub anchor: Option<Res<'w, Anchor>>,
    /// The terrain, for the height above the sea and the pull of the planet.
    terrain: Option<Res<'w, Terrain>>,
    /// How thick the air of the planet is.
    thickness: Option<Res<'w, Thickness>>,
    /// How deep the camera is in the cloud layer.
    clouds: Option<Res<'w, CloudCrossing>>,
    /// The weather at the camera.
    weather: Option<Res<'w, Weather>>,
    /// The time of day.
    clock: Option<Res<'w, DayClock>>,
    /// The colours of the sky.
    palette: Option<Res<'w, Palette>>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

impl Field<'_, '_> {
    /// Returns the seconds of the frame.
    pub fn dt(&self) -> f32 {
        self.time.delta_secs()
    }

    /// Returns where the game camera is in render space.
    pub fn eye(&self) -> Vec3 {
        let eye = self.camera.single();
        eye.map_or(Vec3::ZERO, |eye| eye.translation())
    }

    /// Returns the thickness of the air at the render position `at`, against the sea level at
    /// home, and the pull of the planet there. No air and no pull away from a surface.
    fn air(&self, anchor: &Anchor, at: Vec3) -> (f32, f32) {
        let (Some(terrain), Frame::Surface(_)) = (self.terrain.as_deref(), anchor.frame) else {
            return (0.0, 0.0);
        };
        let globe = terrain.globe();
        let gravity = globe.setting().gravity;
        let [top, ..] = heights(globe.record(), f64::from(gravity), globe.radius());
        let dense = air_density(top as f32, altitude(anchor, globe, at)) / SEA_DENSITY as f32;
        let planet = self.thickness.as_deref().map_or(1.0, |air| air.0);
        (dense.min(1.0) * planet.clamp(0.0, 1.0), gravity)
    }

    /// Reads the flight of the hull drawn at `at` this frame into `flown`, and returns it with
    /// the air around it.
    pub fn read(&self, anchor: &Anchor, at: Vec3, flown: &mut Flown) -> Air {
        let dt = self.dt();
        let vel = match flown.was {
            Some((frame, was)) if frame == anchor.frame && dt > 0.0 => {
                Some((at - anchor.to_render(was)) / dt)
            }
            _ => None,
        };
        flown.was = Some((anchor.frame, anchor.to_frame(at)));
        let (thick, gravity) = self.air(anchor, at);
        let (speed, load) = flown.advance(vel, (up(anchor, at), gravity), dt);
        let depth = self.clouds.as_deref().map_or(0.0, |clouds| clouds.depth);
        let weather = self.weather.as_deref().copied().unwrap_or_default();
        let clock = self.clock.as_deref();
        Air {
            speed,
            load,
            thick,
            moist: moisture(depth, weather.cloud, weather.rain),
            day: clock.map_or(1.0, |clock| daylight(clock.phase)),
        }
    }

    /// Returns the linear colour of the sky straight up, white without a sky.
    fn sky(&self) -> Vec3 {
        self.palette.as_deref().map_or(Vec3::ONE, |palette| {
            let zenith = Color::srgb(palette.zenith.x, palette.zenith.y, palette.zenith.z);
            let zenith = zenith.to_linear();
            Vec3::new(zenith.red, zenith.green, zenith.blue)
        })
    }

    /// Returns how `source` glows in `air` and how its ribbon is drawn, seen from `eye`: a wing
    /// tip a pale line of the colour of the sky that widens like a vortex, a lamp a thin line of
    /// its own colour.
    pub fn style(&self, source: Source, air: &Air, eye: Vec3) -> (f32, Look) {
        let (heat, width, spread, life, tint) = match source {
            Source::Vortex(_) => {
                let pale = self.sky().lerp(Vec3::ONE, 0.55) * 2.4;
                (vortex_heat(air), 0.06, 5.0, VORTEX_LIFE, pale)
            }
            Source::Lamp(side) => {
                let own = Vec3::from_array(TIP_COLORS[side.min(1)]) * 4.0;
                (lamp_heat(air), 0.03, 1.0, lamp_life(air.thick), own)
            }
        };
        let look = Look {
            eye,
            width,
            spread,
            life,
            tint,
        };
        (heat, look)
    }
}
