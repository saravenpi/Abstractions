//! The touchdown of a glide over the ground of the start.

use super::Game;

/// Frames a glide in over the ground of the start may take to touch down: ten seconds, enough for a slope
/// that falls away along the run.
const GLIDE: usize = 300;

/// Runs frames until the body is out of the air, and fails with the state of the body when it is not
/// within [`GLIDE`] frames.
pub fn touches_down(game: &mut Game) {
    for _ in 0..GLIDE {
        if !game.body().gear.state.mode.airborne() {
            return;
        }
        game.step(1);
    }
    let body = game.body();
    let down = !body.gear.state.mode.airborne();
    assert!(
        down,
        "the body to touch down: not within {GLIDE} frames, {body:?}"
    );
}
