//! Taking a rock out of a loaded chunk: its merged props mesh and its compound collider are built
//! again without it, inside the swap budget of the frame.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::Chunks;
use super::budget::Spend;
use super::build::render_only;
use super::embed::Embedding;
use super::plan::Loaded;
use super::spawn::Spawner;
use crate::world::Terrain;
use crate::world::canopy::{Canopy, Crown};
use crate::world::mesh::leaf_coord;
use crate::world::propmesh::merge_props_on;
use crate::world::props::body_collider_on;
use crate::world::rocks::{Rock, RockTaken, Taken};
use crate::world::scatter::Rig;

/// Metres within which a crown of the canopy is the one of a rock that left.
const SAME: f32 = 0.01;

impl Chunks {
    /// Returns the rock that stands at `at`, a point of the frame of the chunk whose props entity is
    /// `props`.
    pub fn rock_under(&self, props: Entity, at: Vec3) -> Option<Rock> {
        let held = self.loaded.values().find(|l| l.props == Some(props))?;
        held.stand.near(at)
    }
}

/// Tells the chunk that holds the rock of every message how much of it is left, so it draws it smaller
/// or not at all.
pub fn note(mut chunks: ResMut<Chunks>, taken: Res<Taken>, mut told: MessageReader<RockTaken>) {
    let Chunks { loaded, owed, .. } = &mut *chunks;
    for blow in told.read() {
        let left = taken.left(blow.id);
        loaded.values_mut().any(|l| l.stand.quarry(blow.id, left));
    }
    *owed |= loaded.values().any(|l| l.stand.dirty());
}

/// Run condition: a chunk still draws a rock that was taken.
pub fn mending(chunks: Res<Chunks>) -> bool {
    chunks.owed
}

/// Puts the new props `mesh` of `l` in place of the old one, and its collider with it while the
/// chunk is solid. A chunk with no prop left loses the entity.
fn swap(commands: &mut Commands, spawner: &mut Spawner, l: &mut Loaded, mesh: Option<Mesh>) {
    let Some(entity) = l.props else { return };
    let Some(mesh) = mesh else {
        commands.entity(entity).try_despawn();
        l.props = None;
        return;
    };
    let handle = spawner.meshes.add(mesh);
    commands.entity(entity).try_insert(Mesh3d(handle));
    match (l.solid, &l.bodies) {
        (true, Some(bodies)) => {
            commands.entity(entity).try_insert(bodies.clone());
        }
        (true, None) => {
            commands.entity(entity).try_remove::<Collider>();
        }
        (false, _) => {}
    }
}

/// Scales the crown of a rock that was drawn smaller by `ratio`.
fn shrink(crown: &mut Crown, ratio: f32) {
    crown.height *= ratio;
    crown.foot *= ratio;
}

/// Builds the props mesh and the collider of chunk `coord` of the chart of `view` again from the
/// props its stand keeps, and takes the crowns of the rocks that left out of its canopy and shrinks those of the rocks that shrank.
fn repair(
    (commands, spawner): (&mut Commands, &mut Spawner),
    l: &mut Loaded,
    (view, coord): (&Terrain, IVec2),
    crowns: &mut Query<&mut Canopy>,
) {
    let warp = Embedding::of(view, coord);
    let (embed, turn) = (|p| warp.point(p), |p| warp.turn(p));
    let rig = Rig {
        embed: &embed,
        turn: &turn,
    };
    let props = l.stand.props();
    let mesh = (!props.is_empty()).then(|| render_only(merge_props_on(props, &rig)));
    l.bodies = body_collider_on(props, &rig);
    let changes = l.stand.settle();
    if let Ok(mut canopy) = crowns.get_mut(l.entity) {
        for (at, ratio) in changes {
            canopy.0.retain(|c| ratio > 0.0 || c.at.distance(at) > SAME);
            let here = canopy.0.iter_mut().filter(|c| c.at.distance(at) <= SAME);
            here.for_each(|c| shrink(c, ratio));
        }
    }
    swap(commands, spawner, l, mesh);
}

/// Builds again the props of the chunks that lost a rock, as many as the swap budget of the frame
/// allows.
pub fn mend(
    mut commands: Commands,
    mut chunks: ResMut<Chunks>,
    mut spawner: Spawner,
    mut crowns: Query<&mut Canopy>,
) {
    let mut spend = Spend::of(chunks.stretch.millis(spawner.shares().swaps));
    let mut waiting = false;
    for (id, l) in chunks.loaded.iter_mut().filter(|(_, l)| l.stand.dirty()) {
        if !spend.more() {
            waiting = true;
            break;
        }
        let view = spawner.view(*id);
        let coord = leaf_coord(&view, *id);
        repair(
            (&mut commands, &mut spawner),
            l,
            (&view, coord),
            &mut crowns,
        );
    }
    chunks.owed = waiting;
}
