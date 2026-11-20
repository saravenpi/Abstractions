//! The lay: once a second, the signs of life around the local robot, read from the watch of the signs
//! and painted on the ground; and their forgetting when the planet they lie on is left or dead.

use bevy::ecs::system::SystemParam;
use bevy::light::NotShadowCaster;
use bevy::math::DVec3;
use bevy::prelude::*;

use super::Trail;
use super::paint::{Canvas, material, mesh, paint};
use super::tune::{MOVE, RADIUS, ROLL};
use crate::fauna::gate::{Spec, living};
use crate::fauna::ground::Heights;
use crate::fauna::herds::{Land, herd_arc, read};
use crate::fauna::place::Spot;
use crate::life::herd::{Search, Watch};
use crate::life::signs;
use crate::life::signs::Mark;
use crate::player::Player;
use crate::screens::Screen;
use crate::space::{Anchor, Clock, Placed};
use crate::world::{Cropped, Terrain};

/// What the lay reads and writes.
#[derive(SystemParam)]
pub struct Lay<'w, 's> {
    commands: Commands<'w, 's>,
    clock: Res<'w, Clock>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    spec: Res<'w, Spec>,
    trail: ResMut<'w, Trail>,
    cropped: ResMut<'w, Cropped>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    placed: Query<'w, 's, &'static mut Placed>,
}

impl Trail {
    /// Returns true when the watch must be rolled again for the robot at unit direction `at` on the
    /// globe of seed `seed` and radius `radius` metres at universe `time`.
    fn stale(&self, (seed, radius): (u64, f64), at: DVec3, time: f64) -> bool {
        let (was, from, then) = self.rolled;
        let far = herd_arc(from, at) * radius > MOVE;
        self.watch.is_none() || was != seed || far || !(then..then + ROLL).contains(&time)
    }

    /// Rolls the watch again when needed, then reads the signs around unit direction `at` on `land`
    /// at universe `time`.
    pub fn read(&mut self, land: Land, at: DVec3, time: f64) {
        let (seed, planet) = (land.globe.seed(), land.globe.radius());
        if self.stale((seed, planet), at, time) {
            let radius = RADIUS + signs::tune::REACH + MOVE;
            let search = Search {
                at,
                radius,
                planet,
                time,
            };
            self.watch = Some(read(land, time, |g| Watch::new(land.life, g, &search)));
            self.rolled = (seed, at, time);
        }
        let Some(watch) = self.watch.as_mut() else {
            return;
        };
        let search = Search {
            at,
            radius: RADIUS,
            planet,
            time,
        };
        let out = &mut self.signs;
        read(land, time, |g| {
            signs::near(land.life, g, watch, &search, out)
        });
    }
}

/// Reads the signs around the local robot and paints them on the ground.
pub fn lay(mut l: Lay, heights: Local<Heights>) {
    let globe = l.terrain.globe().clone();
    let (Some(life), Some(planet), Ok(robot)) =
        (globe.life(), l.spec.planet.clone(), l.robot.single())
    else {
        return;
    };
    let at = l.anchor.to_frame(robot.translation).normalize();
    let land = Land {
        globe: &globe,
        life,
        planet: &planet,
    };
    l.trail.read(land, at, l.clock.seconds);
    crop(&l.trail, &mut l.cropped);
    if l.terrain.is_changed() {
        heights.clear();
    }
    draw(&mut l, Spot::at(at, globe.radius()), &heights);
}

/// Tells the blades where the grazed patches of the trail lie, only when they moved, grew or went.
fn crop(trail: &Trail, cropped: &mut ResMut<Cropped>) {
    let grazed = trail.signs.iter().filter(|s| s.mark == Mark::Grazed);
    let patches = grazed.map(|s| (s.at, s.size * 0.5));
    if !patches.clone().eq(cropped.0.iter().copied()) {
        cropped.0.clear();
        cropped.0.extend(patches);
    }
}

/// Paints the signs of the trail into its mesh in the frame `spot`, spawning the mesh the first time.
fn draw(l: &mut Lay, spot: Spot, heights: &Heights) {
    let placed = Placed {
        pos: spot.origin,
        rot: spot.rot,
    };
    let (entity, handle) = match l.trail.drawn.clone() {
        Some(drawn) => drawn,
        None => spawn(l, placed),
    };
    if let Ok(mut at) = l.placed.get_mut(entity) {
        *at = placed;
    }
    let terrain = &l.terrain;
    let ground = |xz: Vec2| spot.ground_by(terrain, xz, |x, z| heights.mesh(terrain, x, z));
    let canvas = Canvas {
        spot,
        radius: terrain.globe().radius(),
        ground: &ground,
    };
    if let Some(mut mesh) = l.meshes.get_mut(&handle) {
        paint(&mut mesh, &l.trail.signs, &canvas);
    }
}

/// Spawns the entity and the mesh of the signs at `placed`, and returns them.
fn spawn(l: &mut Lay, placed: Placed) -> (Entity, Handle<Mesh>) {
    let handle = l.meshes.add(mesh());
    let look = (
        Mesh3d(handle.clone()),
        MeshMaterial3d(l.materials.add(material())),
        NotShadowCaster,
    );
    let shown = (Transform::default(), Visibility::default());
    let marks = (placed, DespawnOnEnter(Screen::Title));
    let entity = l
        .commands
        .spawn((Name::new("Signs of life"), look, shown, marks))
        .id();
    l.trail.drawn = Some((entity, handle.clone()));
    (entity, handle)
}

/// Despawns the signs and forgets them, when the anchor left a living planet, its life is gone, or the
/// globe is another than the one they were read on.
pub fn forget(
    mut commands: Commands,
    (anchor, terrain): (Option<Res<Anchor>>, Option<Res<Terrain>>),
    mut trail: ResMut<Trail>,
) {
    let seed = terrain.as_ref().map(|t| t.globe().seed());
    let same = trail.watch.is_none() || seed == Some(trail.rolled.0);
    if living(anchor, terrain) && same {
        return;
    }
    if let Some((entity, _)) = trail.drawn.take() {
        commands.entity(entity).despawn();
    }
    trail.signs.clear();
    trail.watch = None;
}
