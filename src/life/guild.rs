//! Guilds, habitats and activity: how a consumer eats, where it lives and when it is out.

use super::chem::Setting;
use super::{Activity, Glow, GlowUse, Guild, Habitat, Reaction, Role, Rung, Tissue};
use crate::universe::hash::Rng;

#[cfg(test)]
mod tests;

/// Pressure, the home air being 1, under which nothing flies.
pub const FLIGHT_AIR: f32 = 0.3;

/// Returns the guild of a consumer at trophic `level` with niche value `niche` on the planet of `setting`.
///
/// Plant eaters sit near level 2 and split by niche: grazers low, browsers and fruit, nectar and
/// seed eaters higher, filter feeders where there is a liquid. Mid levels scavenge, decompose and
/// ride hosts, and the high levels hunt or wait in ambush. Fruit, nectar and hunting all start at
/// the shelled rung: below it the upper levels only scavenge, rot, ride or sieve.
pub fn of(level: f32, niche: f32, setting: &Setting, rng: &mut Rng) -> Guild {
    let wet = setting.world.ocean > 0.0;
    let flowers = setting.rung >= Rung::Shelled;
    let roll = rng.below(10);
    if level < 2.3 {
        match (roll, wet, flowers) {
            (0, true, _) => Guild::Filter,
            (1, _, true) => Guild::Nectar,
            (2, _, true) => Guild::Frugivore,
            (3, _, _) if niche > 0.4 => Guild::Granivore,
            _ if niche > 0.5 => Guild::Browser,
            _ => Guild::Grazer,
        }
    } else if !flowers {
        [
            Guild::Scavenger,
            Guild::Decomposer,
            Guild::Parasite,
            Guild::Filter,
        ][roll as usize % 4]
    } else if level < 3.0 {
        match roll {
            0..2 => Guild::Scavenger,
            2 => Guild::Decomposer,
            3 => Guild::Parasite,
            4..7 => Guild::Ambusher,
            _ => Guild::Hunter,
        }
    } else {
        [Guild::Hunter, Guild::Ambusher][usize::from(roll < 3)]
    }
}

/// Returns where a consumer of `guild` lives on the planet of `setting`: flyers only with enough air,
/// swimmers only where there is a liquid, and filter feeders in it.
pub fn habitat(guild: Guild, setting: &Setting, rng: &mut Rng) -> Habitat {
    let world = &setting.world;
    let air = world.pressure >= FLIGHT_AIR && setting.rung >= Rung::Shelled;
    let sea = world.ocean > 0.0;
    let roll = rng.unit() as f32;
    match guild {
        Guild::Filter if sea => Habitat::Water,
        Guild::Nectar if air => Habitat::Air,
        _ if sea && roll < 0.35 * world.ocean => Habitat::Water,
        _ if sea && roll < 0.4 * world.ocean + 0.1 => Habitat::Shore,
        _ if air && roll > 0.85 => Habitat::Air,
        _ => Habitat::Ground,
    }
}

/// Returns when a consumer of `guild` in `habitat` is out on the planet of `setting`: at night under a
/// dim star or where glow pays, by day for nectar feeders, else mostly by day.
pub fn activity(guild: Guild, habitat: Habitat, setting: &Setting, rng: &mut Rng) -> Activity {
    let dim = setting.world.star < 3600.0 || setting.world.flux < 0.1;
    let glow_pays = guild == Guild::Ambusher || habitat == Habitat::Water;
    match (guild, rng.below(10)) {
        (Guild::Nectar, _) => Activity::Day,
        (_, 0..6) if dim => Activity::Night,
        (_, 0..5) if glow_pays => Activity::Night,
        (_, 0..6) => Activity::Day,
        (_, 6..8) => Activity::Twilight,
        _ => Activity::Night,
    }
}

/// Returns the glow of a species of `role` with `tissues` on the planet of `setting`, if glowing pays it:
/// a lure on a night ambusher, warning on toxic night tissue, belly lights on night swimmers, and
/// courtship at dusk on half of the twilight species.
pub fn glow(role: &Role, tissues: &[Tissue], setting: &Setting, rng: &mut Rng) -> Option<Glow> {
    let toxic = tissues.iter().any(|t| t.reaction != Reaction::Inert);
    let dark = role.activity == Activity::Night;
    let purpose = match (role.guild, role.habitat, role.activity) {
        (Guild::Ambusher, _, _) if dark => GlowUse::Lure,
        _ if dark && toxic => GlowUse::Warning,
        (_, Habitat::Water, _) if dark => GlowUse::Belly,
        (_, _, Activity::Twilight) if rng.below(2) == 0 => GlowUse::Courtship,
        _ => return None,
    };
    Some(Glow {
        purpose,
        colour: setting.chemistry.glow,
        strength: rng.range(0.6, 1.0) as f32,
    })
}
