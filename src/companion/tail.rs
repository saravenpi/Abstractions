//! The tail: the thin trail the companion leaves in flight, in the colour of its own light.
//!
//! It is the path its thruster really went through: the points are laid in the planet frame
//! and never move afterwards, the ribbon fades by age and is hidden behind solid things by the
//! depth test. One ribbon, a third as wide and as long as a trail of the wingsuit.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::Companion;
use super::motion::Float;
use super::ribbon::{Look, Strip, fill, mesh, write};
use super::screen::Display;
use super::sense::Around;
use super::spring::ramp;
use super::tune::{HALF, TAIL_GAIN, TAIL_LIFE, TAIL_SPEED, TAIL_WIDTH};
use super::wake::Wake;
use crate::screens::Screen;

/// The trail of one companion and whether its mesh currently shows anything.
#[derive(Component, Debug, Clone)]
pub struct Tail {
    /// The companion that lays it.
    pub owner: Entity,
    /// Its points, in the planet frame.
    pub wake: Wake,
    /// Whether the mesh currently shows anything.
    pub drawn: bool,
}

/// Returns the additive material of the ribbon, whose colour and alpha come from the vertices.
pub fn material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Add,
        unlit: true,
        fog_enabled: false,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

/// Returns how strongly the tail glows for a companion that has gone `flight` of the way from
/// following on foot to flying, at `speed` metres per second: nothing at rest or on foot.
pub fn heat(flight: f32, speed: f32) -> f32 {
    flight.clamp(0.0, 1.0) * ramp(TAIL_SPEED.0, TAIL_SPEED.1, speed)
}

/// Spawns the ribbon of the companion `owner`, drawn with the shared `material`. It stays at
/// the render origin: its vertices are rewritten in render space each frame from points kept in
/// the planet frame, which is what carries it through a rebase. The mesh is its own, since
/// every trail has its own points. It leaves with the title screen.
pub fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    owner: Entity,
) {
    commands.spawn((
        Tail {
            owner,
            wake: Wake::default(),
            drawn: false,
        },
        Mesh3d(meshes.add(mesh())),
        MeshMaterial3d(material),
        Transform::IDENTITY,
        NoFrustumCulling,
        NotShadowCaster,
        DespawnOnEnter(Screen::Title),
    ));
}

/// Returns the render position of the thruster of a companion at `at`, where the tail leaves
/// from. Seated, `at` is in the space of the pilot seat `seat`.
fn nozzle(at: &Transform, seat: Option<&Transform>) -> Vec3 {
    let under = at.transform_point(Vec3::NEG_Y * (HALF[1] + 0.03));
    seat.map_or(under, |seat| seat.transform_point(under))
}

/// Returns how the tail of a companion with the light of `display` is drawn, seen from `eye`.
fn look(eye: Vec3, display: &Display) -> Look {
    Look {
        eye,
        width: TAIL_WIDTH,
        life: TAIL_LIFE,
        tint: display.tint * TAIL_GAIN,
    }
}

/// Lays the tail of every companion behind its thruster and rewrites the ribbon while any of
/// it glows. A tail with nothing left to show is written once more, to clear it, and then left
/// alone. A tail whose companion is gone goes too.
pub fn draw(
    mut commands: Commands,
    around: Around,
    bots: Query<(&Transform, &Float, &Display), With<Companion>>,
    mut tails: Query<(Entity, &mut Tail, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let anchor = &*around.anchor;
    for (entity, mut tail, handle) in &mut tails {
        let Ok((at, float, display)) = bots.get(tail.owner) else {
            commands.entity(entity).despawn();
            continue;
        };
        let seat = float.seated.then_some(*around.body);
        let glow = heat(float.flight, float.vel.length()) * f32::from(u8::from(!float.seated));
        let from = anchor.to_frame(nozzle(at, seat));
        tail.wake.feed(from, glow, around.dt());
        let live = tail.wake.live(TAIL_LIFE);
        if !live && !tail.drawn {
            continue;
        }
        tail.drawn = live;
        let look = look(around.camera.translation(), display);
        let mut strip = Strip::default();
        fill(
            &tail.wake,
            &look,
            |point| anchor.to_render(point),
            &mut strip,
        );
        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            write(&mut mesh, &strip);
        }
    }
}
