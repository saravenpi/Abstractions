//! How the camera frames the body.

use crate::gear::FlightState;
use crate::gear::Mode;
use crate::player::eyes::*;
use crate::view::Kind;

/// Wings out or a fast fall are framed as flying; walking, the boots and a slide as on foot.
#[test]
fn the_camera_frames_flight_and_fast_falls_as_flying() {
    let at = |mode: Mode, airspeed: f32| {
        kind(&FlightState {
            mode,
            airspeed,
            ..FlightState::default()
        })
    };
    assert_eq!(at(Mode::Glide, 5.0), Kind::Flying);
    assert_eq!(at(Mode::Fly, 60.0), Kind::Flying);
    assert_eq!(at(Mode::Fall, 40.0), Kind::Flying);
    assert_eq!(at(Mode::Fall, 6.0), Kind::OnFoot);
    assert_eq!(at(Mode::BootFall, 6.0), Kind::OnFoot);
    assert_eq!(at(Mode::Slide, 30.0), Kind::OnFoot);
    assert_eq!(at(Mode::Off, 30.0), Kind::OnFoot);
}
