//! What the hands do to the map besides clicking: the wheel, the trackpad and the pinch.
//!
//! A wheel turns by lines and zooms. A trackpad scrolls by pixels and pans the map, as a drag
//! does; with Ctrl held, or with a pinch, it zooms. The zoom stays inside the level shown: a
//! double click goes into a place, and Back comes out of it.

use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::map::SIZE;
use super::pointer::ZOOM_STEP;

/// Pixels of a trackpad scroll with Ctrl held that zoom as much as one line of the wheel.
pub const PIXELS_PER_NOTCH: f32 = 100.0;

/// What one frame of the wheel or the trackpad asks of the map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    /// Zoom by this many notches, closer when positive.
    Zoom(f32),
    /// Pan by these screen pixels, as a drag by them would.
    Pan(Vec2),
}

/// What the hands did to the map lately: how fast it drifts after a drag and the pinch of
/// this frame.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Hand {
    /// Speed of the map in map pixels per second, kept while dragging and coasting after.
    pub drift: Vec2,
    /// Pinch of this frame: the share the trackpad gesture grew by, closer when positive.
    pub pinch: f32,
}

/// Returns what the wheel asks this frame. Lines zoom. Pixels pan, or zoom with Ctrl held.
pub fn turn(scroll: &AccumulatedMouseScroll, ctrl: bool) -> Option<Turn> {
    let turn = match (scroll.unit, ctrl) {
        (MouseScrollUnit::Line, _) => Turn::Zoom(scroll.delta.y),
        (MouseScrollUnit::Pixel, true) => Turn::Zoom(scroll.delta.y / PIXELS_PER_NOTCH),
        (MouseScrollUnit::Pixel, false) => Turn::Pan(scroll.delta),
    };
    match turn {
        Turn::Zoom(0.0) => None,
        Turn::Pan(pixels) if pixels == Vec2::ZERO => None,
        turn => Some(turn),
    }
}

/// Returns the map pixels that `pixels` of the screen are on a map `side` pixels high.
pub fn to_map(pixels: Vec2, side: f32) -> Vec2 {
    match side > 0.0 {
        true => pixels * SIZE / side,
        false => pixels,
    }
}

impl Hand {
    /// Returns the notches this frame zooms by, from the wheel and the pinch.
    pub fn notches(&self, turn: Option<Turn>) -> f32 {
        let wheel = match turn {
            Some(Turn::Zoom(notches)) => notches,
            _ => 0.0,
        };
        wheel + (1.0 + self.pinch).max(0.1).ln() / ZOOM_STEP.ln()
    }
}

/// Returns true while a Ctrl key is held.
pub fn ctrl(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
}

/// Adds up the pinch of this frame.
fn gather(mut pinches: MessageReader<PinchGesture>, mut hand: ResMut<Hand>) {
    let pinch = pinches.read().map(|pinch| pinch.0).sum();
    if hand.pinch != pinch {
        hand.pinch = pinch;
    }
}

/// Registers the hands, which start still each time the map opens.
pub(super) fn plugin(app: &mut App) {
    let still = |mut hand: ResMut<Hand>| *hand = Hand::default();
    app.init_resource::<Hand>()
        .add_systems(OnEnter(super::Computer::Open), still)
        .add_systems(
            Update,
            gather
                .before(super::Input)
                .run_if(super::open::ready)
                .run_if(resource_exists::<Messages<PinchGesture>>),
        );
}
