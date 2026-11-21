//! The rules of one marker over the world, as pure functions: how near its target is for the
//! thing itself to be the sign, how it fades over a band and comes back farther out, what the
//! middle of the view does to it and what its icon and its words look like.

use super::tune;
use crate::menus::hud::context::Context;
use crate::menus::{Marker, MarkerKind};

/// Whether each fade of a marker has run to its end: it comes back only farther out.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Sticky {
    /// The marker lost its words and its full size.
    pub reduced: bool,
    /// The marker is gone.
    pub hidden: bool,
}

/// How near a marker is drawn, before the middle of the view and its neighbours have their say.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// How visible the whole marker is, from 0 (gone) to 1.
    pub alpha: f32,
    /// How whole it is, from 0 (reduced: a small icon without words) to 1.
    pub detail: f32,
}

/// What is drawn of a marker: its icon and its words.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// Opacity of the icon and its arrow, from 0 to 1.
    pub glyph: f32,
    /// Opacity of the words and their strip, from 0 to 1.
    pub label: f32,
    /// Whether the icon is drawn small.
    pub small: bool,
}

/// Returns whether the player follows `marker` on purpose: the course, or their own pin. Such
/// a marker is never reduced and hides only on arrival.
pub fn tracked(marker: &Marker) -> bool {
    marker.kind == MarkerKind::Course || (marker.kind == MarkerKind::Waypoint && marker.owner == 0)
}

/// Returns the place of `marker` in the order that settles who keeps its room when two
/// markers meet: the course, the followed pin, friends, home, the ship, pins, herds.
pub fn rank(marker: &Marker) -> u8 {
    match marker.kind {
        MarkerKind::Course => 0,
        MarkerKind::Waypoint if marker.owner == 0 => 1,
        MarkerKind::Friend => 2,
        MarkerKind::Home => 3,
        MarkerKind::Ship => 4,
        MarkerKind::Waypoint | MarkerKind::Favourite => 5,
        MarkerKind::Herd => 6,
    }
}

/// Returns the length in metres of what a marker of `kind` stands on, as seen from `context`:
/// the real size of the target, counted larger the faster the player travels.
pub fn span(kind: MarkerKind, context: Context) -> f64 {
    let size = match kind {
        MarkerKind::Home => tune::HOME_SPAN,
        MarkerKind::Ship => tune::SHIP_SPAN,
        _ => tune::BODY_SPAN,
    };
    let reach = match context {
        Context::Gliding => tune::GLIDE_REACH,
        Context::ShipLanded => 1.0,
        _ if context.is_ship() => tune::FLIGHT_REACH,
        _ => 1.0,
    };
    size * reach
}

/// Returns how visible something `gap` metres away is when it fades out under `inner` metres:
/// 1 past the band, 0 under it, smooth between. Once the fade has run to 0, `stuck` moves the
/// band out by [`tune::STICK`] until the fade is back at 1, so nothing flickers on a limit.
pub fn fade(gap: f64, inner: f64, stuck: &mut bool) -> f32 {
    if inner <= 0.0 {
        return 1.0;
    }
    let lo = inner * [1.0, tune::STICK][usize::from(*stuck)];
    let t = ((gap - lo) / (lo * (tune::BAND - 1.0))).clamp(0.0, 1.0) as f32;
    if t <= 0.0 {
        *stuck = true;
    } else if t >= 1.0 {
        *stuck = false;
    }
    t * t * (3.0 - 2.0 * t)
}

/// Returns how `marker`, `gap` metres away, is drawn in `context`, and updates what it
/// remembers. A marker the player follows keeps its words and hides only on arrival; the
/// course, which is never reached on foot, never hides. A friend is never hidden: near, only
/// the name is left. Any other marker is reduced, then hidden, as its target grows in view.
pub fn look(marker: &Marker, gap: f64, context: Context, sticky: &mut Sticky) -> Look {
    if tracked(marker) {
        let arrive = [tune::ARRIVE, 0.0][usize::from(marker.kind == MarkerKind::Course)];
        let alpha = fade(gap, arrive, &mut sticky.hidden);
        return Look { alpha, detail: 1.0 };
    }
    let unit = span(marker.kind, context);
    let detail = fade(gap, unit * tune::REDUCE_SPANS, &mut sticky.reduced);
    let alpha = match marker.kind {
        MarkerKind::Friend => 1.0,
        _ => fade(gap, unit * tune::HIDE_SPANS, &mut sticky.hidden),
    };
    Look { alpha, detail }
}

/// Returns the opacity of a marker drawn on its target `angle` radians from the middle of the
/// view: faint in the middle, whole to the side, smooth between.
pub fn centre(angle: f32) -> f32 {
    let (inner, outer) = (tune::CENTRE.0.to_radians(), tune::CENTRE.1.to_radians());
    let t = ((angle - inner) / (outer - inner)).clamp(0.0, 1.0);
    tune::CENTRE_ALPHA + (1.0 - tune::CENTRE_ALPHA) * t * t * (3.0 - 2.0 * t)
}

/// Returns what is drawn of a marker of `kind` that looks like `look`: on its target at `dim`
/// of its opacity, or on the border of the screen for no `dim`. A friend near and in view is a
/// name alone, over their head; on the border they keep their icon and its arrow. Any other
/// reduced marker is a small icon without words.
pub fn face(kind: MarkerKind, look: Look, dim: Option<f32>) -> Face {
    let alpha = look.alpha * dim.unwrap_or(1.0);
    if kind == MarkerKind::Friend {
        let near = [1.0, look.detail][usize::from(dim.is_some())];
        return Face {
            glyph: alpha * near,
            label: alpha,
            small: false,
        };
    }
    Face {
        glyph: alpha * (tune::REDUCED_ALPHA + (1.0 - tune::REDUCED_ALPHA) * look.detail),
        label: alpha * look.detail,
        small: look.detail < 0.5,
    }
}
