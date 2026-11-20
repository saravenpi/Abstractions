//! Voices: how a species sounds, from how its body makes sound, and how high, from its size.
//!
//! The pitch of a call falls with body size: the dominant frequency of animal calls scales about as
//! the mass to the power -0.4, from about 1 kHz for a kilogram (Fletcher 2004, "A simple
//! frequency-scaling rule for animal communication", JASA 115, 2334-2338,
//! <https://doi.org/10.1121/1.1694997>). A long, thin body has a long vocal tract, and a longer tract
//! lowers the formants of the call (Fitch 1997, "Vocal tract length and formant frequency dispersion
//! correlate with body size in rhesus macaques", JASA 102, 1213-1222, <https://doi.org/10.1121/1.421048>).
//!
//! How the body makes the sound picks the timbre. Animals that breathe air push it through a larynx
//! or a syrinx and moan, hoot, whistle, growl or hiss by their way of life. Shelled animals have no
//! lungs to voice with and stridulate, rubbing a ridged file on a scraper, which gives rasps and trains
//! of clicks (<https://en.wikipedia.org/wiki/Stridulation>). Soft bodies only push fluid through
//! themselves, a gurgle. A call made in water is muffled where a listener stands in air: the
//! surface reflects almost all of it (<https://en.wikipedia.org/wiki/Acoustic_impedance>).

use crate::life::body::Locomotion;
use crate::life::{Guild, Habitat, Plan, Rung, Species};

/// Frequency in hertz of the call of a one kilogram animal.
const HZ_PER_KG: f32 = 1000.0;

/// Exponent of the mass in the frequency of a call.
const HZ_EXPONENT: f32 = -0.4;

/// Lowest and highest frequency in hertz of a call.
const HZ_SPAN: (f32, f32) = (45.0, 4000.0);

/// Usual length of a body over the side of a cube of its mass in water, the shape that keeps
/// Fletcher's pitch as it is.
const ELONGATION: f32 = 2.5;

/// Exponent of the elongation over its usual value in the pitch: longer tracts call lower.
const TRACT_EXPONENT: f32 = -0.3;

/// Pitch factor of a long neck, the longest tract of all.
const NECK: f32 = 0.85;

/// Kilograms from which a stridulator rasps rather than clicks.
const RASP_MASS: f32 = 2.0;

/// How a call sounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timbre {
    /// A low, falling moan: grazers that breathe air.
    Low,
    /// A round hoot: browsers.
    Hoot,
    /// A whistle gliding up: flyers, swimmers, seed and fruit eaters.
    Whistle,
    /// A rough growl: hunters.
    Growl,
    /// A breathy hiss: ambushers.
    Hiss,
    /// A gurgle: scavengers and soft bodies.
    Croak,
    /// A train of clicks: small stridulators and crystals.
    Click,
    /// A scraped rasp: big stridulators.
    Rasp,
}

/// Every timbre, in code order.
pub const TIMBRES: [Timbre; 8] = [
    Timbre::Low,
    Timbre::Hoot,
    Timbre::Whistle,
    Timbre::Growl,
    Timbre::Hiss,
    Timbre::Croak,
    Timbre::Click,
    Timbre::Rasp,
];

/// How a body makes a sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Organ {
    /// Air pushed through a voice box.
    Breath,
    /// A file rubbed on a scraper of shell.
    Shell,
    /// Fluid pushed through a soft body.
    Soft,
    /// Facets that knock together.
    Crystal,
}

/// The voice of a species.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voice {
    /// Main frequency in hertz.
    pub hz: f32,
    /// How it sounds.
    pub timbre: Timbre,
    /// Notes in one call, 1 to 4: more for evolved life.
    pub notes: u8,
    /// True when it calls from under water, heard muffled from the air.
    pub muffled: bool,
}

/// Returns how `species` on a planet whose life reached `rung` makes its sounds.
fn organ(species: &Species, rung: Rung) -> Organ {
    match (rung, species.plan) {
        (Rung::Mats | Rung::Soft, _) | (_, Plan::Amorphous | Plan::Radial | Plan::Colonial) => {
            Organ::Soft
        }
        (_, Plan::Crystalline) => Organ::Crystal,
        (Rung::Shelled, _) | (_, Plan::Segmented) => Organ::Shell,
        _ => Organ::Breath,
    }
}

/// Returns the timbre of `species` that makes its sounds with `organ`.
fn timbre(species: &Species, organ: Organ) -> Timbre {
    let flies = species.body.locomotion == Locomotion::Fly;
    match (organ, species.role.guild, species.role.habitat) {
        (Organ::Soft, ..) => Timbre::Croak,
        (Organ::Crystal, ..) => Timbre::Click,
        (Organ::Shell, ..) if species.history.mass >= RASP_MASS => Timbre::Rasp,
        (Organ::Shell, ..) => Timbre::Click,
        (_, Guild::Hunter, _) => Timbre::Growl,
        (_, Guild::Ambusher, _) => Timbre::Hiss,
        (_, Guild::Scavenger, _) => Timbre::Croak,
        (_, Guild::Decomposer | Guild::Parasite | Guild::Producer, _) => Timbre::Click,
        (_, _, Habitat::Air | Habitat::Water) => Timbre::Whistle,
        _ if flies => Timbre::Whistle,
        (_, Guild::Grazer, _) => Timbre::Low,
        (_, Guild::Browser, _) => Timbre::Hoot,
        _ => Timbre::Whistle,
    }
}

/// Returns the pitch factor of the shape of `species` breathing through `organ`: long, thin bodies
/// and long necks have long tracts and call lower.
fn tract(species: &Species, organ: Organ) -> f32 {
    let side = (species.history.mass.max(1e-4) / 1000.0).cbrt();
    let elongation = species.body.length.max(0.01) / side / ELONGATION;
    let shape = elongation.powf(TRACT_EXPONENT).clamp(0.75, 1.3);
    let neck = organ == Organ::Breath && species.body.neck.is_some();
    shape * if neck { NECK } else { 1.0 }
}

/// Returns the voice of `species` on a planet whose life reached `rung`: lower for bigger animals,
/// sounding as its body makes sound.
pub fn voice(species: &Species, rung: Rung) -> Voice {
    let organ = organ(species, rung);
    let mass = species.history.mass.max(1e-4);
    let hz = HZ_PER_KG * mass.powf(HZ_EXPONENT) * tract(species, organ);
    let timbre = timbre(species, organ);
    let notes = match (timbre, rung) {
        (Timbre::Click | Timbre::Rasp, _) => 4,
        (_, Rung::Old) => 3,
        (_, Rung::Skeletal) => 2,
        _ => 1,
    };
    let muffled =
        species.role.habitat == Habitat::Water || species.body.locomotion == Locomotion::Swim;
    Voice {
        hz: hz.clamp(HZ_SPAN.0, HZ_SPAN.1),
        timbre,
        notes,
        muffled,
    }
}
