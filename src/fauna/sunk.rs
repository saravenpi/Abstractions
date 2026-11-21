//! The sunk check: how deep the feet of an animal stand under the drawn ground, and in a dev build a
//! log, once a second, of the animal whose feet stand deepest when they stand more than
//! [`SUNK`](super::tune::SUNK) under it. No species burrows: a foot under the ground is a defect.

use bevy::prelude::*;

use super::beast::Beast;

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

/// The living animals, drawn and not frozen.
#[cfg(feature = "dev")]
type Living = (Without<super::Dead>, Without<super::Frozen>);

/// Logs the living drawn animal whose feet stand deepest under the drawn ground, when they stand
/// more than [`SUNK`](super::tune::SUNK) under it, with its species, its herd and its place in the herd.
#[cfg(feature = "dev")]
pub fn check(
    terrain: Res<crate::world::Terrain>,
    chunks: Option<Res<crate::world::Chunks>>,
    beasts: Query<(&Beast, &super::Creature), Living>,
    heights: Local<super::ground::Heights>,
) {
    heights.clear();
    let chunks = chunks.as_deref();
    let deep = |(beast, creature): (&Beast, &super::Creature)| {
        let spot = beast.spot;
        let drawn = |x: f32, z: f32| heights.drawn(&terrain, chunks, x, z);
        let ground = |p: Vec2| spot.ground_by(&terrain, p, drawn);
        depth(beast, &ground).map(|d| (d, *creature))
    };
    let worst = beasts
        .iter()
        .filter_map(deep)
        .max_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((metres, c)) = worst.filter(|(d, _)| *d > super::tune::SUNK) {
        warn!(
            "fauna: a foot of {:?}, herd {} slot {}, stands {metres:.2} m under the drawn ground",
            c.species, c.herd, c.slot
        );
    }
}

/// Registers the sunk check of a dev build: once a second in play on a living planet, after the walk.
#[cfg(feature = "dev")]
pub(super) fn plugin(app: &mut App) {
    let second = std::time::Duration::from_secs(1);
    app.add_systems(
        Update,
        check
            .after(super::walk::walk)
            .run_if(bevy::time::common_conditions::on_timer(second))
            .run_if(crate::screens::playing)
            .run_if(super::gate::living),
    );
}
