//! The wounds of animals, for everyone: a cut a blow of this machine opened goes out, and a cut that
//! comes back opens in the same animal, bleeding from the time it was made. A spear that stands in a
//! wound plugs it for everyone.

use abstractions_net::wire::{AnimalKey, Wound as Told};
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;

use super::tune::WOUND_KEEP;
use crate::avatar::strike::{Rest, Spear};
use crate::fauna::bleed::tune::PLUG_RANGE;
use crate::fauna::bleed::{Bled, Cut, Wound};
use crate::fauna::{Beast, Creature, Lead, Part};
use crate::net::hunt::Hunted;
use crate::net::hunt::blows::key_of;
use crate::net::hunt::hooks::{self, Echoes};
use crate::net::link::Link;
use crate::space::{Clock, Location};

/// The animals a wound can open on.
type Animals<'w, 's> = Query<'w, 's, (Entity, &'static Beast, Option<&'static mut Wound>)>;

/// The places of the parts and the roots of the animals.
type Places<'w, 's> = (
    Query<'w, 's, (&'static GlobalTransform, &'static Aabb), With<Part>>,
    Query<'w, 's, &'static GlobalTransform, With<Beast>>,
);

/// Tells the server of each wound a blow of this machine opened.
pub fn send(
    mut link: ResMut<Link>,
    (location, clock): (Res<Location>, Res<Clock>),
    mut bled: MessageReader<Bled>,
    animals: Query<&Creature>,
) {
    for Bled { victim, cut } in bled.read() {
        let Ok(creature) = animals.get(*victim) else {
            continue;
        };
        let wound = Told {
            part: cut.part,
            depth: cut.depth,
            width: cut.width,
            vessel: u8::from(cut.major),
            opened: clock.seconds,
            plugged: false,
        };
        hooks::send_wound(&mut link, key_of(&location, creature), wound);
    }
}

/// Returns true when `beast` is the animal `key` names.
fn is(beast: &Beast, key: &AnimalKey) -> bool {
    matches!(beast.lead, Lead::Herd { id, slot } if id == key.herd && slot == key.slot)
}

/// Returns the cut `told` makes at `at`, in the frame of the animal, `now` seconds into the universe.
fn made(told: &Told, at: Vec3, now: f64) -> Cut {
    Cut {
        part: told.part,
        at,
        normal: Vec3::Y,
        width: told.width,
        depth: told.depth,
        major: told.vessel != 0,
        age: (now - told.opened).max(0.0) as f32,
        owed: 0.0,
        plugged: told.plugged,
    }
}

/// Opens `told` on the animal `key` if it is drawn, and returns false while it is not.
fn open(
    (animals, (parts, roots)): (&mut Animals, &Places),
    commands: &mut Commands,
    (key, told): (&AnimalKey, &Told),
    now: f64,
) -> bool {
    let found = animals.iter_mut().find(|(_, beast, _)| is(beast, key));
    let Some((entity, beast, wound)) = found else {
        return false;
    };
    let part = beast.parts.get(usize::from(told.part)).copied().flatten();
    let (Some(Ok(root)), Some(Ok((placed, aabb)))) =
        (Some(roots.get(entity)), part.map(|part| parts.get(part)))
    else {
        return false;
    };
    let middle = placed.transform_point(Vec3::from(aabb.center));
    let cut = made(told, root.affine().inverse().transform_point3(middle), now);
    match wound {
        Some(mut wound) => wound.add(cut),
        None => {
            let mut wound = Wound::default();
            wound.add(cut);
            commands.entity(entity).insert(wound);
        }
    }
    true
}

/// Opens the wounds the others made on the animals drawn here. A wound of an animal that is not
/// drawn yet waits for it, for [`WOUND_KEEP`] seconds.
pub fn play(
    mut echoes: ResMut<Echoes>,
    (location, clock): (Res<Location>, Res<Clock>),
    (mut animals, places): (Animals, Places),
    mut commands: Commands,
) {
    for (key, told) in std::mem::take(&mut echoes.wounds) {
        let here = key.body == location.body;
        let young = clock.seconds - told.opened < WOUND_KEEP;
        let opened = here
            && open(
                (&mut animals, &places),
                &mut commands,
                (&key, &told),
                clock.seconds,
            );
        if young && !opened {
            hooks::keep(&mut echoes.wounds, (key, told));
        }
    }
}

/// Tells the server of each wound a spear plugs that it was not told of.
pub fn plugs(
    (mut link, mut hunted): (ResMut<Link>, ResMut<Hunted>),
    location: Res<Location>,
    spears: Query<(&Transform, &Spear, &ChildOf)>,
    animals: Query<(&Creature, &Wound)>,
) {
    for (at, spear, of) in &spears {
        let Ok((creature, wound)) = animals.get(of.parent()) else {
            continue;
        };
        let near = |cut: &&Cut| at.translation.distance(cut.at) < PLUG_RANGE;
        for cut in wound.cuts.iter().flatten().filter(near) {
            let key = key_of(&location, creature);
            if spear.rest == Rest::Stuck && hunted.plugged.insert((key, cut.part)) {
                hooks::send_plug(&mut link, key, cut.part);
            }
        }
    }
}

/// Plugs the wounds of the parts the others plugged, on the animals drawn here.
pub fn plugged(
    (mut echoes, mut hunted): (ResMut<Echoes>, ResMut<Hunted>),
    mut animals: Query<(&Beast, &mut Wound)>,
) {
    for (key, part) in std::mem::take(&mut echoes.plugs) {
        hunted.plugged.insert((key, part));
        let found = animals.iter_mut().find(|(beast, _)| is(beast, &key));
        let waiting = echoes.wounds.iter().any(|(wounded, _)| *wounded == key);
        if let Some((_, mut wound)) = found {
            let same = |cut: &&mut Cut| cut.part == part;
            wound
                .cuts
                .iter_mut()
                .flatten()
                .filter(same)
                .for_each(|cut| cut.plugged = true);
        } else if waiting {
            hooks::keep(&mut echoes.plugs, (key, part));
        }
    }
}
