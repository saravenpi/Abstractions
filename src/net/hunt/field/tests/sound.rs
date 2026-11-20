//! The sound of a blow on the network: its bytes and the impact that goes out and comes back.

use abstractions_net::wire::Act;

use super::super::codec::{outer_code, outer_of, tool_code, tool_of};
use super::rig::{acts, app, impact, written};
use crate::audio::{Body, Impact, Outer};
use crate::net::hunt::Echoes;

/// The layer and the head of a tool come back from their bytes within a power of two.
#[test]
fn a_sound_survives_its_bytes() {
    assert_eq!(outer_of(outer_code(Outer::Horn)), Outer::Horn);
    let tool = Body {
        stiffness: 70.0,
        mass: 0.5,
    };
    let back = tool_of(tool_code(&tool));
    assert!((back.stiffness / tool.stiffness - 1.0).abs() < 0.5);
    assert!((back.mass / tool.mass - 1.0).abs() < 0.5);
}

/// An impact of this machine goes out once with its place on the planet, and the same impact told
/// back by the server is played and not sent again.
#[test]
fn an_impact_goes_out_and_the_one_that_comes_back_is_played() {
    let (mut app, script) = app();
    app.world_mut().write_message(impact());
    app.update();
    let all = acts(&script);
    let [Act::Impact(sent)] = all.as_slice() else {
        panic!("one impact");
    };
    assert_eq!((sent.joules, sent.cut), (40.0, true));
    assert_eq!(sent.outer, outer_code(Outer::Bone));
    app.world_mut().resource_mut::<Echoes>().impacts.push(*sent);
    app.update();
    let played = written::<Impact>(&app);
    assert_eq!(played.len(), 2);
    assert_eq!(played[1].outer, Outer::Bone);
    assert!(played[1].at.distance(impact().at) < 0.01);
    assert_eq!(acts(&script).len(), 1);
}
