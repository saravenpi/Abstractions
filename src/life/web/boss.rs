//! Boss titans and cave-adapted subterranean species promoted from a planet's food web.

use super::super::body::{self, Locomotion};
use super::super::chem::Setting;
use super::super::kinds::{Glow, GlowUse};
use super::super::{Activity, Guild, Habitat, Plan, Rung, Species, history};
use crate::universe::STANDARD_GRAVITY;
use crate::universe::hash::{Rng, child};

/// Child stream seed for boss and cave designation.
const STREAM: u64 = 0x424F_5353;

/// Dread epithets bestowed upon promoted apex boss titans.
const EPITHETS: [&str; 4] = [
    "the dread colossus",
    "the apex titan",
    "the iron juggernaut",
    "the abyssal stalker",
];

/// Skeletal flyer archetype index avoided by land titans.
const FLYER: u8 = 3;

/// Maximum standing height in metres of a cave-adapted species so it clears cavern ceilings.
const CAVE_HEIGHT: f32 = 3.2;

/// Promotes one apex predator boss (plus an optional colossus juggernaut on rich skeletal worlds)
/// and designates one or two compact subterranean cave species on worlds at or above `Rung::Shelled`.
pub(super) fn promote(species: &mut [Species], planet: u64, setting: &Setting) {
    if setting.rung < Rung::Shelled {
        return;
    }
    let mut rng = Rng::new(child(planet, STREAM));
    if let Some(idx) = pick_boss(species, true) {
        crown(&mut species[idx], planet, setting, &mut rng);
    }
    if setting.rung >= Rung::Skeletal
        && species.len() >= 18
        && let Some(idx) = pick_boss(species, false)
    {
        crown(&mut species[idx], planet, setting, &mut rng);
    }
    let caves = if setting.rung >= Rung::Skeletal { 2 } else { 1 };
    for _ in 0..caves {
        if let Some(idx) = pick_cave(species, &mut rng) {
            adapt_cave(&mut species[idx], planet, setting, &mut rng);
        }
    }
}

/// Returns the best candidate index for a boss promotion: an apex land predator when `predator` is
/// true, or the heaviest remaining land consumer otherwise.
fn pick_boss(species: &[Species], predator: bool) -> Option<usize> {
    let score = |s: &Species| {
        let land = matches!(s.role.habitat, Habitat::Ground | Habitat::Shore);
        let hunter = matches!(s.role.guild, Guild::Hunter | Guild::Ambusher);
        let tier = if predator {
            u8::from(land) * 2 + u8::from(hunter) * 4
        } else {
            u8::from(land) * 4 + u8::from(!hunter)
        };
        (tier, (s.role.level * 100.0) as i32, s.history.mass as i64)
    };
    species
        .iter()
        .enumerate()
        .filter(|(_, s)| s.role.guild != Guild::Producer && !s.boss)
        .max_by_key(|(_, s)| score(s))
        .map(|(i, _)| i)
}

/// Returns the index of an eligible non-boss, non-producer consumer to adapt for caves.
fn pick_cave(species: &[Species], rng: &mut Rng) -> Option<usize> {
    let eligible: Vec<usize> = species
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            s.role.guild != Guild::Producer
                && !s.boss
                && !s.cave
                && matches!(s.role.habitat, Habitat::Ground | Habitat::Shore)
        })
        .map(|(i, _)| i)
        .collect();
    if eligible.is_empty() {
        return None;
    }
    Some(eligible[rng.below(eligible.len() as u32) as usize])
}

/// Rebuilds the life history, body and gravity-capped walking speed of `sp` at `mass` kilograms.
fn rebuild_body(sp: &mut Species, seed: u64, mass: f32, setting: &Setting) {
    let mut hist = history::of(seed, &sp.role, mass, setting.rung, &setting.world);
    let built = body::build(
        seed,
        (sp.plan, sp.archetype),
        &sp.role,
        &hist,
        &setting.world,
    );
    if built.locomotion == Locomotion::Walk {
        hist.speed = hist
            .speed
            .min(history::gait(built.height, setting.world.gravity));
    }
    sp.history = hist;
    sp.body = built;
}

