//! Settling the marks of the compass: every mark is an icon at its true bearing with its
//! distance under it, and none is moved off it. Where two icons meet, the one that ranks first,
//! then the nearer, is drawn whole in front and the other at [`BEHIND`] of its opacity behind
//! it, its distance dimmed with it. A distance that would print over the distance of a mark that
//! settled before it is left out. A crowd of friends past a limit folds into the nearest one
//! with a count.

use std::cmp::Ordering;

use super::show::Seen;
use crate::menus::{MarkerKind, Markers};
use crate::ui::theme;
use crate::ui::units::distance_short;

/// Width in pixels of the room of one icon on the bar: two icons whose centres are nearer than
/// this meet, and the one behind is dimmed.
pub(super) const BASE: f32 = theme::ICON_SMALL + 2.0 * theme::S2;

/// Share of its opacity, from 0 to 1, a mark keeps when it is drawn behind another, over the
/// world and on the compass: faint enough to read as behind, clear enough to be seen.
pub(in crate::menus::hud) const BEHIND: f32 = 0.4;

/// Most friends the bar draws as icons. The ones past it fold into the nearest one.
pub(super) const FRIENDS: usize = 5;

/// Half-width in pixels of the hysteresis band keeping distance labels from flickering near
/// the overlap threshold.
pub(super) const HYSTERESIS: f32 = 4.0;

/// Returns the width in pixels the distance under a mark `gap` metres away takes on the bar,
/// with the gap that keeps two apart.
pub(super) fn wide(gap: f64) -> f32 {
    let chars = (distance_short(gap).chars().count() as f32).max(5.0);
    chars * theme::FONT_SMALL * theme::GLYPH_ASPECT + theme::S2
}

/// Folds the friends of `seen` past [`FRIENDS`], farthest first, into the nearest friend, and
/// counts them in its `more`.
fn fold(seen: &mut Vec<Seen>, markers: &Markers) {
    let friend = |mark: &Seen| markers.0[mark.index].kind == MarkerKind::Friend;
    let mut near: Vec<usize> = (0..seen.len()).filter(|&i| friend(&seen[i])).collect();
    if near.len() <= FRIENDS {
        return;
    }
    near.sort_by(|&a, &b| seen[a].gap.total_cmp(&seen[b].gap));
    seen[near[0]].more = near.len() - FRIENDS;
    let mut gone: Vec<usize> = near[FRIENDS..].to_vec();
    gone.sort_unstable_by(|a, b| b.cmp(a));
    gone.into_iter().for_each(|i| {
        seen.remove(i);
    });
}

/// Returns which of `a` and `b`, two marks of `markers`, settles first and is drawn in front:
/// the one that ranks first, then the nearer.
fn first(a: &Seen, b: &Seen, markers: &Markers) -> Ordering {
    let rank = |mark: &Seen| markers.0[mark.index].rank();
    let step = |mark: &Seen| (mark.gap * 0.5).round();
    rank(a)
        .cmp(&rank(b))
        .then_with(|| step(a).total_cmp(&step(b)))
        .then_with(|| a.index.cmp(&b.index))
}

/// Settles `seen`, the first to settle first: a mark whose icon meets the icon of one before it
/// is dimmed to [`BEHIND`], and its distance is left out when it would meet a distance already
/// said, keeping a [`HYSTERESIS`] pixel band around the threshold for marks listed in `was_said`.
pub(super) fn settle(seen: &mut [Seen], was_said: &[usize]) {
    for i in 0..seen.len() {
        let (before, rest) = seen.split_at_mut(i);
        let mark = &mut rest[0];
        if before.iter().any(|other| (other.at - mark.at).abs() < BASE) {
            mark.alpha *= BEHIND;
        }
        let margin = if was_said.contains(&mark.index) {
            -HYSTERESIS
        } else {
            HYSTERESIS
        };
        let (at, room) = (mark.at, mark.wide);
        let clear = |other: &Seen| (other.at - at).abs() >= (other.wide + room) * 0.5 + margin;
        mark.said = before.iter().filter(|other| other.said).all(clear);
    }
}

/// Settles every mark of `seen` at its bearing with `was_said` hysteresis on distance labels,
/// and orders them last to first, so the pool, which draws its later marks over its earlier ones,
/// puts the first in front.
pub(super) fn space_steady(seen: &mut Vec<Seen>, markers: &Markers, was_said: &[usize]) {
    fold(seen, markers);
    seen.iter_mut().for_each(|mark| mark.wide = wide(mark.gap));
    seen.sort_by(|a, b| first(a, b, markers));
    settle(seen, was_said);
    seen.reverse();
}

/// Settles every mark of `seen` at its bearing without prior frame history and orders them
/// last to first, so the pool, which draws its later marks over its earlier ones, puts the
/// first in front.
pub(super) fn space(seen: &mut Vec<Seen>, markers: &Markers) {
    space_steady(seen, markers, &[]);
}
