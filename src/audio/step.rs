//! One footfall as a one-shot cue: what the foot lands on, how hard, which foot, and where it
//! is for the listener.

use super::cue::StepSurface;

/// Levels of the force in a packed step.
const FORCES: f32 = 30.0;

/// Levels of the side and of the nearness in a packed step. Even, so that straight ahead packs
/// to itself.
const PLACES: f32 = 14.0;

/// One footfall, positioned for the listener.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    /// What the foot lands on.
    pub surface: StepSurface,
    /// How hard it lands, from 0 to 1: the speed of the sole into the ground, 1 at a hard
    /// landing. A walk at home is near a half.
    pub force: f32,
    /// Where it is: -1 at the listener's left to 1 at the right.
    pub side: f32,
    /// How loud it arrives, 0 out of earshot to 1 under the listener.
    pub near: f32,
    /// Whether it is the left foot. The two feet of a body do not sound alike.
    pub left: bool,
    /// Whether both feet land at once from a jump or a fall, with no roll from heel to toe.
    pub landing: bool,
    /// Whether it is heard through the body alone: the listener's own step where there is no air.
    pub body: bool,
}

impl Step {
    /// Returns a walking step of the listener's own right foot on `surface`.
    pub const fn on(surface: StepSurface) -> Self {
        Self {
            surface,
            force: 0.5,
            side: 0.0,
            near: 1.0,
            left: false,
            landing: false,
            body: false,
        }
    }

    /// Packs the step into 19 bits: surface 3, force 5, side 4, nearness 4, then the three flags.
    pub(super) fn pack(self) -> u32 {
        let force = (self.force.clamp(0.0, 1.0) * FORCES).round() as u32;
        let side = ((self.side.clamp(-1.0, 1.0) + 1.0) * 0.5 * PLACES).round() as u32;
        let near = (self.near.clamp(0.0, 1.0) * PLACES).round() as u32;
        let flags = u32::from(self.left) | u32::from(self.landing) << 1 | u32::from(self.body) << 2;
        self.surface as u32 | force << 3 | side << 8 | near << 12 | flags << 16
    }

    /// Unpacks a step packed by [`Step::pack`], or nothing when the bits hold none.
    pub(super) fn unpack(bits: u32) -> Option<Self> {
        let field = |shift: u32, width: u32| (bits >> shift) & ((1 << width) - 1);
        Some(Self {
            surface: *StepSurface::ALL.get(field(0, 3) as usize)?,
            force: field(3, 5) as f32 / FORCES,
            side: field(8, 4) as f32 / PLACES * 2.0 - 1.0,
            near: field(12, 4) as f32 / PLACES,
            left: field(16, 1) == 1,
            landing: field(17, 1) == 1,
            body: field(18, 1) == 1,
        })
    }
}
