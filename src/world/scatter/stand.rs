//! The stands of the species of each producer group: which of its kin holds a patch, and how thick the
//! group grows there, with the chance of a group shared among its species rather than added to.

use bevy::prelude::*;

use super::cover::{edge, shade, thickness};
use super::grow::Grower;
use super::pass::{Ctx, Ground};

/// Most producer groups a planet holds stands for.
pub const GROUPS: usize = 16;

/// Half the width of the stand of a species in the kin key, and the share it keeps outside it.
const KIN: (f32, f32) = (0.4, 0.12);

/// Returns how well a species of `niche` holds a patch of kin key `key`: fully at the heart of its stand,
/// and a little everywhere, so stands mix at their edges.
fn kin(niche: f32, key: f32) -> f32 {
    (1.0 - (key - niche).abs() / KIN.0).max(KIN.1)
}

/// Returns, for the species `growers` in a patch of kin key `key`, the share of its group each one takes
/// there: its abundance times how well it holds the patch, over the same for its whole group.
pub fn stands(growers: &[Grower], key: f32) -> impl Fn(&Grower) -> f32 {
    let mut totals = [0f32; GROUPS];
    for g in growers {
        totals[g.group] += g.share * kin(g.niche, key);
    }
    move |g: &Grower| g.share * kin(g.niche, key) / totals[g.group].max(1e-6)
}

/// Returns how thick plants of the big pass when `big` grow at chart position `w` over `at`, `rise` metres
/// above the sea, and the kin key there: the cover, the moisture, coasts, slopes and the shade of big plants.
pub fn lift(ctx: &Ctx, (w, at): (Vec2, &Ground), rise: f32, big: bool) -> (f32, f32) {
    let [cover, key, wet] = ctx.cover.as_ref().map_or([1.0, 0.5, 0.5], |c| c.at(w));
    let under = if big { 1.0 } else { shade(&ctx.canopy, w) };
    (thickness(cover, wet) * edge(at, rise, big) * under, key)
}
