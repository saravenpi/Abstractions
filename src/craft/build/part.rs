//! Parts: grip, shaft, head and counterweight, each sized from its mass and its material's density.

use std::f32::consts::PI;

use super::Hands;
use super::inertia::Piece;
use super::tune::ENGAGE;
use crate::craft::matter::Material;

/// The shape of a head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Form {
    /// A blade, as long and high as it is wide, an eighth of that thick.
    Edge,
    /// A four-sided point, four times as long as its base is wide.
    Point,
    /// A block.
    Mass,
}

/// The head, socketed on the far end of the shaft.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Head {
    /// Its shape.
    pub form: Form,
    /// Mass in kilograms, 0.05 to 2.
    pub mass: f32,
    /// What it is made of.
    pub material: Material,
}

impl Head {
    /// Returns the length of the head along the axis, in metres, from its volume m/ρ.
    pub fn size(&self) -> f32 {
        let volume = self.mass / self.material.density;
        match self.form {
            Form::Edge => (8.0 * volume).cbrt(),
            Form::Point => 4.0 * (0.75 * volume).cbrt(),
            Form::Mass => volume.cbrt(),
        }
    }

    /// Returns the width that meets the target, in metres: the length of edge that bites, the base of
    /// a point, the side of a block.
    pub fn width(&self) -> f32 {
        match self.form {
            Form::Edge => self.size().min(ENGAGE),
            Form::Point => self.size() / 4.0,
            Form::Mass => self.size(),
        }
    }

    /// Returns the area that meets the target in a blunt blow, in square metres.
    pub fn area(&self) -> f32 {
        match self.form {
            Form::Edge => self.width() * self.size() / 8.0,
            Form::Point | Form::Mass => self.width() * self.width(),
        }
    }
}

/// The shaft, a round rod from the hand to the head.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shaft {
    /// Length from the hand to the head, in metres, 0.2 to 1.4.
    pub length: f32,
    /// Diameter in metres.
    pub diameter: f32,
    /// What it is made of.
    pub material: Material,
}

impl Shaft {
    /// Returns the mass of one metre of shaft, ρ·π·d²/4, in kilograms per metre.
    pub fn line(&self) -> f32 {
        self.material.density * PI * self.diameter * self.diameter / 4.0
    }
}

/// A counterweight at the butt, behind the hand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Weight {
    /// Mass in kilograms.
    pub mass: f32,
    /// What it is made of.
    pub material: Material,
}

/// A whole tool, resolved to materials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Build {
    /// Length of the grip behind the hand, in metres, 0.10 to 0.15: the butt of the shaft.
    pub grip: f32,
    /// The shaft.
    pub shaft: Shaft,
    /// The head.
    pub head: Head,
    /// The counterweight, if any.
    pub counterweight: Option<Weight>,
    /// The sinew or resin that binds the sockets, or nothing for a bare friction fit.
    pub binding: Option<Material>,
    /// How it is held.
    pub hands: Hands,
}

impl Build {
    /// Returns the grip, shaft, head and counterweight as point masses about the hand. The hand sits
    /// at the front of the grip; the counterweight sits at the butt.
    pub fn pieces(&self) -> [Piece; 4] {
        let line = self.shaft.line();
        let (length, grip) = (self.shaft.length, self.grip);
        let weight = self.counterweight.map_or(Piece::default(), |w| Piece {
            mass: w.mass,
            at: -grip,
            gyration: (w.mass / w.material.density).cbrt() / 12f32.sqrt(),
        });
        [
            Piece::rod(line * grip, -grip, 0.0),
            Piece::rod(line * length, 0.0, length),
            Piece {
                mass: self.head.mass,
                at: length,
                gyration: self.head.size() / 12f32.sqrt(),
            },
            weight,
        ]
    }

    /// Returns the reach from the hand to the far end of the head, in metres.
    pub fn reach(&self) -> f32 {
        self.shaft.length + self.head.size() / 2.0
    }
}
