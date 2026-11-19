//! What the hands do to the map besides clicking: the wheel, the trackpad and the pinch.
//!
//! The wheel zooms, by lines or by the pixels of a trackpad or a smooth wheel, and so does a
//! pinch; a drag pans. The zoom stays inside the level shown: a double click goes into a place,
//! and Back comes out of it.

use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::map::SIZE;
use super::pointer::ZOOM_STEP;

/// Pixels of a trackpad or smooth wheel scroll that zoom as much as one line of the wheel.
pub const PIXELS_PER_NOTCH: f32 = 100.0;

/// What the hands did to the map lately: how fast it drifts after a drag and the pinch of
/// this frame.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Hand {
    /// Speed of the map in map pixels per second, kept while dragging and coasting after.
    pub drift: Vec2,
    /// Pinch of this frame: the share the trackpad gesture grew by, closer when positive.
    pub pinch: f32,
}

/// Returns the notches the wheel turned this frame, closer when positive. A mouse wheel gives
/// lines; a trackpad or a smooth wheel gives pixels, of which [`PIXELS_PER_NOTCH`] make one.
pub fn turn(scroll: &AccumulatedMouseScroll) -> f32 {
    match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_NOTCH,
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
    /// Returns the notches this frame zooms by, from the `wheel` and the pinch.
    pub fn notches(&self, wheel: f32) -> f32 {
        wheel + (1.0 + self.pinch).max(0.1).ln() / ZOOM_STEP.ln()
    }
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