/// Crowns `sp` as a solitary towering boss titan with hardened dark tissues, bioluminescent accents
/// and a dread epithet.
fn crown(sp: &mut Species, planet: u64, setting: &Setting, rng: &mut Rng) {
    let heavy = (STANDARD_GRAVITY as f32 / setting.world.gravity.max(1.0))
        .powi(3)
        .clamp(0.4, 3.0);
    let mass = (rng.range(18_000.0, 95_000.0) as f32 * heavy).min(240_000.0);
    sp.boss = true;
    sp.cave = false;
    sp.role.habitat = Habitat::Ground;
    if setting.rung >= Rung::Skeletal {
        sp.plan = Plan::Skeletal;
        if sp.archetype % body::archetypes(Plan::Skeletal) == FLYER {
            sp.archetype = 0;
        }
    }
    let seed = child(planet, 0x5350_0000 + u64::from(sp.id.index));
    rebuild_body(sp, seed, mass, setting);
    sp.history.group_min = 1;
    sp.history.group_max = 1;
    style_boss(sp, setting, rng);
}

/// Applies boss tissue hardening, menacing dark/apex colouration, bioluminescent glow and epithet.
fn style_boss(sp: &mut Species, setting: &Setting, rng: &mut Rng) {
    for t in &mut sp.tissues {
        t.hardness = (t.hardness + 1.5).min(10.0);
        t.toughness = (t.toughness + 0.2).min(1.0);
    }
    if let Some(main) = sp.tissues.first_mut() {
        main.colour = [
            (main.colour[0] * 0.35 + 0.10).min(0.28),
            (main.colour[1] * 0.22 + 0.03).min(0.16),
            (main.colour[2] * 0.25 + 0.04).min(0.18),
        ];
    }
    let g = setting.chemistry.glow;
    let colour = [
        (g[0] * 0.6 + 0.4).min(1.0),
        (g[1] * 0.4 + 0.1).min(1.0),
        (g[2] * 0.4 + 0.08).min(1.0),
    ];
    sp.glow = Some(Glow {
        purpose: GlowUse::Warning,
        colour,
        strength: 0.9,
    });
    if let Some(last) = sp.tissues.last_mut() {
        last.glow = 0.9;
    }
    sp.epithet = EPITHETS[rng.below(EPITHETS.len() as u32) as usize].to_string();
}

/// Adapts `sp` for subterranean caves: compact scale fitting under 3.2 m ceilings, dark-adapted
/// nocturnal eyes, pale/obsidian/crystalline hide and a bioluminescent lure.
fn adapt_cave(sp: &mut Species, planet: u64, setting: &Setting, rng: &mut Rng) {
    sp.cave = true;
    sp.role.habitat = Habitat::Ground;
    sp.role.activity = Activity::Night;
    if sp.plan == Plan::Skeletal && sp.archetype % body::archetypes(Plan::Skeletal) == FLYER {
        sp.archetype = 0;
    }
    let seed = child(planet, 0x5350_0000 + u64::from(sp.id.index));
    let mut mass = rng.range(18.0, 420.0) as f32;
    rebuild_body(sp, seed, mass, setting);
    if sp.body.height > CAVE_HEIGHT {
        let ratio = (CAVE_HEIGHT * 0.9 / sp.body.height).clamp(0.2, 0.95);
        mass = (mass * ratio.powi(3)).max(18.0);
        rebuild_body(sp, seed, mass, setting);
    }
    style_cave(sp, setting, rng);
}

/// Applies pale, obsidian or crystalline cavern hide and a bioluminescent lure glow to `sp`.
fn style_cave(sp: &mut Species, setting: &Setting, rng: &mut Rng) {
    let palettes = [[0.72, 0.74, 0.78], [0.10, 0.11, 0.14], [0.45, 0.62, 0.76]];
    let shade = palettes[rng.below(palettes.len() as u32) as usize];
    if let Some(main) = sp.tissues.first_mut() {
        main.colour = shade;
        main.gloss = (main.gloss + 0.25).min(1.0);
    }
    let g = setting.chemistry.glow;
    let colour = [
        (g[0] * 0.5 + 0.2).min(1.0),
        (g[1] * 0.6 + 0.35).min(1.0),
        (g[2] * 0.6 + 0.4).min(1.0),
    ];
    sp.glow = Some(Glow {
        purpose: GlowUse::Lure,
        colour,
        strength: 0.85,
    });
    if let Some(last) = sp.tissues.last_mut() {
        last.glow = 0.85;
    }
}
