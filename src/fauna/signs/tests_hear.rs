//! Tests of the hearing: calls fade with distance and carry farther for big animals, no frame sends
//! more than [`CALLS`] calls, and every call heard comes from a herd that is drawn.

use bevy::prelude::*;

use super::Sung;
use super::hear::level;
use super::stage::app;
use super::tune::{CALLS, HEAR};
use crate::fauna::Dead;
use crate::fauna::beast::Beast;
use crate::fauna::stage::run;
use crate::fauna::want::Lead;
use crate::life::signs::calls::Cry;
use crate::space::Clock;

/// A call is quieter farther away and louder from a bigger animal at the same distance.
#[test]
fn calls_fade_with_distance_and_carry_with_size() {
    assert!(level(40.0, 10.0) > level(40.0, 80.0));
    assert!(level(400.0, 80.0) > level(40.0, 80.0));
    assert!((level(1.0, 12.0) - 0.5).abs() < 1e-6);
}

/// Over two minutes in a herd, the hearing sends some calls and never more than [`CALLS`] a frame.
#[test]
fn calls_per_frame_are_bounded() {
    let (mut app, _) = app(true);
    run(&mut app, 120.0, 0.25);
    let sent = &app.world().resource::<super::stage::Sent>().0;
    let total: usize = sent.iter().sum();
    println!("{total} calls in {} frames", sent.len());
    assert!(sent.iter().all(|n| *n <= CALLS));
    assert!(total > 0, "a herd stayed silent for two minutes");
}

/// Returns the herds with a living member drawn in `world`.
fn drawn_herds(world: &mut World) -> Vec<u64> {
    let mut beasts = world.query_filtered::<&Beast, Without<Dead>>();
    let herd = |b: &Beast| match b.lead {
        Lead::Herd { id, .. } => Some(id),
        _ => None,
    };
    beasts.iter(world).filter_map(herd).collect()
}

/// Over a minute in a herd, every call heard in a hearing, but the death cry of an animal just killed,
/// comes from a herd with a living member drawn at that moment: what the player hears, they can find.
#[test]
fn every_call_heard_comes_from_a_drawn_herd() {
    let (mut app, _) = app(true);
    let mut checked = 0;
    for _ in 0..240 {
        run(&mut app, HEAR, HEAR);
        let world = app.world_mut();
        let drawn = drawn_herds(world);
        let now = world.resource::<Clock>().seconds;
        let recent = |c: &crate::life::signs::calls::Calling| {
            c.at > now - f64::from(HEAR) && c.cry != Cry::Death
        };
        for (call, _) in world
            .resource::<Sung>()
            .calls
            .iter()
            .filter(|(c, _)| recent(c))
        {
            assert!(
                drawn.contains(&call.herd),
                "herd {} heard at {} s, none of it drawn",
                call.herd,
                call.at
            );
            checked += 1;
        }
    }
    println!("{checked} calls heard, every one from a drawn herd");
    assert!(checked > 0, "no call heard in a minute");
}
