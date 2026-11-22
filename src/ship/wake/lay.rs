//! Laying the trails of every ship: each hull gets five ribbons, two wing tip vortices, two
//! lamp streaks and a contrail, fed from where the hull is drawn and written through the strip of
//! the wingsuit trails.

use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::field::Field;
use super::{Air, Flown};
use crate::gear::{Kit, Strip, Trail, ribbon_mesh, show, trace};
use crate::screens::Screen;
use crate::ship::Engine;
use crate::ship::exhaust::{Exhaust, HullLamp, lamp_place};
use crate::space::{Anchor, Placed, Rebased};

/// Where the vortex leaves the left wing, in hull axes: the trailing edge of the tip. The right
/// one mirrors it.
const WING_TIP: Vec3 = Vec3::new(-3.2, -0.2, 2.35);

/// Least metres between two points of a vortex trail and of a lamp streak, and the jump in
/// metres over which either starts over: a ship in air covers under 60 m in a frame of 0.1 s,
/// even boosted, and one that moves twice that far was put somewhere else or flies too fast
/// to leave a line.
const SPACING: (f64, f64, f64) = (2.5, 2.0, 120.0);

/// Points of a ribbon its whole life is spread over at speed: a few under the 64 a trail
/// keeps. At 275 m/s a vortex that glows 0.8 s is 220 m long, which 2.5 m points would cut at
/// 160 m.
const ROOM: f64 = 60.0;

/// Metres between two points of a contrail, so that its ribbon holds seven kilometres, and the
/// jump in metres over which it starts over: a climb to orbit covers up to a kilometre in a
/// long frame.
const CONTRAIL_SPACING: (f64, f64) = (120.0, 1500.0);

/// Where the contrail starts behind the nozzles, in hull axes: the plume has mixed with enough
/// air there for its water to freeze.
const CONTRAIL_START: Vec3 = Vec3::new(0.0, 0.1, 12.0);

/// What lays a ribbon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The vortex of a wing tip: 0 for the left, 1 for the right.
    Vortex(usize),
    /// The navigation lamp of a winglet: 0 for the left, 1 for the right.
    Lamp(usize),
    /// The contrail behind the main engines.
    Contrail,
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
        Source::Contrail => CONTRAIL_START,
    }
}

/// Gives every ship that has none its five ribbons, and takes away the ribbons of the ships that
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
        let pairs = sides.flat_map(|side| [Source::Vortex(side), Source::Lamp(side)]);
        for source in pairs.chain([Source::Contrail]) {
            commands.spawn((
                Streak {
                    source,
                    ship,
                    trail: Trail::default(),
                    drawn: false,
                },
                Rebased,
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

/// The hulls that leave trails: where each is in the frame of the anchor, whether it shows,
/// and how it flew.
///
/// The pose is the [`Placed`] one, which is the truth of this frame. The `Transform` of a hull
/// is copied from it after `Update`, so here it still holds the pose of the frame before, under
/// an anchor that has already moved on with the ship: a trail fed from it lies a frame of
/// flight away from the wing.
type Hulls<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Placed,
        &'static Visibility,
        &'static mut Flown,
        Option<&'static Engine>,
    ),
>;

/// The ribbons, which are no hulls.
type Ribbons<'w, 's> =
    Query<'w, 's, (&'static mut Streak, &'static Mesh3d, &'static mut Transform), Without<Flown>>;

/// The flight of one ship this frame: the ship, its pose and the air it flies in.
type Flight = (Entity, Placed, Air);

/// Returns the metres between two points of a ribbon of `source` laid at `speed` metres per
/// second that lives `life` seconds: the least spacing of [`SPACING`], stretched at speed so that
/// the whole life of the ribbon fits in [`ROOM`] points.
fn spacing(source: Source, speed: f32, life: f32) -> f64 {
    let least = match source {
        Source::Vortex(_) | Source::Contrail => SPACING.0,
        Source::Lamp(_) => SPACING.1,
    };
    least.max(f64::from(speed * life) / ROOM)
}

/// Returns where `source` of a hull posed as `placed` is, in the frame of the anchor.
pub fn tip(placed: &Placed, source: Source) -> DVec3 {
    placed.pos + placed.rot * emitter(source).as_dvec3()
}

/// Reads the flight of every ship from its pose and its engines, and keeps it on the ship for
/// its cone. A hidden ship flies in no air, so it lays nothing.
fn observe(field: &Field, anchor: &Anchor, hulls: &mut Hulls, flights: &mut Vec<Flight>) {
    flights.clear();
    for (ship, placed, shown, mut flown, engine) in hulls.iter_mut() {
        let read = Air {
            engine: engine.map_or(0.0, |engine| engine.level),
            ..field.read(anchor, placed.pos, &mut flown)
        };
        let air = if *shown != Visibility::Hidden {
            read
        } else {
            Air::default()
        };
        flown.air = air;
        flights.push((ship, *placed, air));
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
    let (eye, dt) = (field.eye(), field.dt());
    observe(&field, anchor, &mut hulls, &mut flights);
    for (mut streak, handle, mut transform) in &mut ribbons {
        let Some((_, placed, air)) = flights.iter().find(|flight| flight.0 == streak.ship) else {
            continue;
        };
        let (heat, look) = field.style(streak.source, air, eye);
        let spacing = match streak.source {
            Source::Contrail => CONTRAIL_SPACING,
            source => (spacing(source, air.speed, look.life), SPACING.2),
        };
        let streak = &mut *streak;
        let point = tip(placed, streak.source);
        streak.trail.feed_by(point, heat, dt, spacing);
        *transform = Transform::IDENTITY;
        if let Some(head) = trace(&streak.trail, &mut streak.drawn, (look, anchor), &mut strip) {
            show(head, (handle, &mut transform), &strip, &mut meshes);
        }
    }
}
