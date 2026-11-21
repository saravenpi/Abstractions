//! The sunk check: how deep the feet of a walking animal stand under the drawn ground, and, once a
//! second, the walkers whose feet stand more than [`SUNK`] under it set back on it; a dev build also
//! logs the deepest. No species burrows: a foot under the ground is a defect.
//!
//! Only walkers stand on their feet. The feet of a flyer, a swimmer or a floater are where its legs
//! rested when it came into view, and its plan never moves them: a swimmer drawn on the bed of a lake
//! keeps feet the hang of its legs under the bed, metres deep for a big one, while it swims over them.

use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use super::beast::Beast;
use super::ground::Heights;
use super::tune::SUNK;
use super::{Creature, Dead, Frozen};
use crate::life::body::Locomotion;
use crate::world::{Chunks, Terrain};

/// The living animals, drawn and not frozen.
type Living = (Without<Dead>, Without<Frozen>);

/// Returns how deep the deepest foot of `beast` stands under the ground `ground` of its frame, in
/// metres, negative when every foot is over it; nothing for a body without feet.
pub fn depth(beast: &Beast, ground: &dyn Fn(Vec2) -> f32) -> Option<f32> {
    let under = |at: Vec3| ground(at.xz()) - at.y;
    beast
        .motion
        .feet
        .iter()
        .map(|f| under(f.at))
        .reduce(f32::max)
}

/// Sets `beast` back on `ground` when a foot of it stands more than [`SUNK`] under it: every foot and
/// where it lands no lower than the ground under it, and the body held no lower than the ground under
/// its root. Returns how deep its deepest foot stood.
pub fn mend(beast: &mut Beast, ground: &dyn Fn(Vec2) -> f32) -> Option<f32> {
    let deep = depth(beast, ground).filter(|d| *d > SUNK)?;
    for foot in &mut beast.motion.feet {
        foot.at.y = foot.at.y.max(ground(foot.at.xz()));
        foot.to.y = foot.to.y.max(ground(foot.to.xz()));
    }
    let root = &mut beast.motion.root;
    let base = ground(root.at.xz());
    root.hold.at.x = root.hold.at.x.max(base);
    root.floor = base;
    Some(deep)
}

/// Sets back on the drawn ground every living walker whose feet sank under it, and in a dev build logs
/// the deepest with its species, its herd and its place in the herd.
pub fn check(
    (terrain, chunks): (Res<Terrain>, Option<Res<Chunks>>),
    mut beasts: Query<(&mut Beast, &Creature), Living>,
    heights: Local<Heights>,
) {
    let Some(life) = terrain.globe().life() else {
        return;
    };
    heights.clear();
    let chunks = chunks.as_deref();
    let mut worst: Option<(f32, Creature)> = None;
    for (mut beast, creature) in &mut beasts {
        let species = life.species.get(usize::from(beast.index));
        if !species.is_some_and(|s| s.body.locomotion == Locomotion::Walk) {
            continue;
        }
        let spot = beast.spot;
        let drawn = |x: f32, z: f32| heights.drawn(&terrain, chunks, x, z);
        let ground = |p: Vec2| spot.ground_by(&terrain, p, drawn);
        if let Some(deep) = mend(&mut beast, &ground) {
            worst = worst.filter(|w| w.0 >= deep).or(Some((deep, *creature)));
        }
    }
    #[cfg(feature = "dev")]
    if let Some((metres, c)) = worst {
        warn!(
            "fauna: a foot of {:?}, herd {} slot {}, stood {metres:.2} m under the drawn ground, set back on it",
            c.species, c.herd, c.slot
        );
    }
    #[cfg(not(feature = "dev"))]
    let _ = worst;
}

/// Registers the sunk check: once a second in play on a living planet, after the walk.
pub(super) fn plugin(app: &mut App) {
    let second = std::time::Duration::from_secs(1);
    app.add_systems(
        Update,
        check
            .after(super::walk::walk)
            .run_if(on_timer(second))
            .run_if(crate::screens::playing)
            .run_if(super::gate::living),
    );
}
