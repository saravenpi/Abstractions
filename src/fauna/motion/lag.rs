//! The gait table: how far behind the stride clock each leg lifts, by leg count, rank and side.
//!
//! A lag is a share of a stride. A foot lifts when the clock passes its duty factor plus its lag,
//! so a larger lag lifts later. Bipeds alternate. Quadrupeds walk a lateral sequence (left hind,
//! left fore, right hind, right fore, a quarter apart), trot in diagonal pairs and gallop with the
//! hind pair then the fore pair. Hexapods run an alternating tripod. Longer bodies carry a
//! metachronal wave from the back to the front, the two sides half a stride apart.

use super::gait::Mix;

/// Lags of a quadruped by gait (walk, trot, gallop), rank (fore, hind) and side (left, right).
const QUADRUPED: [[[f32; 2]; 2]; 3] = [
    [[0.25, 0.75], [0.0, 0.5]],
    [[0.5, 0.0], [0.0, 0.5]],
    [[0.5, 0.6], [0.0, 0.1]],
];

/// Lags of a biped by side, the same in every gait.
const BIPED: [f32; 2] = [0.0, 0.5];

/// Where a leg sits on its body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seat {
    /// Number of legs on the body.
    pub legs: usize,
    /// Number of ranks from front to back.
    pub ranks: u8,
    /// Rank of the leg, 0 at the front.
    pub rank: u8,
    /// -1 left, 1 right.
    pub side: i8,
}

/// Returns the lag of the leg at `seat` in gait `gait` (0 walk, 1 trot, 2 gallop).
pub fn of(seat: Seat, gait: usize) -> f32 {
    let side = usize::from(seat.side > 0);
    let rank = usize::from(seat.rank);
    match seat.legs {
        2 => BIPED[side],
        4 => QUADRUPED[gait][rank.min(1)][side],
        6 => 0.5 * ((rank + side) % 2) as f32,
        _ => {
            let ranks = f32::from(seat.ranks.max(1));
            let back = ranks - 1.0 - rank as f32;
            (back / ranks + 0.5 * side as f32).rem_euclid(1.0)
        }
    }
}

/// Returns `to` moved onto the turn of `from`, so that the two are less than half a stride apart.
fn near(from: f32, to: f32) -> f32 {
    let gap = to - from;
    from + gap - gap.round()
}

/// Returns the lag of the leg at `seat` in the gait blend `mix`, along the short way round.
pub fn blended(seat: Seat, mix: &Mix) -> f32 {
    let walk = of(seat, 0);
    let steady = walk + (near(walk, of(seat, 1)) - walk) * mix.trot;
    let lag = steady + (near(steady, of(seat, 2)) - steady) * mix.gallop;
    lag.rem_euclid(1.0)
}
