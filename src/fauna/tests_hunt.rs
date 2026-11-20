//! Tests of hunts as drawn: hunters close in on the timeline of a hunt, the kill leaves a carcass
//! where it fell, and scavengers come to it.

use super::rig::{SEED, kill, planet};
use super::stage::{run, stage, staying};
use super::tune::VISIT_DELAY;
use super::want::{Lead, drawn};
use crate::life::{Guild, Species};

/// Returns true when `s` eats carcasses and has a body to draw.
fn scavenges(s: &Species) -> bool {
    matches!(s.role.guild, Guild::Scavenger | Guild::Decomposer) && drawn(s)
}

/// The hunters of a kill are drawn near the kill point as the chase ends, then the carcass lies there,
/// with its visitors once they have had time to come.
#[test]
fn hunts_show_on_the_timeline() {
    let (body, globe, life) = planet(true);
    let life = life.expect("the rig's home planet lives");
    let hunt = kill(&globe, &life, body).expect("a kill near the rig's spots");
    println!("seed {SEED}: a kill at {:.0} s", hunt.outcome_time());
    let mut app = stage(globe, body, hunt.to, hunt.outcome_time() - 3.0);
    run(&mut app, 2.0, 0.1);
    let hunters = staying(&mut app)
        .into_iter()
        .filter(|b| b.lead.key().0 == hunt.hunter);
    assert!(hunters.count() > 0, "no hunter drawn on the chase");
    run(&mut app, VISIT_DELAY as f32 + 6.0, 0.1);
    let beasts = staying(&mut app);
    assert!(
        beasts
            .iter()
            .any(|b| matches!(b.lead, Lead::Carcass { .. })),
        "no carcass after the kill"
    );
    let visits = beasts
        .iter()
        .filter(|b| matches!(b.lead, Lead::Visit { .. }))
        .count();
    let scavengers = life.species.iter().any(scavenges);
    println!("{visits} visitors at the carcass");
    assert!(
        !scavengers || visits > 0,
        "no scavenger came to the carcass"
    );
}
