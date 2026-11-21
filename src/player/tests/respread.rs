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

/// Gliding east by north-east with the view north, 70 degrees off, the wings folded at speed
/// and spread again a second later: folded, the body turns to the view; spread again, the wings
/// open toward the view and the path turns onto it within two seconds, the nose never swinging
/// back, and once the path is north it stays north. The velocity never jumps. Before, the wings
/// opened again along the old path, whatever the view.
#[test]
fn spreading_again_opens_toward_the_view() {
    let mut app = launch(Vec3::new(30.0, -7.0, -11.0), 2.0);
    let mut last = body(&mut app, PlayerId::LOCAL).2.state.velocity;
    let (mut kick, mut reached) = (0.0_f32, None);
    for frame in 0..120 {
        let asked: &[Intent] = if frame % 30 == 0 && frame < 60 {
            &[Intent::Wings]
        } else {
            &[]
        };
        let (_, gear, _) = hold(&mut app, 1, asked);
        kick = kick.max((gear.state.velocity - last).length());
        last = gear.state.velocity;
        let (nose, path) = (wrap(gear.state.servo.yaw), wrap(heading(&gear)));
        let what = format!("frame {frame}: nose {nose}, path {path}");
        assert!(
            frame < 36 || nose.abs() < 0.2,
            "{what}: the nose swung away"
        );
        reached = reached.or((frame > 30 && path.abs() < 0.1).then_some(frame));
        assert!(
            reached.is_none() || path.abs() < 0.15,
            "{what}: the path went back"
        );
    }
    assert!(
        reached.is_some_and(|frame| frame <= 90),
        "reached the view at {reached:?}"
    );
    assert_eq!(body(&mut app, PlayerId::LOCAL).2.state.mode, Mode::Glide);
    assert!(kick < KICK, "the velocity jumped by {kick} m/s in a frame");
}

/// Holding W dives and gains speed, the path steep but never past the vertical, and the body
/// never on its back.
#[test]
fn holding_w_dives_and_gains_speed() {
    let mut app = launch(Vec3::new(0.0, -7.0, -30.0), 3.0);
    let start = body(&mut app, PlayerId::LOCAL).2.state.airspeed;
    let (_, gear, _) = hold(&mut app, 150, &[keys(1.0, 0.0)]);
    let (fast, steep) = (gear.state.airspeed, path(&gear));
    assert!(fast > start + 15.0, "{fast} m/s from {start}");
    assert!(steep < -0.5 && steep > -1.58, "path {steep}");
    assert!((gear.state.frame() * Vec3::Y).y > -0.3, "on its back");
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
/// they may take: within 4 s the path is back within 20 degrees of the horizon.
#[test]
fn a_slow_spread_pulls_out_of_its_dive_soon() {
    let mut app = launch(Vec3::new(0.0, -6.0, -3.0), 0.0);
    let (_, gear, _) = hold(&mut app, 240, &[]);
    let angle = path(&gear).to_degrees();
    assert!(angle > -20.0, "still diving at {angle} degrees");
    assert!(gear.state.airspeed > 15.0, "{} m/s", gear.state.airspeed);
}
