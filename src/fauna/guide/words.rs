//! Plain words for the facts of a species: its plan, how it moves and eats, where and when it
//! lives, and its size and weight.

use crate::life::body::{Body, Locomotion};
use crate::life::{Activity, Energy, Guild, Habitat, Plan, Tolerance};

/// Names of the growth architectures of plants, in the order of the contract.
pub const GROWTHS: [&str; 8] = [
    "Trunk and branches",
    "Rosette",
    "Mat",
    "Vine",
    "Bulb",
    "Stalk and cap",
    "Kelp",
    "Crystal lattice",
];

/// Returns the short name of what a species is in the list: its growth for a plant, else its guild.
pub fn kind(guild: Guild, archetype: u8) -> &'static str {
    match guild {
        Guild::Producer => GROWTHS[usize::from(archetype) % GROWTHS.len()],
        Guild::Grazer => "Grazer",
        Guild::Browser => "Browser",
        Guild::Frugivore => "Fruit eater",
        Guild::Nectar => "Nectar drinker",
        Guild::Granivore => "Seed eater",
        Guild::Filter => "Filter feeder",
        Guild::Hunter => "Hunter",
        Guild::Ambusher => "Ambusher",
        Guild::Scavenger => "Scavenger",
        Guild::Decomposer => "Decomposer",
        Guild::Parasite => "Parasite",
    }
}

/// Returns what a species of `guild` eats; producers live on `energy`.
pub fn diet(guild: Guild, energy: Energy) -> &'static str {
    match (guild, energy) {
        (Guild::Producer, Energy::Light) => "Starlight",
        (Guild::Producer, Energy::Chemistry) => "Chemicals from vents",
        (Guild::Producer, Energy::Heat) => "Heat from hot ground",
        (Guild::Grazer, _) => "Low plants",
        (Guild::Browser, _) => "High plants",
        (Guild::Frugivore, _) => "Fruit",
        (Guild::Nectar, _) => "Nectar",
        (Guild::Granivore, _) => "Seeds",
        (Guild::Filter, _) => "Small food it sieves",
        (Guild::Hunter, _) => "Prey it chases",
        (Guild::Ambusher, _) => "Prey it waits for",
        (Guild::Scavenger, _) => "Dead animals",
        (Guild::Decomposer, _) => "Dead matter",
        (Guild::Parasite, _) => "A host it lives on",
    }
}

/// Returns the body plan in plain words.
pub fn plan(plan: Plan) -> &'static str {
    match plan {
        Plan::Skeletal => "Jointed skeleton",
        Plan::Amorphous => "Soft slime",
        Plan::Radial => "Bell and tentacles",
        Plan::Segmented => "Chain of segments",
        Plan::Colonial => "Swarm",
        Plan::Crystalline => "Crystal body",
    }
}

/// Returns how a body moves.
pub fn moves(locomotion: Locomotion) -> &'static str {
    match locomotion {
        Locomotion::Walk => "Walks",
        Locomotion::Slither => "Slithers",
        Locomotion::Pulse => "Pulses",
        Locomotion::Ooze => "Oozes",
        Locomotion::Swarm => "Swarms",
        Locomotion::Tumble => "Hops and rolls",
        Locomotion::Fly => "Flies",
        Locomotion::Swim => "Swims",
    }
}

/// Returns where a species lives.
pub fn habitat(habitat: Habitat) -> &'static str {
    match habitat {
        Habitat::Ground => "On the ground",
        Habitat::Air => "In the air",
        Habitat::Water => "In water",
        Habitat::Shore => "Along shores",
    }
}

/// Returns when a species is out.
pub fn active(activity: Activity) -> &'static str {
    match activity {
        Activity::Day => "By day",
        Activity::Night => "By night",
        Activity::Twilight => "At dawn and dusk",
    }
}

/// Returns the limbs of `body`: its legs, wings and tentacles, or "None".
pub fn limbs(body: &Body) -> String {
    let count = |n: usize, one: &str| match n {
        0 => None,
        1 => Some(format!("1 {one}")),
        n => Some(format!("{n} {one}s")),
    };
    let wings = body.wings.map(|w| format!("wings {:.1} m across", w.span));
    let parts: Vec<String> = [
        count(body.legs.len(), "leg"),
        wings,
        count(body.tentacles.len(), "tentacle"),
    ]
    .into_iter()
    .flatten()
    .collect();
    match parts.is_empty() {
        true => "None".to_string(),
        false => parts.join(", "),
    }
}

/// Returns the climate of `tolerance`: its temperatures in degrees Celsius and how wet.
pub fn climate(tolerance: &Tolerance) -> (String, String) {
    let wet = |m: f32| ["dry", "fair", "damp", "wet"][((m * 4.0) as usize).min(3)];
    let celsius = |kelvin: f32| (kelvin - 273.15).round() as i32;
    let (cold, heat) = (celsius(tolerance.cold), celsius(tolerance.heat));
    let (dry, damp) = (wet(tolerance.dry), wet(tolerance.wet));
    let moisture = match dry == damp {
        true => format!("Always {dry}"),
        false => format!("From {dry} to {damp}"),
    };
    (format!("{cold} to {heat} C"), moisture)
}
