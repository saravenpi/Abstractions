//! Choosing the markers to draw and remembering where each slot drew last frame.

use bevy::prelude::*;

use super::place::{MAX, Placed, choose};
use crate::menus::hud::compass::{Around, FAVOURITE_REACH};
use crate::menus::hud::context::{Context, marker_shown};
use crate::menus::{Marker, MarkerKind, Markers};
use crate::ui::theme;

/// Metres under which a marker is hidden: it is right here.
const HERE: f64 = 5.0;

/// Pixels a marker on the border must come inside the safe rectangle before it goes back on the
/// screen: it stops the icon from flickering between the two forms.
const HYSTERESIS: f32 = 8.0;

/// One marker chosen to show this frame.
#[derive(Debug, Clone, Copy)]
pub struct Pick {
    /// Its index in [`Markers`].
    pub index: usize,
    /// Its direction in the camera frame.
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
}

/// The adapter between the markers and the pin system: whether `marker` is the active pin,
/// which shows at any distance. Today it is the player's own waypoint. When `Places` lands,
/// replace the body by a comparison with `Places::active()`.
pub fn is_active(marker: &Marker) -> bool {
    marker.kind == MarkerKind::Waypoint && marker.owner == 0
}

/// Returns whether `marker` at `gap` metres is worth drawing: the course and the active pin at
/// any distance, other pins within reach, the rest unless right here.
fn worth(marker: &Marker, gap: f64) -> bool {
    let pin = matches!(marker.kind, MarkerKind::Waypoint | MarkerKind::Favourite);
    gap > HERE && (!pin || is_active(marker) || gap <= FAVOURITE_REACH)
}

/// Returns the colour a marker is drawn in: the accent for the course, the pin colour for pins,
/// its own for friends, white for home and the ship.
pub fn tint(marker: &Marker) -> Color {
    match marker.kind {
        MarkerKind::Course => theme::ACCENT,
        MarkerKind::Waypoint | MarkerKind::Favourite => theme::PIN,
        MarkerKind::Friend => marker.color,
        _ => theme::TEXT,
    }
}

/// The markers of this frame and the memory of the slots. It lives in a `Local`, so nothing
/// is allocated per frame once the vectors have grown.
#[derive(Default)]
pub struct State {
    /// The markers worth drawing.
    picks: Vec<Pick>,
    /// For each pick, whether it must show and its distance.
    items: Vec<(bool, f64)>,
    /// The picks to draw, in slot order.
    order: Vec<usize>,
    /// What each slot drew last frame.
    pub memory: [Memory; MAX],
    /// The text of the distance being written.
    pub words: String,
}

impl State {
    /// Collects the markers worth drawing in `context` and chooses at most [`MAX`] of them.
    pub fn gather(&mut self, around: &Around, markers: &Markers, context: Context) {
        self.picks.clear();
        self.items.clear();
        for (index, marker) in markers.0.iter().enumerate() {
            let shown = marker_shown(context, marker.kind) && marker.kind != MarkerKind::Herd;
            let Some((dir, gap)) = around.seen(marker).filter(|_| shown) else {
                continue;
            };
            if worth(marker, gap) {
                self.picks.push(Pick { index, dir, gap });
                let always = marker.kind == MarkerKind::Course || is_active(marker);
                self.items.push((always, gap));
            }
        }
        choose(&self.items, &mut self.order);
    }

    /// Returns the pick drawn in slot `at`, if any.
    pub fn chosen(&self, at: usize) -> Option<Pick> {
        self.order.get(at).map(|chosen| self.picks[*chosen])
    }

    /// Returns the extra inset the slot `at` needs to leave the border: it stays there until it
    /// is well inside.
    pub fn stickiness(&self, at: usize) -> f32 {
        [0.0, HYSTERESIS][usize::from(self.memory[at].edge)]
    }

    /// Records that slot `at` drew `kind` at `index` as `placed`, and returns where its icon is
    /// drawn: it follows its target smoothly, and jumps when the slot shows another marker.
    pub fn settle(
        &mut self,
        at: usize,
        who: (MarkerKind, usize),
        placed: Placed,
        follow: f32,
    ) -> Vec2 {
        let before = self.memory[at];
        let pos = match before.who == Some(who) {
            true => before.pos.lerp(placed.pos, follow),
            false => placed.pos,
        };
        self.memory[at] = Memory {
            who: Some(who),
            pos,
            edge: placed.arrow.is_some(),
        };
        pos
    }
}
