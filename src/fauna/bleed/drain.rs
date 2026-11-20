//! Draining: every open wound lets blood out each frame, throws it as drops, lowers the health of
//! the animal by the blood it lost and closes as it clots.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::throw::{jet_of, throw};
use super::tune::{EMIT, PARCEL, PLUG_RANGE, RATE};
use super::wound::{Circuit, Cut, Wound};
use crate::avatar::strike::{Rest, Spear};
use crate::craft::matter::Grade;
use crate::effects::Fling;
use crate::fauna::attack::{Dead, slay};
use crate::fauna::beast::Beast;
use crate::fauna::herds::Herds;
use crate::life::blood::{pulse, species as blood_of, speed, tune::LETHAL};
use crate::settings::{Persistent, Settings};
use crate::space::Clock;
use crate::world::Terrain;

/// Advances every cut of `wound` by `dt` seconds under `circuit`, calling `emit` with the cut and the
/// volume of each drop it throws, and returns the blood lost this step in cubic metres and the mean
/// flow of all the cuts in cubic metres per second. A cut whose flow stopped is closed.
///
/// A cut throws at most [`RATE`] drops a second and [`EMIT`] a frame: a stronger flow throws bigger
/// drops, and the blood is lost all the same.
pub fn advance(
    wound: &mut Wound,
    circuit: &Circuit,
    dt: f32,
    plugged: impl Fn(&Cut) -> bool,
    mut emit: impl FnMut(&Cut, f32),
) -> (f32, f32) {
    let (mut lost, mut mean) = (0.0, 0.0);
    for slot in wound.cuts.iter_mut() {
        let Some(cut) = slot.as_mut() else { continue };
        cut.age += dt;
        let q = circuit.rate(cut, wound.lost, cut.plugged || plugged(cut));
        if q <= 0.0 {
            *slot = None;
            continue;
        }
        let beat = if cut.major {
            pulse(cut.age, circuit.beat)
        } else {
            1.0
        };
        cut.owed += q * beat * dt;
        let parcel = PARCEL.max(q / RATE);
        let mut thrown = 0;
        while cut.owed >= parcel && thrown < EMIT {
            emit(cut, parcel);
            cut.owed -= parcel;
            thrown += 1;
        }
        cut.owed = cut.owed.min(parcel);
        lost += q * beat * dt;
        mean += q;
    }
    (lost, mean)
}

/// What draining reads and writes besides the animals.
#[derive(SystemParam)]
pub struct Drain<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    settings: Res<'w, Persistent<Settings>>,
    clock: Res<'w, Clock>,
    herds: ResMut<'w, Herds>,
    flings: MessageWriter<'w, Fling>,
    spears: Query<'w, 's, (&'static Spear, &'static Transform, &'static ChildOf)>,
}

/// Returns whether a cut is plugged: a spear stuck in the animal `entity` has its point within
/// [`PLUG_RANGE`] of it.
fn plugs<'a>(
    spears: &'a Query<(&Spear, &Transform, &ChildOf)>,
    entity: Entity,
) -> impl Fn(&Cut) -> bool + 'a {
    move |cut| {
        spears.iter().any(|(spear, at, of)| {
            of.parent() == entity
                && spear.rest == Rest::Stuck
                && at.translation.distance(cut.at) < PLUG_RANGE
        })
    }
}

/// Takes `lost` cubic metres of blood off the health of `beast`, a share of its health that is the
/// share of its blood: the lethal loss is all of it. An animal with none left is slain.
fn weaken(d: &mut Drain, (entity, beast): (Entity, &mut Beast), (lost, volume): (f32, f32)) {
    let harm = beast.wits.health.max * lost / (volume * LETHAL).max(f32::EPSILON);
    if beast.wits.health.hurt(harm) {
        let now = d.clock.seconds;
        let (herds, terrain) = (&mut *d.herds, &*d.terrain);
        slay(
            &mut d.commands,
            (entity, beast),
            (herds, now),
            (terrain, Grade::Poor),
        );
    }
}

/// Lets the wounds of `beast` bleed one frame: drops fly off, the blood lost goes into its wound and
/// lowers its health, and its `leaking` flag says whether any flow is left.
fn tend(d: &mut Drain, who: (Entity, &mut Beast, &mut Wound, &GlobalTransform)) {
    let (entity, beast, wound, placed) = who;
    let globe = d.terrain.globe().clone();
    let known = globe
        .life()
        .and_then(|l| Some((l, l.species.get(usize::from(beast.index))?)));
    let blood = known.and_then(|(l, s)| blood_of(l, s));
    let (Some((_, species)), Some(blood)) = (known, blood) else {
        d.commands.entity(entity).remove::<Wound>();
        return;
    };
    let circuit = Circuit::of(species, &blood, globe.gravity() as f32);
    let (dt, shows, index) = (d.time.delta_secs(), d.settings.gameplay.blood, beast.index);
    let key = (d.time.elapsed_secs() * 1000.0) as u32;
    let fast = speed(circuit.pressure * circuit.left(wound.lost));
    let (plugged, mut n) = (plugs(&d.spears, entity), 0u32);
    let (lost, mean) = advance(wound, &circuit, dt, plugged, |cut, volume| {
        n += 1;
        if shows {
            let jet = jet_of(placed, cut, &blood, (index, fast));
            throw(&mut d.flings, &jet, (volume, key.wrapping_add(n * 131)));
        }
    });
    wound.lost += lost;
    beast.wits.leaking = mean > 0.0;
    weaken(d, (entity, beast), (lost, circuit.volume));
    if !wound.open() {
        d.commands.entity(entity).remove::<Wound>();
    }
}

/// Lets every wounded animal bleed: a dead one stops, a wound that closed goes away.
pub fn drain(
    mut d: Drain,
    mut beasts: Query<(Entity, &mut Beast, &mut Wound, &GlobalTransform, Has<Dead>)>,
) {
    for (entity, mut beast, mut wound, placed, dead) in &mut beasts {
        if dead {
            beast.wits.leaking = false;
            d.commands.entity(entity).remove::<Wound>();
            continue;
        }
        tend(&mut d, (entity, &mut beast, &mut wound, placed));
    }
}
