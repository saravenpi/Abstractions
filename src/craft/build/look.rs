//! The look of a tool: what its icon is drawn from, the same wherever the tool shows. The figure of
//! its head tells its kind, the length of its shaft and the tint of each material are the second
//! cue. Two tools with one look draw one picture.

use super::{Build, Intent};
use crate::craft::matter::{Class, Stuff};

/// The kind of a tool as its head shows it, and the word it is called by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Figure {
    /// A short single-edged blade.
    Knife,
    /// A long single-edged blade.
    Blade,
    /// A heavy edge across the shaft.
    Axe,
    /// A leaf point on a long shaft.
    Spear,
    /// A small point on a long shaft, balanced to throw.
    Javelin,
    /// A shaft thickened at its end.
    Club,
    /// A heavy block across the shaft.
    Hammer,
}

impl Figure {
    /// Every figure.
    pub const ALL: [Figure; 7] = [
        Figure::Knife,
        Figure::Blade,
        Figure::Axe,
        Figure::Spear,
        Figure::Javelin,
        Figure::Club,
        Figure::Hammer,
    ];

    /// Returns the figure of a tool for `intent` whose head holds `share` of its mass, `reach`
    /// metres long.
    pub fn of(intent: Intent, share: f32, reach: f32) -> Figure {
        match intent {
            Intent::Cut if share > 0.6 => Figure::Axe,
            Intent::Cut if reach < 0.5 => Figure::Knife,
            Intent::Cut => Figure::Blade,
            Intent::Pierce => Figure::Spear,
            Intent::Crush if share > 0.6 => Figure::Hammer,
            Intent::Crush => Figure::Club,
            Intent::Throw => Figure::Javelin,
        }
    }

    /// Returns the word for a tool of this figure.
    pub fn word(self) -> &'static str {
        match self {
            Figure::Knife => "knife",
            Figure::Blade => "blade",
            Figure::Axe => "axe",
            Figure::Spear => "spear",
            Figure::Javelin => "javelin",
            Figure::Club => "club",
            Figure::Hammer => "hammer",
        }
    }
}

/// The tone a material is drawn in: one grey for each family, no colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tint {
    /// Stone and glass, in full white.
    Stone,
    /// Bone and antler.
    Bone,
    /// Horn and shell.
    Horn,
    /// Wood, resin and hide, the dimmest.
    Wood,
}

impl Tint {
    /// Every tint, brightest first.
    pub const ALL: [Tint; 4] = [Tint::Stone, Tint::Bone, Tint::Horn, Tint::Wood];

    /// Returns the tint of `stuff`.
    pub fn of(stuff: &Stuff) -> Tint {
        let Stuff::Tissue { class, .. } = stuff else {
            return Tint::Stone;
        };
        match class {
            Class::Glass => Tint::Stone,
            Class::Bone | Class::Antler => Tint::Bone,
            Class::Keratin | Class::Cuticle => Tint::Horn,
            Class::Hide | Class::Wood | Class::Resin => Tint::Wood,
        }
    }

    /// Returns the opacity of white the tint is drawn with, 0 to 255: one step of the text tones
    /// apart, so each reads as its own grey on a dark panel.
    pub fn alpha(self) -> u8 {
        match self {
            Tint::Stone => 255,
            Tint::Bone => 222,
            Tint::Horn => 186,
            Tint::Wood => 150,
        }
    }
}

/// What the icon of a tool is drawn from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Look {
    /// What the tool is for, which names its fallback icon.
    pub intent: Intent,
    /// The figure of its head, or nothing when it has no parts to draw.
    pub figure: Option<Figure>,
    /// How long it is, 0 shortest to 3 longest.
    pub length: u8,
    /// The tint of its head.
    pub head: Tint,
    /// The tint of its shaft.
    pub shaft: Tint,
}

/// Reaches in metres that start each length after the shortest.
const LENGTHS: [f32; 3] = [0.5, 0.8, 1.1];

impl Look {
    /// Returns the look of `build` used for `intent`, its head of `head` and its shaft of `shaft`.
    pub fn of(build: &Build, intent: Intent, head: Option<&Stuff>, shaft: Option<&Stuff>) -> Look {
        let total = super::mass(&build.pieces()).max(f32::EPSILON);
        let reach = build.reach();
        Look {
            intent,
            figure: Some(Figure::of(intent, build.head.mass / total, reach)),
            length: LENGTHS.iter().filter(|from| reach >= **from).count() as u8,
            head: head.map_or(Tint::Stone, Tint::of),
            shaft: shaft.map_or(Tint::Wood, Tint::of),
        }
    }

    /// Returns the look of a tool for `intent` with no parts to draw: its icon is the intent's.
    pub fn bare(intent: Intent) -> Look {
        Look {
            intent,
            figure: None,
            length: 0,
            head: Tint::Stone,
            shaft: Tint::Wood,
        }
    }

    /// Returns a number that differs for every look, for the key of its cached picture.
    pub fn code(&self) -> u16 {
        let figure = self.figure.map_or(0, |f| f as u16 + 1);
        let fields = [
            (self.intent as u16, 2),
            (figure, 3),
            (u16::from(self.length.min(3)), 2),
            (self.head as u16, 2),
            (self.shaft as u16, 2),
        ];
        fields
            .into_iter()
            .fold(0, |code, (value, bits)| code << bits | value)
    }
}
