//! The vapour cone: a pale cone of condensed water that wraps the rear of the hull for a moment as
//! the ship passes the speed of sound in wet air.
//!
//! The speed of sound comes from the temperature of the air at the height of the ship and the gas
//! mix of the planet, and the wetness from the clouds and the rain at the camera, so the cone shows
//! only where the air is dense and wet enough, and nowhere in a vacuum or on a dry world.

use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::ShipSet;
use super::body::Ship;
use super::drag::{SEA_DENSITY, density};
use super::telemetry::ShipTelemetry;
use super::view::HULL_RADIUS;
use crate::render::CloudCrossing;
use crate::screens::Screen;
use crate::space::{Anchor, Frame, Location, Placed};
use crate::speed::tune::CONE_ALPHA;
use crate::speed::{cone, mach, moisture, sound_speed, temperature_at};
use crate::universe::{PlanetSpec, Universe, heights};
use crate::weather::Weather;

/// Share of the hull radius the cone is long, wide at its base, and sits behind the hull centre.
const SHAPE: (f32, f32, f32) = (1.7, 0.95, 0.5);

/// Linear colour of the vapour: white with a hint of blue.
const TINT: Vec3 = Vec3::new(0.93, 0.96, 1.0);

/// The cone, one entity with its own material.
#[derive(Component, Debug, Clone, Copy)]
pub struct Vapour;

/// The planet the ship flies over and the top of its weather layer in metres, kept so that the
/// planet is not rebuilt every frame.
#[derive(Debug, Default)]
pub struct Here(Option<(PlanetSpec, f64)>);

/// What the cone is read from.
#[derive(SystemParam)]
pub struct Air<'w> {
    /// The render anchor.
    anchor: Res<'w, Anchor>,
    /// The planet the player is at.
    location: Res<'w, Location>,
    /// The universe being played.
    universe: Res<'w, Universe>,
    /// The instruments of the ship.
    telemetry: Res<'w, ShipTelemetry>,
    /// How deep the camera is in the cloud layer, when the sky is drawn.
    clouds: Option<Res<'w, CloudCrossing>>,
    /// The weather at the camera.
    weather: Option<Res<'w, Weather>>,
}

/// Spawns the hidden cone when gameplay starts, unless it exists.
pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<(), With<Vapour>>,
) {
    if !old.is_empty() {
        return;
    }
    let material = StandardMaterial {
        base_color: Color::linear_rgba(TINT.x, TINT.y, TINT.z, 0.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        fog_enabled: false,
        double_sided: true,
        cull_mode: None,
        ..default()
    };
    commands.spawn((
        Vapour,
        Mesh3d(meshes.add(Cone::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(material)),
        Transform::from_scale(Vec3::ZERO),
        NotShadowCaster,
        DespawnOnEnter(Screen::Title),
    ));
}

impl Air<'_> {
    /// Returns how strongly the cone shows around a ship at `height` metres over the sea of the
    /// planet `here`, from 0 to 1.
    fn strength(&self, here: &(PlanetSpec, f64), height: f64) -> f32 {
        let (spec, limit) = here;
        let record = &spec.record;
        let scale = (spec.scale_height(), *limit);
        let cold = temperature_at(record.temperature, record.mix.kappa(), scale, height);
        let sound = sound_speed(cold, &record.mix);
        let thick = (density(spec, height) / SEA_DENSITY) as f32;
        let depth = self.clouds.as_deref().map_or(0.0, |clouds| clouds.depth);
        let weather = self.weather.as_deref().copied().unwrap_or_default();
        let wet = moisture(depth, weather.cloud, weather.rain);
        cone(mach(self.telemetry.surface_speed as f32, sound), thick, wet)
    }

    /// Returns the planet of the player, built again only when the body changes.
    fn planet<'a>(&self, here: &'a mut Here) -> &'a (PlanetSpec, f64) {
        let known = here
            .0
            .as_ref()
            .is_some_and(|(spec, _)| spec.id == self.location.body);
        if !known {
            let spec = self.universe.planet(self.location.body);
            let top = heights(&spec.record, spec.gravity, spec.radius)[2];
            here.0 = Some((spec, top));
        }
        here.0.as_ref().expect("the planet was just set")
    }
}

/// Fits the cone around the rear of the hull of the ship at `placed`, apex toward the nose.
fn fit(transform: &mut Transform, anchor: &Anchor, placed: &Placed) {
    let nose = anchor.rotation_to_render(placed.rot) * Vec3::NEG_Z;
    let across = SHAPE.1 * HULL_RADIUS * 2.0;
    transform.translation = anchor.to_render(placed.pos) - nose * (SHAPE.2 * HULL_RADIUS);
    transform.rotation = Quat::from_rotation_arc(Vec3::Y, nose);
    transform.scale = Vec3::new(across, SHAPE.0 * HULL_RADIUS, across);
}

/// Fits the cone around the rear of the hull and sizes its alpha by the strength of the cone, or
/// hides it. The material is written only when the strength moves.
pub fn draw(
    air: Air,
    ship: Single<(&Ship, &Placed)>,
    mut cone: Single<(&mut Transform, &MeshMaterial3d<StandardMaterial>), With<Vapour>>,
    (mut here, mut last): (Local<Here>, Local<f32>),
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (ship, placed) = *ship;
    let strength = match air.anchor.frame == ship.frame && matches!(ship.frame, Frame::Surface(_)) {
        true => air.strength(air.planet(&mut here), air.telemetry.altitude_sea),
        false => 0.0,
    };
    let (transform, handle) = &mut *cone;
    if strength <= 0.0 {
        if transform.scale != Vec3::ZERO {
            transform.scale = Vec3::ZERO;
        }
        *last = 0.0;
        return;
    }
    fit(transform, &air.anchor, placed);
    if (strength - *last).abs() > 0.002 || *last == 0.0 {
        *last = strength;
        if let Some(material) = materials.get_mut(&handle.0) {
            material.base_color = Color::linear_rgba(TINT.x, TINT.y, TINT.z, CONE_ALPHA * strength);
        }
    }
}

/// Registers the cone, which is fitted with the other reports of the ship once it is placed.
pub(super) fn plugin(app: &mut App) {
    let wanted = resource_exists::<Universe>
        .and_then(resource_exists::<Location>)
        .and_then(any_with_component::<Vapour>);
    app.add_systems(OnEnter(Screen::Gameplay), spawn)
        .add_systems(Update, draw.in_set(ShipSet::Report).run_if(wanted));
}
