//! Choosing the markers to draw and remembering what each slot drew last frame.

use bevy::prelude::*;

use super::place::{MAX, Placed, choose};
use super::rule::{self, Look, Sticky};
use super::tune::FRIEND_LIFT;
use crate::menus::hud::compass::{Around, FAVOURITE_REACH};
use crate::menus::hud::context::{Context, marker_shown};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::settings::WorldMarkers;

/// Pixels a marker on the border must come inside the safe rectangle before it goes back on the
/// screen: it stops the icon from flickering between the two forms.
const HYSTERESIS: f32 = 8.0;

/// One marker chosen to show this frame.
#[derive(Debug, Clone, Copy)]
pub struct Pick {
    /// Its index in [`Markers`].
    pub index: usize,
    /// The direction its icon is drawn in, in the camera frame.
    pub dir: Vec3,
    /// Its distance in metres.
    pub gap: f64,
}

/// What a slot drew last frame.
#[derive(Debug, Default, Clone, Copy)]
pub struct Memory {
    /// The marker it showed: its kind and index.
    pub who: Option<(MarkerKind, usize)>,
    /// Where its icon was, in pixels.
    pub pos: Vec2,
    /// Whether it sat on the border.
    pub edge: bool,
    /// Which of its fades have run to their end.
    pub sticky: Sticky,
}

/// Returns whether `marker` at `gap` metres is one the player asked to see at `level`, within
/// reach: at Minimal only what they follow and friends, and a pin they do not follow only
/// while it is close enough to walk to.
fn wanted(marker: &Marker, gap: f64, level: WorldMarkers) -> bool {
    let pin = matches!(marker.kind, MarkerKind::Waypoint | MarkerKind::Favourite);
    let kept = rule::tracked(marker) || marker.kind == MarkerKind::Friend;
    match level {
        WorldMarkers::Off => false,
        WorldMarkers::Minimal => kept,
        WorldMarkers::All => kept || !pin || gap <= FAVOURITE_REACH,
    }
}

/// Returns the direction the icon of a marker of `kind` is drawn in, for a target `gap` metres
/// away in `dir`: over the head of a friend, on the target itself for the rest.
fn lifted(kind: MarkerKind, dir: Vec3, gap: f64) -> Vec3 {
    match kind {
        MarkerKind::Friend => (dir * gap as f32 + Vec3::Y * FRIEND_LIFT)
            .try_normalize()
            .unwrap_or(dir),
        _ => dir,
    }
}

/// The markers of this frame and the memory of the slots. It lives in a `Local`, so nothing
/// is allocated per frame once the vectors have grown.
#[derive(Default)]
pub struct State {
    /// The markers worth drawing.
    picks: Vec<Pick>,
    /// For each pick, its rank and its distance.
    items: Vec<(u8, f64)>,
    /// The picks to draw, in slot order.
    order: Vec<usize>,
    /// What each slot drew last frame.
    pub memory: [Memory; MAX],
    /// The words being written.
    pub words: String,
}

impl State {
    /// Collects the markers the player asked for at `level` that show in `context`, and chooses
    /// at most [`MAX`] of them, by rank. Herds have no marker over the world: the animals are
    /// their own sign.
    pub fn gather(
        &mut self,
        around: &Around,
        markers: &Markers,
        (context, level): (Context, WorldMarkers),
    ) {
        self.picks.clear();
        self.items.clear();
        for (index, marker) in markers.0.iter().enumerate() {
            let shown = marker_shown(context, marker.kind) && marker.kind != MarkerKind::Herd;
            let Some((dir, gap)) = around.seen(marker).filter(|_| shown) else {
                continue;
            };
            if wanted(marker, gap, level) {
                let dir = lifted(marker.kind, dir, gap);
                self.picks.push(Pick { index, dir, gap });
                self.items.push((rule::rank(marker), gap));
            }
        }
        choose(&self.items, &mut self.order);
    }

    /// Returns the pick drawn in slot `at`, if any.
    pub fn chosen(&self, at: usize) -> Option<Pick> {
        self.order.get(at).map(|chosen| self.picks[*chosen])
    }

    /// Returns how far inside the safe rectangle the slot `at` must come to leave the border:
    /// it stays there until it is well inside.
    pub fn stickiness(&self, at: usize) -> f32 {
        [0.0, HYSTERESIS][usize::from(self.memory[at].edge)]
    }

    /// Records that slot `at` drew `who` as `placed`, and returns where its icon is drawn: it
    /// follows its target smoothly, and jumps and forgets its fades when the slot shows another
    /// marker.
    pub fn settle(
        &mut self,
        at: usize,
        who: (MarkerKind, usize),
        placed: Placed,
        follow: f32,
    ) -> Vec2 {
        let before = self.memory[at];
        let same = before.who == Some(who);
        let pos = match same {
            true => before.pos.lerp(placed.pos, follow),
            false => placed.pos,
        };
        self.memory[at] = Memory {
            who: Some(who),
            pos,
            edge: placed.arrow.is_some(),
            sticky: [Sticky::default(), before.sticky][usize::from(same)],
        };
        pos
    }

    /// Returns how near the marker of slot `at` is drawn in `context`, remembering its fades.
    pub fn look(&mut self, at: usize, marker: &Marker, gap: f64, context: Context) -> Look {
        rule::look(marker, gap, context, &mut self.memory[at].sticky)
    }
}
