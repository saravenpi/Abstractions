//! Laying the trails of every ship: each hull gets four ribbons, two wing tip vortices and two
//! lamp streaks, fed from where the hull is drawn and written through the strip of the wingsuit
//! trails.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::field::Field;
use super::{Air, Flown};
use crate::gear::{Kit, Strip, Trail, ribbon_mesh, show, trace};
use crate::screens::Screen;
use crate::ship::exhaust::{Exhaust, HullLamp, lamp_place};
use crate::space::Anchor;

/// Where the vortex leaves the left wing, in hull axes: the trailing edge of the tip. The right
/// one mirrors it.
const WING_TIP: Vec3 = Vec3::new(-3.2, -0.2, 2.35);

/// Metres between two points of a vortex trail and of a lamp streak, and the jump in metres
/// over which either starts over: a ship in air covers under 60 m in a frame, and one that
/// moves farther was put somewhere else or flies too fast to leave a line.
const SPACING: (f64, f64, f64) = (6.0, 2.0, 60.0);

/// What lays a ribbon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The vortex of a wing tip: 0 for the left, 1 for the right.
    Vortex(usize),
    /// The navigation lamp of a winglet: 0 for the left, 1 for the right.
    Lamp(usize),
}

/// One ribbon of a ship.
#[derive(Component, Debug, Clone)]
pub struct Streak {
    /// What lays it.
    pub source: Source,
    /// The ship it belongs to.
    pub ship: Entity,
    /// Its points, in the frame of the anchor.
    pub trail: Trail,
    /// Whether its mesh shows anything.
    pub drawn: bool,
}

/// Returns where `source` is in hull axes.
pub fn emitter(source: Source) -> Vec3 {
    match source {
        Source::Vortex(side) => Vec3::new(
            WING_TIP.x * [1.0, -1.0][side.min(1)],
            WING_TIP.y,
            WING_TIP.z,
        ),
        Source::Lamp(side) => lamp_place(HullLamp::Tip(side)),
    }
}

/// Gives every ship that has none its four ribbons, and takes away the ribbons of the ships that
/// are gone.
pub fn spawn(
    mut commands: Commands,
    (kit, mut meshes): (Option<Res<Kit>>, ResMut<Assets<Mesh>>),
    new: Query<Entity, (With<Exhaust>, Without<Flown>)>,
    streaks: Query<(Entity, &Streak)>,
    ships: Query<(), With<Exhaust>>,
) {
    for (ribbon, streak) in &streaks {
        if !ships.contains(streak.ship) {
            commands.entity(ribbon).try_despawn();
        }
    }
    let Some(kit) = kit else {
        return;
    };
    for ship in &new {
        commands.entity(ship).insert(Flown::default());
        let sides = [0, 1].into_iter();
        for source in sides.flat_map(|side| [Source::Vortex(side), Source::Lamp(side)]) {
            commands.spawn((
                Streak {
                    source,
                    ship,
                    trail: Trail::default(),
                    drawn: false,
                },
                Mesh3d(meshes.add(ribbon_mesh())),
                MeshMaterial3d(kit.glow.clone()),
                Transform::IDENTITY,
                NoFrustumCulling,
                NotShadowCaster,
                DespawnOnEnter(Screen::Title),
            ));
        }
    }
}

/// The hulls that leave trails: where each is drawn, whether it shows, and how it flew.
type Hulls<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Visibility,
        &'static mut Flown,
    ),
>;

/// The ribbons, which are no hulls.
type Ribbons<'w, 's> =
    Query<'w, 's, (&'static mut Streak, &'static Mesh3d, &'static mut Transform), Without<Flown>>;

/// The flight of one ship this frame: the ship, where it is drawn and the air it flies in.
type Flight = (Entity, Transform, Air);

/// Reads the flight of every ship from where it is drawn. A hidden ship flies in no air, so it
/// lays nothing.
fn observe(field: &Field, anchor: &Anchor, hulls: &mut Hulls, flights: &mut Vec<Flight>) {
    flights.clear();
    for (ship, at, shown, mut flown) in hulls.iter_mut() {
        let air = field.read(anchor, at.translation, &mut flown);
        let seen = *shown != Visibility::Hidden;
        flights.push((ship, *at, if seen { air } else { Air::default() }));
    }
}

/// Reads the flight of every ship, feeds its four trails and rewrites the meshes of the ones
/// that glow.
pub fn draw(
    field: Field,
    mut hulls: Hulls,
    mut ribbons: Ribbons,
    (mut meshes, mut strip): (ResMut<Assets<Mesh>>, Local<Strip>),
    mut flights: Local<Vec<Flight>>,
) {
    let Some(anchor) = field.anchor.as_deref() else {
        return;
    };
    let eye = field
        .camera
        .single()
        .map_or(Vec3::ZERO, |eye| eye.translation());
    let dt = field.dt();
    observe(&field, anchor, &mut hulls, &mut flights);
    for (mut streak, handle, mut transform) in &mut ribbons {
        let Some((_, at, air)) = flights.iter().find(|flight| flight.0 == streak.ship) else {
            continue;
        };
        let (heat, look) = field.style(streak.source, air, eye);
        let point = anchor.to_frame(at.transform_point(emitter(streak.source)));
        let spacing = match streak.source {
            Source::Vortex(_) => SPACING.0,
            Source::Lamp(_) => SPACING.1,
        };
        let streak = &mut *streak;
        streak.trail.feed_by(point, heat, dt, (spacing, SPACING.2));
        if let Some(head) = trace(&streak.trail, &mut streak.drawn, (look, anchor), &mut strip) {
            show(head, (handle, &mut transform), &strip, &mut meshes);
        }
    }
}
