//! The trails of the other bodies: each body that lays its own gets two wingtip ribbons and a
//! boot exhaust, fed from its posed body, its pose and how it moves, the way the trails of the
//! player are fed from its gear. A body seated in a ship or hidden lays nothing.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;

use super::kit::Kit;
use super::lay::redraw;
use super::ribbon::{Strip, mesh};
use super::scene::{Limbs, SOLES, Scene};
use super::trail::Trail;
use super::trails::{Ribbon, Source};
use crate::avatar::{self, AvatarPose, Marked, Marks};
use crate::gear::aero::grip;
use crate::gear::{FlightState, Mode, air_density};
use crate::render::{GameCamera, Palette};
use crate::screens::Screen;
use crate::space::{Anchor, Frame, altitude, up};
use crate::universe::air;
use crate::wind::Thickness;
use crate::world::Terrain;

/// Marks a body that lays its own trails. It keeps where the body was on the frame before and
/// the flight its trails were fed from this frame.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Wake {
    /// The anchor frame and the frame position of the body on the frame before.
    was: Option<(Frame, DVec3)>,
    /// The flight of the body this frame.
    scene: Scene,
}

/// Ties a ribbon to the body that lays it.
#[derive(Component, Debug, Clone, Copy)]
pub struct Laid(pub Entity);

/// What the trails of the other bodies are read from.
#[derive(SystemParam)]
pub struct Field<'w, 's> {
    /// Frame clock.
    time: Res<'w, Time>,
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// The terrain, for the height above the sea.
    terrain: Option<Res<'w, Terrain>>,
    /// How thick the air of the planet is.
    thickness: Option<Res<'w, Thickness>>,
    /// The colours of the sky.
    palette: Option<Res<'w, Palette>>,
    /// The game camera.
    camera: Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
}

/// The bodies that lay their own trails.
type Bodies<'w, 's> = Query<
    'w,
    's,
    (
        &'static GlobalTransform,
        &'static AvatarPose,
        &'static Marked,
        &'static Visibility,
        &'static mut Wake,
    ),
>;

/// Gives every new body that lays its own trails its three ribbons, and takes away the ribbons
/// of the bodies that are gone.
pub fn spawn(
    mut commands: Commands,
    (kit, mut meshes): (Res<Kit>, ResMut<Assets<Mesh>>),
    new: Query<Entity, Added<Wake>>,
    ribbons: Query<(Entity, &Laid)>,
    bodies: Query<(), With<Wake>>,
) {
    for (ribbon, laid) in &ribbons {
        if bodies.get(laid.0).is_err() {
            commands.entity(ribbon).try_despawn();
        }
    }
    for body in &new {
        for source in [Source::Tip(0), Source::Tip(1), Source::Boots] {
            commands.spawn((
                Ribbon {
                    source,
                    trail: Trail::default(),
                    drawn: false,
                },
                Laid(body),
                Mesh3d(meshes.add(mesh())),
                MeshMaterial3d(kit.glow.clone()),
                Transform::IDENTITY,
                NoFrustumCulling,
                NotShadowCaster,
                DespawnOnEnter(Screen::Title),
            ));
        }
    }
}

/// Returns the gear mode a body in `mode` shows: the flight modes as they are, the rest off.
fn gear(mode: avatar::Mode) -> Mode {
    match mode {
        avatar::Mode::BootFall => Mode::BootFall,
        avatar::Mode::Glide => Mode::Glide,
        avatar::Mode::Fly => Mode::Fly,
        _ => Mode::Off,
    }
}

/// Returns the grip of the gear at the render position `at` and how much thinner than at home
/// the air of the planet is, as [`Scene`] holds them.
fn thickness(field: &Field, anchor: &Anchor, at: Vec3) -> (f32, f32) {
    let density = field.terrain.as_deref().map_or(0.0, |terrain| {
        let globe = terrain.globe();
        let height = altitude(anchor, globe, at) - SOLES;
        air_density(air(globe.radius())[0] as f32, height)
    });
    let thick = field.thickness.as_deref().map_or(1.0, |air| air.0);
    (grip(density), 1.0 - thick.clamp(0.0, 1.0))
}

/// Returns the flight of a body drawn at `at` in `pose`, whose gear sits at `marks`, moving
/// at the render velocity `vel`.
fn read(
    field: &Field,
    anchor: &Anchor,
    (at, pose, marks): (&Transform, &AvatarPose, &Marks),
    vel: Vec3,
) -> Scene {
    let vertical = up(anchor, at.translation).as_dvec3();
    let tilt = DQuat::from_rotation_arc(DVec3::Y, vertical).as_quat();
    let (sigma, thin) = thickness(field, anchor, at.translation);
    let camera = field
        .camera
        .single()
        .map_or(at.translation, |eye| eye.translation());
    Scene {
        active: pose.mode != avatar::Mode::Seated,
        state: FlightState {
            mode: gear(pose.mode),
            velocity: tilt.inverse() * vel,
            wings: pose.wings,
            thrust: pose.thrust,
            airspeed: pose.airspeed,
            load: pose.load,
            ..FlightState::default()
        },
        body: at.translation,
        tilt,
        rot: at.rotation,
        eye: camera,
        sigma,
        thin,
        dt: field.time.delta_secs(),
        limbs: Some(Limbs::of(marks, at)),
        ..Scene::default()
    }
}

/// Reads the flight of every body that lays its own trails. A hidden one lays nothing.
fn observe(field: &Field, anchor: &Anchor, bodies: &mut Bodies) {
    let dt = field.time.delta_secs();
    for (global, pose, marks, shown, mut wake) in bodies.iter_mut() {
        let at = global.compute_transform();
        let vel = match wake.was {
            Some((frame, was)) if frame == anchor.frame && dt > 0.0 => {
                (at.translation - anchor.to_render(was)) / dt
            }
            _ => Vec3::ZERO,
        };
        wake.was = Some((anchor.frame, anchor.to_frame(at.translation)));
        wake.scene = match shown {
            Visibility::Hidden => Scene::default(),
            _ => read(field, anchor, (&at, pose, &marks.0), vel),
        };
    }
}

/// Feeds the trails of every body that lays its own from its flight this frame.
pub fn draw(
    field: Field,
    mut bodies: Bodies,
    mut ribbons: Query<(&Laid, &mut Ribbon, &Mesh3d, &mut Transform)>,
    (mut meshes, mut strip): (ResMut<Assets<Mesh>>, Local<Strip>),
) {
    let Some(anchor) = field.anchor.as_deref() else {
        return;
    };
    observe(&field, anchor, &mut bodies);
    for (laid, mut ribbon, handle, mut transform) in &mut ribbons {
        let scene = bodies
            .get(laid.0)
            .map_or(Scene::default(), |body| body.4.scene);
        let against = (&scene, anchor, field.palette.as_deref());
        redraw(
            (&mut ribbon, handle, &mut transform),
            against,
            &mut strip,
            &mut meshes,
        );
    }
}
