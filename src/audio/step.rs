//! One footfall as a one-shot cue: what the foot lands on, how hard and how heavy, and how long it
//! stays down. The feet of the robots send it as a [`super::Sound`] from where it lands, the
//! treads play it.

use super::cue::StepSurface;

/// How a body meets the ground.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Contact {
    /// One foot of a walk or a run: the heel strikes, then the sole rolls onto the toe.
    Stride,
    /// Both feet at once from a jump, a fall or a glide: flat, with no roll.
    Landing,
    /// The body slides along the ground: a scrape with no strike.
    Skid,
}

/// Every contact, in code order.
const CONTACTS: [Contact; 3] = [Contact::Stride, Contact::Landing, Contact::Skid];

/// Unit each packed field is stored in, in the order of [`Step::fields`].
const UNITS: [f32; 10] = [1.0, 1.0, 1.0 / 63.0, 0.25, 4.0, 0.008, 0.01, 0.04, 1.0, 1.0];

/// Width in bits of each packed field, in the order of [`Step::fields`]: 40 bits in all.
const WIDTHS: [u32; 10] = [4, 2, 6, 6, 5, 5, 6, 4, 1, 1];

/// One footfall.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    /// What the foot lands on.
    pub surface: StepSurface,
    /// How the body meets the ground.
    pub contact: Contact,
    /// Speed of the sole into the ground over the speed of a hard landing, from 0 to 1. The
    /// pressure the strike makes follows it.
    pub force: f32,
    /// Weight the foot carries while it is down, in weights of the robot at home: more under a
    /// run, under a landing, or on a heavier world. The grains of the ground follow it.
    pub load: f32,
    /// Mass in kilograms that strikes the ground: a leg, or both legs on a landing. A heavier
    /// strike stays longer on the ground and sounds lower.
    pub mass: f32,
    /// Seconds from the strike of the heel to the toe.
    pub roll: f32,
    /// Seconds from the strike to the moment the foot leaves the ground.
    pub lift: f32,
    /// Depth in metres of the water the foot lands in.
    pub depth: f32,
    /// Whether it is the left foot. The two feet of a body do not sound alike.
    pub left: bool,
    /// Whether the ground is cold enough for packed snow to squeak, under about -10 degrees.
    pub cold: bool,
}

impl Step {
    /// Returns a walking step of a right foot on `surface` at home.
    pub const fn on(surface: StepSurface) -> Self {
        Self {
            surface,
            contact: Contact::Stride,
            force: 0.13,
            load: 2.9,
            mass: 12.9,
            roll: 0.05,
            lift: 0.07,
            depth: 0.0,
            left: false,
            cold: false,
        }
    }

    /// Returns the fields as numbers, in packing order.
    fn fields(&self) -> [f32; 10] {
        [
            self.surface as u8 as f32,
            self.contact as u8 as f32,
            self.force,
            self.load,
            self.mass,
            self.roll,
            self.lift,
            self.depth,
            f32::from(u8::from(self.left)),
            f32::from(u8::from(self.cold)),
        ]
    }

    /// Packs the step into 40 bits. Each field is rounded to its unit and held in its width.
    pub(super) fn pack(self) -> u64 {
        let mut shift = 0;
        let mut bits = 0_u64;
        for ((value, unit), width) in self.fields().into_iter().zip(UNITS).zip(WIDTHS) {
            let most = (1_u64 << width) - 1;
            let level = (value.max(0.0) / unit).round() as u64;
            bits |= level.min(most) << shift;
            shift += width;
        }
        bits
    }

    /// Unpacks a step packed by [`Step::pack`], or nothing when the bits hold none.
    pub(super) fn unpack(bits: u64) -> Option<Self> {
        let mut shift = 0;
        let mut f = [0.0; 10];
        for ((field, unit), width) in f.iter_mut().zip(UNITS).zip(WIDTHS) {
            *field = ((bits >> shift) & ((1_u64 << width) - 1)) as f32 * unit;
            shift += width;
        }
        Some(Self {
            surface: *StepSurface::ALL.get(f[0] as usize)?,
            contact: *CONTACTS.get(f[1] as usize)?,
            force: f[2],
            load: f[3],
            mass: f[4],
            roll: f[5],
            lift: f[6],
            depth: f[7],
            left: f[8] > 0.5,
            cold: f[9] > 0.5,
        })
    }
}
