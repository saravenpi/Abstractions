//! The frame of a ship in a save file: the text older saves hold, written when the save kept
//! its own copy of the frame, still loads into the frame of the game.

use bevy::math::IVec3;

use crate::ship::ShipState;
use crate::space::Frame;
use crate::universe::{BodyId, GalaxyId, StarId};

/// The fields of a ship record that are not its frame.
const REST: &str =
    "pos = [1.0, 2.0, 3.0]\nvel = [0.0, 0.0, 0.0]\nrot = [0.0, 0.0, 0.0, 1.0]\nlanded = true\n";

/// The star the saved frames below name.
fn star() -> StarId {
    StarId {
        galaxy: GalaxyId(IVec3::new(1, 2, -3)),
        sector: IVec3::new(5, -4, 17),
        index: 2,
    }
}

/// Reads a record with the frame text `frame`, written as the save wrote it, and checks that
/// it is `expected`, and that it survives being written and read again.
fn reads_as(frame: &str, expected: Frame) {
    let text = format!("{REST}{frame}");
    let state: ShipState = toml::from_str(&text).unwrap();
    assert_eq!(state.frame, expected);
    let again: ShipState = toml::from_str(&toml::to_string(&state).unwrap()).unwrap();
    assert_eq!(again, state);
}

/// A save written with a surface frame, a system frame and a galaxy frame loads as it did.
#[test]
fn saved_frames_load_as_they_were_written() {
    let body = BodyId {
        star: star(),
        index: 3,
        moon: 0,
    };
    let surface = "\n[frame.Surface]\nindex = 3\nmoon = 0\n\n[frame.Surface.star]\n\
        galaxy = [1, 2, -3]\nsector = [5, -4, 17]\nindex = 2\n";
    let system = "\n[frame.System]\ngalaxy = [1, 2, -3]\nsector = [5, -4, 17]\nindex = 2\n";
    let galaxy = "\n[frame]\nGalaxy = [3, -1, 8]\n";
    reads_as(surface, Frame::Surface(body));
    reads_as(system, Frame::System(star()));
    reads_as(galaxy, Frame::Galaxy(GalaxyId(IVec3::new(3, -1, 8))));
}
