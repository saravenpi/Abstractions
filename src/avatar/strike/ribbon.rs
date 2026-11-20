//! The ribbon of a tool: the surface its edge swept in the last moments, drawn as a translucent strip
//! in the frame of the body, one per armed body.
//!
//! The mesh has a fixed number of vertices, written in place each frame like the ribbons of the gear
//! trails, so nothing is allocated. A thrust sweeps a line and a slash an arc.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::act::Strike;
use super::canvas::{material, mesh, write};
use super::held::Held;
use super::show::HEAD;
use super::trail::{ROWS, Trail};
use super::tune::{TRAIL_LIFE, TRAIL_PEAK, TRAIL_TINT};

/// Share of the opacity the edge nearer the grip keeps: the ribbon fades toward the hand.
const INNER: f32 = 0.25;

/// The ribbon of a body, on its own entity under the body.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Ribbon {
    /// The path of the edge.
    pub trail: Trail,
}

/// Marks a body that has a ribbon, and names it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Trailing(pub Entity);

/// Returns the colour of the ribbon: white with [`TRAIL_TINT`] of the colour of the head, linear RGB.
pub fn tint() -> Vec3 {
    let head = Color::srgb(HEAD[0], HEAD[1], HEAD[2]).to_linear();
    Vec3::ONE.lerp(Vec3::new(head.red, head.green, head.blue), TRAIL_TINT)
}

/// Writes the ribbon of `trail` into `positions` and `colours`, two slots a row, newest row first.
/// Rows the trail has not laid yet fold onto its oldest and are clear.
pub fn fill(trail: &Trail, positions: &mut [[f32; 3]], colours: &mut [[f32; 4]]) {
    let tint = tint();
    let last = trail.row(trail.len().saturating_sub(1)).unwrap_or_default();
    for index in 0..ROWS {
        let row = trail.row(index).unwrap_or(last);
        let young = (1.0 - row.age / TRAIL_LIFE).clamp(0.0, 1.0);
        let alpha = if index < trail.len() {
            TRAIL_PEAK * row.glow * young * young
        } else {
            0.0
        };
        for (side, share) in [(0, INNER), (1, 1.0)] {
            positions[index * 2 + side] = row.edge[side].to_array();
            colours[index * 2 + side] = tint.extend(alpha * share).to_array();
        }
    }
}

/// Gives every body that holds a tool and has no ribbon one, on its own entity under the body.
pub fn rig(
    mut commands: Commands,
    (mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    bodies: Query<Entity, (With<Held>, Without<Trailing>)>,
    mut shared: Local<Option<Handle<StandardMaterial>>>,
) {
    let look = shared
        .get_or_insert_with(|| materials.add(material()))
        .clone();
    for body in &bodies {
        let ribbon = commands
            .spawn((
                Name::new("Tool trail"),
                Ribbon::default(),
                Mesh3d(meshes.add(mesh())),
                MeshMaterial3d(look.clone()),
                Transform::IDENTITY,
                Visibility::Hidden,
                NoFrustumCulling,
                NotShadowCaster,
                ChildOf(body),
            ))
            .id();
        commands.entity(body).insert(Trailing(ribbon));
    }
}

/// Returns the edge of the tool whose grip and tip are `blade` and whose head is `head` metres long:
/// the end of the shaft the head covers, the one nearer the grip first.
pub fn edge(blade: [Vec3; 2], head: f32) -> [Vec3; 2] {
    let [grip, tip] = blade;
    let way = (tip - grip).normalize_or_zero();
    [tip - way * head.min(tip.distance(grip)), tip]
}

/// Lays the trail of every armed body from the path of its tip this frame, shows its ribbon while
/// anything glows and hides it when nothing does or when no tool is held.
pub fn trace(
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    bodies: Query<(&Strike, Option<&Held>, &Trailing)>,
    mut ribbons: Query<(&mut Ribbon, &Mesh3d, &mut Visibility)>,
) {
    let dt = time.delta_secs().max(1e-4);
    for (strike, held, trailing) in &bodies {
        let Ok((mut ribbon, handle, mut shown)) = ribbons.get_mut(trailing.0) else {
            continue;
        };
        let [now, was] = strike.blade;
        let speed = now[1].distance(was[1]) / dt;
        match held.filter(|h| h.out) {
            Some(held) => ribbon
                .trail
                .feed(edge(now, held.build.head.size()), speed, dt),
            None => ribbon.trail.feed([Vec3::ZERO; 2], 0.0, dt),
        }
        let empty = ribbon.trail.is_empty();
        shown.set_if_neq(if empty {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        if let (false, Some(mut mesh)) = (empty, meshes.get_mut(&handle.0)) {
            write(&mut mesh, &ribbon.trail);
        }
    }
}
