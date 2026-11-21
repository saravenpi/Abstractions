//! Spacing the marks of the compass: every mark is an icon on the one row of the bar at its
//! bearing, two that would overlap are nudged sideways by the least that clears them, and a crowd
//! of friends past a limit folds into the last one with a count.

use super::show::Seen;
use crate::menus::hud::needle::WIDTH;
use crate::menus::{MarkerKind, Markers};
use crate::ui::theme;

/// Width in pixels of the room of one mark, which is also the least distance between the centres
/// of two marks on the bar.
pub(super) const BASE: f32 = theme::ICON_SMALL + 2.0 * theme::S2;

/// Most friends the bar draws as icons. The ones past it fold into the last one drawn.
pub(super) const FRIENDS: usize = 5;

/// Moves the places `at`, in ascending order, the least that leaves [`BASE`] pixels between two
/// neighbours: the marks of a crowd spread evenly around where the crowd was.
pub(super) fn spread(at: &mut [f32]) {
    let mut blocks: Vec<(f32, usize)> = Vec::new();
    for (i, place) in at.iter().enumerate() {
        blocks.push((place - i as f32 * BASE, 1));
        while let [.., (a, n), (b, m)] = blocks[..] {
            if a / n as f32 <= b / m as f32 {
                break;
            }
            blocks.truncate(blocks.len() - 2);
            blocks.push((a + b, n + m));
        }
    }
    let mut i = 0;
    for (sum, count) in blocks {
        for _ in 0..count {
            at[i] = sum / count as f32 + i as f32 * BASE;
            i += 1;
        }
    }
}

/// Keeps the places `at`, in ascending order, with their room on the bar, moving them together
/// when the first is too far left or the last too far right.
pub(super) fn inside(at: &mut [f32]) {
    let reach = BASE * 0.5;
    let (Some(first), Some(last)) = (at.first().copied(), at.last().copied()) else {
        return;
    };
    let push = (reach - first).max(0.0) - (last - (WIDTH - reach)).max(0.0);
    at.iter_mut().for_each(|place| *place += push);
}

/// Folds the friends of `seen` past [`FRIENDS`], farthest first, into the last friend kept, and
/// counts them in its `more`.
fn fold(seen: &mut Vec<Seen>, markers: &Markers) {
    let friend = |mark: &Seen| markers.0[mark.index].kind == MarkerKind::Friend;
    let mut near: Vec<usize> = (0..seen.len()).filter(|&i| friend(&seen[i])).collect();
    near.sort_by(|&a, &b| seen[a].gap.total_cmp(&seen[b].gap));
    let Some(&kept) = near.get(FRIENDS.saturating_sub(1)) else {
        return;
    };
    seen[kept].more = near.len() - FRIENDS;
    let mut gone: Vec<usize> = near[FRIENDS..].to_vec();
    gone.sort_unstable_by(|a, b| b.cmp(a));
    gone.into_iter().for_each(|i| {
        seen.remove(i);
    });
}

/// Gives each of `seen` its place `at` on the bar: spread by bearing so no two overlap, then
/// ordered farthest first so the nearest is drawn in front.
pub(super) fn space(seen: &mut Vec<Seen>, markers: &Markers) {
    fold(seen, markers);
    seen.sort_by(|a, b| a.at.total_cmp(&b.at));
    let mut at: Vec<f32> = seen.iter().map(|mark| mark.at).collect();
    spread(&mut at);
    inside(&mut at);
    seen.iter_mut()
        .zip(at)
        .for_each(|(mark, place)| mark.at = place);
    seen.sort_by(|a, b| b.gap.total_cmp(&a.gap));
}
