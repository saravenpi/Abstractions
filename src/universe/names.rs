//! Names: several weighted generators, so most stars carry a catalogue code and few a proper name.

use super::hash::Rng;

/// Sounds a syllable may start with. The empty ones make some syllables start on their vowel.
const ONSETS: [&str; 28] = [
    "", "", "b", "c", "d", "f", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "z", "th",
    "sh", "br", "dr", "kr", "st", "tr", "vr", "ly", "qu",
];

/// Vowel sounds, the plain ones listed more often.
const VOWELS: [&str; 14] = [
    "a", "a", "e", "e", "i", "o", "o", "u", "ae", "ia", "ei", "ou", "ar", "or",
];

/// Sounds a word may end with, half of them silent.
const CODAS: [&str; 12] = ["", "", "", "", "", "", "n", "r", "s", "l", "x", "th"];

/// Greek letters used as a prefix of some star names.
const GREEK: [&str; 10] = [
    "Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta", "Eta", "Theta", "Sigma", "Omega",
];

/// Roman numerals of the moons of a planet, from the closest one out.
const NUMERALS: [&str; 6] = ["I", "II", "III", "IV", "V", "VI"];

/// Returns one entry of a list, every entry being as likely.
fn pick<'a>(rng: &mut Rng, list: &[&'a str]) -> &'a str {
    list[rng.below(list.len() as u32) as usize]
}

/// Returns a made-up capitalised word of two or three syllables.
fn word(rng: &mut Rng) -> String {
    let mut word = String::with_capacity(12);
    for _ in 0..2 + rng.below(2) {
        word.push_str(pick(rng, &ONSETS));
        word.push_str(pick(rng, &VOWELS));
    }
    word.push_str(pick(rng, &CODAS));
    word[..1].make_ascii_uppercase();
    word
}

/// Returns a capital letter.
fn letter(rng: &mut Rng) -> char {
    char::from(b'A' + rng.below(26) as u8)
}

/// Returns a catalogue code like `A-21` or `KX-407`: one or two letters and a number below 10,000.
fn code(rng: &mut Rng) -> String {
    let mut code = String::from(letter(rng));
    if rng.below(2) == 0 {
        code.push(letter(rng));
    }
    let digits = 1 + rng.below(4);
    format!("{code}-{}", 1 + rng.below(10u32.pow(digits) - 1))
}

/// Returns the name of a star: a code for 64 in 100, else a Greek prefix, a numbered word or a plain word.
pub(super) fn star(seed: u64) -> String {
    let mut rng = Rng::new(seed);
    match rng.below(100) {
        0..64 => code(&mut rng),
        64..78 => format!("{} {}", pick(&mut rng, &GREEK), word(&mut rng)),
        78..90 => format!("{} {}", word(&mut rng), 1 + rng.below(99)),
        _ => word(&mut rng),
    }
}

/// Returns the name of a galaxy: a proper name for the home galaxy and 3 in 10 others, else a code.
pub(super) fn galaxy(seed: u64, is_home: bool) -> String {
    let mut rng = Rng::new(seed);
    if is_home || rng.below(10) < 3 {
        return word(&mut rng);
    }
    format!("GX-{}", 1 + rng.below(9999))
}

/// Returns the name of planet number `index` of a star: the star name and a letter from `b` on.
pub(super) fn planet(star: &str, index: u8) -> String {
    format!("{star} {}", char::from(b'b' + index))
}

/// Returns the name of moon number `rank`, from 1, of a planet: the planet name and a Roman
/// numeral, as astronomers number the moons of Jupiter.
pub(super) fn moon(planet: &str, rank: u8) -> String {
    let numeral = NUMERALS[usize::from(rank.clamp(1, 6)) - 1];
    format!("{planet} {numeral}")
}
