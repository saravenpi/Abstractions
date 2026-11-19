//! The wingsuit flown by intents in the controller: folding keeps the momentum, spreading again
//! flies on where the body goes, the forward key dives, and the hands-off glide is about four
//! to one.

use bevy::prelude::*;

use super::glider::{heading, keys, launch, path};
use super::stage::body;
use super::wings::hold;
use crate::gear::Mode;
use crate::player::wrap;
use crate::sim::{Intent, PlayerId};

/// Largest change of velocity in one frame of a thirtieth of a second across a fold and a
/// spread, in metres per second: about 4 g, what the wings may pull.
const KICK: f32 = 3.0;

/// Gliding east with the view north, the wings folded at speed and spread again a second
/// later: the nose heading starts on the flight path and stays within 5 degrees of it, the path
/// keeps heading east, and the velocity never jumps.
#[test]
fn spreading_again_flies_on_where_the_body_goes() {
    let mut app = launch(Vec3::new(30.0, -7.0, 0.0), 2.0);
    let east = heading(&body(&mut app, PlayerId::LOCAL).2);
    let mut last = body(&mut app, PlayerId::LOCAL).2.state.velocity;
    let mut kick = 0.0_f32;
    for frame in 0..120 {
        let asked: &[Intent] = match frame {
            0 | 30 => &[Intent::Wings],
            _ => &[],
        };
        let (_, gear, _) = hold(&mut app, 1, asked);
        kick = kick.max((gear.state.velocity - last).length());
        last = gear.state.velocity;
        let off = wrap(gear.state.servo.yaw - heading(&gear)).to_degrees();
        assert!(
            off.abs() < 5.0,
            "frame {frame}: the nose is {off} degrees off the path"
        );
        assert!(
            wrap(heading(&gear) - east).abs() < 0.09,
            "frame {frame}: turned away"
        );
    }
    let gear = body(&mut app, PlayerId::LOCAL).2;
    assert_eq!(gear.state.mode, Mode::Glide);
    assert!(kick < KICK, "the velocity jumped by {kick} m/s in a frame");
}

/// Holding W dives and gains speed, the path no steeper than the dive limit.
#[test]
fn holding_w_dives_and_gains_speed() {
    let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 3.0);
    let start = body(&mut app, PlayerId::LOCAL).2.state.airspeed;
    let (_, gear, _) = hold(&mut app, 90, &[keys(1.0, 0.0)]);
    let (fast, steep) = (gear.state.airspeed, path(&gear));
    assert!(fast > start + 15.0, "{fast} m/s from {start}");
    assert!(steep > crate::gear::tune::DIVE_LIMIT - 0.05, "path {steep}");
}

/// Hands off, the glide settles at about four metres forward for each metre down.
#[test]
fn the_hands_off_glide_is_about_four_to_one() {
    let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 8.0);
    let v = body(&mut app, PlayerId::LOCAL).2.state.velocity;
    let ratio = v.with_y(0.0).length() / -v.y;
    println!(
        "hands-off glide ratio {ratio:4.2} at {:4.1} m/s",
        v.length()
    );
    assert!((3.5..=4.6).contains(&ratio), "ratio {ratio}");
}

/// Spread from a slow fall, hands off, the wings dive for speed and then pull out at the load
/// they may take: within 2.5 s the path is back within 20 degrees of the horizon.
#[test]
fn a_slow_spread_pulls_out_of_its_dive_soon() {
    let mut app = launch(Vec3::new(0.0, -6.0, -3.0), 0.0);
    let (_, gear, _) = hold(&mut app, 75, &[]);
    let angle = path(&gear).to_degrees();
    assert!(angle > -20.0, "still diving at {angle} degrees");
    assert!(gear.state.airspeed > 15.0, "{} m/s", gear.state.airspeed);
}
